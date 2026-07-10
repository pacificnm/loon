# API Reference

## Library Modules

```
ui/src/lib/
├── api.ts           # Backend API functions
├── config.ts        # Desktop config loading
├── player.ts        # Player utilities
└── tauri.ts         # Tauri helpers
```

---

## api.ts - Backend API Functions

### `apiFetch`

**Signature:**
```typescript
async function apiFetch<T>(path: string, init?: RequestInit): Promise<T>
```

**Purpose:** Internal fetch wrapper with config-based URL resolution.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `string` | API endpoint path (e.g., `/api/movies`) |
| `init` | `RequestInit` | Optional fetch options |

**Returns:** `Promise<T>` - Parsed JSON response

**Error Handling:**
```typescript
if (!response.ok) {
  const body = await response.text()
  throw new Error(
    body.trim() || `HTTP ${response.status} from ${serverUrl}${path}`
  )
}
```

---

### `fetchHealth`

**Signature:**
```typescript
export async function fetchHealth(): Promise<HealthResponse>
```

**Endpoint:** `GET /api/health`

**Returns:**
```typescript
interface HealthResponse {
  status: string        // e.g., "ok"
  movies_count: number  // Total movies in library
}
```

**Usage:**
```typescript
const health = await fetchHealth()
console.log(`${health.status}: ${health.movies_count} movies`)
```

---

### `fetchMovies`

**Signature:**
```typescript
export async function fetchMovies(): Promise<MovieSummary[]>
```

**Endpoint:** `GET /api/movies`

**Returns:** `MovieSummary[]`

**Response Shape:**
```typescript
{
  movies: MovieSummary[]
}
```

**Usage:**
```typescript
const movies = await fetchMovies()
// Returns array directly, not wrapped object
```

---

### `fetchMovieDetail`

**Signature:**
```typescript
export async function fetchMovieDetail(slug: string): Promise<MovieDetail>
```

**Endpoint:** `GET /api/movies/:slug`

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | Movie slug identifier |

**Returns:** `MovieDetail` - Full movie information

**Usage:**
```typescript
const movie = await fetchMovieDetail('alien-1979')
```

---

### `setFavorite`

**Signature:**
```typescript
export async function setFavorite(
  slug: string,
  favorite: boolean
): Promise<void>
```

**Endpoint:** `PUT /api/movies/:slug/favorite`

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | Movie slug identifier |
| `favorite` | `boolean` | New favorite status |

**Request Body:**
```json
{ "favorite": true }
```

**Usage:**
```typescript
await setFavorite('alien-1979', true)
```

---

### `setMovieTmdbMatch`

**Signature:**
```typescript
export async function setMovieTmdbMatch(
  slug: string,
  tmdbId: string
): Promise<MovieDetail>
```

**Endpoint:** `PUT /api/movies/:slug/match`

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | Movie slug identifier |
| `tmdbId` | `string` | TMDB movie ID (numeric string) |

**Request Body:**
```json
{ "tmdb_id": "348" }
```

**Returns:** `MovieDetail` - Updated movie with new metadata

**Usage:**
```typescript
const updated = await setMovieTmdbMatch('alien-1979', '348')
```

---

### `getStreamUrl`

**Signature:**
```typescript
export function getStreamUrl(slug: string): string
```

**Purpose:** Constructs stream URL for a movie.

**Returns:** `/stream/:slug`

**Usage:**
```typescript
const url = getStreamUrl('alien-1979')
// Returns: "/stream/alien-1979"
```

---

### `fetchLibraryStatus`

**Signature:**
```typescript
export async function fetchLibraryStatus(): Promise<LibraryStatusResponse>
```

**Endpoint:** `GET /api/library/status`

**Returns:**
```typescript
interface LibraryStatusResponse {
  state: 'idle' | 'scanning'
  last_scan_at: string | null
  last_scan_duration_secs: number
  movies_count: number
  scan_in_progress: boolean
  progress: ScanProgress | null
}
```

---

### `startScanStream`

**Signature:**
```typescript
export async function* startScanStream(
  full: boolean = false
): AsyncGenerator<ScanStreamEvent>
```

**Endpoint:** `POST /api/library/scan`

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `full` | `boolean` | `false` | Full metadata refresh |

**Returns:** `AsyncGenerator<ScanStreamEvent>` - SSE event stream

**Event Types:**
```typescript
type ScanStreamEvent =
  | { type: 'started', scan_id?: string }
  | { type: 'progress', progress?: ScanProgress }
  | { type: 'complete', movies_count?: number, duration_secs?: number }
  | { type: 'error', message?: string }
```

**SSE Format:**
```
event: progress
data: {"phase":"enriching","files_seen":100,...}

event: complete
data: {"movies_count":50,"duration_secs":120}
```

