import { useState } from 'react'
import type { MovieSummary } from '../types'

interface MovieTableProps {
  movies: MovieSummary[]
  onMovieSelect: (slug: string) => void
}

export function MovieTable({ movies, onMovieSelect }: MovieTableProps) {
  const [sortTitleAsc, setSortTitleAsc] = useState(true)
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null)

  const sortedMovies = [...movies].sort((a, b) => {
    const order = a.title.toLowerCase().localeCompare(b.title.toLowerCase())
    return sortTitleAsc ? order : -order
  })

  const formatFileSize = (bytes: number | null): string => {
    if (!bytes) return '—'
    const gb = bytes / (1024 * 1024 * 1024)
    if (gb >= 1) return `${gb.toFixed(2)} GB`
    const mb = bytes / (1024 * 1024)
    if (mb >= 1) return `${mb.toFixed(1)} MB`
    const kb = bytes / 1024
    if (kb >= 1) return `${kb.toFixed(0)} KB`
    return `${bytes} B`
  }

  const handleRowClick = (slug: string) => {
    setSelectedSlug(slug)
    onMovieSelect(slug)
  }

  return (
    <div className="overflow-hidden rounded-loon-md border border-loon-border bg-loon-surface">
      <div className="overflow-auto">
        <table className="w-full">
          <thead className="sticky top-0 bg-loon-surface border-b border-loon-border">
            <tr>
              <th className="px-4 py-3 text-left text-sm font-medium text-loon-muted">
                <button
                  onClick={() => setSortTitleAsc(!sortTitleAsc)}
                  className="hover:text-loon-fg"
                >
                  Title {sortTitleAsc ? '↑' : '↓'}
                </button>
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-loon-muted">
                Year
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-loon-muted">
                File
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-loon-muted">
                Size
              </th>
            </tr>
          </thead>
          <tbody>
            {sortedMovies.map((movie) => (
              <tr
                key={movie.slug}
                onClick={() => handleRowClick(movie.slug)}
                className={`cursor-pointer border-b border-loon-border/50 transition-colors ${
                  selectedSlug === movie.slug
                    ? 'bg-loon-primary/10'
                    : 'hover:bg-loon-border/30'
                }`}
              >
                <td className="px-4 py-2.5 text-sm text-loon-fg">{movie.title}</td>
                <td className="px-4 py-2.5 text-sm text-loon-muted">
                  {movie.year?.toString() || '—'}
                </td>
                <td className="px-4 py-2.5 text-sm font-mono text-loon-muted">
                  <span className="truncate block max-w-xs" title={movie.relative_path}>
                    {movie.relative_path}
                  </span>
                </td>
                <td className="px-4 py-2.5 text-sm text-loon-muted">
                  {formatFileSize(movie.size_bytes)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
