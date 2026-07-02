//! Health check handler.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::models::HealthResponse;
use crate::state;

/// `GET /api/health`
pub async fn health(_ctx: RequestContext) -> HttpResult {
    let app = state::app_state();
    let movies_count = app
        .repo
        .movie_count()
        .unwrap_or_else(|_| app.catalog.read().expect("catalog lock").len());

    Json(HealthResponse {
        status: "ok",
        service: "loon-server",
        version: env!("CARGO_PKG_VERSION"),
        movies_count,
        library_scanned_at: state::library_scanned_at(),
    })
    .into_response()
}
