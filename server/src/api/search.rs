//! `GET /api/search`

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::error::invalid_request;
use crate::models::SearchResponse;
use crate::services::catalog::LoonMovieRecord;
use crate::state;

/// Searches movies by title.
pub async fn search(ctx: RequestContext) -> HttpResult {
    let query = ctx
        .query("q")
        .ok_or_else(|| invalid_request("query parameter `q` is required"))?
        .trim()
        .to_string();

    if query.len() < 2 {
        return Err(invalid_request(
            "query parameter `q` must be at least 2 characters",
        ));
    }

    let limit = parse_limit(ctx.query("limit"), 20, 50);
    let records = state::repo()
        .search_movies(&query, limit)
        .map_err(map_repo_error)?;
    let movies: Vec<_> = records.iter().map(LoonMovieRecord::to_summary).collect();
    let total = movies.len();

    Json(SearchResponse {
        query,
        movies,
        total,
    })
    .into_response()
}

fn parse_limit(raw: Option<&str>, default: u32, max: u32) -> u32 {
    raw.and_then(|value| value.parse().ok())
        .unwrap_or(default)
        .clamp(1, max)
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
}
