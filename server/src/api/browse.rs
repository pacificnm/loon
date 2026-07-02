//! `GET /api/browse`

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::error::library_scanning;
use crate::services::browse::build_browse;
use crate::state;

/// Returns the Netflix-style home browse feed.
pub async fn browse(_ctx: RequestContext) -> HttpResult {
    if state::app_state().scan.is_running() {
        return Err(library_scanning());
    }

    let response = build_browse(&state::repo()).map_err(|error| {
        nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
    })?;

    Json(response).into_response()
}
