//! Favorite toggle handler.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::error::{invalid_request, movie_not_found};
use crate::models::{FavoriteRequest, FavoriteResponse};
use crate::state;

/// `PUT /api/movies/:slug/favorite`
pub async fn set_favorite(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let repo = state::repo();

    if repo.get_by_slug(&slug).map_err(map_repo_error)?.is_none() {
        return Err(movie_not_found(&slug));
    }

    let favorite = if ctx.body().is_empty() {
        !repo.is_favorite(&slug).map_err(map_repo_error)?
    } else {
        let body: FavoriteRequest = ctx
            .json()
            .map_err(|_| invalid_request("invalid JSON body"))?;
        body.favorite.ok_or_else(|| {
            invalid_request("request body must include `favorite` or be omitted to toggle")
        })?
    };

    repo.set_favorite(&slug, favorite).map_err(map_repo_error)?;

    if let Ok(mut catalog) = state::catalog().write() {
        if let Some(record) = catalog.get_mut(&slug) {
            record.is_favorite = favorite;
        }
    }

    Json(FavoriteResponse { slug, favorite }).into_response()
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
}
