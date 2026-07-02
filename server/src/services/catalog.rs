//! In-memory movie catalog built from library scans.

use std::collections::HashMap;
use std::path::Path;

use nest_media::{MovieMetadata, PersonCredit};
use nest_media_library::{MovieScanCandidate, ScanResult};
use tracing::warn;

use crate::models::{CastMemberDto, CrewMemberDto, MovieDetail, MovieSummary};
use crate::services::artwork::{ArtworkKind, ArtworkRuntime};
use crate::services::enrichment::ScanArtworkMap;
use crate::services::slug::unique_movie_slug;

/// One playable movie in Loon's catalog.
#[derive(Debug, Clone)]
pub struct LoonMovieRecord {
    /// Stable Nest media id (typically `file:{relative_path}`).
    pub media_id: String,
    /// URL-safe movie identifier.
    pub slug: String,
    /// Path relative to `media_root`.
    pub relative_path: String,
    /// Display title.
    pub title: String,
    /// Original release title when known.
    pub original_title: Option<String>,
    /// Release year when known.
    pub year: Option<u16>,
    /// Runtime in minutes when known.
    pub runtime_minutes: Option<u16>,
    /// Plot summary when known.
    pub summary: Option<String>,
    /// Genre names when enriched.
    pub genres: Vec<String>,
    /// Poster image URL when enriched.
    pub poster_url: Option<String>,
    /// Backdrop image URL when enriched.
    pub backdrop_url: Option<String>,
    /// Cast credits when enriched.
    pub cast: Vec<CastMemberDto>,
    /// Crew credits when enriched.
    pub crew: Vec<CrewMemberDto>,
    /// TMDB movie id when known.
    pub tmdb_id: Option<String>,
    /// IMDb id when known.
    pub imdb_id: Option<String>,
    /// Unix timestamp when this file was last scanned.
    pub scanned_at: u64,
    /// Whether the user marked this movie as a favorite.
    pub is_favorite: bool,
    /// Last known watch position in seconds.
    pub watch_progress_seconds: Option<u32>,
    /// Duration associated with the last progress update.
    pub watch_duration_seconds: Option<u32>,
}

/// In-memory catalog indexed by slug.
#[derive(Debug, Clone, Default)]
pub struct LoonCatalog {
    by_slug: HashMap<String, LoonMovieRecord>,
}

impl LoonCatalog {
    /// Creates an empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of movies in the catalog.
    pub fn len(&self) -> usize {
        self.by_slug.len()
    }

    /// Returns whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.by_slug.is_empty()
    }

    /// Returns all movies sorted by title as API summaries.
    pub fn list(&self) -> Vec<MovieSummary> {
        let mut movies: Vec<_> = self.by_slug.values().map(record_to_summary).collect();
        movies.sort_by(|left, right| left.title.cmp(&right.title));
        movies
    }

    /// Looks up a movie by slug.
    pub fn get(&self, slug: &str) -> Option<&LoonMovieRecord> {
        self.by_slug.get(slug)
    }

    /// Looks up a mutable movie by slug.
    pub fn get_mut(&mut self, slug: &str) -> Option<&mut LoonMovieRecord> {
        self.by_slug.get_mut(slug)
    }

    /// Returns all movie records in the catalog.
    pub fn records(&self) -> Vec<LoonMovieRecord> {
        self.by_slug.values().cloned().collect()
    }

    /// Replaces the catalog contents.
    pub fn replace(&mut self, records: Vec<LoonMovieRecord>) {
        self.by_slug.clear();
        for record in records {
            self.insert(record);
        }
    }

    /// Inserts a movie record, replacing any existing entry with the same slug.
    pub fn insert(&mut self, record: LoonMovieRecord) {
        self.by_slug.insert(record.slug.clone(), record);
    }
}

impl LoonMovieRecord {
    /// Converts to a browse-grid summary DTO.
    pub fn to_summary(&self) -> MovieSummary {
        record_to_summary(self)
    }

    /// Converts to a detail DTO including the stream URL.
    pub fn to_detail(&self) -> MovieDetail {
        MovieDetail {
            slug: self.slug.clone(),
            title: self.title.clone(),
            original_title: self.original_title.clone(),
            year: self.year,
            runtime_minutes: self.runtime_minutes,
            summary: self.summary.clone(),
            genres: self.genres.clone(),
            poster_url: ArtworkRuntime::proxy_url(
                &self.slug,
                ArtworkKind::Poster,
                &self.poster_url,
            ),
            backdrop_url: ArtworkRuntime::proxy_url(
                &self.slug,
                ArtworkKind::Backdrop,
                &self.backdrop_url,
            ),
            cast: self.cast.clone(),
            crew: self.crew.clone(),
            is_favorite: self.is_favorite,
            watch_progress_seconds: self.watch_progress_seconds,
            stream_url: format!("/stream/{}", self.slug),
        }
    }
}

