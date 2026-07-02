//! TMDB metadata enrichment after filesystem discovery.

use std::collections::HashMap;
use std::path::Path;

use nest_media::{ExternalMediaId, MetadataProvider, MovieSearchQuery, MovieSearchResult};
use nest_media_library::{MovieScanCandidate, ScanResult};
use nest_tmdb::ImageSize;
use tracing::{debug, info, warn};

use crate::services::tmdb::TmdbRuntime;

/// Artwork URLs keyed by scanned file `relative_path`.
pub type ScanArtworkMap = HashMap<String, ScanArtwork>;

/// Poster and backdrop URL for one catalog movie.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanArtwork {
    /// HTTPS poster URL.
    pub poster_url: Option<String>,
    /// HTTPS backdrop URL.
    pub backdrop_url: Option<String>,
}

/// Enriches all scan candidates with TMDB metadata and artwork URLs.
pub async fn enrich_with_tmdb(result: &mut ScanResult, tmdb: &TmdbRuntime) -> ScanArtworkMap {
    let mut artwork = ScanArtworkMap::new();

    for candidate in &mut result.candidates {
        enrich_candidate(candidate, tmdb, &mut artwork).await;
    }

    let enriched = result
        .candidates
        .iter()
        .filter(|candidate| candidate.metadata.is_some())
        .count();
    info!(
        enriched,
        total = result.candidates.len(),
        "TMDB enrichment finished"
    );

    artwork
}

/// Enriches one candidate via TMDB search + fetch.
pub async fn enrich_candidate(
    candidate: &mut MovieScanCandidate,
    tmdb: &TmdbRuntime,
    artwork: &mut ScanArtworkMap,
) {
    let path = candidate.file.relative_path.clone();
    let year = candidate.guessed_year;

    let Some(results) = search_tmdb(tmdb, &search_title_variants(candidate)).await else {
        warn!(path = %path, "TMDB search returned no results");
        return;
    };

    let best = pick_best_search_result(&results, year);
    debug!(
        path = %path,
        picked = %best.title,
        picked_year = ?best.year,
        guessed_year = ?year,
        candidates = results.len(),
        "picked TMDB search result"
    );

    match tmdb.provider.fetch_movie(best.external_id.clone()).await {
        Ok(fetch) => {
            if let Some(urls) = artwork_urls(tmdb, &fetch.poster_path, &fetch.backdrop_path).await {
                artwork.insert(path.clone(), urls);
            }
            candidate.metadata = Some(fetch.metadata);
        }
        Err(error) => {
            warn!(
                path = %path,
                error = %error.message(),
                "TMDB metadata fetch failed"
            );
        }
    }
}

/// Fetches TMDB metadata for a known movie id (no title search).
pub async fn enrich_candidate_by_tmdb_id(
    candidate: &mut MovieScanCandidate,
    tmdb_id: u32,
    tmdb: &TmdbRuntime,
    artwork: &mut ScanArtworkMap,
) {
    let path = candidate.file.relative_path.clone();
    let external_id = ExternalMediaId::new(format!("tmdb:{tmdb_id}"));

    match tmdb.provider.fetch_movie(external_id).await {
        Ok(fetch) => {
            if let Some(urls) = artwork_urls(tmdb, &fetch.poster_path, &fetch.backdrop_path).await {
                artwork.insert(path.clone(), urls);
            }
            candidate.metadata = Some(fetch.metadata);
        }
        Err(error) => {
            warn!(
                path = %path,
                tmdb_id,
                error = %error.message(),
                "TMDB fetch by id failed"
            );
        }
    }
}

async fn search_tmdb(tmdb: &TmdbRuntime, titles: &[String]) -> Option<Vec<MovieSearchResult>> {
    for title in titles {
        let query = MovieSearchQuery::new(title.clone());
        if let Ok(results) = tmdb.provider.search_movie(query).await {
            if !results.is_empty() {
                return Some(results);
            }
        }
    }

    None
}

/// Picks the best TMDB hit using guessed year for disambiguation only.
///
/// TMDB's `year` search parameter is a strict release-date filter (not a ranking
/// hint). It often returns zero results when the title is slightly off, and it
/// can exclude the theatrical release when secondary dates differ. We search by
/// title only and use the AI-guessed year locally to break ties.
fn pick_best_search_result(
    results: &[MovieSearchResult],
    guessed_year: Option<u16>,
) -> &MovieSearchResult {
    let Some(guessed_year) = guessed_year else {
        return &results[0];
    };

    results
        .iter()
        .enumerate()
        .min_by_key(|(index, result)| (year_distance(result.year, guessed_year), *index))
        .map(|(_, result)| result)
        .unwrap_or(&results[0])
}

fn year_distance(result_year: Option<u16>, guessed_year: u16) -> u32 {
    match result_year {
        Some(year) => year.abs_diff(guessed_year) as u32,
        None => u32::MAX,
    }
}

async fn artwork_urls(
    tmdb: &TmdbRuntime,
    poster_path: &Option<String>,
    backdrop_path: &Option<String>,
) -> Option<ScanArtwork> {
    let poster_url = match poster_path.as_deref().filter(|path| !path.is_empty()) {
        Some(path) => Some(tmdb.images.poster_url(path, ImageSize::W500).await),
        None => None,
    };
    let backdrop_url = match backdrop_path.as_deref().filter(|path| !path.is_empty()) {
        Some(path) => Some(tmdb.images.backdrop_url(path, ImageSize::W1280).await),
        None => None,
    };

    if poster_url.is_none() && backdrop_url.is_none() {
        return None;
    }

    Some(ScanArtwork {
        poster_url,
        backdrop_url,
    })
}

