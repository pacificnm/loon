import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
  fetchLibraryStatus,
  LoonApiError,
  streamLibraryScan,
} from '../api/client';
import type { LibraryStatusResponse } from '../api/types';
import { FocusButton } from '../components/FocusButton';
import { getServerUrl } from '../config';
import { formatScanEvent, formatScanProgress } from '../utils/scanLog';
import adminStyles from './AdminPage.module.css';
import styles from './page.module.css';

export function AdminPage() {
  const server = getServerUrl();
  const [status, setStatus] = useState<LibraryStatusResponse | null>(null);
  const [logLines, setLogLines] = useState<string[]>([]);
  const [streaming, setStreaming] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);
  const abortRef = useRef<AbortController | null>(null);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'admin-page',
    preferredChildFocusKey: 'admin-scan',
  });

  const appendLog = useCallback((line: string) => {
    setLogLines((previous) => [...previous, line]);
  }, []);

  const refreshStatus = useCallback(async () => {
    const next = await fetchLibraryStatus(server);
    setStatus(next);
    return next;
  }, [server]);

  useEffect(() => {
    void refreshStatus().catch((error: unknown) => {
      appendLog(error instanceof Error ? error.message : 'Failed to load library status');
    });
  }, [appendLog, refreshStatus]);

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logLines]);

  useEffect(() => {
    return () => {
      abortRef.current?.abort();
    };
  }, []);

  useEffect(() => {
    if (!status?.scan_in_progress || streaming) {
      return;
    }

    const intervalId = window.setInterval(() => {
      void refreshStatus().catch(() => {
        /* polling errors are non-fatal */
      });
    }, 2000);

    return () => window.clearInterval(intervalId);
  }, [refreshStatus, status?.scan_in_progress, streaming]);

  const runScan = useCallback(
    async (full: boolean) => {
      if (streaming) {
        return;
      }

      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;
      setStreaming(true);
      appendLog(full ? '--- Starting full metadata scan ---' : '--- Starting library scan ---');

      try {
        await streamLibraryScan(
          server,
          { full, signal: controller.signal },
          (event) => {
            appendLog(formatScanEvent(event));
          },
        );
        await refreshStatus();
      } catch (error) {
        if (error instanceof LoonApiError && error.code === 'scan_already_running') {
          appendLog('Scan already running on server');
          await refreshStatus();
        } else if (error instanceof Error && error.name === 'AbortError') {
          appendLog('Scan stream cancelled');
        } else {
          appendLog(error instanceof Error ? error.message : 'Scan failed');
        }
      } finally {
        setStreaming(false);
        abortRef.current = null;
      }
    },
    [appendLog, refreshStatus, server, streaming],
  );

  const lastScanLabel =
    status?.last_scan_at && status.last_scan_at !== '0'
      ? status.last_scan_at
      : 'never';

  const showRemoteProgress = Boolean(status?.scan_in_progress && !streaming && status.progress);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>Admin</h1>
      <div className={styles.content}>
        <FocusContext.Provider value={focusKey}>
          <div ref={ref} className={adminStyles.panel}>
            <div className={adminStyles.statusBar}>
              <span>
                <span className={adminStyles.statusLabel}>State: </span>
                <span
                  className={
                    status?.scan_in_progress ? adminStyles.statusScanning : adminStyles.statusValue
                  }
                >
                  {status?.state ?? '…'}
                </span>
              </span>
              <span>
                <span className={adminStyles.statusLabel}>Movies: </span>
                <span className={adminStyles.statusValue}>{status?.movies_count ?? '…'}</span>
              </span>
              <span>
                <span className={adminStyles.statusLabel}>Last scan: </span>
                <span className={adminStyles.statusValue}>{lastScanLabel}</span>
              </span>
              {status && status.last_scan_duration_secs > 0 ? (
                <span>
                  <span className={adminStyles.statusLabel}>Duration: </span>
                  <span className={adminStyles.statusValue}>
                    {status.last_scan_duration_secs}s
                  </span>
                </span>
              ) : null}
            </div>

            {showRemoteProgress && status?.progress ? (
              <p className={adminStyles.liveProgress}>
                Scan in progress — {formatScanProgress(status.progress)}
              </p>
            ) : null}

            <div className={adminStyles.actions}>
              <FocusButton
                focusKey="admin-scan"
                label={streaming ? 'Scanning…' : 'Scan library'}
                onPress={() => void runScan(false)}
              />
              <FocusButton
                focusKey="admin-scan-full"
                label="Full metadata scan"
                onPress={() => void runScan(true)}
              />
              <FocusButton
                focusKey="admin-refresh"
                label="Refresh status"
                onPress={() => {
                  void refreshStatus().catch((error: unknown) => {
                    appendLog(
                      error instanceof Error ? error.message : 'Failed to refresh status',
                    );
                  });
                }}
              />
              <FocusButton
                focusKey="admin-clear"
                label="Clear log"
                onPress={() => setLogLines([])}
              />
            </div>

            <div ref={logRef} className={adminStyles.log} aria-live="polite">
              {logLines.length === 0 ? (
                <p className={adminStyles.logEmpty}>
                  Scan output will appear here. Use Scan library for an incremental rescan, or
                  Full metadata scan to re-fetch TMDB data for every movie.
                </p>
              ) : (
                logLines.map((line, index) => (
                  <div key={`${index}-${line}`} className={adminStyles.logLine}>
                    {line}
                  </div>
                ))
              )}
            </div>
          </div>
        </FocusContext.Provider>
      </div>
    </div>
  );
}