/// Builds a catalog from persisted movie records.
pub fn catalog_from_records(records: Vec<LoonMovieRecord>) -> LoonCatalog {
    let mut catalog = LoonCatalog::new();
    for record in records {
        catalog.insert(record);
    }
    catalog
}

/// Builds a catalog from a library scan result.
///
/// Every discovered video file becomes a catalog entry. The only exclusion is
/// path traversal in `relative_path` (security).
pub fn catalog_from_scan(result: ScanResult, artwork: &ScanArtworkMap) -> LoonCatalog {
    let mut catalog = LoonCatalog::new();

    for candidate in result.candidates {
        if candidate.file.relative_path.contains("..") {
            warn!(
                path = %candidate.file.relative_path,
                "rejecting candidate with path traversal"
            );
            continue;
        }

        let art = artwork.get(&candidate.file.relative_path);
        let record = record_from_candidate(&candidate, art, &catalog.by_slug);
        catalog.insert(record);
    }

    catalog
}

fn record_from_candidate(
    candidate: &MovieScanCandidate,
    artwork: Option<&crate::services::enrichment::ScanArtwork>,
    existing: &HashMap<String, LoonMovieRecord>,
) -> LoonMovieRecord {
    let existing_keys: HashMap<String, ()> = existing.keys().cloned().map(|k| (k, ())).collect();

    if let Some(metadata) = candidate.metadata.as_ref() {
        return record_from_metadata(candidate, metadata, artwork, &existing_keys);
    }

    let title = display_title(candidate);
    let year = candidate.guessed_year;
    let slug = unique_movie_slug(&title, year, &candidate.file.relative_path, &existing_keys);

    LoonMovieRecord {
        media_id: media_id_for_path(&candidate.file.relative_path),
        slug,
        relative_path: candidate.file.relative_path.clone(),
        title,
        original_title: None,
        year,
        runtime_minutes: None,
        summary: None,
        genres: Vec::new(),
        poster_url: artwork.and_then(|art| art.poster_url.clone()),
        backdrop_url: artwork.and_then(|art| art.backdrop_url.clone()),
        cast: Vec::new(),
        crew: Vec::new(),
        tmdb_id: None,
        imdb_id: None,
        scanned_at: 0,
        is_favorite: false,
        watch_progress_seconds: None,
        watch_duration_seconds: None,
    }
}

fn record_from_metadata(
    candidate: &MovieScanCandidate,
    metadata: &MovieMetadata,
    artwork: Option<&crate::services::enrichment::ScanArtwork>,
    existing: &HashMap<String, ()>,
) -> LoonMovieRecord {
    let title = metadata.title.clone();
    let year = metadata.year.or(candidate.guessed_year);
    let slug = unique_movie_slug(&title, year, &candidate.file.relative_path, existing);

    LoonMovieRecord {
        media_id: media_id_for_path(&candidate.file.relative_path),
        slug,
        relative_path: candidate.file.relative_path.clone(),
        title,
        original_title: metadata.original_title.clone(),
        year,
        runtime_minutes: metadata
            .runtime_seconds
            .map(|seconds| (seconds / 60) as u16),
        summary: metadata.summary.clone(),
        genres: metadata.genres.clone(),
        poster_url: artwork.and_then(|art| art.poster_url.clone()),
        backdrop_url: artwork.and_then(|art| art.backdrop_url.clone()),
        cast: map_cast(&metadata.cast),
        crew: map_crew(&metadata.crew),
        tmdb_id: metadata.external_ids.tmdb_id.clone(),
        imdb_id: metadata.external_ids.imdb_id.clone(),
        scanned_at: 0,
        is_favorite: false,
        watch_progress_seconds: None,
        watch_duration_seconds: None,
    }
}

fn map_cast(credits: &[PersonCredit]) -> Vec<CastMemberDto> {
    credits
        .iter()
        .map(|credit| CastMemberDto {
            name: credit.name.clone(),
            character: credit.character.clone(),
        })
        .collect()
}

fn map_crew(credits: &[PersonCredit]) -> Vec<CrewMemberDto> {
    credits
        .iter()
        .map(|credit| CrewMemberDto {
            name: credit.name.clone(),
            job: Some(credit.role.clone()),
        })
        .collect()
}

fn display_title(candidate: &MovieScanCandidate) -> String {
    if let Some(title) = candidate
        .guessed_title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
    {
        return title.to_string();
    }

    Path::new(&candidate.file.relative_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .map(slug_stem_to_title)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| candidate.file.relative_path.clone())
}

