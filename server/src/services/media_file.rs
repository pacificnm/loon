//! Media file metadata helpers.

use std::path::Path;

use crate::models::MovieFileInfo;
use crate::services::catalog::LoonMovieRecord;

/// MIME type for a media file extension.
pub fn content_type_for_extension(ext: Option<&str>) -> &'static str {
    match ext.map(str::to_ascii_lowercase).as_deref() {
        Some("mp4") | Some("m4v") | Some("mov") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        _ => "application/octet-stream",
    }
}

/// MIME type derived from a relative library path.
pub fn content_type_for_path(relative_path: &str) -> &'static str {
    let extension = Path::new(relative_path)
        .extension()
        .and_then(|ext| ext.to_str());
    content_type_for_extension(extension)
}

/// Builds API file metadata from a catalog record.
pub fn file_info_from_record(record: &LoonMovieRecord) -> MovieFileInfo {
    let path = Path::new(&record.relative_path);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(record.relative_path.as_str())
        .to_string();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    MovieFileInfo {
        filename,
        relative_path: record.relative_path.clone(),
        extension,
        size_bytes: record.size_bytes,
        content_type: content_type_for_path(&record.relative_path).to_string(),
        modified_at: record.modified_secs,
        scanned_at: (record.scanned_at > 0).then_some(record.scanned_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CastMemberDto, CrewMemberDto};
    use crate::services::catalog::LoonMovieRecord;

    fn sample_record() -> LoonMovieRecord {
        LoonMovieRecord {
            media_id: "file:Movies/Alien (1979)/Alien (1979).mp4".into(),
            slug: "alien-1979".into(),
            relative_path: "Movies/Alien (1979)/Alien (1979).mp4".into(),
            title: "Alien".into(),
            original_title: None,
            year: Some(1979),
            runtime_minutes: Some(117),
            summary: None,
            genres: vec!["Horror".into()],
            poster_url: None,
            backdrop_url: None,
            cast: Vec::<CastMemberDto>::new(),
            crew: Vec::<CrewMemberDto>::new(),
            tmdb_id: Some("348".into()),
            imdb_id: Some("tt0078748".into()),
            scanned_at: 1_700_000_000,
            size_bytes: Some(4_589_934_592),
            modified_secs: Some(1_699_000_000),
            is_favorite: false,
            watch_progress_seconds: None,
            watch_duration_seconds: None,
        }
    }

    #[test]
    fn builds_file_info_from_record() {
        let info = file_info_from_record(&sample_record());
        assert_eq!(info.filename, "Alien (1979).mp4");
        assert_eq!(info.extension.as_deref(), Some("mp4"));
        assert_eq!(info.content_type, "video/mp4");
        assert_eq!(info.size_bytes, Some(4_589_934_592));
        assert_eq!(info.modified_at, Some(1_699_000_000));
        assert_eq!(info.scanned_at, Some(1_700_000_000));
    }
}
