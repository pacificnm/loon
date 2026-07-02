//! Watch progress handler.

use std::time::{SystemTime, UNIX_EPOCH};

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::db::WatchProgress;
use crate::error::{invalid_request, movie_not_found};
use crate::models::{ProgressRequest, ProgressResponse};
use crate::state;

/// `PUT /api/movies/:slug/progress`
pub async fn save_progress(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let body: ProgressRequest = ctx
        .json()
        .map_err(|_| invalid_request("invalid JSON body"))?;

    if state::repo()
        .get_by_slug(&slug)
        .map_err(map_repo_error)?
        .is_none()
    {
        return Err(movie_not_found(&slug));
    }

    let progress = WatchProgress {
        position_seconds: body.position_seconds,
        duration_seconds: body.duration_seconds,
    };
    state::repo()
        .save_progress(&slug, &progress)
        .map_err(map_repo_error)?;

    let finished = body.duration_seconds.is_some_and(|duration| {
        duration > 0 && body.position_seconds as f64 / duration as f64 > 0.9
    });

    if let Ok(mut catalog) = state::catalog().write() {
        if let Some(record) = catalog.get_mut(&slug) {
            if finished {
                record.watch_progress_seconds = None;
                record.watch_duration_seconds = None;
            } else {
                record.watch_progress_seconds = Some(body.position_seconds);
                record.watch_duration_seconds = body.duration_seconds;
            }
        }
    }

    Json(ProgressResponse {
        slug,
        position_seconds: body.position_seconds,
        duration_seconds: body.duration_seconds,
        updated_at: iso_timestamp(now_secs()),
    })
    .into_response()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn iso_timestamp(secs: u64) -> String {
    // Good enough for v0.2; full RFC3339 can replace later.
    format!("{secs}")
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
}
