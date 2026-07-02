//! Netflix-style browse feed builder.

use nest_error::NestResult;

use crate::db::LibraryRepository;
use crate::models::{BrowseResponse, BrowseRow, MovieSummary};
use crate::services::catalog::LoonMovieRecord;

const ROW_LIMIT: u32 = 20;
const MIN_GENRE_MOVIES: u32 = 3;

/// Builds the home browse feed from SQLite.
pub fn build_browse(repo: &LibraryRepository) -> NestResult<BrowseResponse> {
    let mut rows = Vec::new();

    let continue_watching = records_to_summaries(&repo.list_continue_watching(ROW_LIMIT)?);
    if !continue_watching.is_empty() {
        rows.push(BrowseRow {
            slug: "continue-watching".into(),
            title: "Continue Watching".into(),
            row_type: "continue_watching".into(),
            movies: continue_watching,
        });
    }

    let recently_added = records_to_summaries(&repo.list_recently_added(ROW_LIMIT)?);
    let hero = select_hero(&recently_added);
    if !recently_added.is_empty() {
        rows.push(BrowseRow {
            slug: "recently-added".into(),
            title: "Recently Added".into(),
            row_type: "recently_added".into(),
            movies: recently_added,
        });
    }

    let favorites = records_to_summaries(&repo.list_favorites(ROW_LIMIT)?);
    if !favorites.is_empty() {
        rows.push(BrowseRow {
            slug: "favorites".into(),
            title: "Favorites".into(),
            row_type: "favorites".into(),
            movies: favorites,
        });
    }

    for genre in repo.list_genres()? {
        if genre.count < MIN_GENRE_MOVIES {
            continue;
        }
        let movies = records_to_summaries(&repo.list_by_genre(&genre.name, ROW_LIMIT)?);
        if movies.is_empty() {
            continue;
        }
        rows.push(BrowseRow {
            slug: format!("genre-{}", slugify(&genre.name)),
            title: genre.name.clone(),
            row_type: "genre".into(),
            movies,
        });
    }

    Ok(BrowseResponse { hero, rows })
}

fn select_hero(recently_added: &[MovieSummary]) -> Option<MovieSummary> {
    recently_added
        .iter()
        .find(|movie| movie.backdrop_url.is_some())
        .or_else(|| recently_added.first())
        .cloned()
}

fn records_to_summaries(records: &[LoonMovieRecord]) -> Vec<MovieSummary> {
    records.iter().map(LoonMovieRecord::to_summary).collect()
}

fn slugify(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
