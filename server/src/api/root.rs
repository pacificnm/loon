//! Root handler — browser-friendly index of API routes.

use nest_http_serve::{HttpResult, Json, RequestContext};

use crate::models::RootResponse;

/// `GET /`
pub async fn root(_ctx: RequestContext) -> HttpResult {
    Json(RootResponse::default()).into_response()
}
