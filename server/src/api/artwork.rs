//! Artwork proxy handler.

use nest_http_serve::{HttpResponse, HttpResult, HttpStatus, RequestContext};
use tracing::warn;

use crate::error::{artwork_not_found, invalid_request, movie_not_found};
use crate::services::artwork::ArtworkKind;
use crate::services::catalog::LoonMovieRecord;
use crate::state;

/// `GET /api/artwork/:slug/:kind`
pub async fn artwork(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    let kind = ArtworkKind::parse(ctx.param("kind")?)
        .ok_or_else(|| invalid_request("kind must be `poster` or `backdrop`"))?;

    let record = lookup_record(slug).ok_or_else(|| movie_not_found(slug))?;
    let source_url = match kind {
        ArtworkKind::Poster => record.poster_url.clone(),
        ArtworkKind::Backdrop => record.backdrop_url.clone(),
    }
    .ok_or_else(|| artwork_not_found(slug, kind))?;

    let app = state::app_state();

    if let Some(artwork) = app.artwork.as_ref() {
        if let Ok(Some(cached)) = artwork.get_cached(slug, kind) {
            return Ok(image_response(cached.bytes, &cached.content_type));
        }

        match artwork.fetch_and_cache(slug, kind, &source_url).await {
            Ok(payload) => return Ok(image_response(payload.bytes, &payload.content_type)),
            Err(error) => {
                warn!(
                    slug,
                    kind = kind.as_str(),
                    error = %error,
                    "artwork cache fetch failed; redirecting to source URL"
                );
            }
        }
    }

    Ok(HttpResponse::empty(HttpStatus(302)).with_header("location", source_url))
}

fn lookup_record(slug: &str) -> Option<LoonMovieRecord> {
    if let Ok(Some(record)) = state::repo().get_by_slug(slug) {
        return Some(record);
    }

    state::catalog()
        .read()
        .ok()
        .and_then(|catalog| catalog.get(slug).cloned())
}

fn image_response(bytes: Vec<u8>, content_type: &str) -> HttpResponse {
    HttpResponse::new(HttpStatus::OK, bytes).with_header("content-type", content_type)
}