/// Fetches poster and backdrop URLs for TMDB path tokens.
pub async fn artwork_for_paths(
    tmdb: &TmdbRuntime,
    poster_path: &Option<String>,
    backdrop_path: &Option<String>,
) -> Option<ScanArtwork> {
    artwork_urls(tmdb, poster_path, backdrop_path).await
}

/// Returns ordered unique title variants to try against TMDB.
pub fn search_title_variants(candidate: &MovieScanCandidate) -> Vec<String> {
    let mut variants: Vec<String> = Vec::new();

    if let Some(title) = candidate.guessed_title.as_deref() {
        push_variant(&mut variants, normalize_search_title(title));
        if let Some(spaced) = split_concatenated_title(title) {
            push_variant(&mut variants, spaced);
        }
        push_variant(&mut variants, split_camel_case_title(title));
    }

    if let Some(stem) = Path::new(&candidate.file.relative_path)
        .file_stem()
        .and_then(|name| name.to_str())
    {
        let from_path = normalize_search_title(&stem.replace(['_', '.'], " "));
        push_variant(&mut variants, from_path);
    }

    if variants.is_empty() {
        push_variant(&mut variants, candidate.file.relative_path.clone());
    }

    variants
}

fn push_variant(variants: &mut Vec<String>, value: String) {
    let value = value.trim().to_string();
    if value.is_empty() {
        return;
    }
    if variants
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&value))
    {
        return;
    }
    variants.push(value);
}

/// Returns the primary title used for TMDB search.
pub fn search_title(candidate: &MovieScanCandidate) -> String {
    search_title_variants(candidate)
        .into_iter()
        .next()
        .unwrap_or_else(|| candidate.file.relative_path.clone())
}

fn normalize_search_title(title: &str) -> String {
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn split_camel_case_title(title: &str) -> String {
    let chars: Vec<char> = title.chars().collect();
    let mut result = String::new();

    for (index, ch) in chars.iter().enumerate() {
        if index > 0 {
            let previous = chars[index - 1];
            let next = chars.get(index + 1).copied();
            let split_before = previous.is_ascii_lowercase() && ch.is_ascii_uppercase()
                || previous.is_ascii_uppercase()
                    && ch.is_ascii_uppercase()
                    && next.is_some_and(|next| next.is_ascii_lowercase());
            if split_before {
                result.push(' ');
            }
        }
        result.push(*ch);
    }

    normalize_search_title(&result)
}

fn split_concatenated_title(title: &str) -> Option<String> {
    if title.contains(' ') {
        return None;
    }

    let lower = title.to_ascii_lowercase();
    const SUFFIXES: [&str; 12] = [
        "management",
        "movie",
        "story",
        "night",
        "wars",
        "men",
        "woman",
        "dragon",
        "king",
        "queen",
        "part",
        "chapter",
    ];

    for suffix in SUFFIXES {
        if let Some(pos) = lower.find(suffix) {
            if pos > 0 {
                let (left, right) = title.split_at(pos);
                return Some(normalize_search_title(&format!("{left} {right}")));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use nest_media_library::{MovieScanCandidate, ScanItemStatus, ScannedFile};

    use super::*;

    fn candidate(path: &str, title: Option<&str>, year: Option<u16>) -> MovieScanCandidate {
        MovieScanCandidate {
            file: ScannedFile {
                relative_path: path.into(),
                size_bytes: 1,
                modified_secs: None,
            },
            guessed_title: title.map(str::to_string),
            guessed_year: year,
            inspection: None,
            metadata: None,
            status: ScanItemStatus::New,
        }
    }

    #[test]
    fn splits_concatenated_management_title() {
        assert_eq!(
            split_concatenated_title("Angermanagement").as_deref(),
            Some("Anger management")
        );
    }

    #[test]
    fn splits_camel_case_title() {
        assert_eq!(split_camel_case_title("BladeRunner"), "Blade Runner");
    }

    #[test]
    fn search_title_variants_includes_spaced_alternate() {
        let variants = search_title_variants(&candidate(
            "angermanagment2003.mp4",
            Some("Angermanagement"),
            Some(2003),
        ));
        assert!(variants.iter().any(|title| title.contains(' ')));
    }

    #[test]
    fn pick_best_prefers_closest_year() {
        let results = vec![
            MovieSearchResult {
                external_id: nest_media::ExternalMediaId::new("tmdb:1"),
                title: "Blade Runner 2049".into(),
                year: Some(2017),
                summary: None,
            },
            MovieSearchResult {
                external_id: nest_media::ExternalMediaId::new("tmdb:2"),
                title: "Blade Runner".into(),
                year: Some(1982),
                summary: None,
            },
        ];
        let best = pick_best_search_result(&results, Some(1982));
        assert_eq!(best.title, "Blade Runner");
    }

    #[test]
    fn pick_best_falls_back_to_first_when_no_year() {
        let results = vec![
            MovieSearchResult {
                external_id: nest_media::ExternalMediaId::new("tmdb:1"),
                title: "First".into(),
                year: Some(2000),
                summary: None,
            },
            MovieSearchResult {
                external_id: nest_media::ExternalMediaId::new("tmdb:2"),
                title: "Second".into(),
                year: Some(1999),
                summary: None,
            },
        ];
        let best = pick_best_search_result(&results, None);
        assert_eq!(best.title, "First");
    }
}
