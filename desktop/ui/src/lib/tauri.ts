/** True when the UI runs inside the Tauri webview. */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** Opens a URL in the system default browser using Tauri's opener plugin. */
export async function openUrl(url: string): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('plugin:opener|open_url', { url })
  } catch (err) {
    // Fallback: try window.open for external URLs
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}

async function appWindow() {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  return getCurrentWindow()
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
