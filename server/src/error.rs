//! API error helpers.

use nest_http_serve::{HttpError, HttpStatus, ServeError};

/// Returns a 404 JSON error for an unknown movie slug.
pub fn movie_not_found(slug: &str) -> ServeError {
    api_error(
        HttpStatus::NOT_FOUND,
        "movie_not_found",
        format!("No movie with slug '{slug}'"),
    )
}

/// Returns a 400 JSON error for invalid client input.
pub fn invalid_request(message: impl Into<String>) -> ServeError {
    api_error(HttpStatus::BAD_REQUEST, "invalid_request", message.into())
}

/// Returns a 409 JSON error when a scan is already running.
pub fn scan_already_running() -> ServeError {
    api_error(
        HttpStatus::CONFLICT,
        "scan_already_running",
        "A library scan is already in progress".into(),
    )
}

/// Returns a 503 JSON error when the library is scanning.
pub fn library_scanning() -> ServeError {
    api_error(
        HttpStatus::SERVICE_UNAVAILABLE,
        "library_scanning",
        "Library scan in progress".into(),
    )
}

/// Returns a 404 JSON error when artwork is unavailable for a movie.
pub fn artwork_not_found(slug: &str, kind: crate::services::artwork::ArtworkKind) -> ServeError {
    api_error(
        HttpStatus::NOT_FOUND,
        "artwork_not_found",
        format!("No {} artwork for movie '{slug}'", kind.as_str()),
    )
}

fn api_error(status: HttpStatus, code: &str, message: String) -> ServeError {
    ServeError::from(HttpError::from_status(status, message).with_code(code))
}
