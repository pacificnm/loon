//! Loon admin configuration.

use nest_config::ConfigService;
use nest_core::AppContext;
use nest_error::NestError;
use nest_error::NestResult;
use serde::Deserialize;

const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000";

/// Admin GUI settings loaded from `[loon-admin]` in config.
#[derive(Debug, Clone, Deserialize)]
pub struct LoonAdminConfig {
    /// Loon server base URL.
    #[serde(default = "default_server_url")]
    pub server_url: String,
}

impl Default for LoonAdminConfig {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
        }
    }
}

fn default_server_url() -> String {
    DEFAULT_SERVER_URL.to_string()
}

impl LoonAdminConfig {
    /// Loads settings from config when present, otherwise uses defaults.
    pub fn from_context(ctx: &AppContext) -> NestResult<Self> {
        let mut config = Self::default();

        if let Ok(config_service) = ctx.service::<ConfigService>() {
            if let Ok(Some(section)) = config_service.document().optional_section("loon-admin") {
                config = section;
            }
        }

        config.server_url = config.server_url.trim_end_matches('/').to_string();
        if config.server_url.is_empty() {
            return Err(NestError::config("loon-admin.server_url must not be empty"));
        }

        Ok(config)
    }
}
