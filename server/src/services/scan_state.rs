//! Background library scan coordination.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use nest_media_library::ScanStats;

/// High-level scan phase for progress reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanPhase {
    /// Walking the media library on disk.
    Discovering,
    /// Guessing filenames and fetching TMDB metadata.
    Enriching,
    /// Writing results to SQLite.
    Persisting,
}

/// Live scan progress counters.
#[derive(Debug, Clone, Default, serde::Serialize, PartialEq, Eq)]
pub struct ScanProgress {
    /// Current scan phase.
    pub phase: Option<ScanPhase>,
    /// Files examined so far.
    pub files_seen: u32,
    /// Candidates discovered.
    pub candidates: u32,
    /// Non-fatal errors.
    pub errors: u32,
    /// Movies enriched so far in the current phase.
    pub enriched: u32,
    /// Movies scheduled for enrichment in this scan.
    pub total_to_enrich: u32,
    /// File currently being enriched.
    pub current_path: Option<String>,
}

/// Shared scan state for status polling and single-flight enforcement.
#[derive(Debug, Default)]
pub struct ScanCoordinator {
    running: Mutex<bool>,
    last_scan_at: AtomicU64,
    last_duration_secs: AtomicU64,
    progress: Mutex<Option<ScanProgress>>,
}

impl ScanCoordinator {
    /// Attempts to mark a scan as running. Returns false if one is already active.
    pub fn try_start(&self) -> bool {
        let mut guard = self.running.lock().expect("scan mutex poisoned");
        if *guard {
            return false;
        }
        *guard = true;
        *self.progress.lock().expect("scan mutex poisoned") = Some(ScanProgress::default());
        true
    }

    /// Marks the scan finished and stores timing metadata.
    pub fn finish(&self, scanned_at: u64, started: Instant, stats: ScanStats) {
        {
            let mut guard = self.running.lock().expect("scan mutex poisoned");
            *guard = false;
        }
        *self.progress.lock().expect("scan mutex poisoned") = None;
        self.last_scan_at.store(scanned_at, Ordering::SeqCst);
        self.last_duration_secs
            .store(started.elapsed().as_secs(), Ordering::SeqCst);
        let _ = stats;
    }

    /// Returns whether a scan is currently running.
    pub fn is_running(&self) -> bool {
        *self.running.lock().expect("scan mutex poisoned")
    }

    /// Updates progress counters during a scan.
    pub fn set_progress(&self, progress: ScanProgress) {
        if let Ok(mut snapshot) = self.progress.lock() {
            *snapshot = Some(progress);
        }
    }

    /// Returns the latest progress snapshot when scanning.
    pub fn progress(&self) -> Option<ScanProgress> {
        self.progress.lock().expect("scan mutex poisoned").clone()
    }

    /// Returns the unix timestamp of the last completed scan.
    pub fn last_scan_at(&self) -> u64 {
        self.last_scan_at.load(Ordering::SeqCst)
    }

    /// Returns the duration of the last completed scan in seconds.
    pub fn last_scan_duration_secs(&self) -> u64 {
        self.last_duration_secs.load(Ordering::SeqCst)
    }
}
