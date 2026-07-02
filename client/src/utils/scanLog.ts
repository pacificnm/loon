import type { ScanProgress, ScanStreamEvent } from '../api/types';

function timestamp(): string {
  return new Date().toLocaleTimeString();
}

function formatPhase(phase: ScanProgress['phase']): string {
  if (!phase) {
    return 'working';
  }
  return phase.replace('_', ' ');
}

/** Human-readable line for one scan SSE event. */
export function formatScanEvent(event: ScanStreamEvent): string {
  switch (event.type) {
    case 'started':
      return `[${timestamp()}] Scan started (${event.scan_id})`;
    case 'progress': {
      const progress = event.progress;
      let line = `[${timestamp()}] ${formatPhase(progress.phase)}: files ${progress.files_seen}, candidates ${progress.candidates}`;
      if (progress.errors > 0) {
        line += `, errors ${progress.errors}`;
      }
      if (progress.phase === 'enriching' && progress.total_to_enrich > 0) {
        line += `, enriched ${progress.enriched}/${progress.total_to_enrich}`;
      }
      if (progress.current_path) {
        line += ` — ${progress.current_path}`;
      }
      return line;
    }
    case 'complete':
      return `[${timestamp()}] Complete: ${event.movies_count} movies in ${event.duration_secs}s (files ${event.stats.files_seen}, candidates ${event.stats.candidates}, errors ${event.stats.errors})`;
    case 'error':
      return `[${timestamp()}] Error: ${event.message}`;
    default:
      return `[${timestamp()}] ${JSON.stringify(event)}`;
  }
}

/** Snapshot line for polled library status progress. */
export function formatScanProgress(progress: ScanProgress): string {
  let line = `${formatPhase(progress.phase)}: files ${progress.files_seen}, candidates ${progress.candidates}`;
  if (progress.phase === 'enriching' && progress.total_to_enrich > 0) {
    line += `, enriched ${progress.enriched}/${progress.total_to_enrich}`;
  }
  if (progress.current_path) {
    line += ` — ${progress.current_path}`;
  }
  return line;
}
