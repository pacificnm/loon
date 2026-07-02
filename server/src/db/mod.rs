//! SQLite persistence for the movie library.

mod migrations;
mod repository;

pub use migrations::loon_migrations;
pub use repository::{
    GenreCount, LibraryRepository, MovieListQuery, MovieSort, StoredFile, WatchProgress,
};

use std::path::Path;

use nest_data_sqlite::{migration::apply_migrations, SqliteConfig, SqliteConnection};
use nest_error::{NestError, NestResult};

/// Opens the Loon SQLite database and applies pending migrations.
pub fn open_database(path: &Path) -> NestResult<LibraryRepository> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            NestError::io(format!(
                "failed to create database directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    let conn = SqliteConnection::open(&SqliteConfig::file(path)).map_err(map_data_error)?;
    apply_migrations(&conn, &loon_migrations()).map_err(map_data_error)?;
    Ok(LibraryRepository::new(conn))
}

fn map_data_error(error: nest_data::DataError) -> NestError {
    NestError::data(error.message()).with_source(error)
}
