import type {
  HealthResponse,
  MovieSummary,
  MovieDetail,
  ScanStreamEvent,
  LibraryStatusResponse,
} from '../types'
import { loadDesktopConfig } from './config'

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const { serverUrl } = await loadDesktopConfig()
  const response = await fetch(`${serverUrl}${path}`, init)
  if (!response.ok) {
    const body = await response.text()
    throw new Error(
      body.trim() || `HTTP ${response.status} from ${serverUrl}${path}`,
    )
  }
  return response.json() as Promise<T>
}

/** Confirms the Loon backend is reachable. */
export async function fetchHealth(): Promise<HealthResponse> {
  return apiFetch<HealthResponse>('/api/health')
}

/** Lists movies from the Loon server API. */
export async function fetchMovies(): Promise<MovieSummary[]> {
  const data = await apiFetch<{ movies: MovieSummary[] }>('/api/movies')
  return data.movies ?? []
}

/** Fetches full details for a single movie by slug. */
export async function fetchMovieDetail(slug: string): Promise<MovieDetail> {
  return apiFetch<MovieDetail>(`/api/movies/${encodeURIComponent(slug)}`)
}

/** Sets the favorite status for a movie. */
export async function setFavorite(slug: string, favorite: boolean): Promise<void> {
  await apiFetch(`/api/movies/${encodeURIComponent(slug)}/favorite`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ favorite }),
  })
}

/** Gets the stream URL for a movie. */
export function getStreamUrl(slug: string): string {
  return `/stream/${slug}`
}

/** Gets the current library scan status. */
export async function fetchLibraryStatus(): Promise<LibraryStatusResponse> {
  return apiFetch<LibraryStatusResponse>('/api/library/status')
}

/**
 * Starts a library scan and streams progress events via Server-Sent Events.
 * Returns an async iterator that yields ScanStreamEvent objects.
 */
export async function* startScanStream(full: boolean = false): AsyncGenerator<ScanStreamEvent> {
  const { serverUrl } = await loadDesktopConfig()
  const response = await fetch(`${serverUrl}/api/library/scan`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ full }),
  })

  if (!response.ok) {
    const body = await response.text()
    throw new Error(
      body.trim() || `HTTP ${response.status} from ${serverUrl}/api/library/scan`,
    )
  }

  const reader = response.body?.getReader()
  if (!reader) {
    throw new Error('Response body is not readable')
  }

  const decoder = new TextDecoder()
  let buffer = ''

  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })

      // Parse SSE events (format: "event: <name>\ndata: <json>\n\n")
      const parts = buffer.split('\n\n')
      buffer = parts.pop() || ''

      for (const part of parts) {
        const lines = part.split('\n')
        const eventLine = lines.find((l) => l.startsWith('event: '))
        const dataLine = lines.find((l) => l.startsWith('data: '))

        if (eventLine && dataLine) {
          const eventType = eventLine.slice(7).trim()
          const dataStr = dataLine.slice(5).trim()
          try {
            const data = JSON.parse(dataStr) as ScanStreamEvent
            yield { ...data, type: eventType as ScanStreamEvent['type'] }
          } catch {
            // Skip malformed JSON
          }
        }
      }
    }
  } finally {
    reader.releaseLock()
  }
}
