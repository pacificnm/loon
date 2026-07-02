//! Manual TMDB match handler.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::error::{invalid_request, movie_not_found, tmdb_not_configured};
use crate::models::MatchRequest;
use crate::services::tmdb_match::{parse_tmdb_id, rematch_movie_by_tmdb_id};
use crate::state;

/// `PUT /api/movies/:slug/match`
pub async fn set_tmdb_match(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let body: MatchRequest = ctx
        .json()
        .map_err(|_| invalid_request("invalid JSON body"))?;
    let tmdb_id = parse_tmdb_id(&body.tmdb_id)?;

    let app = state::app_state();
    let Some(tmdb) = app.tmdb.as_ref() else {
        return Err(tmdb_not_configured());
    };

    if app.repo.get_by_slug(&slug).map_err(map_repo_error)?.is_none() {
        return Err(movie_not_found(&slug));
    }

    let record = rematch_movie_by_tmdb_id(
        &slug,
        tmdb_id,
        &app.repo,
        tmdb,
        app.artwork.as_ref(),
        app.config.library.id.as_str(),
    )
    .await
    .map_err(map_repo_error)?;

    if let Ok(mut catalog) = state::catalog().write() {
        catalog.insert(record.clone());
    }

    Json(record.to_detail()).into_response()
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(error)
}
