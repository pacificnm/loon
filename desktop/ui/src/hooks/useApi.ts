import { useState, useEffect, useCallback } from 'react'
import { fetchHealth, fetchMovies } from '../lib/api'
import { loadDesktopConfig } from '../lib/config'
import type { MovieSummary } from '../types'

export function useApi() {
  const [serverUrl, setServerUrl] = useState<string | null>(null)
  const [movies, setMovies] = useState<MovieSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refreshMovies = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const config = await loadDesktopConfig()
      setServerUrl(config.serverUrl)
      await fetchHealth()
      const list = await fetchMovies()
      setMovies(list)
    } catch (err: unknown) {
      const detail = err instanceof Error ? err.message : String(err)
      setError(detail)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refreshMovies()
  }, [refreshMovies])

  return { movies, loading, error, refreshMovies, serverUrl }
}