fn slug_stem_to_title(stem: &str) -> String {
    stem.replace(['_', '.'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn media_id_for_path(relative_path: &str) -> String {
    format!("file:{relative_path}")
}

fn record_to_summary(record: &LoonMovieRecord) -> MovieSummary {
    MovieSummary {
        slug: record.slug.clone(),
        title: record.title.clone(),
        year: record.year,
        runtime_minutes: record.runtime_minutes.unwrap_or(0),
        poster_url: ArtworkRuntime::proxy_url(
            &record.slug,
            ArtworkKind::Poster,
            &record.poster_url,
        ),
        backdrop_url: ArtworkRuntime::proxy_url(
            &record.slug,
            ArtworkKind::Backdrop,
            &record.backdrop_url,
        ),
        summary: record.summary.clone().unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use nest_media::{ExternalIds, ExternalMediaId, MediaTracks};
    use nest_media_library::{ScanItemStatus, ScannedFile};

    use super::*;
    use crate::services::enrichment::ScanArtwork;

    fn candidate(path: &str, title: Option<&str>, year: Option<u16>) -> MovieScanCandidate {
        MovieScanCandidate {
            file: ScannedFile {
                relative_path: path.into(),
                size_bytes: 1024,
                modified_secs: None,
            },
            guessed_title: title.map(str::to_string),
            guessed_year: year,
            inspection: None,
            metadata: None,
            status: ScanItemStatus::New,
        }
    }

    fn metadata(title: &str, year: u16) -> MovieMetadata {
        MovieMetadata {
            external_id: ExternalMediaId::new("tmdb:1"),
            title: title.into(),
            original_title: Some(title.into()),
            sort_title: None,
            year: Some(year),
            runtime_seconds: Some(117 * 60),
            rating: None,
            summary: Some("In space no one can hear you scream.".into()),
            genres: vec!["Horror".into()],
            cast: vec![nest_media::PersonCredit::new(
                "Sigourney Weaver",
                "Actor",
                Some("Ripley".into()),
            )],
            crew: vec![nest_media::PersonCredit::new(
                "Ridley Scott",
                "Director",
                None,
            )],
            tracks: MediaTracks::new(),
            external_ids: ExternalIds::new(),
        }
    }

    #[test]
    fn builds_record_from_candidate_with_year() {
        let record = record_from_candidate(
            &candidate(
                "Movies/Alien (1979)/Alien (1979).mp4",
                Some("Alien"),
                Some(1979),
            ),
            None,
            &HashMap::new(),
        );
        assert_eq!(record.slug, "alien-1979");
    }

    #[test]
    fn includes_file_without_year() {
        let record = record_from_candidate(
            &candidate(
                "the_chronicles_of_narnia_the_lion_the_witch_and_the_wardrobe.mp4",
                Some("the chronicles of narnia the lion the witch and the wardrobe"),
                None,
            ),
            None,
            &HashMap::new(),
        );
        assert_eq!(record.year, None);
        assert!(record.slug.contains("chronicles"));
        assert!(record.title.contains("narnia"));
    }

    #[test]
    fn falls_back_to_filename_when_title_missing() {
        let record = record_from_candidate(
            &candidate("Movies/standalone.mp4", None, None),
            None,
            &HashMap::new(),
        );
        assert_eq!(record.title, "standalone");
        assert_eq!(record.slug, "standalone");
    }

    #[test]
    fn catalog_includes_every_candidate() {
        let mut result = ScanResult {
            library_id: nest_media_library::LibraryId::new("main"),
            started_at: 0,
            finished_at: 0,
            candidates: vec![
                candidate("a.mp4", Some("Alpha"), None),
                candidate("b.mp4", None, None),
            ],
            errors: Vec::new(),
            stats: Default::default(),
        };
        result.finished_at = 1;

        let catalog = catalog_from_scan(result, &ScanArtworkMap::new());
        assert_eq!(catalog.len(), 2);
    }

    #[test]
    fn uses_tmdb_metadata_for_title_slug_and_artwork() {
        let path = "Movies/Alien (1979)/Alien (1979).mp4";
        let mut candidate = candidate(path, Some("Alien"), Some(1979));
        candidate.metadata = Some(metadata("Alien", 1979));

        let mut artwork = ScanArtworkMap::new();
        artwork.insert(
            path.into(),
            ScanArtwork {
                poster_url: Some("https://image.tmdb.org/t/p/w500/poster.jpg".into()),
                backdrop_url: Some("https://image.tmdb.org/t/p/w1280/backdrop.jpg".into()),
            },
        );

        let record = record_from_candidate(&candidate, artwork.get(path), &HashMap::new());
        assert_eq!(record.slug, "alien-1979");
        assert_eq!(record.title, "Alien");
        assert_eq!(
            record.summary.as_deref(),
            Some("In space no one can hear you scream.")
        );
        assert_eq!(record.genres, vec!["Horror"]);
        assert_eq!(record.cast.len(), 1);
        assert_eq!(record.crew[0].job.as_deref(), Some("Director"));
        assert!(record.poster_url.is_some());
    }
}
