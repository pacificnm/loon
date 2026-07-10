import { useEffect, useState } from 'react'
import { loadDesktopConfig } from '../lib/config'
import { fetchHealth } from '../lib/api'

export function SettingsPanel() {
  const [serverUrl, setServerUrl] = useState<string | null>(null)
  const [configPath, setConfigPath] = useState<string | null>(null)
  const [health, setHealth] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadDesktopConfig()
      .then(async (config) => {
        setServerUrl(config.serverUrl)
        setConfigPath(config.configPath)
        const status = await fetchHealth()
        setHealth(`${status.status} (${status.movies_count} movies)`)
      })
      .catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err)
        setError(message)
      })
  }, [])

  return (
    <div className="rounded-loon-lg border border-loon-border bg-loon-surface p-6">
      <h2 className="mb-2 text-lg font-medium text-loon-fg">Settings</h2>
      <p className="mb-4 text-sm text-loon-muted">
        Loon Admin desktop connects only to the Loon backend API.
      </p>

      <div className="space-y-4">
        <div>
          <h3 className="mb-1 text-sm font-medium text-loon-muted">Config file</h3>
          <p className="font-mono text-sm text-loon-fg break-all">
            {configPath ?? '~/.config/loon/config.toml'}
          </p>
        </div>

        <div>
          <h3 className="mb-1 text-sm font-medium text-loon-muted">Backend API</h3>
          {error ? (
            <p className="text-sm text-loon-error">{error}</p>
          ) : (
            <p className="font-mono text-sm text-loon-fg">{serverUrl ?? 'Loading…'}</p>
          )}
        </div>

        <div>
          <h3 className="mb-1 text-sm font-medium text-loon-muted">Health</h3>
          <p className="font-mono text-sm text-loon-fg">{health ?? (error ? '—' : 'Checking…')}</p>
        </div>
      </div>
    </div>
  )
}
