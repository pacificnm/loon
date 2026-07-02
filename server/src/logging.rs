//! nest-logging initialization from `config.toml`.

use nest_config::{ConfigDocument, ConfigLoader};
use nest_error::NestResult;
use nest_logging::{LogLevel, LoggingConfig};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::config::Cli;

#[derive(Debug, Deserialize)]
struct LoggingSection {
    level: Option<String>,
    directory: Option<String>,
    file: Option<String>,
}

/// Installs the Nest logging subscriber from `[logging]` in config.toml.
pub fn init_from_cli(cli: &Cli) -> NestResult<()> {
    let loaded = ConfigLoader::file_or_search("loon", Some(cli.config.clone())).load()?;
    let config = build_logging_config(&loaded.document)?;
    nest_logging::init(config)?;
    Ok(())
}

fn build_logging_config(document: &ConfigDocument) -> NestResult<LoggingConfig> {
    let mut config = LoggingConfig::for_cli("loon-server");

    if document.has_section("logging") {
        let section: LoggingSection = document.section("logging")?;
        if let Some(level) = section.level.as_deref() {
            if let Ok(parsed) = level.parse::<LogLevel>() {
                config.level = parsed;
            }
        }
        if let Some(directory) = section.directory {
            config = config.with_file(directory);
        } else if let Some(file) = section.file {
            let path = PathBuf::from(file);
            let directory = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            config = config.with_file(directory);
        }
    }

    Ok(config)
}
