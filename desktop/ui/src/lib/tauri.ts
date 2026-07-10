/** True when the UI runs inside the Tauri webview. */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** Launches ffplay/mpv for a movie stream via the Tauri host. */
export async function playStream(slug: string, title?: string): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('plugin:loon|play_stream', { slug, title: title ?? null })
}

async function appWindow() {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  return getCurrentWindow()
}

export async function hideWindow(): Promise<void> {
  if (!isTauri()) return
  await (await appWindow()).hide()
}

export async function closeWindow(): Promise<void> {
  if (!isTauri()) return
  await (await appWindow()).close()
}

export async function minimizeWindow(): Promise<void> {
  if (!isTauri()) return
  await (await appWindow()).minimize()
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (!isTauri()) return
  await (await appWindow()).toggleMaximize()
}

/** Opens an https URL in the system default browser. */
export async function openUrl(url: string): Promise<void> {
  if (isTauri()) {
    const { openUrl: open } = await import('@tauri-apps/plugin-opener')
    await open(url)
    return
  }
  window.open(url, '_blank', 'noopener,noreferrer')
}

/** Returns the desktop app version when running inside Tauri. */
export async function getAppVersion(): Promise<string | null> {
  if (!isTauri()) return null
  try {
    const app = await import('@tauri-apps/api/app')
    return await app.getVersion()
  } catch {
    return null
  }
}
