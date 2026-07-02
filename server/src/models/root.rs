//! Root index response.

use serde::Serialize;

/// Service index returned at `GET /`.
#[derive(Debug, Clone, Serialize)]
pub struct RootResponse {
    /// Service name.
    pub service: &'static str,
    /// Human-readable hint for browser visits.
    pub message: &'static str,
    /// JSON API entry points.
    pub endpoints: RootEndpoints,
}

/// Links to primary API routes.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RootEndpoints {
    /// Health check.
    pub health: &'static str,
    /// Movie list.
    pub movies: &'static str,
}

impl Default for RootResponse {
    fn default() -> Self {
        Self {
            service: "loon-server",
            message: "Loon API — use the endpoints below (no web UI at / yet)",
            endpoints: RootEndpoints {
                health: "/api/health",
                movies: "/api/movies",
            },
        }
    }
}
