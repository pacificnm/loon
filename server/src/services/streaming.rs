//! HTTP byte-range video streaming.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use nest_http_serve::{HttpResponse, HttpResult, HttpStatus, RequestContext};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::error::movie_not_found;
use crate::state;

/// `GET /stream/:slug`
pub async fn stream_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    let state = state::app_state();
    let relative_path = {
        let catalog = state.catalog.read().expect("catalog lock poisoned");
        catalog.get(slug).map(|record| record.relative_path.clone())
    }
    .or_else(|| {
        state
            .repo
            .get_by_slug(slug)
            .ok()
            .flatten()
            .map(|record| record.relative_path)
    })
    .ok_or_else(|| movie_not_found(slug))?;

    let file_path = resolve_media_path(&state.media_root, &relative_path)?;
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|_| movie_not_found(slug))?;

    let total = metadata.len();
    if total == 0 {
        return Ok(empty_video_response(content_type_for_path(&file_path)));
    }

    let range_header = ctx.header("range");
    let content_type = content_type_for_path(&file_path);

    match parse_range(range_header, total) {
        RangeParseResult::Full => stream_full_file(&file_path, total, content_type, slug).await,
        RangeParseResult::Partial { start, end } => {
            stream_partial_file(&file_path, start, end, total, content_type, slug).await
        }
        RangeParseResult::Invalid => Ok(invalid_range_response(total)),
    }
}

#[derive(Debug)]
enum RangeParseResult {
    Full,
    Partial { start: u64, end: u64 },
    Invalid,
}

fn parse_range(header: Option<&str>, total: u64) -> RangeParseResult {
    let Some(header) = header else {
        return RangeParseResult::Full;
    };

    let Some(spec) = header.strip_prefix("bytes=") else {
        return RangeParseResult::Invalid;
    };

    let spec = spec.split(',').next().unwrap_or(spec).trim();
    if spec.is_empty() {
        return RangeParseResult::Invalid;
    }

    if let Some((start, end)) = spec.split_once('-') {
        if start.is_empty() {
            // suffix range: bytes=-500
            let suffix: u64 = match end.parse() {
                Ok(value) if value > 0 => value,
                _ => return RangeParseResult::Invalid,
            };
            if suffix >= total {
                return RangeParseResult::Partial {
                    start: 0,
                    end: total.saturating_sub(1),
                };
            }
            return RangeParseResult::Partial {
                start: total - suffix,
                end: total - 1,
            };
        }

        let start: u64 = match start.parse() {
            Ok(value) => value,
            Err(_) => return RangeParseResult::Invalid,
        };

        if start >= total {
            return RangeParseResult::Invalid;
        }

        let end = if end.is_empty() {
            total - 1
        } else {
            match end.parse::<u64>() {
                Ok(value) => value.min(total - 1),
                Err(_) => return RangeParseResult::Invalid,
            }
        };

        if end < start {
            return RangeParseResult::Invalid;
        }

        return RangeParseResult::Partial { start, end };
    }

    RangeParseResult::Invalid
}

async fn stream_full_file(path: &Path, total: u64, content_type: &str, slug: &str) -> HttpResult {
    let mut file = File::open(path).await.map_err(|_| movie_not_found(slug))?;
    let mut buffer = Vec::with_capacity(total.min(8 * 1024 * 1024) as usize);
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| movie_not_found(slug))?;

    Ok(HttpResponse::new(HttpStatus::OK, buffer)
        .with_header("content-type", content_type)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", total.to_string()))
}

async fn stream_partial_file(
    path: &Path,
    start: u64,
    end: u64,
    total: u64,
    content_type: &str,
    slug: &str,
) -> HttpResult {
    let length = end - start + 1;
    let mut file = File::open(path).await.map_err(|_| movie_not_found(slug))?;
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| movie_not_found(slug))?;

    let mut buffer = vec![0_u8; length as usize];
    file.read_exact(&mut buffer)
        .await
        .map_err(|_| movie_not_found(slug))?;

    Ok(HttpResponse::new(HttpStatus(206), buffer)
        .with_header("content-type", content_type)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", length.to_string())
        .with_header("content-range", format!("bytes {start}-{end}/{total}")))
}

fn invalid_range_response(total: u64) -> HttpResponse {
    HttpResponse::empty(HttpStatus(416)).with_header("content-range", format!("bytes */{total}"))
}

fn empty_video_response(content_type: &str) -> HttpResponse {
    HttpResponse::empty(HttpStatus::OK)
        .with_header("content-type", content_type)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", "0")
}

fn resolve_media_path(
    media_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, nest_http_serve::ServeError> {
    if relative_path.contains("..") {
        return Err(movie_not_found("invalid"));
    }

    let path = media_root.join(relative_path);
    let canonical = std::fs::canonicalize(&path).map_err(|_| movie_not_found("missing"))?;
    let root = std::fs::canonicalize(media_root).map_err(|_| movie_not_found("missing"))?;

    if !canonical.starts_with(&root) {
        return Err(movie_not_found("invalid"));
    }

    Ok(canonical)
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp4") | Some("m4v") | Some("mov") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open_ended_range() {
        match parse_range(Some("bytes=0-"), 1000) {
            RangeParseResult::Partial { start, end } => {
                assert_eq!(start, 0);
                assert_eq!(end, 999);
            }
            other => panic!("expected partial range, got {other:?}"),
        }
    }

    #[test]
    fn parses_fixed_range() {
        match parse_range(Some("bytes=10-19"), 100) {
            RangeParseResult::Partial { start, end } => {
                assert_eq!((start, end), (10, 19));
            }
            other => panic!("expected partial range, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_bounds_range() {
        assert!(matches!(
            parse_range(Some("bytes=200-300"), 100),
            RangeParseResult::Invalid
        ));
    }
}
