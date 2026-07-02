//! Movie browse handlers.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::db::{MovieListQuery, MovieSort};
use crate::error::{invalid_request, movie_not_found};
use crate::models::MovieListResponse;
use crate::services::catalog::LoonMovieRecord;
use crate::state;

/// `GET /api/movies`
pub async fn list_movies(ctx: RequestContext) -> HttpResult {
    let paginated = ctx.query("page").is_some() || ctx.query("limit").is_some();

    if paginated {
        let page = parse_u32(ctx.query("page"), 1)?.max(1);
        let limit = parse_u32(ctx.query("limit"), 50)?.clamp(1, 100);
        let sort = match ctx.query("sort").unwrap_or("title") {
            "year" => MovieSort::Year,
            "recently_added" => MovieSort::RecentlyAdded,
            "title" => MovieSort::Title,
            other => return Err(invalid_request(format!("invalid sort value: {other}"))),
        };
        let genre = ctx.query("genre").map(str::to_string);

        let query = MovieListQuery {
            page,
            limit,
            sort,
            genre,
        };
        let total = state::repo().count_movies(&query).map_err(map_repo_error)?;
        let records = state::repo().list_movies(&query).map_err(map_repo_error)?;
        let movies: Vec<_> = records.iter().map(LoonMovieRecord::to_summary).collect();
        let pages = total.div_ceil(limit as usize) as u32;

        return Json(MovieListResponse {
            movies,
            total,
            page: Some(page),
            limit: Some(limit),
            pages: Some(pages),
        })
        .into_response();
    }

    let catalog = state::catalog();
    let guard = catalog.read().expect("catalog lock poisoned");
    let movies = guard.list();
    let total = movies.len();

    Json(MovieListResponse {
        movies,
        total,
        page: None,
        limit: None,
        pages: None,
    })
    .into_response()
}

/// `GET /api/movies/:slug`
pub async fn get_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;

    if let Ok(Some(record)) = state::repo().get_by_slug(slug) {
        return Json(record.to_detail()).into_response();
    }

    let catalog = state::catalog();
    let guard = catalog.read().expect("catalog lock poisoned");
    let movie = guard
        .get(slug)
        .map(LoonMovieRecord::to_detail)
        .ok_or_else(|| movie_not_found(slug))?;

    Json(movie).into_response()
}

fn parse_u32(raw: Option<&str>, default: u32) -> Result<u32, nest_http_serve::ServeError> {
    match raw {
        Some(value) => value
            .parse()
            .map_err(|_| invalid_request(format!("invalid integer: {value}"))),
        None => Ok(default),
    }
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
}
