//! Movie API DTOs.

use serde::{Deserialize, Serialize};

/// Health check payload.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: &'static str,
    /// Service name.
    pub service: &'static str,
    /// Server version.
    pub version: &'static str,
    /// Number of movies in the catalog.
    pub movies_count: usize,
    /// Unix timestamp of the last library scan.
    pub library_scanned_at: u64,
}

/// Movie list response.
#[derive(Debug, Clone, Serialize)]
pub struct MovieListResponse {
    /// Movies in this page.
    pub movies: Vec<MovieSummary>,
    /// Total movies matching the query.
    pub total: usize,
    /// Current page when paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Page size when paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Total number of pages when paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<u32>,
}

/// Movie card shown in browse grids.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MovieSummary {
    /// URL-safe movie identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Release year when known.
    pub year: Option<u16>,
    /// Runtime in minutes.
    pub runtime_minutes: u16,
    /// Poster image URL (HTTPS), if available.
    pub poster_url: Option<String>,
    /// Backdrop image URL (HTTPS), if available.
    pub backdrop_url: Option<String>,
    /// Short plot summary.
    pub summary: String,
}

/// Full movie detail for the webOS detail screen.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MovieDetail {
    /// URL-safe movie identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Original title when different from the display title.
    pub original_title: Option<String>,
    /// Release year.
    pub year: Option<u16>,
    /// Runtime in minutes.
    pub runtime_minutes: Option<u16>,
    /// Plot summary.
    pub summary: Option<String>,
    /// Genre names.
    pub genres: Vec<String>,
    /// Poster image URL.
    pub poster_url: Option<String>,
    /// Backdrop image URL.
    pub backdrop_url: Option<String>,
    /// Cast members.
    pub cast: Vec<CastMemberDto>,
    /// Crew members.
    pub crew: Vec<CrewMemberDto>,
    /// Whether the movie is marked as a favorite.
    pub is_favorite: bool,
    /// Last known watch position in seconds.
    pub watch_progress_seconds: Option<u32>,
    /// Relative stream URL for playback.
    pub stream_url: String,
}

/// Cast member in a movie detail response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CastMemberDto {
    /// Person name.
    pub name: String,
    /// Character name.
    pub character: Option<String>,
}

/// Crew member in a movie detail response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrewMemberDto {
    /// Person name.
    pub name: String,
    /// Job title.
    pub job: Option<String>,
}
