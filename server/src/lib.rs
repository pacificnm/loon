//! Loon movie server library.

#![allow(clippy::result_large_err)]

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod logging;
pub mod models;
pub mod services;
pub mod state;

pub use config::{LoonCacheConfig, ServerConfig};

use std::sync::{Arc, RwLock};

use nest_error::NestResult;
use nest_http_serve::{HttpServer, RouteGroup, TestServer};

use crate::db::open_database;
use crate::services::ai::AiRuntime;
use crate::services::artwork::ArtworkRuntime;
use crate::services::scan_service::{load_catalog_from_db, scan_and_persist, ScanOptions};
use crate::services::scan_state::ScanCoordinator;
use crate::services::tmdb::TmdbRuntime;
use crate::state::{init_state, AppState};

/// Builds the root route group.
pub fn root_routes() -> RouteGroup {
    RouteGroup::new("").get("/", api::root::root)
}

/// Builds the `/api` route group.
pub fn api_routes() -> RouteGroup {
    RouteGroup::new("/api")
        .get("/health", api::health::health)
        .get("/browse", api::browse::browse)
        .get("/movies", api::movies::list_movies)
        .get("/movies/:slug", api::movies::get_movie)
        .put("/movies/:slug/favorite", api::favorites::set_favorite)
        .put("/movies/:slug/match", api::match_handler::set_tmdb_match)
        .put("/movies/:slug/progress", api::progress::save_progress)
        .get("/search", api::search::search)
        .get("/genres", api::genres::list_genres)
        .post("/library/scan", api::library::start_scan)
        .get("/library/status", api::library::library_status)
        .get("/artwork/:slug/:kind", api::artwork::artwork)
}

/// Builds the stream route group.
pub fn stream_routes() -> RouteGroup {
    RouteGroup::new("").get("/stream/:slug", api::stream::stream_movie)
}

/// Scans the media library and initializes application state.
pub async fn init_app(config: &ServerConfig, force_scan: bool) -> NestResult<()> {
    init_app_with_options(config, force_scan, false).await
}

/// Initializes application state, optionally skipping the startup scan.
pub async fn init_app_with_options(
    config: &ServerConfig,
    force_scan: bool,
    skip_initial_scan: bool,
) -> NestResult<()> {
    let repo = Arc::new(open_database(&config.db_path())?);
    let tmdb = match config.tmdb.as_ref() {
        Some(tmdb_config) => Some(TmdbRuntime::from_config(tmdb_config)?),
        None => None,
    };
    let ai = match config.ai.as_ref() {
        Some(ai_config) => Some(AiRuntime::from_config(ai_config)?),
        None => None,
    };
    let artwork = match config.cache.as_ref() {
        Some(cache_config) if cache_config.enabled => {
            Some(ArtworkRuntime::from_config(cache_config)?)
        }
        _ => None,
    };

    let needs_scan = !skip_initial_scan && (force_scan || repo.movie_count()? == 0);
    let (catalog, scanned_at) = if needs_scan {
        let result = scan_and_persist(
            config,
            &repo,
            tmdb.as_ref(),
            ai.as_ref(),
            ScanOptions {
                full_metadata: force_scan,
            },
            None,
            artwork.as_ref(),
        )
        .await?;
        (load_catalog_from_db(&repo)?, result.scanned_at)
    } else {
        (
            load_catalog_from_db(&repo)?,
            repo.load_all()
                .ok()
                .and_then(|records| records.iter().map(|record| record.scanned_at).max())
                .unwrap_or(0),
        )
    };

    let scan = Arc::new(ScanCoordinator::default());
    scan.finish(scanned_at, std::time::Instant::now(), Default::default());

    init_state(AppState {
        catalog: Arc::new(RwLock::new(catalog)),
        repo,
        media_root: config.media_root.clone(),
        library_scanned_at: std::sync::Mutex::new(scanned_at),
        scan,
        config: Arc::new(config.clone()),
        tmdb,
        ai,
        artwork,
    });

    Ok(())
}

/// Starts the HTTP server on the given bind address.
pub async fn run(config: ServerConfig, force_scan: bool) -> NestResult<()> {
    init_app(&config, force_scan).await?;

    HttpServer::builder()
        .name("loon-server")
        .bind(&config.bind)
        .routes(root_routes())
        .routes(api_routes())
        .routes(stream_routes())
        .run()
        .await
        .map_err(|error| nest_error::NestError::network(error.message()))
}

/// Starts an in-process test server (binds `127.0.0.1:0`).
pub async fn spawn_test_server() -> Result<TestServer, nest_http_serve::ServeError> {
    let temp = tempfile::tempdir().map_err(|error| {
        nest_http_serve::ServeError::from(nest_error::NestError::io(error.to_string()))
    })?;
    let data_dir = temp.path().to_path_buf();
    let config = ServerConfig::test_with_data_dir(Some(data_dir));

    spawn_test_server_with_config(config, Some(temp), false).await
}

/// Starts an in-process test server from an explicit configuration.
pub async fn spawn_test_server_with_config(
    config: ServerConfig,
    temp: Option<tempfile::TempDir>,
    skip_initial_scan: bool,
) -> Result<TestServer, nest_http_serve::ServeError> {
    init_app_with_options(&config, true, skip_initial_scan)
        .await
        .map_err(|error| {
            nest_http_serve::ServeError::from(nest_error::NestError::config(error.to_string()))
        })?;

    let server = HttpServer::builder()
        .name("loon-server")
        .routes(root_routes())
        .routes(api_routes())
        .routes(stream_routes())
        .spawn()
        .await?;

    if let Some(temp) = temp {
        std::mem::forget(temp);
    }
    Ok(server)
}
