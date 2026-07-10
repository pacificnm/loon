import { useEffect, useState } from 'react'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faChevronLeft, faPlay, faHeart, faPenToSquare } from '@fortawesome/free-solid-svg-icons'
import type { MovieDetail as MovieDetailType } from '../types'
import { loadDesktopConfig } from '../lib/config'
import { setFavorite } from '../lib/api'
import { openUrl } from '../lib/tauri'

export interface MovieDetailProps {
  movie: MovieDetailType
  onBack: () => void
}

export function MovieDetail({ movie, onBack }: MovieDetailProps) {
  const [serverUrl, setServerUrl] = useState<string>('')
  const [isFavorite, setIsFavorite] = useState<boolean>(movie.is_favorite)
  const [favoriteLoading, setFavoriteLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadDesktopConfig()
      .then(config => {
        console.log('Loaded server URL:', config.serverUrl)
        setServerUrl(config.serverUrl)
      })
      .catch(err => setError(`Failed to load config: ${err}`))
  }, [])

  const posterUrl = movie.poster_url && serverUrl ? `${serverUrl}${movie.poster_url}` : null
  const backdropUrl = movie.backdrop_url && serverUrl ? `${serverUrl}${movie.backdrop_url}` : null

  const handlePlay = async () => {
    if (!serverUrl) {
      setError('Server URL not loaded yet')
      return
    }
    try {
      const streamUrl = `${serverUrl}/stream/${movie.slug}`
      console.log('Opening stream:', streamUrl)
      await openUrl(streamUrl)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      setError(`Failed to open stream: ${msg}`)
    }
  }

  const handleToggleFavorite = async () => {
    if (!serverUrl) {
      setError('Server URL not loaded yet')
      return
    }
    setFavoriteLoading(true)
    try {
      console.log('Toggling favorite for:', movie.slug, 'to:', !isFavorite)
      await setFavorite(movie.slug, !isFavorite)
      setIsFavorite(!isFavorite)
      setError(null)
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      console.error('Favorite error:', err)
      setError(`Failed to toggle favorite: ${msg}`)
    } finally {
      setFavoriteLoading(false)
    }
  }

  const handleEdit = async () => {
    if (!serverUrl) {
      setError('Server URL not loaded yet')
      return
    }
    // Open TMDB match dialog - for now just log
    console.log('Edit movie:', movie.slug)
    setError('Edit not implemented yet')
  }

  const formatRuntime = (minutes: number): string => {
    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60
    return `${hours}h ${mins}m`
  }

  const formatMetaLine = (): string | null => {
    const parts: string[] = []
    if (movie.year) parts.push(movie.year.toString())
    if (movie.runtime_minutes > 0) parts.push(formatRuntime(movie.runtime_minutes))
    if (movie.genres.length > 0) parts.push(movie.genres.join(' · '))
    return parts.length > 0 ? parts.join(' · ') : null
  }

  const crewLines = (): Array<{ label: string; names: string }> => {
    const lines: Array<{ label: string; names: string }> = []
    
    const directors = movie.crew
      .filter(m => m.job?.toLowerCase() === 'director')
      .map(m => m.name)
    
    if (directors.length > 0) {
      lines.push({ label: 'Director', names: directors.join(', ') })
    }

    const producers = movie.crew
      .filter(m => {
        const job = m.job?.toLowerCase()
        return job === 'producer' || job === 'executive producer'
      })
      .map(m => m.name)
    
    if (producers.length > 0) {
      lines.push({ label: 'Producers', names: producers.join(', ') })
    }

    return lines
  }

  return (
    <div className="h-full overflow-auto">
      {error && (
        <div className="mb-4 rounded-loon-md border border-loon-error/20 bg-loon-error/10 p-3">
          <p className="text-sm text-loon-error">{error}</p>
          <button
            onClick={() => setError(null)}
            className="mt-2 text-xs text-loon-error hover:underline"
          >
            Dismiss
          </button>
        </div>
      )}

      <button
        onClick={onBack}
        className="mb-4 flex items-center gap-2 text-sm text-loon-muted hover:text-loon-fg"
      >
        <FontAwesomeIcon icon={faChevronLeft} className="w-4 h-4" />
        Back to Library
      </button>

      {/* Hero Section */}
      <div className="relative mb-6 rounded-loon-lg overflow-hidden">
        {backdropUrl && (
          <div
            className="absolute inset-0 bg-cover bg-center"
            style={{ backgroundImage: `url(${backdropUrl})` }}
          >
            <div className="absolute inset-0 bg-gradient-to-t from-loon-bg via-loon-bg/80 to-transparent" />
          </div>
        )}

        <div className="relative z-10 flex gap-6 p-6">
          {posterUrl && (
            <img
              src={posterUrl}
              alt={movie.title}
              className="h-64 w-44 rounded-loon-md object-cover shadow-lg"
            />
          )}
          
          <div className="flex flex-col justify-end">
            <h1 className="text-3xl font-semibold text-loon-fg">{movie.title}</h1>
            {movie.original_title && movie.original_title !== movie.title && (
              <p className="mt-1 text-lg text-loon-muted">{movie.original_title}</p>
            )}
            {formatMetaLine() && (
              <p className="mt-2 text-sm text-loon-accent">{formatMetaLine()}</p>
            )}
            
            <div className="mt-4 flex gap-3">
              <button
                onClick={handlePlay}
                className="flex items-center gap-2 rounded-loon-sm bg-loon-primary px-4 py-2 text-sm font-medium text-loon-bg hover:opacity-90"
              >
                <FontAwesomeIcon icon={faPlay} className="w-4 h-4" />
                Play
              </button>
              <button
                onClick={handleToggleFavorite}
                disabled={favoriteLoading}
                className="flex items-center gap-2 rounded-loon-sm border border-loon-border px-4 py-2 text-sm font-medium text-loon-fg hover:bg-loon-border/50 disabled:opacity-50"
              >
                <FontAwesomeIcon
                  icon={faHeart}
                  className={`w-4 h-4 ${isFavorite ? 'text-loon-error' : ''}`}
                />
                {favoriteLoading ? 'Saving...' : isFavorite ? 'Unfavorite' : 'Favorite'}
              </button>
              <button
                onClick={handleEdit}
                className="flex items-center gap-2 rounded-loon-sm border border-loon-border px-4 py-2 text-sm font-medium text-loon-fg hover:bg-loon-border/50"
              >
                <FontAwesomeIcon icon={faPenToSquare} className="w-4 h-4" />
                Edit
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Overview */}
      {(movie.summary || crewLines().length > 0) && (
        <div className="mb-6">
          <h2 className="mb-2 text-lg font-medium text-loon-fg">Overview</h2>
          {movie.summary && (
            <p className="text-sm text-loon-fg mb-3">{movie.summary}</p>
          )}
          {crewLines().map(({ label, names }) => (
            <div key={label} className="flex gap-3 text-sm">
              <span className="text-loon-muted">{label}</span>
              <span className="text-loon-fg">{names}</span>
            </div>
          ))}
        </div>
      )}

      {/* Cast */}
      {movie.cast.length > 0 && (
        <div className="mb-6">
          <h2 className="mb-3 text-lg font-medium text-loon-fg">Top Billed Cast</h2>
          <div className="flex gap-4 overflow-x-auto">
            {movie.cast.slice(0, 12).map((member, index) => (
              <div key={index} className="flex-shrink-0 w-32">
                {member.profile_url ? (
                  <img
                    src={member.profile_url}
                    alt={member.name}
                    className="h-48 w-full rounded-loon-md object-cover"
                  />
                ) : (
                  <div className="flex h-48 w-full items-center justify-center rounded-loon-md bg-loon-surface text-loon-muted">
                    {member.name.charAt(0)}
                  </div>
                )}
                <p className="mt-2 text-sm font-medium text-loon-fg">{member.name}</p>
                {member.character && (
                  <p className="text-xs text-loon-muted">{member.character}</p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* File Info */}
      <div>
        <h2 className="mb-3 text-lg font-medium text-loon-fg">File & Media Info</h2>
        <div className="grid grid-cols-2 gap-2 text-sm">
          {movie.file.filename && (
            <div className="flex">
              <span className="w-32 text-loon-muted">File name</span>
              <span className="text-loon-fg">{movie.file.filename}</span>
            </div>
          )}
          {movie.file.relative_path && (
            <div className="flex">
              <span className="w-32 text-loon-muted">Path</span>
              <span className="text-loon-fg">{movie.file.relative_path}</span>
            </div>
          )}
          {movie.file.extension && (
            <div className="flex">
              <span className="w-32 text-loon-muted">Format</span>
              <span className="text-loon-fg">{movie.file.extension.toUpperCase()}</span>
            </div>
          )}
          {movie.imdb_id && (
            <div className="flex">
              <span className="w-32 text-loon-muted">IMDb</span>
              <span className="text-loon-fg">{movie.imdb_id}</span>
            </div>
          )}
          {movie.is_favorite && (
            <div className="flex">
              <span className="w-32 text-loon-muted">Favorite</span>
              <span className="text-loon-fg">yes</span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
