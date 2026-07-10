import { useState } from 'react'
import { AppShell } from './components/AppShell'
import { LibraryPanel } from './components/LibraryPanel'
import { ScanPanel } from './components/ScanPanel'
import { SettingsPanel } from './components/SettingsPanel'
import { useApi } from './hooks/useApi'
import { fetchMovieDetail } from './lib/api'
import type { MovieDetail } from './types'

type Section = 'library' | 'scan' | 'settings'

export function App() {
  const [activeSection, setActiveSection] = useState<Section>('library')
  const [selectedMovie, setSelectedMovie] = useState<MovieDetail | null>(null)
  const [movieLoading, setMovieLoading] = useState(false)
  const { movies, loading, error, refreshMovies } = useApi()

  const handleMovieSelect = async (slug: string) => {
    setMovieLoading(true)
    try {
      const movie = await fetchMovieDetail(slug)
      setSelectedMovie(movie)
    } catch (err) {
      console.error('Failed to fetch movie details:', err)
    } finally {
      setMovieLoading(false)
    }
  }

  const handleBack = () => {
    setSelectedMovie(null)
  }

  const renderContent = () => {
    switch (activeSection) {
      case 'library':
        return (
          <LibraryPanel
            movies={movies}
            loading={loading}
            error={error}
            selectedMovie={selectedMovie}
            movieLoading={movieLoading}
            onMovieSelect={handleMovieSelect}
            onBack={handleBack}
            onRefresh={refreshMovies}
          />
        )
      case 'scan':
        return <ScanPanel />
      case 'settings':
        return <SettingsPanel />
      default:
        return null
    }
  }

  return (
    <AppShell
      activeSection={activeSection}
      onSectionChange={setActiveSection}
      title="Loon Admin"
      subtitle="Desktop library manager"
    >
      {renderContent()}
    </AppShell>
  )
}
