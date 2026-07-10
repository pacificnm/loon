import { useEffect, useId, useState } from 'react'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faPenToSquare, faXmark } from '@fortawesome/free-solid-svg-icons'
import type { MovieDetail } from '../types'
import { setMovieTmdbMatch } from '../lib/api'
import { openUrl } from '../lib/tauri'

export interface TmdbEditDialogProps {
  open: boolean
  movie: MovieDetail
  onClose: () => void
  onSaved: (updated: MovieDetail) => void
}

function parseTmdbNumericId(raw: string): number | null {
  const trimmed = raw.trim()
  if (!trimmed) return null
  const numeric = trimmed.startsWith('tmdb:') ? trimmed.slice('tmdb:'.length) : trimmed
  const id = Number.parseInt(numeric.trim(), 10)
  return Number.isFinite(id) && id > 0 ? id : null
}

function tmdbMovieUrl(id: number): string {
  return `https://www.themoviedb.org/movie/${id}`
}

export function TmdbEditDialog({ open, movie, onClose, onSaved }: TmdbEditDialogProps) {
  const titleId = useId()
  const descId = useId()
  const [tmdbId, setTmdbId] = useState(movie.tmdb_id ?? '')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    setTmdbId(movie.tmdb_id ?? '')
    setError(null)
    setSaving(false)
  }, [open, movie.tmdb_id])

  useEffect(() => {
    if (!open) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !saving) {
        onClose()
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [open, onClose, saving])

  if (!open) {
    return null
  }

  const linkedId = parseTmdbNumericId(tmdbId)
  const tmdbLink = linkedId ? tmdbMovieUrl(linkedId) : null

  const handleOpenTmdb = async () => {
    if (!tmdbLink) return
    try {
      await openUrl(tmdbLink)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      setError(`Failed to open browser: ${msg}`)
    }
  }

  const handleSave = async () => {
    const trimmed = tmdbId.trim()
    if (!trimmed) {
      setError('Enter a TMDB movie id')
      return
    }
    if (!parseTmdbNumericId(trimmed)) {
      setError('TMDB id must be a numeric movie id (for example 348)')
      return
    }

    setSaving(true)
    setError(null)
    try {
      const updated = await setMovieTmdbMatch(movie.slug, trimmed)
      onSaved(updated)
      onClose()
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      setError(msg.includes('tmdb_not_configured')
        ? 'TMDB is not configured on the server'
        : `Failed to update TMDB match: ${msg}`)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={() => {
        if (!saving) onClose()
      }}
      role="presentation"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descId}
        className="w-full max-w-md overflow-hidden rounded-loon-lg border border-loon-border bg-loon-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-loon-border px-5 py-3">
          <div className="flex items-center gap-2">
            <FontAwesomeIcon icon={faPenToSquare} className="h-4 w-4 text-loon-accent" />
            <h2 id={titleId} className="text-sm font-semibold text-loon-fg">
              Edit TMDB match
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={saving}
            className="rounded-loon-sm p-1 text-loon-muted hover:bg-loon-border/50 hover:text-loon-fg disabled:opacity-50"
            aria-label="Close"
          >
            <FontAwesomeIcon icon={faXmark} className="h-3.5 w-3.5" />
          </button>
        </header>

        <div id={descId} className="space-y-4 px-5 py-5">
          <p className="text-sm text-loon-muted">{movie.title}</p>

          <div className="space-y-2">
            <label htmlFor="tmdb-edit-input" className="text-sm font-medium text-loon-fg">
              TMDB movie id
            </label>
            <p className="text-xs text-loon-muted">
              Enter the numeric id from themoviedb.org (for example 348 for Alien).
            </p>
            <input
              id="tmdb-edit-input"
              type="text"
              inputMode="numeric"
              placeholder="348"
              value={tmdbId}
              disabled={saving}
              autoFocus
              onChange={(event) => setTmdbId(event.target.value)}
              className="w-full rounded-loon-sm border border-loon-border bg-loon-bg px-3 py-2 text-sm text-loon-fg placeholder-loon-muted focus:border-loon-primary focus:outline-none disabled:opacity-50"
            />
          </div>

          {tmdbLink ? (
            <button
              type="button"
              onClick={() => void handleOpenTmdb()}
              disabled={saving}
              className="text-sm text-loon-accent hover:underline disabled:opacity-50"
            >
              Open on TMDB
            </button>
          ) : (
            <p className="text-xs text-loon-muted">
              Enter a valid id to open the movie page in your browser.
            </p>
          )}

          {error ? (
            <p className="rounded-loon-sm border border-loon-error/20 bg-loon-error/10 px-3 py-2 text-sm text-loon-error">
              {error}
            </p>
          ) : null}
        </div>

        <footer className="flex justify-end gap-2 border-t border-loon-border px-5 py-3">
          <button
            type="button"
            onClick={onClose}
            disabled={saving}
            className="rounded-loon-sm border border-loon-border px-4 py-2 text-sm font-medium text-loon-fg hover:bg-loon-border/50 disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void handleSave()}
            disabled={saving}
            className="rounded-loon-sm bg-loon-primary px-4 py-2 text-sm font-medium text-loon-bg hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </footer>
      </div>
    </div>
  )
}
