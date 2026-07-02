//! Library scan API DTOs.

use serde::{Deserialize, Serialize};

use crate::services::scan_state::ScanProgress;

/// `POST /api/library/scan` request body.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ScanStartRequest {
    /// When true, re-fetch TMDB metadata for every movie.
    pub full: bool,
}

/// `GET /api/library/status` response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LibraryStatusResponse {
    /// High-level state (`idle` or `scanning`).
    pub state: String,
    /// ISO-8601 timestamp of the last completed scan.
    pub last_scan_at: Option<String>,
    /// Duration of the last completed scan in seconds.
    pub last_scan_duration_secs: u64,
    /// Number of movies in the library.
    pub movies_count: usize,
    /// Whether a scan is currently running.
    pub scan_in_progress: bool,
    /// Live progress while scanning.
    pub progress: Option<ScanProgress>,
}

/// `GET /api/search` response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching movies.
    pub movies: Vec<super::MovieSummary>,
    /// Number of matches returned.
    pub total: usize,
}

/// `GET /api/genres` response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GenresResponse {
    /// Genres with movie counts.
    pub genres: Vec<GenreEntry>,
}

/// One genre row in the genres list.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GenreEntry {
    /// Genre name.
    pub name: String,
    /// Number of movies tagged with this genre.
    pub count: u32,
}

/// `PUT /api/movies/:slug/favorite` request.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct FavoriteRequest {
    /// Desired favorite state. Omit to toggle.
    pub favorite: Option<bool>,
}

/// `PUT /api/movies/:slug/favorite` response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FavoriteResponse {
    /// Movie slug.
    pub slug: String,
    /// Current favorite state.
    pub favorite: bool,
}

/// `PUT /api/movies/:slug/progress` request.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProgressRequest {
    /// Playback position in seconds.
    pub position_seconds: u32,
    /// Total duration in seconds when known.
    pub duration_seconds: Option<u32>,
}

/// `PUT /api/movies/:slug/progress` response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProgressResponse {
    /// Movie slug.
    pub slug: String,
    /// Saved position in seconds.
    pub position_seconds: u32,
    /// Duration associated with this update.
    pub duration_seconds: Option<u32>,
    /// ISO-8601 update timestamp.
    pub updated_at: String,
}
