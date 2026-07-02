//! Loon server API types (mirrors webOS client).

#![allow(dead_code)]

use serde::Deserialize;

/// Movie card in browse/list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct MovieSummary {
    /// URL-safe movie identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Release year when known.
    pub year: Option<u16>,
    /// Runtime in minutes.
    pub runtime_minutes: u16,
    /// Poster image URL when available.
    pub poster_url: Option<String>,
    /// Backdrop image URL when available.
    pub backdrop_url: Option<String>,
    /// Short plot summary.
    pub summary: String,
}

/// `GET /api/movies` response body.
#[derive(Debug, Clone, Deserialize)]
pub struct MovieListResponse {
    /// Movies in this page.
    pub movies: Vec<MovieSummary>,
    /// Total movies matching the query.
    pub total: usize,
}

/// On-disk media file metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct MovieFileInfo {
    /// Base file name.
    pub filename: String,
    /// Path relative to the library root.
    pub relative_path: String,
}

/// Full movie detail for admin editing.
#[derive(Debug, Clone, Deserialize)]
pub struct MovieDetail {
    /// URL-safe movie identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Original title when different from display title.
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
    /// Whether the movie is marked as a favorite.
    pub is_favorite: bool,
    /// TMDB movie id when known.
    pub tmdb_id: Option<String>,
    /// IMDb id when known.
    pub imdb_id: Option<String>,
    /// On-disk media file metadata.
    pub file: MovieFileInfo,
}

/// `GET /api/health` response body.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    /// Service status string.
    pub status: String,
    /// Number of movies in the catalog.
    pub movies_count: usize,
}
