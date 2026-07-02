//! Backfill TMDB person ids on cast members from movie credits.

use nest_error::NestResult;
use nest_media::ExternalMediaId;
use nest_tmdb::{ImageSize, TmdbImageService, DEFAULT_IMAGE_BASE_URL};

use crate::services::catalog::LoonMovieRecord;
use crate::services::tmdb::TmdbRuntime;

/// Fills missing `tmdb_person_id` values from TMDB movie credits.
pub async fn backfill_cast_person_ids(
    record: &mut LoonMovieRecord,
    tmdb: &TmdbRuntime,
) -> NestResult<bool> {
    if record.cast.iter().all(|member| member.tmdb_person_id.is_some()) {
        return Ok(false);
    }

    let Some(tmdb_id) = record
        .tmdb_id
        .as_deref()
        .and_then(parse_tmdb_movie_id)
    else {
        return Ok(false);
    };

    let external_id = ExternalMediaId::new(format!("tmdb:{tmdb_id}"));
    let fetch = tmdb
        .provider
        .fetch_movie(external_id)
        .await
        .map_err(|error| {
            nest_error::NestError::service(format!(
                "TMDB credits fetch failed: {}",
                error.message()
            ))
        })?;

    let mut changed = false;
    for member in &mut record.cast {
        if member.tmdb_person_id.is_some() {
            continue;
        }
        let Some(credit) = fetch
            .metadata
            .cast
            .iter()
            .find(|credit| credit.name.eq_ignore_ascii_case(&member.name))
        else {
            continue;
        };
        if let Some(person_id) = credit.tmdb_person_id {
            member.tmdb_person_id = Some(person_id);
            changed = true;
        }
        if member.profile_url.is_none() {
            if let Some(path) = credit.profile_path.as_deref().filter(|path| !path.is_empty()) {
                member.profile_url = Some(TmdbImageService::profile_url_with_base(
                    DEFAULT_IMAGE_BASE_URL,
                    path,
                    ImageSize::W185,
                ));
                changed = true;
            }
        }
    }

    Ok(changed)
}

fn parse_tmdb_movie_id(raw: &str) -> Option<u32> {
    raw.strip_prefix("tmdb:")
        .unwrap_or(raw)
        .trim()
        .parse()
        .ok()
}