**Usage:**
```typescript
for await (const event of startScanStream()) {
  if (event.type === 'progress') {
    console.log(`Enriched: ${event.progress?.enriched}`)
  } else if (event.type === 'complete') {
    console.log(`Scan complete: ${event.movies_count} movies`)
  }
}
```

**Implementation Details:**
```typescript
const reader = response.body?.getReader()
const decoder = new TextDecoder()
let buffer = ''

// Parse SSE format: "event: <name>\ndata: <json>\n\n"
const parts = buffer.split('\n\n')
const eventLine = lines.find(l => l.startsWith('event: '))
const dataLine = lines.find(l => l.startsWith('data: '))
```

---

## config.ts - Config Loading

### Types

```typescript
interface DesktopConfig {
  serverUrl: string
  configPath: string
  playerPath?: string
}
```

---

### `loadDesktopConfig`

**Signature:**
```typescript
export async function loadDesktopConfig(): Promise<DesktopConfig>
```

**Purpose:** Loads configuration from Tauri backend.

**Returns:** `DesktopConfig` with normalized values

**Caching:**
```typescript
let cached: DesktopConfig | null = null

export async function loadDesktopConfig(): Promise<DesktopConfig> {
  if (cached) return cached
  // ... load and cache
}
```

**Normalization:**
```typescript
cached = {
  serverUrl: response.serverUrl.trim().replace(/\/$/, ''),
  configPath: response.configPath,
  playerPath: response.playerPath?.trim() || undefined,
}
```

**Tauri Command:**
```typescript
const response = await invoke<{
  serverUrl: string
  configPath: string
  playerPath?: string | null
}>('plugin:loon|get_config')
```

**Error Conditions:**
- Tauri not initialized
- Config file missing
- `server_url` empty in config

**Usage:**
```typescript
const config = await loadDesktopConfig()
console.log(`API: ${config.serverUrl}`)
```

---

## tauri.ts - Tauri Helpers

### `isTauri`

**Signature:**
```typescript
export function isTauri(): boolean
```

**Purpose:** Detects if running inside Tauri webview.

**Detection:**
```typescript
return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
```

**Usage:**
```typescript
if (isTauri()) {
  // Use Tauri APIs
} else {
  // Use browser fallbacks
}
```

---

### `playStream`

**Signature:**
```typescript
export async function playStream(
  slug: string,
  title?: string
): Promise<void>
```

**Purpose:** Launches player window via Tauri command.

**Tauri Command:**
```typescript
const { invoke } = await import('@tauri-apps/api/core')
await invoke('plugin:loon|play_stream', { slug, title: title ?? null })
```

**Usage:**
```typescript
await playStream('alien-1979', 'Alien (1979)')
```

---

### Window Management

#### `hideWindow`

```typescript
export async function hideWindow(): Promise<void>
```

**Usage:** Player window close (hides instead of destroying)

---

#### `closeWindow`

```typescript
export async function closeWindow(): Promise<void>
```

**Usage:** Close main window

---

#### `minimizeWindow`

```typescript
export async function minimizeWindow(): Promise<void>
```

**Usage:** Minimize window to taskbar

---

#### `toggleMaximizeWindow`

```typescript
export async function toggleMaximizeWindow(): Promise<void>
```

**Usage:** Toggle between maximized and restored state

---

### `openUrl`

**Signature:**
```typescript
export async function openUrl(url: string): Promise<void>
```

**Purpose:** Opens URL in system default browser.

**Tauri Implementation:**
```typescript
if (isTauri()) {
  const { openUrl: open } = await import('@tauri-apps/plugin-opener')
  await open(url)
}
```

**Browser Fallback:**
```typescript
window.open(url, '_blank', 'noopener,noreferrer')
```

**Usage:**
```typescript
await openUrl('https://www.themoviedb.org/movie/348')
```

---

## player.ts - Player Utilities

### Types

```typescript
interface PlayerRoute {
  slug: string
  title: string
  streamUrl?: string
}

interface PlayerLoadPayload {
  slug: string
  title: string
  streamUrl: string
}
```

---

### `parsePlayerQuery`

**Signature:**
```typescript
export function parsePlayerQuery(): PlayerRoute | null
```

**Purpose:** Parses URL query parameters for player initialization.

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `slug` | Yes | Movie identifier |
| `title` | No | Display title (defaults to slug) |
| `streamUrl` | No | Pre-built stream URL |

**Implementation:**
```typescript
const params = new URLSearchParams(window.location.search)
const slug = params.get('slug')?.trim()
if (!slug) return null

const title = params.get('title')?.trim() || slug
const streamUrl = params.get('streamUrl')?.trim() || undefined

return { slug, title, streamUrl }
```

**Usage:**
```typescript
// URL: player.html?slug=alien-1979&title=Alien
const route = parsePlayerQuery()
// Returns: { slug: 'alien-1979', title: 'Alien' }
```

---

