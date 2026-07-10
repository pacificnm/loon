import { useEffect, useState, type ReactNode } from 'react'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faMinus,
  faWindowMaximize,
  faWindowRestore,
  faXmark,
} from '@fortawesome/free-solid-svg-icons'
import {
  closeWindow,
  isTauri,
  minimizeWindow,
  toggleMaximizeWindow,
} from '../lib/tauri'

const CONTROL_WIDTH = 46

/** Minimize / maximize / close for frameless Tauri windows. */
export function WindowControls() {
  const [maximized, setMaximized] = useState(false)
  const tauri = isTauri()

  useEffect(() => {
    if (!tauri) return

    let cancelled = false
    let unlisten: (() => void) | undefined

    void (async () => {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const win = getCurrentWindow()
      const current = await win.isMaximized()
      if (!cancelled) setMaximized(current)
      unlisten = await win.onResized(async () => {
        const next = await win.isMaximized()
        if (!cancelled) setMaximized(next)
      })
    })()

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [tauri])

  if (!tauri) return null

  return (
    <div className="flex h-full items-stretch">
      <ControlButton label="Minimize" onClick={() => void minimizeWindow()}>
        <FontAwesomeIcon icon={faMinus} className="h-3 w-3" />
      </ControlButton>
      <ControlButton
        label={maximized ? 'Restore' : 'Maximize'}
        onClick={() => void toggleMaximizeWindow()}
      >
        <FontAwesomeIcon
          icon={maximized ? faWindowRestore : faWindowMaximize}
          className="h-3 w-3"
        />
      </ControlButton>
      <ControlButton
        label="Close"
        onClick={() => void closeWindow()}
        danger
      >
        <FontAwesomeIcon icon={faXmark} className="h-3.5 w-3.5" />
      </ControlButton>
    </div>
  )
}

function ControlButton({
  label,
  onClick,
  danger,
  children,
}: {
  label: string
  onClick: () => void
  danger?: boolean
  children: ReactNode
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      style={{ width: CONTROL_WIDTH }}
      className={[
        'flex h-full items-center justify-center transition-colors',
        danger
          ? 'text-loon-fg/90 hover:bg-red-600 hover:text-white'
          : 'text-loon-fg/85 hover:bg-loon-border/60 hover:text-loon-fg',
      ].join(' ')}
    >
      {children}
    </button>
  )
}
