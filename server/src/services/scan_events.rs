//! Scan progress events streamed to API clients.

use nest_media_library::ScanStats;
use tokio::sync::mpsc;

use crate::services::scan_state::{ScanCoordinator, ScanPhase, ScanProgress};

/// Server-sent event emitted while a library scan runs.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScanStreamEvent {
    /// Scan accepted and running.
    Started {
        /// Scan identifier.
        scan_id: String,
    },
    /// Progress snapshot.
    Progress {
        /// Live counters.
        progress: ScanProgress,
    },
    /// Scan finished successfully.
    Complete {
        /// Scan identifier.
        scan_id: String,
        /// Movies in the library after the scan.
        movies_count: usize,
        /// Wall-clock duration in seconds.
        duration_secs: u64,
        /// Final scan statistics.
        stats: ScanStats,
    },
    /// Scan failed.
    Error {
        /// Scan identifier.
        scan_id: String,
        /// Failure message.
        message: String,
    },
}

impl ScanStreamEvent {
    /// SSE event name for this payload.
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::Started { .. } => "started",
            Self::Progress { .. } => "progress",
            Self::Complete { .. } => "complete",
            Self::Error { .. } => "error",
        }
    }
}

/// Reports scan progress to the coordinator and optional SSE channel.
#[derive(Clone)]
pub struct ScanReporter {
    scan_id: String,
    coordinator: Option<std::sync::Arc<ScanCoordinator>>,
    events: Option<mpsc::Sender<ScanStreamEvent>>,
}

impl ScanReporter {
    /// Creates a reporter for one scan run.
    pub fn new(
        scan_id: impl Into<String>,
        coordinator: Option<std::sync::Arc<ScanCoordinator>>,
        events: Option<mpsc::Sender<ScanStreamEvent>>,
    ) -> Self {
        Self {
            scan_id: scan_id.into(),
            coordinator,
            events,
        }
    }

    /// Emits a started event.
    pub async fn started(&self) {
        self.emit(ScanStreamEvent::Started {
            scan_id: self.scan_id.clone(),
        })
        .await;
    }

    /// Emits a progress snapshot.
    pub async fn progress(&self, progress: ScanProgress) {
        if let Some(coordinator) = &self.coordinator {
            coordinator.set_progress(progress.clone());
        }
        self.emit(ScanStreamEvent::Progress { progress }).await;
    }

    /// Emits a successful completion event.
    pub async fn complete(&self, movies_count: usize, duration_secs: u64, stats: ScanStats) {
        self.emit(ScanStreamEvent::Complete {
            scan_id: self.scan_id.clone(),
            movies_count,
            duration_secs,
            stats,
        })
        .await;
    }

    /// Emits a failure event.
    pub async fn error(&self, message: impl Into<String>) {
        self.emit(ScanStreamEvent::Error {
            scan_id: self.scan_id.clone(),
            message: message.into(),
        })
        .await;
    }

    async fn emit(&self, event: ScanStreamEvent) {
        if let Some(sender) = &self.events {
            let _ = sender.send(event).await;
        }
    }
}

/// Builds a progress snapshot for a scan phase.
pub fn scan_progress(
    phase: ScanPhase,
    stats: &ScanStats,
    enriched: u32,
    total_to_enrich: u32,
    current_path: Option<String>,
) -> ScanProgress {
    ScanProgress {
        phase: Some(phase),
        files_seen: stats.files_seen,
        candidates: stats.candidates,
        errors: stats.errors,
        enriched,
        total_to_enrich,
        current_path,
    }
}
