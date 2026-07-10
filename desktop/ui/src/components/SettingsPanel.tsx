import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { loadDesktopConfig } from '../lib/config'
import { fetchHealth, fetchLibraryStatus } from '../lib/api'
import { getAppVersion, openUrl } from '../lib/tauri'
import type { HealthResponse, LibraryStatusResponse } from '../types'

function SettingsSection({
  title,
  children,
}: {
  title: string
  children: ReactNode
}) {
  return (
    <section className="rounded-loon-md bg-loon-bg p-4">
      <h3 className="mb-3 text-sm font-medium text-loon-fg">{title}</h3>
      <dl className="space-y-2 text-sm">{children}</dl>
    </section>
  )
}

function SettingsRow({
  label,
  children,
}: {
  label: string
  children: ReactNode
}) {
  return (
    <div className="grid grid-cols-[9rem_1fr] items-start gap-3">
      <dt className="text-loon-muted">{label}</dt>
      <dd className="min-w-0 text-loon-fg">{children}</dd>
    </div>
  )
}

function formatUnixTime(seconds: number | undefined): string {
  if (!seconds || seconds <= 0) return 'Never'
  return new Date(seconds * 1000).toLocaleString()
}

function formatDuration(secs: number): string {
  const mins = Math.floor(secs / 60)
  const remainingSecs = secs % 60
  if (mins > 0) return `${mins}m ${remainingSecs}s`
  return `${secs}s`
}

function playerLabel(playerPath: string | undefined): string {
  if (!playerPath) return 'Built-in player window'
  return `${playerPath} (configured, not used)`
}

export function SettingsPanel() {
  const [serverUrl, setServerUrl] = useState<string | null>(null)
  const [configPath, setConfigPath] = useState<string | null>(null)
  const [playerPath, setPlayerPath] = useState<string | undefined>(undefined)
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const [health, setHealth] = useState<HealthResponse | null>(null)
  const [libraryStatus, setLibraryStatus] = useState<LibraryStatusResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadSettings = useCallback(async (showRefresh = false) => {
    if (showRefresh) {
      setRefreshing(true)
    } else {
      setLoading(true)
    }
    setError(null)

    try {
      const [config, version] = await Promise.all([
        loadDesktopConfig(),
        getAppVersion(),
      ])
      setServerUrl(config.serverUrl)
      setConfigPath(config.configPath)
      setPlayerPath(config.playerPath)
      setAppVersion(version)

      const [healthResult, statusResult] = await Promise.all([
        fetchHealth(),
        fetchLibraryStatus(),
      ])
      setHealth(healthResult)
      setLibraryStatus(statusResult)
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      setHealth(null)
      setLibraryStatus(null)
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }, [])

  useEffect(() => {
    void loadSettings()
  }, [loadSettings])

  const connected = !error && health?.status === 'ok'
  const lastScanLabel = libraryStatus?.last_scan_at
    ? new Date(libraryStatus.last_scan_at).toLocaleString()
    : formatUnixTime(health?.library_scanned_at)

  return (
    <div className="rounded-loon-lg border border-loon-border bg-loon-surface p-6">
      <div className="mb-4 flex items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-medium text-loon-fg">Settings</h2>
          <p className="mt-1 text-sm text-loon-muted">
            Desktop connection and server status. Edit{' '}
            <code className="text-loon-accent">~/.config/loon/config.toml</code> and restart to
            change the backend URL.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span
            className={`rounded-full px-3 py-1 text-xs ${
              loading
                ? 'bg-loon-muted/10 text-loon-muted'
                : connected
                  ? 'bg-loon-success/10 text-loon-success'
                  : 'bg-loon-error/10 text-loon-error'
            }`}
          >
            {loading ? 'Loading…' : connected ? 'Connected' : 'Unreachable'}
          </span>
          <button
            type="button"
            onClick={() => void loadSettings(true)}
            disabled={loading || refreshing}
            className="rounded-loon-sm border border-loon-border px-3 py-1.5 text-sm font-medium text-loon-fg hover:bg-loon-border/50 disabled:opacity-50"
          >
            {refreshing ? 'Refreshing…' : 'Refresh'}
          </button>
        </div>
      </div>

      {error ? (
        <div className="mb-4 rounded-loon-md border border-loon-error/20 bg-loon-error/10 p-3">
          <p className="text-sm text-loon-error">{error}</p>
        </div>
      ) : null}

      <div className="space-y-4">
        <SettingsSection title="Connection">
          <SettingsRow label="Backend API">
            {serverUrl ? (
              <div className="space-y-1">
                <p className="break-all font-mono text-sm">{serverUrl}</p>
                <button
                  type="button"
                  onClick={() => void openUrl(serverUrl)}
                  className="text-xs text-loon-accent hover:underline"
                >
                  Open in browser
                </button>
              </div>
            ) : (
              'Loading…'
            )}
          </SettingsRow>
          <SettingsRow label="Health">
            {health ? (
              <span>
                {health.status}
                {health.service ? ` · ${health.service}` : ''}
                {health.version ? ` v${health.version}` : ''}
              </span>
            ) : (
              '—'
            )}
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Library">
          <SettingsRow label="Movies">
            {(libraryStatus?.movies_count ?? health?.movies_count)?.toLocaleString() ?? '—'}
          </SettingsRow>
          <SettingsRow label="Scan state">
            {libraryStatus?.scan_in_progress
              ? 'Scanning'
              : libraryStatus?.state ?? '—'}
          </SettingsRow>
          <SettingsRow label="Last scan">{lastScanLabel}</SettingsRow>
          {libraryStatus && libraryStatus.last_scan_duration_secs > 0 ? (
            <SettingsRow label="Last duration">
              {formatDuration(libraryStatus.last_scan_duration_secs)}
            </SettingsRow>
          ) : null}
        </SettingsSection>

        <SettingsSection title="Desktop">
          <SettingsRow label="App version">{appVersion ?? '0.1.0'}</SettingsRow>
          <SettingsRow label="Config file">
            <p className="break-all font-mono text-sm">
              {configPath ?? '~/.config/loon/config.toml'}
            </p>
          </SettingsRow>
          <SettingsRow label="Video player">
            <div>
              <p>{playerPath ? playerLabel(playerPath) : playerLabel(undefined)}</p>
              <p className="mt-1 text-xs text-loon-muted">
                Play opens the built-in Loon player window. Use the Scan tab to refresh library
                metadata; manual TMDB edits are preserved.
              </p>
            </div>
          </SettingsRow>
        </SettingsSection>
      </div>
    </div>
  )
}
