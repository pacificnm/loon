//! Manual TMDB id rematch for mis-identified movies.

use nest_error::{NestError, NestResult};
use nest_media::ExternalMediaId;
use nest_http_serve::ServeError;

use crate::db::LibraryRepository;
use crate::error::invalid_request;
use crate::services::artwork::ArtworkRuntime;
use crate::services::catalog::{apply_tmdb_fetch, LoonMovieRecord};
use crate::services::enrichment::artwork_for_paths;
use crate::services::tmdb::TmdbRuntime;

/// Parses a TMDB movie id from user input (`348`, `tmdb:348`, etc.).
pub fn parse_tmdb_id(raw: &str) -> Result<u32, ServeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(invalid_request("tmdb_id is required"));
    }

    let numeric = trimmed
        .strip_prefix("tmdb:")
        .unwrap_or(trimmed)
        .trim();

    numeric
        .parse::<u32>()
        .map_err(|_| invalid_request("tmdb_id must be a numeric TMDB movie id"))
}

/// Fetches TMDB metadata for `tmdb_id` and updates the movie in SQLite.
pub async fn rematch_movie_by_tmdb_id(
    slug: &str,
    tmdb_id: u32,
    repo: &LibraryRepository,
    tmdb: &TmdbRuntime,
    artwork_cache: Option<&ArtworkRuntime>,
    library_id: &str,
) -> NestResult<LoonMovieRecord> {
    let mut record = repo
        .get_by_slug(slug)?
        .ok_or_else(|| NestError::data(format!("movie '{slug}' not found")))?;

    let stored = repo.get_file_by_path(&record.relative_path)?;
    let external_id = ExternalMediaId::new(format!("tmdb:{tmdb_id}"));
    let fetch = tmdb.provider.fetch_movie(external_id).await.map_err(|error| {
        NestError::service(format!("TMDB fetch failed: {}", error.message()))
    })?;

    let artwork =
        artwork_for_paths(tmdb, &fetch.poster_path, &fetch.backdrop_path).await;
    if let Some(cache) = artwork_cache {
        let _ = cache.invalidate_movie(slug);
    }

    apply_tmdb_fetch(&mut record, tmdb_id, &fetch.metadata, artwork.as_ref());
    record.tmdb_locked = true;

    let size_bytes = stored
        .as_ref()
        .map(|file| file.size_bytes)
        .or(record.size_bytes)
        .unwrap_or(0);
    let modified_secs = stored
        .as_ref()
        .and_then(|file| file.modified_secs)
        .or(record.modified_secs);

    repo.upsert_movie(
        library_id,
        &record,
        record.scanned_at,
        size_bytes,
        modified_secs,
    )?;

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_tmdb_id() {
        assert_eq!(parse_tmdb_id("348").unwrap(), 348);
    }

    #[test]
    fn parses_prefixed_tmdb_id() {
        assert_eq!(parse_tmdb_id("tmdb:411").unwrap(), 411);
    }

    #[test]
    fn rejects_empty_tmdb_id() {
        assert!(parse_tmdb_id("  ").is_err());
    }
}
