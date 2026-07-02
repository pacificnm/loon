//! Poster and backdrop cache + fetch helpers.

use std::sync::Arc;

use nest_cache::{Cache, CacheEntry, CacheKey};
use nest_cache_file::{set_with_content_type, FileCacheAdapter, FileCacheConfig};
use nest_error::NestResult;
use nest_http::HttpRequest;
use nest_http_client::{HttpClientConfig, HttpClientService};
use tracing::warn;

use crate::config::LoonCacheConfig;

/// Artwork variant served by the proxy route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtworkKind {
    /// Poster image.
    Poster,
    /// Backdrop image.
    Backdrop,
}

impl ArtworkKind {
    /// Parses a route `:kind` parameter.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "poster" => Some(Self::Poster),
            "backdrop" => Some(Self::Backdrop),
            _ => None,
        }
    }

    /// Returns the route segment for this kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Poster => "poster",
            Self::Backdrop => "backdrop",
        }
    }
}

/// Cached or remote artwork payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkPayload {
    /// Raw image bytes.
    pub bytes: Vec<u8>,
    /// HTTP content type.
    pub content_type: String,
}

/// Runtime artwork cache used by the proxy route.
#[derive(Clone)]
pub struct ArtworkRuntime {
    cache: Cache,
    adapter: Arc<FileCacheAdapter>,
    http: HttpClientService,
}

impl ArtworkRuntime {
    /// Builds the artwork cache from resolved settings.
    pub fn from_config(config: &LoonCacheConfig) -> NestResult<Self> {
        let mut file_config = FileCacheConfig::new(&config.root);
        if let Some(max_bytes) = config.max_bytes {
            file_config = file_config.with_max_bytes(max_bytes);
        }

        let adapter = Arc::new(FileCacheAdapter::new(file_config).map_err(|error| {
            nest_error::NestError::io(format!("artwork cache init failed: {error}"))
        })?);
        let http =
            HttpClientService::new(HttpClientConfig::default().with_user_agent("loon-server/0.1"))?;

        Ok(Self {
            cache: Cache::new(adapter.clone()),
            adapter,
            http,
        })
    }

    /// Returns the proxy URL for an artwork kind when remote artwork exists.
    pub fn proxy_url(slug: &str, kind: ArtworkKind, remote_url: &Option<String>) -> Option<String> {
        remote_url
            .as_ref()
            .map(|_| format!("/api/artwork/{slug}/{}", kind.as_str()))
    }

    /// Returns cached artwork when present.
    pub fn get_cached(&self, slug: &str, kind: ArtworkKind) -> NestResult<Option<ArtworkPayload>> {
        let key = cache_key(slug, kind);
        let Some(bytes) = self
            .cache
            .get_bytes(&key)
            .map_err(|error| nest_error::NestError::io(error.to_string()))?
        else {
            return Ok(None);
        };

        let content_type = self
            .adapter
            .metadata_for(&key)
            .map_err(|error| nest_error::NestError::io(error.to_string()))?
            .and_then(|meta| meta.content_type)
            .unwrap_or_else(|| "application/octet-stream".into());

        Ok(Some(ArtworkPayload {
            bytes,
            content_type,
        }))
    }

    /// Fetches remote artwork, stores it in the cache, and returns the bytes.
    pub async fn fetch_and_cache(
        &self,
        slug: &str,
        kind: ArtworkKind,
        source_url: &str,
    ) -> NestResult<ArtworkPayload> {
        let response = self
            .http
            .send(HttpRequest::get(source_url))
            .await
            .map_err(|error| {
                nest_error::NestError::network(format!(
                    "artwork fetch failed for {source_url}: {error}"
                ))
            })?;

        let content_type = response
            .headers
            .get("content-type")
            .map(str::to_string)
            .unwrap_or_else(|| guess_content_type(source_url));
        let bytes = response.body;

        let entry = CacheEntry {
            key: cache_key(slug, kind),
            value: bytes.clone(),
            tags: artwork_tags(slug),
            expires_at: None,
        };

        if let Err(error) = set_with_content_type(&self.adapter, entry, content_type.clone()) {
            warn!(slug, kind = kind.as_str(), error = %error, "failed to store artwork in cache");
        }

        Ok(ArtworkPayload {
            bytes,
            content_type,
        })
    }

    /// Invalidates cached artwork for one movie slug.
    pub fn invalidate_movie(&self, slug: &str) -> NestResult<u64> {
        self.cache
            .invalidate_tag(&movie_tag(slug))
            .map_err(|error| nest_error::NestError::io(error.to_string()))
    }
}

fn cache_key(slug: &str, kind: ArtworkKind) -> CacheKey {
    CacheKey::scoped("loon", &["artwork", slug, kind.as_str()])
}

fn artwork_tags(slug: &str) -> Vec<String> {
    vec![movie_tag(slug), "artwork".into()]
}

fn movie_tag(slug: &str) -> String {
    format!("movie:{slug}")
}

fn guess_content_type(url: &str) -> String {
    if url.contains(".png") {
        "image/png".into()
    } else if url.contains(".webp") {
        "image/webp".into()
    } else {
        "image/jpeg".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_url_requires_remote_source() {
        assert_eq!(
            ArtworkRuntime::proxy_url("alien-1979", ArtworkKind::Poster, &Some("https://x".into())),
            Some("/api/artwork/alien-1979/poster".into())
        );
        assert_eq!(
            ArtworkRuntime::proxy_url("alien-1979", ArtworkKind::Poster, &None),
            None
        );
    }

    #[tokio::test]
    async fn fetch_and_cache_round_trip() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/poster.jpg"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_bytes(b"jpeg-bytes")
                    .insert_header("content-type", "image/jpeg"),
            )
            .mount(&server)
            .await;

        let temp = tempfile::tempdir().unwrap();
        let runtime = ArtworkRuntime::from_config(&LoonCacheConfig {
            enabled: true,
            root: temp.path().to_path_buf(),
            max_bytes: None,
        })
        .unwrap();

        let payload = runtime
            .fetch_and_cache(
                "alien-1979",
                ArtworkKind::Poster,
                &format!("{}/poster.jpg", server.uri()),
            )
            .await
            .unwrap();
        assert_eq!(payload.bytes, b"jpeg-bytes");
        assert_eq!(payload.content_type, "image/jpeg");

        let cached = runtime
            .get_cached("alien-1979", ArtworkKind::Poster)
            .unwrap()
            .expect("cached artwork");
        assert_eq!(cached.bytes, b"jpeg-bytes");
    }
}
