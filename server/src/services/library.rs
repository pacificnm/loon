//! Library scan orchestration at startup.

use nest_error::NestResult;
use nest_media_library::ScanResult;

use crate::config::ServerConfig;

/// Runs filesystem discovery for the configured library.
pub fn discover_library(config: &ServerConfig) -> NestResult<ScanResult> {
    let files = nest_file::FileService::with_config(
        nest_file::FileServiceConfig::scoped(&config.media_root).allow_create_dirs(true),
    )?;

    let scanner = nest_media_library::LibraryScanner::new(files);
    Ok(scanner.discover(&config.library)?)
}
