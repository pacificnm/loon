import { useEffect } from 'react'
import { useScan } from '../hooks/useScan'

export function ScanPanel() {
  const {
    status,
    isScanning,
    progress,
    error,
    startScan,
    refreshStatus,
    clearError,
  } = useScan()

  useEffect(() => {
    refreshStatus()
  }, [refreshStatus])

  const handleIncrementalScan = () => startScan(false)
  const handleFullScan = () => startScan(true)

  const formatPhase = (phase: string | null): string => {
    switch (phase) {
      case 'discovering':
        return 'Discovering files'
      case 'enriching':
        return 'Enriching metadata'
      case 'persisting':
        return 'Saving results'
      default:
        return 'Idle'
    }
  }

  const formatDuration = (secs: number): string => {
    const mins = Math.floor(secs / 60)
    const remainingSecs = secs % 60
    if (mins > 0) {
      return `${mins}m ${remainingSecs}s`
    }
    return `${secs}s`
  }

  return (
    <div className="rounded-loon-lg border border-loon-border bg-loon-surface p-6">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-lg font-medium text-loon-fg">Scan</h2>
        {status && (
          <span
            className={`rounded-full px-3 py-1 text-xs ${
              isScanning
                ? 'bg-loon-primary/10 text-loon-primary'
                : 'bg-loon-muted/10 text-loon-muted'
            }`}
          >
            {isScanning ? 'Scanning' : 'Idle'}
          </span>
        )}
      </div>

      <p className="mb-4 text-sm text-loon-muted">
        Scan your media library for new or changed files.
      </p>

      {error && (
        <div className="mb-4 rounded-loon-md border border-loon-error/20 bg-loon-error/10 p-3">
          <p className="text-sm text-loon-error">{error}</p>
          <button
            onClick={clearError}
            className="mt-2 text-xs text-loon-error hover:underline"
          >
            Dismiss
          </button>
        </div>
      )}

      <div className="mb-4 flex gap-2">
        <button
          onClick={handleIncrementalScan}
          disabled={isScanning}
          className="rounded-loon-md bg-loon-primary px-4 py-2 text-sm font-medium text-loon-primary-fg hover:bg-loon-primary/90 disabled:opacity-50"
        >
          {isScanning ? 'Scanning...' : 'Scan for Changes'}
        </button>
        <button
          onClick={handleFullScan}
          disabled={isScanning}
          className="rounded-loon-md border border-loon-border bg-loon-bg px-4 py-2 text-sm font-medium text-loon-fg hover:bg-loon-bg/80 disabled:opacity-50"
        >
          Full Refresh
        </button>
      </div>

      {isScanning && progress && (
        <div className="rounded-loon-md bg-loon-bg p-4">
          <div className="mb-3 flex items-center justify-between">
            <span className="text-sm font-medium text-loon-fg">
              {formatPhase(progress.phase)}
            </span>
            {progress.total_to_enrich > 0 && (
              <span className="text-xs text-loon-muted">
                {progress.enriched}/{progress.total_to_enrich} enriched
              </span>
            )}
          </div>

          {progress.total_to_enrich > 0 && (
            <div className="mb-3 h-2 w-full overflow-hidden rounded-full bg-loon-border">
              <div
                className="h-full bg-loon-primary transition-all"
                style={{
                  width: `${
                    (progress.enriched / progress.total_to_enrich) * 100
                  }%`,
                }}
              />
            </div>
          )}

          <div className="grid grid-cols-3 gap-4 text-xs">
            <div>
              <span className="text-loon-muted">Files seen</span>
              <p className="font-medium text-loon-fg">
                {progress.files_seen.toLocaleString()}
              </p>
            </div>
            <div>
              <span className="text-loon-muted">Candidates</span>
              <p className="font-medium text-loon-fg">
                {progress.candidates.toLocaleString()}
              </p>
            </div>
            <div>
              <span className="text-loon-muted">Errors</span>
              <p className="font-medium text-loon-fg">
                {progress.errors.toLocaleString()}
              </p>
            </div>
          </div>

          {progress.current_path && (
            <div className="mt-3 truncate text-xs text-loon-muted">
              Processing: {progress.current_path}
            </div>
          )}
        </div>
      )}

      {!isScanning && status && (
        <div className="rounded-loon-md bg-loon-bg p-4 text-sm">
          <div className="flex justify-between">
            <span className="text-loon-muted">Last scan</span>
            <span className="text-loon-fg">
              {status.last_scan_at
                ? new Date(status.last_scan_at).toLocaleString()
                : 'Never'}
            </span>
          </div>
          {status.last_scan_duration_secs > 0 && (
            <div className="mt-2 flex justify-between">
              <span className="text-loon-muted">Duration</span>
              <span className="text-loon-fg">
                {formatDuration(status.last_scan_duration_secs)}
              </span>
            </div>
          )}
          <div className="mt-2 flex justify-between">
            <span className="text-loon-muted">Movies</span>
            <span className="text-loon-fg">
              {status.movies_count.toLocaleString()}
            </span>
          </div>
        </div>
      )}
    </div>
  )
}
