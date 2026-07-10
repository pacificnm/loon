import { useCallback, useEffect, useRef, useState } from 'react'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faSpinner } from '@fortawesome/free-solid-svg-icons'
import { WindowControls } from './components/WindowControls'
import { loadDesktopConfig } from './lib/config'
import type { PlayerLoadPayload, PlayerRoute } from './lib/player'
import { hideWindow, isTauri } from './lib/tauri'

type PlaybackPhase = 'idle' | 'loading' | 'playing' | 'error'

const TITLE_BAR_HEIGHT = 32

interface PlayerAppProps {
  initial: PlayerRoute | null
}

export function PlayerApp({ initial }: PlayerAppProps) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const [slug, setSlug] = useState(initial?.slug ?? '')
  const [title, setTitle] = useState(initial?.title ?? 'Loon Player')
  const [streamUrl, setStreamUrl] = useState<string | null>(initial?.streamUrl ?? null)
  const [phase, setPhase] = useState<PlaybackPhase>(initial ? 'loading' : 'idle')
  const [error, setError] = useState<string | null>(null)
  const [statusText, setStatusText] = useState(
    initial ? 'Connecting to stream…' : 'Waiting for playback…',
  )

  const applyLoad = useCallback((payload: PlayerLoadPayload) => {
    setSlug(payload.slug)
    setTitle(payload.title)
    setStreamUrl(payload.streamUrl)
    setPhase('loading')
    setError(null)
    setStatusText('Loading movie…')
  }, [])

  useEffect(() => {
    if (streamUrl || !slug) {
      return
    }
    loadDesktopConfig()
      .then((config) => {
        setStreamUrl(`${config.serverUrl}/stream/${encodeURIComponent(slug)}`)
      })
      .catch((err) => {
        const message = err instanceof Error ? err.message : String(err)
        setPhase('error')
        setError(message)
      })
  }, [slug, streamUrl])

  useEffect(() => {
    if (!isTauri()) {
      return
    }
    let unlisten: (() => void) | undefined
    void import('@tauri-apps/api/event').then(({ listen }) => {
      void listen<PlayerLoadPayload>('player:load', (event) => {
        applyLoad(event.payload)
        void import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
          void getCurrentWindow().setTitle(event.payload.title)
        })
      }).then((dispose) => {
        unlisten = dispose
      })
    })
    return () => {
      unlisten?.()
    }
  }, [applyLoad])

  useEffect(() => {
    const video = videoRef.current
    if (!video || !streamUrl) {
      return
    }
    setPhase('loading')
    setError(null)
    setStatusText('Buffering video…')
    video.src = streamUrl
    void video.play().catch(() => {
      /* user can press play if autoplay is blocked */
    })
  }, [streamUrl])

  const handleClose = async () => {
    if (!isTauri()) {
      window.close()
      return
    }
    await hideWindow()
  }

  const subtitle = slug || 'Waiting for playback'

  return (
    <div className="flex h-screen flex-col bg-loon-bg text-loon-fg">
      <header
        className="relative flex shrink-0 items-stretch border-b border-loon-border bg-loon-surface text-[13px]"
        style={{ height: TITLE_BAR_HEIGHT }}
      >
        {isTauri() ? (
          <div className="min-w-0 flex-1" data-tauri-drag-region />
        ) : null}

        <div className="relative z-10 flex h-full shrink-0 items-stretch">
          {isTauri() ? <WindowControls onClose={handleClose} /> : null}
        </div>

        {isTauri() ? (
          <p
            className="pointer-events-none absolute inset-0 z-0 flex flex-col items-center justify-center px-32"
            aria-hidden
          >
            <span className="truncate text-[12px] font-medium text-loon-fg">{title}</span>
            <span className="truncate text-[10px] text-loon-muted">{subtitle}</span>
          </p>
        ) : (
          <div className="min-w-0 flex-1 px-4 flex items-center">
            <div>
              <h1 className="text-sm font-semibold text-loon-fg leading-tight">{title}</h1>
              <p className="text-[11px] text-loon-muted leading-tight">{subtitle}</p>
            </div>
          </div>
        )}
      </header>

      <div className="relative min-h-0 flex-1 bg-black">
        {streamUrl ? (
          <video
            key={streamUrl}
            ref={videoRef}
            className="h-full w-full object-contain"
            controls
            playsInline
            onLoadStart={() => {
              setPhase('loading')
              setStatusText('Loading movie…')
            }}
            onWaiting={() => {
              setPhase('loading')
              setStatusText('Buffering…')
            }}
            onCanPlay={() => {
              setPhase('playing')
              setStatusText('')
            }}
            onPlaying={() => {
              setPhase('playing')
              setStatusText('')
            }}
            onError={() => {
              setPhase('error')
              setError('Playback failed. Check the server stream and try again.')
            }}
          />
        ) : null}

        {(phase === 'loading' || phase === 'idle') && !error ? (
          <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-3 bg-black/70">
            <FontAwesomeIcon
              icon={faSpinner}
              className="h-10 w-10 animate-spin text-loon-primary"
            />
            <p className="text-sm text-loon-muted">{statusText}</p>
          </div>
        ) : null}

        {phase === 'error' && error ? (
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-black/80 px-6 text-center">
            <p className="text-sm text-loon-error">{error}</p>
            <button
              type="button"
              onClick={() => void handleClose()}
              className="rounded-loon-sm border border-loon-border px-4 py-2 text-sm hover:bg-loon-border/40"
            >
              Close
            </button>
          </div>
        ) : null}
      </div>
    </div>
  )
}
