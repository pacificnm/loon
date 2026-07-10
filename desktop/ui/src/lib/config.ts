import { invoke } from '@tauri-apps/api/core'

export interface DesktopConfig {
  serverUrl: string
  configPath: string
  playerPath?: string
}

let cached: DesktopConfig | null = null

/** Loads API base URL from ~/.config/loon/config.toml via the Tauri host. */
export async function loadDesktopConfig(): Promise<DesktopConfig> {
  if (cached) return cached
  const response = await invoke<{
    serverUrl: string
    configPath: string
    playerPath?: string | null
  }>('plugin:loon|get_config')
  if (!response.serverUrl?.trim()) {
    throw new Error('serverUrl is not configured in ~/.config/loon/config.toml')
  }
  cached = {
    serverUrl: response.serverUrl.trim().replace(/\/$/, ''),
    configPath: response.configPath,
    playerPath: response.playerPath?.trim() || undefined,
  }
  return cached
}
