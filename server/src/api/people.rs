//! Person API handlers.

use nest_http_serve::{Json, RequestContext};

use crate::error::{invalid_request, tmdb_not_configured};
use crate::services::person::{get_person_detail, get_person_for_cast};
use crate::state;

/// `GET /api/people/:tmdb_id` — person detail with library known-for movies.
pub async fn get_person(ctx: RequestContext) -> nest_http_serve::HttpResult {
    let tmdb_person_id = parse_person_id(ctx.param("tmdb_id")?)?;
    let app = state::app_state();
    let tmdb = app.tmdb.as_ref().ok_or_else(tmdb_not_configured)?;

    let movies = {
        let catalog = state::catalog();
        let guard = catalog.read().expect("catalog lock poisoned");
        guard.records()
    };

    let detail = get_person_detail(
        tmdb_person_id,
        &app.repo,
        &movies,
        tmdb,
    )
    .await
    .map_err(nest_http_serve::ServeError::from)?;

    Json(detail).into_response()
}

/// `GET /api/people/resolve?movie_slug=&name=` — person detail for a cast member.
pub async fn resolve_person(ctx: RequestContext) -> nest_http_serve::HttpResult {
    let movie_slug = ctx
        .query("movie_slug")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request("movie_slug is required"))?;
    let cast_name = ctx
        .query("name")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request("name is required"))?;

    let app = state::app_state();
    let tmdb = app.tmdb.as_ref().ok_or_else(tmdb_not_configured)?;

    let movies = {
        let catalog = state::catalog();
        let guard = catalog.read().expect("catalog lock poisoned");
        guard.records()
    };

    let detail = get_person_for_cast(
        movie_slug,
        cast_name,
        &app.repo,
        &movies,
        tmdb,
        app.config.library.id.as_str(),
    )
        .await
        .map_err(nest_http_serve::ServeError::from)?;

    Json(detail).into_response()
}

fn parse_person_id(raw: &str) -> Result<u32, nest_http_serve::ServeError> {
    let trimmed = raw.trim();
    let numeric = trimmed.strip_prefix("tmdb:").unwrap_or(trimmed).trim();
    numeric
        .parse::<u32>()
        .map_err(|_| invalid_request("tmdb_id must be a numeric TMDB person id"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_person_id() {
        assert_eq!(parse_person_id("85").unwrap(), 85);
    }
}
