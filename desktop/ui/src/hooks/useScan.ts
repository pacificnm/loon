import { useState, useCallback, useRef } from 'react'
import { fetchLibraryStatus, startScanStream } from '../lib/api'
import type { ScanProgress, LibraryStatusResponse } from '../types'

export interface UseScanResult {
  /** Current scan state */
  status: LibraryStatusResponse | null
  /** Whether a scan is currently running */
  isScanning: boolean
  /** Latest progress snapshot */
  progress: ScanProgress | null
  /** Error message if scan failed */
  error: string | null
  /** Start a new scan (full=false for incremental, full=true for full metadata refresh) */
  startScan: (full?: boolean) => Promise<void>
  /** Refresh the scan status from the server */
  refreshStatus: () => Promise<void>
  /** Clear any error state */
  clearError: () => void
}

export function useScan(): UseScanResult {
  const [status, setStatus] = useState<LibraryStatusResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const streamingRef = useRef(false)

  const refreshStatus = useCallback(async () => {
    try {
      const result = await fetchLibraryStatus()
      setStatus(result)
    } catch (err: unknown) {
      // Don't set error state for status polling failures - they're transient
      console.error('Failed to fetch library status:', err)
    }
  }, [])

  const startScan = useCallback(
    async (full: boolean = false) => {
      if (streamingRef.current) {
        return
      }

      streamingRef.current = true
      setError(null)

      try {
        for await (const event of startScanStream(full)) {
          if (event.type === 'progress' && event.progress) {
            setStatus((prev) => ({
              ...(prev ?? {
                state: 'scanning' as const,
                last_scan_at: null,
                last_scan_duration_secs: 0,
                movies_count: 0,
                scan_in_progress: true,
                progress: null,
              }),
              state: 'scanning',
              scan_in_progress: true,
              progress: event.progress ?? null,
            }))
          } else if (event.type === 'complete') {
            setStatus((prev) => ({
              ...(prev ?? {
                state: 'idle' as const,
                last_scan_at: null,
                last_scan_duration_secs: 0,
                movies_count: 0,
                scan_in_progress: false,
                progress: null,
              }),
              state: 'idle',
              scan_in_progress: false,
              movies_count: event.movies_count ?? prev?.movies_count ?? 0,
              last_scan_duration_secs: event.duration_secs ?? 0,
              progress: null,
            }))
            streamingRef.current = false
          } else if (event.type === 'error') {
            setError(event.message ?? 'Scan failed')
            setStatus((prev) =>
              prev
                ? { ...prev, state: 'idle', scan_in_progress: false }
                : null,
            )
            streamingRef.current = false
          }
        }
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Scan failed'
        setError(message)
        streamingRef.current = false
        // Refresh status to get accurate state after error
        await refreshStatus()
      }
    },
    [refreshStatus],
  )

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  return {
    status,
    isScanning: status?.scan_in_progress ?? false,
    progress: status?.progress ?? null,
    error,
    startScan,
    refreshStatus,
    clearError,
  }
}
