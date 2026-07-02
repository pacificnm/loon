//! Library scan handlers.

use std::convert::Infallible;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use nest_http_serve::{HttpResponse, HttpResult, RequestContext};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::error;

use crate::error::{invalid_request, scan_already_running};
use crate::models::{LibraryStatusResponse, ScanStartRequest};
use crate::services::scan_events::{ScanReporter, ScanStreamEvent};
use crate::services::scan_service::{load_catalog_from_db, scan_and_persist, ScanOptions};
use crate::state;

/// `POST /api/library/scan` — runs a scan and streams progress as Server-Sent Events.
pub async fn start_scan(ctx: RequestContext) -> HttpResult {
    let app = state::app_state();
    if !app.scan.try_start() {
        return Err(scan_already_running());
    }

    let full = if ctx.body().is_empty() {
        false
    } else {
        let body: ScanStartRequest = ctx
            .json()
            .map_err(|_| invalid_request("invalid JSON body"))?;
        body.full
    };

    let scan_id = format!("scan-{}", now_secs());
    let (events_tx, events_rx) = mpsc::channel::<ScanStreamEvent>(64);

    let config = app.config.clone();
    let repo = app.repo.clone();
    let scan = app.scan.clone();
    let tmdb = app.tmdb.clone();
    let ai = app.ai.clone();
    let artwork = app.artwork.clone();
    let reporter = ScanReporter::new(scan_id.clone(), Some(scan.clone()), Some(events_tx));

    tokio::spawn(async move {
        reporter.started().await;
        let started = Instant::now();
        let result = scan_and_persist(
            &config,
            &repo,
            tmdb.as_ref(),
            ai.as_ref(),
            ScanOptions {
                full_metadata: full,
            },
            Some(&reporter),
            artwork.as_ref(),
        )
        .await;

        match result {
            Ok(result) => {
                if let Ok(catalog) = load_catalog_from_db(&repo) {
                    state::replace_catalog(catalog);
                    state::set_library_scanned_at(result.scanned_at);
                }
                scan.finish(result.scanned_at, started, result.stats.clone());
                reporter
                    .complete(
                        result.movies_count,
                        started.elapsed().as_secs(),
                        result.stats,
                    )
                    .await;
            }
            Err(err) => {
                error!(error = %err, "background library scan failed");
                scan.finish(now_secs(), started, Default::default());
                reporter.error(err.to_string()).await;
            }
        }
    });

    let stream = ReceiverStream::new(events_rx).map(|event| -> Result<Bytes, Infallible> {
        let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
        Ok(Bytes::from(format!(
            "event: {}\ndata: {}\n\n",
            event.event_name(),
            payload
        )))
    });

    HttpResponse::event_stream(stream)
}

/// `GET /api/library/status`
pub async fn library_status(_ctx: RequestContext) -> HttpResult {
    let app = state::app_state();
    let movies_count = app.repo.movie_count().map_err(map_repo_error)?;
    let scan_in_progress = app.scan.is_running();
    let last_scan_at = app.scan.last_scan_at();
    let last_scan_duration_secs = app.scan.last_scan_duration_secs();

    nest_http_serve::Json(LibraryStatusResponse {
        state: if scan_in_progress {
            "scanning".into()
        } else {
            "idle".into()
        },
        last_scan_at: if last_scan_at > 0 {
            Some(iso_timestamp(last_scan_at))
        } else {
            None
        },
        last_scan_duration_secs,
        movies_count,
        scan_in_progress,
        progress: app.scan.progress(),
    })
    .into_response()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn iso_timestamp(secs: u64) -> String {
    format!("{secs}")
}

fn map_repo_error(error: nest_error::NestError) -> nest_http_serve::ServeError {
    nest_http_serve::ServeError::from(nest_error::NestError::data(error.to_string()))
}
