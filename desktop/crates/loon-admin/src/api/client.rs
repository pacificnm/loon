//! Blocking HTTP client for the Loon server API.

use nest_error::{NestError, NestResult};

use super::types::{MovieDetail, MovieListResponse};

/// Client for Loon server REST endpoints.
#[derive(Debug, Clone)]
pub struct LoonApiClient {
    base_url: String,
    http: reqwest::blocking::Client,
}

impl LoonApiClient {
    /// Creates a client for the given server base URL.
    pub fn new(base_url: impl Into<String>) -> NestResult<Self> {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|error| NestError::service(format!("HTTP client init failed: {error}")))?;

        Ok(Self {
            base_url: trim_trailing_slash(base_url.into()),
            http,
        })
    }

    /// `GET /api/movies`
    pub fn list_movies(&self) -> NestResult<MovieListResponse> {
        self.get("/api/movies")
    }

    /// `GET /api/movies/:slug`
    pub fn get_movie(&self, slug: &str) -> NestResult<MovieDetail> {
        let path = format!("/api/movies/{}", urlencoding::encode(slug));
        self.get(&path)
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> NestResult<T> {
        let url = format!("{}{path}", self.base_url);
        let response = self.http.get(&url).send().map_err(map_request_error)?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| NestError::service(format!("failed to read response: {error}")))?;

        if !status.is_success() {
            return Err(NestError::service(format!(
                "GET {path} failed ({status}): {body}"
            )));
        }

        serde_json::from_str(&body).map_err(|error| {
            NestError::service(format!("failed to decode {path} response: {error}"))
        })
    }
}

fn trim_trailing_slash(url: String) -> String {
    url.trim_end_matches('/').to_string()
}

fn map_request_error(error: reqwest::Error) -> NestError {
    NestError::service(format!("HTTP request failed: {error}"))
}
