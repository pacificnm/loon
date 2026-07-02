//! Movie browse handlers.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::db::{MovieListQuery, MovieSort};
use crate::error::{invalid_request, movie_not_found};
use crate::models::MovieListResponse;
use crate::services::cast_backfill::backfill_cast_person_ids;
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
    let slug = ctx.param("slug")?.to_string();
    let app = state::app_state();

    let mut record = if let Ok(Some(record)) = state::repo().get_by_slug(&slug) {
        record
    } else {
        let catalog = state::catalog();
        let guard = catalog.read().expect("catalog lock poisoned");
        guard
            .get(&slug)
            .cloned()
            .ok_or_else(|| movie_not_found(&slug))?
    };

    if let Some(tmdb) = app.tmdb.as_ref() {
        if backfill_cast_person_ids(&mut record, tmdb)
            .await
            .map_err(map_repo_error)?
        {
            persist_cast_backfill(&app, &record).map_err(map_repo_error)?;
        }
    }

    Json(record.to_detail()).into_response()
}

fn persist_cast_backfill(
    app: &std::sync::Arc<crate::state::AppState>,
    record: &LoonMovieRecord,
) -> nest_error::NestResult<()> {
    let stored = app.repo.get_file_by_path(&record.relative_path)?;
    let (size_bytes, modified_secs) = stored
        .map(|file| (file.size_bytes, file.modified_secs))
        .unwrap_or((record.size_bytes.unwrap_or(0), record.modified_secs));

    app.repo.upsert_movie(
        app.config.library.id.as_str(),
        record,
        record.scanned_at,
        size_bytes,
        modified_secs,
    )?;

    if let Ok(mut catalog) = state::catalog().write() {
        catalog.insert(record.clone());
    }

    Ok(())
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
