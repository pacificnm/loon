//! Artwork cache configuration.

use std::path::{Path, PathBuf};

use nest_error::NestResult;
use serde::Deserialize;

/// `[cache]` section in config.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct CacheSection {
    /// Whether the on-disk artwork cache is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Cache root relative to `data_dir` when not absolute.
    #[serde(default = "default_cache_root")]
    pub root: PathBuf,
    /// Optional maximum cache size in megabytes (LRU deferred).
    pub max_mb: Option<u64>,
}

/// Resolved artwork cache settings.
#[derive(Debug, Clone)]
pub struct LoonCacheConfig {
    /// Whether caching is active.
    pub enabled: bool,
    /// Absolute cache directory path.
    pub root: PathBuf,
    /// Optional maximum cache size in bytes.
    pub max_bytes: Option<u64>,
}

impl LoonCacheConfig {
    /// Resolves cache settings from config and data directory.
    pub fn resolve(section: CacheSection, data_dir: &Path) -> NestResult<Self> {
        let root = if section.root.is_absolute() {
            section.root
        } else {
            data_dir.join(section.root)
        };

        Ok(Self {
            enabled: section.enabled,
            root,
            max_bytes: section.max_mb.map(|mb| mb * 1024 * 1024),
        })
    }
}

fn default_cache_root() -> PathBuf {
    PathBuf::from("cache")
}
