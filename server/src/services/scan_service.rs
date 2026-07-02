//! Library scan, TMDB enrichment, and SQLite persistence.

use std::collections::HashSet;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use nest_error::NestResult;
use nest_media_library::{MovieScanCandidate, ScanResult, ScanStats};
use tracing::info;

use crate::config::ServerConfig;
use crate::db::{LibraryRepository, StoredFile};
use crate::services::ai::AiRuntime;
use crate::services::artwork::ArtworkRuntime;
use crate::services::catalog::{catalog_from_records, catalog_from_scan, LoonCatalog, LoonMovieRecord};
use crate::services::enrichment::{enrich_candidate, enrich_candidate_by_tmdb_id, ScanArtworkMap};
use crate::services::filename_guess::guess_movie_from_filename;
use crate::services::library::discover_library;
use crate::services::scan_events::{scan_progress, ScanReporter};
use crate::services::scan_state::ScanPhase;
use crate::services::tmdb::TmdbRuntime;

/// Options controlling scan behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScanOptions {
    /// Re-fetch TMDB metadata for every movie that is not manually locked.
    pub full_metadata: bool,
}

/// Result of a completed library scan.
#[derive(Debug, Clone)]
pub struct ScanRunResult {
    /// Unix timestamp when the scan finished.
    pub scanned_at: u64,
    /// Number of movies in the library after persistence.
    pub movies_count: usize,
    /// Scan statistics.
    pub stats: ScanStats,
}

