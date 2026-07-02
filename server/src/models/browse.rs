//! Browse feed DTOs.

use serde::Serialize;

use super::MovieSummary;

/// Netflix-style home feed.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BrowseResponse {
    /// Featured hero movie when available.
    pub hero: Option<MovieSummary>,
    /// Ordered browse rows.
    pub rows: Vec<BrowseRow>,
}

/// One horizontal browse row.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BrowseRow {
    /// Stable row identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Row kind for client layout.
    pub row_type: String,
    /// Movies in this row.
    pub movies: Vec<MovieSummary>,
}
