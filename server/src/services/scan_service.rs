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
use crate::services::catalog::{catalog_from_records, catalog_from_scan, LoonCatalog};
use crate::services::enrichment::{enrich_candidate, ScanArtworkMap};
use crate::services::filename_guess::guess_movie_from_filename;
use crate::services::library::discover_library;
use crate::services::scan_events::{scan_progress, ScanReporter};
use crate::services::scan_state::ScanPhase;
use crate::services::tmdb::TmdbRuntime;

/// Options controlling scan behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScanOptions {
    /// Re-fetch TMDB metadata for every movie.
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
        let needs_tmdb =
            options.full_metadata || should_refresh_metadata(existing_file.as_ref(), candidate);
        if needs_tmdb {
            refreshed_paths.insert(path);
        }
    }

    let total_to_enrich = refreshed_paths.len() as u32;
    let mut enriched = 0u32;

    for candidate in &mut result.candidates {
        let path = candidate.file.relative_path.clone();
        let existing_file = repo.get_file_by_path(&path)?;
        let needs_tmdb =
            options.full_metadata || should_refresh_metadata(existing_file.as_ref(), candidate);

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

            if let Some(ai) = ai {
                apply_ai_filename_guess(ai, candidate, &path).await;
            }
            if let Some(tmdb) = tmdb {
                enrich_candidate(candidate, tmdb, &mut artwork_map).await;
            }
            enriched += 1;
        } else if let Some(existing_file) = existing_file {
            if let Some(existing_movie) = repo.get_by_media_id(&existing_file.movie_id)? {
                candidate.guessed_title = Some(existing_movie.title.clone());
                candidate.guessed_year = existing_movie.year;
            }
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

        if !options.full_metadata {
            if let Some(stored) = repo.get_file_by_path(&record.relative_path)? {
                if candidate.is_some_and(|item| !should_refresh_metadata(Some(&stored), item)) {
                    if let Some(existing_movie) = repo.get_by_media_id(&stored.movie_id)? {
                        record = existing_movie;
                    }
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