/// Runs discovery, incremental TMDB enrichment, and SQLite upsert.
pub async fn scan_and_persist(
    config: &ServerConfig,
    repo: &LibraryRepository,
    tmdb: Option<&TmdbRuntime>,
    ai: Option<&AiRuntime>,
    options: ScanOptions,
    reporter: Option<&ScanReporter>,
    artwork_cache: Option<&ArtworkRuntime>,
) -> NestResult<ScanRunResult> {
    let started = Instant::now();
    let library_id = config.library.id.as_str().to_string();
    let config = config.clone();

    if let Some(reporter) = reporter {
        reporter
            .progress(scan_progress(
                ScanPhase::Discovering,
                &ScanStats::default(),
                0,
                0,
                None,
            ))
            .await;
    }

    let mut result = tokio::task::spawn_blocking(move || discover_library(&config))
        .await
        .map_err(|error| {
            nest_error::NestError::task(format!("library scan task join failed: {error}"))
        })??;

    let scanned_at = now_secs();
    result.finished_at = scanned_at;
    let stats = result.stats.clone();

    if let Some(reporter) = reporter {
        reporter
            .progress(scan_progress(ScanPhase::Enriching, &stats, 0, 0, None))
            .await;
    }

    let mut artwork_map = ScanArtworkMap::new();
    let mut refreshed_paths = HashSet::new();

    for candidate in &result.candidates {
        let path = candidate.file.relative_path.clone();
        let existing_file = repo.get_file_by_path(&path)?;
        let existing_movie = existing_movie_for_path(repo, &path)?;
        if should_enrich_metadata(options, existing_file.as_ref(), existing_movie.as_ref(), candidate)
        {
            refreshed_paths.insert(path);
        }
    }

    let total_to_enrich = refreshed_paths.len() as u32;
    let mut enriched = 0u32;

    for candidate in &mut result.candidates {
        let path = candidate.file.relative_path.clone();
        let existing_file = repo.get_file_by_path(&path)?;
        let existing_movie = existing_movie_for_path(repo, &path)?;
        let needs_tmdb = should_enrich_metadata(
            options,
            existing_file.as_ref(),
            existing_movie.as_ref(),
            candidate,
        );

        if needs_tmdb {
            if let Some(reporter) = reporter {
                reporter
                    .progress(scan_progress(
                        ScanPhase::Enriching,
                        &stats,
                        enriched,
                        total_to_enrich,
                        Some(path.clone()),
                    ))
                    .await;
            }

            if let Some(tmdb) = tmdb {
                if let Some(tmdb_id) = stored_tmdb_id(existing_movie.as_ref()) {
                    enrich_candidate_by_tmdb_id(candidate, tmdb_id, tmdb, &mut artwork_map).await;
                } else {
                    if let Some(ai) = ai {
                        apply_ai_filename_guess(ai, candidate, &path).await;
                    }
                    enrich_candidate(candidate, tmdb, &mut artwork_map).await;
                }
            }
            enriched += 1;
        } else if let Some(existing_movie) = existing_movie {
            candidate.guessed_title = Some(existing_movie.title.clone());
            candidate.guessed_year = existing_movie.year;
        }
    }

    if let Some(reporter) = reporter {
        reporter
            .progress(scan_progress(
                ScanPhase::Persisting,
                &stats,
                enriched,
                total_to_enrich,
                None,
            ))
            .await;
    }

    let catalog = catalog_from_scan(
        ScanResult {
            library_id: result.library_id.clone(),
            started_at: result.started_at,
            finished_at: scanned_at,
            candidates: result.candidates.clone(),
            errors: result.errors.clone(),
            stats: stats.clone(),
        },
        &artwork_map,
    );

    for mut record in catalog.records() {
        let candidate = result
            .candidates
            .iter()
            .find(|item| item.file.relative_path == record.relative_path);

        if refreshed_paths.contains(&record.relative_path) {
            if let Some(artwork_cache) = artwork_cache {
                let _ = artwork_cache.invalidate_movie(&record.slug);
            }
        }

        let (size_bytes, modified_secs) = candidate
            .map(|item| (item.file.size_bytes, item.file.modified_secs))
            .unwrap_or((0, None));

        let existing_file = repo.get_file_by_path(&record.relative_path)?;
        let existing_movie = existing_file
            .as_ref()
            .and_then(|file| repo.get_by_media_id(&file.movie_id).ok().flatten());

        if let Some(existing) = existing_movie {
            if existing.tmdb_locked {
                record = existing;
            } else if !options.full_metadata {
                if candidate.is_some_and(|item| {
                    existing_file
                        .as_ref()
                        .is_some_and(|stored| !should_refresh_metadata(Some(stored), item))
                }) {
                    record = existing;
                }
            }
        }

        record.scanned_at = scanned_at;
        repo.upsert_movie(&library_id, &record, scanned_at, size_bytes, modified_secs)?;
    }

    let seen_paths: Vec<String> = result
        .candidates
        .iter()
        .map(|candidate| candidate.file.relative_path.clone())
        .collect();
    let _removed = repo.delete_orphans(&library_id, &seen_paths)?;

    let movies_count = repo.movie_count()?;
    info!(
        movies_count,
        elapsed_ms = started.elapsed().as_millis(),
        "library scan persisted"
    );

    Ok(ScanRunResult {
        scanned_at,
        movies_count,
        stats,
    })
}

/// Loads the in-memory catalog from SQLite.
pub fn load_catalog_from_db(repo: &LibraryRepository) -> NestResult<LoonCatalog> {
    Ok(catalog_from_records(repo.load_all()?))
}

fn existing_movie_for_path(
    repo: &LibraryRepository,
    relative_path: &str,
) -> NestResult<Option<LoonMovieRecord>> {
    Ok(match repo.get_file_by_path(relative_path)? {
        Some(file) => repo.get_by_media_id(&file.movie_id)?,
        None => None,
    })
}

fn should_enrich_metadata(
    options: ScanOptions,
    existing_file: Option<&StoredFile>,
    existing_movie: Option<&LoonMovieRecord>,
    candidate: &MovieScanCandidate,
) -> bool {
    if existing_movie.is_some_and(|movie| movie.tmdb_locked) {
        return false;
    }
    options.full_metadata || should_refresh_metadata(existing_file, candidate)
}

