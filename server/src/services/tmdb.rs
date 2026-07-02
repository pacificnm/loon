//! TMDB client wiring for library enrichment.

use nest_error::NestResult;
use nest_http_client::{HttpClientConfig, HttpClientService};
use nest_tmdb::{TmdbClient, TmdbConfig, TmdbImageService, TmdbMetadataProvider};

/// Runtime TMDB services used during library scan.
#[derive(Clone)]
pub struct TmdbRuntime {
    /// TMDB metadata provider.
    pub provider: TmdbMetadataProvider,
    /// Builds poster and backdrop URLs from TMDB path tokens.
    pub images: TmdbImageService,
}

impl TmdbRuntime {
    /// Builds TMDB services from resolved configuration.
    pub fn from_config(config: &TmdbConfig) -> NestResult<Self> {
        let http =
            HttpClientService::new(HttpClientConfig::default().with_user_agent("loon-server/0.1"))?;
        let client = TmdbClient::new(http, config.clone())?;
        let images = TmdbImageService::new(client.image_base_url());
        let provider = TmdbMetadataProvider::new(client);

        Ok(Self { provider, images })
    }
}
