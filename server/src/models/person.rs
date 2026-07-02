//! Person API DTOs.

use serde::Serialize;

use crate::models::MovieSummary;

/// One library movie featuring a person.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct KnownForMovie {
    /// URL-safe movie identifier.
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Release year when known.
    pub year: Option<u16>,
    /// Poster proxy URL when available.
    pub poster_url: Option<String>,
    /// Character played in this movie.
    pub character: Option<String>,
}

/// Full person detail for the webOS actor screen.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PersonDetail {
    /// TMDB person id.
    pub tmdb_person_id: u32,
    /// Display name.
    pub name: String,
    /// Biography text.
    pub biography: Option<String>,
    /// ISO date string when known.
    pub birthday: Option<String>,
    /// ISO date string when known.
    pub deathday: Option<String>,
    /// Birth place label.
    pub place_of_birth: Option<String>,
    /// Profile image URL when known.
    pub profile_url: Option<String>,
    /// Primary department (e.g. Acting).
    pub known_for_department: Option<String>,
    /// TMDB gender code when known.
    pub gender: Option<i32>,
    /// Alternate names.
    pub also_known_as: Vec<String>,
    /// Movies in the local library featuring this person.
    pub known_for: Vec<KnownForMovie>,
}

impl KnownForMovie {
    /// Builds from a browse summary plus optional character name.
    pub fn from_summary(summary: MovieSummary, character: Option<String>) -> Self {
        Self {
            slug: summary.slug,
            title: summary.title,
            year: summary.year,
            poster_url: summary.poster_url,
            character,
        }
    }
}