fn stored_tmdb_id(movie: Option<&LoonMovieRecord>) -> Option<u32> {
    let raw = movie?.tmdb_id.as_deref()?;
    let numeric = raw.strip_prefix("tmdb:").unwrap_or(raw).trim();
    numeric.parse().ok()
}

fn should_refresh_metadata(existing: Option<&StoredFile>, candidate: &MovieScanCandidate) -> bool {
    let Some(existing) = existing else {
        return true;
    };
    existing.size_bytes != candidate.file.size_bytes
        || existing.modified_secs != candidate.file.modified_secs
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

async fn apply_ai_filename_guess(ai: &AiRuntime, candidate: &mut MovieScanCandidate, path: &str) {
    if let Some(guess) = guess_movie_from_filename(ai, path).await {
        candidate.guessed_title = Some(guess.search_title);
        if guess.likely_year.is_some() {
            candidate.guessed_year = guess.likely_year;
        }
    }
}

#[cfg(test)]
mod tests {
    use nest_media_library::{MovieScanCandidate, ScanItemStatus, ScannedFile};

    use super::*;

    fn candidate(path: &str, size: u64, modified: Option<u64>) -> MovieScanCandidate {
        MovieScanCandidate {
            file: ScannedFile {
                relative_path: path.into(),
                size_bytes: size,
                modified_secs: modified,
            },
            guessed_title: None,
            guessed_year: None,
            inspection: None,
            metadata: None,
            status: ScanItemStatus::New,
        }
    }

    fn stored_file(size: u64, modified: Option<u64>) -> StoredFile {
        StoredFile {
            movie_id: "file:test.mp4".into(),
            relative_path: "test.mp4".into(),
            size_bytes: size,
            modified_secs: modified,
            scanned_at: 1,
        }
    }

    fn movie(tmdb_id: Option<&str>, locked: bool) -> LoonMovieRecord {
        LoonMovieRecord {
            media_id: "file:test.mp4".into(),
            slug: "test".into(),
            relative_path: "test.mp4".into(),
            title: "Test".into(),
            original_title: None,
            year: Some(2000),
            runtime_minutes: None,
            summary: None,
            genres: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            cast: Vec::new(),
            crew: Vec::new(),
            tmdb_id: tmdb_id.map(str::to_string),
            imdb_id: None,
            tmdb_locked: locked,
            scanned_at: 1,
            size_bytes: Some(100),
            modified_secs: Some(10),
            is_favorite: false,
            watch_progress_seconds: None,
            watch_duration_seconds: None,
        }
    }

    #[test]
    fn locked_movie_skips_metadata_refresh_even_on_full_scan() {
        let item = candidate("test.mp4", 100, Some(10));
        assert!(!should_enrich_metadata(
            ScanOptions {
                full_metadata: true,
            },
            Some(&stored_file(100, Some(10))),
            Some(&movie(Some("348"), true)),
            &item,
        ));
    }

    #[test]
    fn unchanged_file_skips_metadata_refresh() {
        let item = candidate("test.mp4", 100, Some(10));
        assert!(!should_enrich_metadata(
            ScanOptions::default(),
            Some(&stored_file(100, Some(10))),
            Some(&movie(Some("348"), false)),
            &item,
        ));
    }

    #[test]
    fn changed_file_refreshes_unlocked_movie() {
        let item = candidate("test.mp4", 200, Some(10));
        assert!(should_enrich_metadata(
            ScanOptions::default(),
            Some(&stored_file(100, Some(10))),
            Some(&movie(Some("348"), false)),
            &item,
        ));
    }

    #[test]
    fn parses_stored_tmdb_id() {
        assert_eq!(stored_tmdb_id(Some(&movie(Some("348"), false))), Some(348));
        assert_eq!(
            stored_tmdb_id(Some(&movie(Some("tmdb:411"), false))),
            Some(411)
        );
    }
}
