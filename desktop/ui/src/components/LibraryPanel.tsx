import { useState, useMemo } from 'react'
import type { MovieSummary, MovieDetail } from '../types'
import { MovieTable } from './MovieTable'
import { MovieDetail as MovieDetailView } from './MovieDetail'

interface LibraryPanelProps {
  movies: MovieSummary[]
  loading: boolean
  error: string | null
  selectedMovie: MovieDetail | null
  movieLoading: boolean
  onMovieSelect: (slug: string) => void
  onBack: () => void
  onRefresh: () => void
  onMovieUpdated?: (movie: MovieDetail) => void
}

export function LibraryPanel({
  movies,
  loading,
  error,
  selectedMovie,
  movieLoading,
  onMovieSelect,
  onBack,
  onRefresh,
  onMovieUpdated,
}: LibraryPanelProps) {
  const [searchQuery, setSearchQuery] = useState('')

  const filteredMovies = useMemo(() => {
    if (!searchQuery.trim()) return movies
    const query = searchQuery.toLowerCase().trim()
    return movies.filter(
      (movie) =>
        movie.title.toLowerCase().includes(query) ||
        movie.relative_path.toLowerCase().includes(query) ||
        (movie.year && movie.year.toString().includes(query)),
    )
  }, [movies, searchQuery])

  if (selectedMovie) {
    if (movieLoading) {
      return (
        <div className="flex h-full items-center justify-center">
          <div className="text-loon-muted">Loading movie details...</div>
        </div>
      )
    }
    return (
      <MovieDetailView
        movie={selectedMovie}
        onBack={onBack}
        onMovieUpdated={onMovieUpdated}
      />
    )
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-loon-muted">Loading movies...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-loon-error">{error}</div>
      </div>
    )
  }

  return (
    <div className="h-full">
      <div className="mb-4 flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <input
            type="text"
            placeholder="Search by title, path, or year..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 max-w-md rounded-loon-sm border border-loon-border bg-loon-bg px-3 py-1.5 text-sm text-loon-fg placeholder-loon-muted focus:outline-none focus:border-loon-primary"
          />
          {searchQuery && (
            <span className="text-xs text-loon-muted">
              {filteredMovies.length} of {movies.length}
            </span>
          )}
        </div>
        <button
          onClick={onRefresh}
          className="shrink-0 rounded-loon-sm bg-loon-primary px-3 py-1.5 text-sm font-medium text-loon-bg hover:opacity-90"
        >
          Refresh
        </button>
      </div>

      <MovieTable movies={filteredMovies} onMovieSelect={onMovieSelect} />
    </div>
  )
}
