//! Desktop config from `~/.config/loon/config.toml`.

use std::path::{Path, PathBuf};

use nest_config::ConfigLoader;
use nest_error::{NestError, NestResult};
use serde::Deserialize;

/// `[loon-admin]` section — backend API base URL (no trailing path).
#[derive(Debug, Clone, Deserialize)]
pub struct LoonAdminSection {
    pub server_url: String,
    /// Optional path to ffplay, mpv, or another HTTP-capable player binary.
    #[serde(default)]
    pub player_path: Option<String>,
}

/// Loaded desktop configuration.
#[derive(Debug, Clone)]
pub struct LoonDesktopConfig {
    pub config_path: PathBuf,
    pub server_url: String,
    pub player_path: Option<String>,
}

impl LoonDesktopConfig {
    pub fn load(path: &Path) -> NestResult<Self> {
        let loaded = ConfigLoader::file_or_search("loon", Some(path.to_path_buf())).load()?;
        let section: LoonAdminSection = loaded.document.section("loon-admin")?;
        let server_url = section.server_url.trim().trim_end_matches('/').to_string();
        if server_url.is_empty() {
            return Err(NestError::config("[loon-admin].server_url must not be empty"));
        }
        if !server_url.starts_with("http://") && !server_url.starts_with("https://") {
            return Err(NestError::config(format!(
                "[loon-admin].server_url must be an http(s) URL, got: {server_url}"
            )));
        }
        Ok(Self {
            config_path: path.to_path_buf(),
            server_url,
            player_path: section.player_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_parses_server_url() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            "[loon-admin]\nserver_url = \"http://192.168.88.10:3000\"\n",
        )
        .unwrap();
        let config = LoonDesktopConfig::load(&path).unwrap();
        assert_eq!(config.server_url, "http://192.168.88.10:3000");
    }
}
