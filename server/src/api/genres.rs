//! `GET /api/genres`

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::models::{GenreEntry, GenresResponse};
use crate::state;

/// Returns distinct genres with counts.
pub async fn list_genres(_ctx: RequestContext) -> HttpResult {
    let genres = state::repo()
        .list_genres()
        .map_err(|error| {
            nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
        })?
        .into_iter()
        .map(|genre| GenreEntry {
            name: genre.name,
            count: genre.count,
        })
        .collect();

    Json(GenresResponse { genres }).into_response()
}
