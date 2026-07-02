//! CLI and server configuration loaded from `config.toml`.

pub mod cache;

use std::path::{Path, PathBuf};

use clap::Parser;
use nest_ai_ollama::OllamaConfig;
use nest_config::{ConfigLoader, ConfigService};
use nest_error::{NestError, NestResult};
use nest_media_library::MediaLibraryConfig;
use nest_tmdb::{TmdbConfig, NEST_TMDB_API_KEY_MISSING};
use serde::Deserialize;
use tracing::info;

pub use cache::{CacheSection, LoonCacheConfig};

/// Loon movie server for LG webOS.
#[derive(Debug, Parser)]
#[command(name = "loon-server", version)]
pub struct Cli {
    /// Path to config TOML.
    #[arg(long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Override HTTP bind address (host:port).
    #[arg(long)]
    pub bind: Option<String>,

    /// Run a full library scan on startup even when the database already has movies.
    #[arg(long)]
    pub force_scan: bool,
}

/// `[loon]` section in config.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct LoonSection {
    /// HTTP bind address.
    pub bind: String,
    /// Persistent data directory (SQLite, cache — Phase 3).
    pub data_dir: Option<PathBuf>,
    /// Absolute path to the media library root on disk.
    ///
    /// Library scan roots in `[media-library].roots` are relative to this path.
    /// Example: `media_root = "/mnt/media"` with `roots = ["Movies"]` scans
    /// `/mnt/media/Movies/`.
    pub media_root: PathBuf,
}

/// `[media-library]` section in config.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct MediaLibrarySection {
    /// Library identifier.
    pub id: String,
    /// Subdirectories under `media_root` to scan.
    pub roots: Vec<String>,
    /// Video file extensions without a leading dot.
    pub video_extensions: Option<Vec<String>>,
}

/// `[ai]` section in config.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct AiSection {
    /// Whether AI filename guessing is enabled.
    #[serde(default = "default_ai_enabled")]
    pub enabled: bool,
    /// Provider id (`ollama` in v0.1).
    #[serde(default = "default_ai_provider")]
    pub provider: String,
    /// Inference base URL.
    #[serde(default = "default_ai_base_url")]
    pub base_url: String,
    /// Default model id.
    #[serde(default = "default_ai_model")]
    pub model: String,
    /// Minimum model confidence before applying a guess.
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
}

/// Resolved AI settings for filename guessing.
#[derive(Debug, Clone)]
pub struct LoonAiConfig {
    /// Ollama connection settings.
    pub ollama: OllamaConfig,
    /// Minimum model confidence before applying a guess.
    pub min_confidence: f32,
}

/// Resolved server settings.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the HTTP listener.
    pub bind: String,
    /// Persistent data directory.
    pub data_dir: Option<PathBuf>,
    /// Absolute path to the media library root.
    pub media_root: PathBuf,
    /// Media library scan settings.
    pub library: MediaLibraryConfig,
    /// TMDB settings when enrichment is enabled.
    pub tmdb: Option<TmdbConfig>,
    /// AI filename guessing settings when enabled.
    pub ai: Option<LoonAiConfig>,
    /// Artwork cache settings when enabled.
    pub cache: Option<LoonCacheConfig>,
}