## TypeScript Types

### Movie Types

**File:** `types/index.ts`

#### `MovieSummary`

```typescript
interface MovieSummary {
  slug: string
  title: string
  year: number | null
  runtime_minutes: number
  poster_url: string | null
  backdrop_url: string | null
  summary: string | null
  relative_path: string
  size_bytes: number | null
}
```

---

#### `MovieDetail`

```typescript
interface MovieDetail extends MovieSummary {
  original_title: string | null
  genres: string[]
  cast: CastMember[]
  crew: CrewMember[]
  is_favorite: boolean
  watch_progress_seconds: number | null
  tmdb_id: string | null
  imdb_id: string | null
  file: MovieFileInfo
  stream_url: string
}
```

---

#### `CastMember`

```typescript
interface CastMember {
  name: string
  character: string | null
  profile_url: string | null
  tmdb_person_id: number | null
}
```

---

#### `CrewMember`

```typescript
interface CrewMember {
  name: string
  job: string | null
}
```

---

#### `MovieFileInfo`

```typescript
interface MovieFileInfo {
  filename: string
  relative_path: string
  extension: string | null
  size_bytes: number | null
  content_type: string | null
  modified_at: number | null
  scanned_at: number | null
}
```

---

### Scan Types

#### `ScanPhase`

```typescript
type ScanPhase = 'discovering' | 'enriching' | 'persisting'
```

---

#### `ScanProgress`

```typescript
interface ScanProgress {
  phase: ScanPhase | null
  files_seen: number
  candidates: number
  errors: number
  enriched: number
  total_to_enrich: number
  current_path: string | null
}
```

**Field Descriptions:**
| Field | Description |
|-------|-------------|
| `phase` | Current scan phase |
| `files_seen` | Total files scanned |
| `candidates` | Files identified as movies |
| `errors` | Files that failed processing |
| `enriched` | Movies with metadata fetched |
| `total_to_enrich` | Total candidates needing enrichment |
| `current_path` | File currently being processed |

---

#### `ScanStreamEvent`

```typescript
interface ScanStreamEvent {
  type: 'started' | 'progress' | 'complete' | 'error'
  scan_id?: string
  progress?: ScanProgress
  movies_count?: number
  duration_secs?: number
  stats?: ScanStats
  message?: string
}
```

---

#### `ScanStats`

```typescript
interface ScanStats {
  files_seen: number
  candidates: number
  errors: number
}
```

---

#### `LibraryStatusResponse`

```typescript
interface LibraryStatusResponse {
  state: 'idle' | 'scanning'
  last_scan_at: string | null
  last_scan_duration_secs: number
  movies_count: number
  scan_in_progress: boolean
  progress: ScanProgress | null
}
```

---

### Health Types

#### `HealthResponse`

```typescript
interface HealthResponse {
  status: string
  movies_count: number
}
```

---

## Hook Reference

### useApi

**File:** `hooks/useApi.ts`

**Signature:**
```typescript
function useApi(): {
  movies: MovieSummary[]
  loading: boolean
  error: string | null
  refreshMovies: () => Promise<void>
  serverUrl: string | null
}
```

**Internal State:**
```typescript
const [movies, setMovies] = useState<MovieSummary[]>([])
const [loading, setLoading] = useState(true)
const [error, setError] = useState<string | null>(null)
const [serverUrl, setServerUrl] = useState<string | null>(null)
```

**Refresh Flow:**
1. Set loading state
2. Load desktop config (get server URL)
3. Fetch health check (verify connection)
4. Fetch movies list
5. Update state or error

---

### useScan

**File:** `hooks/useScan.ts`

**Signature:**
```typescript
function useScan(): {
  status: LibraryStatusResponse | null
  isScanning: boolean
  progress: ScanProgress | null
  error: string | null
  startScan: (full?: boolean) => Promise<void>
  refreshStatus: () => Promise<void>
  clearError: () => void
}
```

**Internal State:**
```typescript
const [status, setStatus] = useState<LibraryStatusResponse | null>(null)
const [error, setError] = useState<string | null>(null)
const streamingRef = useRef(false)  // Prevent concurrent scans
```

**Scan Event Handling:**
```typescript
for await (const event of startScanStream(full)) {
  if (event.type === 'progress') {
    setStatus(prev => ({
      ...prev,
      state: 'scanning',
      scan_in_progress: true,
      progress: event.progress
    }))
  } else if (event.type === 'complete') {
    setStatus(prev => ({
      ...prev,
      state: 'idle',
      scan_in_progress: false,
      movies_count: event.movies_count
    }))
  } else if (event.type === 'error') {
    setError(event.message)
    setStatus(prev => prev ? { ...prev, state: 'idle' } : null)
  }
}
```

**Derived Values:**
```typescript
isScanning: status?.scan_in_progress ?? false
progress: status?.progress ?? null
```
