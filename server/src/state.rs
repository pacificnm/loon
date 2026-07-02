//! Shared application state.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::config::ServerConfig;
use crate::db::LibraryRepository;
use crate::services::ai::AiRuntime;
use crate::services::artwork::ArtworkRuntime;
use crate::services::catalog::LoonCatalog;
use crate::services::scan_state::ScanCoordinator;
use crate::services::tmdb::TmdbRuntime;

/// Runtime state initialized at startup.
pub struct AppState {
    /// Movie catalog — reload after library scans.
    pub catalog: Arc<RwLock<LoonCatalog>>,
    /// SQLite library repository.
    pub repo: Arc<LibraryRepository>,
    /// Absolute media root used for scoped file access.
    pub media_root: PathBuf,
    /// Unix timestamp when the library scan last finished.
    pub library_scanned_at: Mutex<u64>,
    /// Background scan coordination.
    pub scan: Arc<ScanCoordinator>,
    /// Server configuration.
    pub config: Arc<ServerConfig>,
    /// TMDB runtime when enrichment is enabled.
    pub tmdb: Option<TmdbRuntime>,
    /// AI runtime when filename guessing is enabled.
    pub ai: Option<AiRuntime>,
    /// Artwork cache when enabled.
    pub artwork: Option<ArtworkRuntime>,
}

static STATE: std::sync::RwLock<Option<Arc<AppState>>> = std::sync::RwLock::new(None);

/// Stores application state (call once before serving requests).
pub fn init_state(state: AppState) {
    let mut guard = STATE.write().expect("app state lock poisoned");
    *guard = Some(Arc::new(state));
}

/// Returns the initialized application state.
pub fn app_state() -> Arc<AppState> {
    STATE
        .read()
        .expect("app state lock poisoned")
        .clone()
        .expect("app state not initialized — call init_app before handling requests")
}

/// Returns the movie catalog for read-mostly handlers.
pub fn catalog() -> Arc<RwLock<LoonCatalog>> {
    app_state().catalog.clone()
}

/// Returns the library repository.
pub fn repo() -> Arc<LibraryRepository> {
    app_state().repo.clone()
}

/// Replaces the in-memory catalog after a scan or mutation.
pub fn replace_catalog(catalog: LoonCatalog) {
    let app = app_state();
    let mut guard = app.catalog.write().expect("catalog lock poisoned");
    *guard = catalog;
}

/// Updates the last scan timestamp in app state.
pub fn set_library_scanned_at(scanned_at: u64) {
    *app_state()
        .library_scanned_at
        .lock()
        .expect("scan timestamp lock poisoned") = scanned_at;
}

/// Returns the last scan timestamp.
pub fn library_scanned_at() -> u64 {
    *app_state()
        .library_scanned_at
        .lock()
        .expect("scan timestamp lock poisoned")
}