impl ServerConfig {
    /// Returns the SQLite database file path.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("data"))
            .join("loon.db")
    }

    /// Loads configuration from the path given on the CLI (with bind override).
    pub fn load(cli: &Cli) -> NestResult<Self> {
        let loaded = ConfigLoader::file_or_search("loon", Some(cli.config.clone())).load()?;
        let service = ConfigService::new(loaded);

        let loon: LoonSection = service
            .section("loon")
            .map_err(|error| missing_section_error("loon", &cli.config, error))?;
        let media_library: MediaLibrarySection = service
            .section("media-library")
            .map_err(|error| missing_section_error("media-library", &cli.config, error))?;

        let bind = cli.bind.clone().unwrap_or(loon.bind);
        let library = build_library_config(media_library);
        let tmdb = load_tmdb(&service);
        let ai = load_ai(&service);
        let data_dir = loon
            .data_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("data"));
        let cache = load_cache(&service, &data_dir);

        Ok(Self {
            bind,
            data_dir: loon.data_dir,
            media_root: loon.media_root,
            library,
            tmdb,
            ai,
            cache,
        })
    }

    /// Test configuration pointing at `server/tests/fixtures`.
    pub fn test() -> Self {
        Self::test_with_data_dir(None)
    }

    /// Test configuration with an optional data directory for SQLite.
    pub fn test_with_data_dir(data_dir: Option<PathBuf>) -> Self {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixtures = manifest.join("tests/fixtures");

        Self {
            bind: "127.0.0.1:0".into(),
            data_dir,
            media_root: fixtures.join("media"),
            library: build_library_config(MediaLibrarySection {
                id: "main".into(),
                roots: vec!["Movies".into()],
                video_extensions: Some(vec!["mp4".into(), "mkv".into()]),
            }),
            tmdb: None,
            ai: None,
            cache: None,
        }
    }

    /// Test configuration with artwork cache enabled.
    pub fn test_with_cache(data_dir: PathBuf, cache_root: PathBuf) -> Self {
        let mut config = Self::test_with_data_dir(Some(data_dir));
        config.cache = Some(LoonCacheConfig {
            enabled: true,
            root: cache_root,
            max_bytes: None,
        });
        config
    }

    /// Test configuration for AI + TMDB enrichment integration tests.
    pub fn enrichment_test(
        media_root: PathBuf,
        data_dir: PathBuf,
        ollama: OllamaConfig,
        tmdb: TmdbConfig,
    ) -> Self {
        Self {
            bind: "127.0.0.1:0".into(),
            data_dir: Some(data_dir),
            media_root,
            library: build_library_config(MediaLibrarySection {
                id: "main".into(),
                roots: vec![".".into()],
                video_extensions: Some(vec!["mp4".into(), "mkv".into()]),
            }),
            tmdb: Some(tmdb),
            ai: Some(LoonAiConfig {
                ollama,
                min_confidence: 0.5,
            }),
            cache: None,
        }
    }
}

fn load_cache(service: &ConfigService, data_dir: &Path) -> Option<LoonCacheConfig> {
    let section: CacheSection = service.section("cache").ok()?;
    if !section.enabled {
        return None;
    }

    match LoonCacheConfig::resolve(section, data_dir) {
        Ok(config) => {
            info!(root = %config.root.display(), "artwork cache enabled");
            Some(config)
        }
        Err(error) => {
            info!(error = %error, "artwork cache disabled due to config error");
            None
        }
    }
}

fn load_tmdb(service: &ConfigService) -> Option<TmdbConfig> {
    match TmdbConfig::from_config_service(service) {
        Ok(config) => Some(config),
        Err(error) if error.code() == Some(NEST_TMDB_API_KEY_MISSING) => {
            info!("TMDB_API_KEY not set; metadata enrichment disabled");
            None
        }
        Err(error) => {
            info!(
                error = %error,
                "TMDB config unavailable; metadata enrichment disabled"
            );
            None
        }
    }
}

fn build_library_config(section: MediaLibrarySection) -> MediaLibraryConfig {
    let mut config = MediaLibraryConfig::new(section.id, section.roots);
    if let Some(extensions) = section.video_extensions {
        config = config.with_video_extensions(extensions);
    }
    config
}

fn load_ai(service: &ConfigService) -> Option<LoonAiConfig> {
    let section: AiSection = service.section("ai").ok()?;
    if !section.enabled {
        return None;
    }
    if section.provider != "ollama" {
        info!(
            provider = %section.provider,
            "unsupported AI provider; filename guessing disabled"
        );
        return None;
    }

    info!(
        model = %section.model,
        base_url = %section.base_url,
        "AI filename guessing enabled"
    );

    Some(LoonAiConfig {
        ollama: OllamaConfig::new(section.base_url, section.model),
        min_confidence: section.min_confidence,
    })
}

fn default_ai_enabled() -> bool {
    true
}

fn default_ai_provider() -> String {
    "ollama".into()
}

fn default_ai_base_url() -> String {
    nest_ai_ollama::DEFAULT_BASE_URL.into()
}

fn default_ai_model() -> String {
    nest_ai_ollama::DEFAULT_MODEL.into()
}

fn default_min_confidence() -> f32 {
    0.5
}

fn missing_section_error(section: &str, path: &Path, error: NestError) -> NestError {
    NestError::config(format!(
        "missing or invalid [{section}] in {}",
        path.display()
    ))
    .with_source(error)
    .with_help(format!(
        "Copy config.example.toml to {} and set media_root to your movie folder.",
        path.display()
    ))
}
