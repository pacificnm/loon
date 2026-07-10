# React Frontend Documentation

## Application Structure

```
ui/src/
├── App.tsx              # Main application component
├── PlayerApp.tsx        # Player window component
├── main.tsx             # App entry point
├── player-main.tsx      # Player entry point
├── vite-env.d.ts        # Vite type declarations
├── index.css            # Global styles (Tailwind)
├── components/          # React components
│   ├── AppShell.tsx
│   ├── WindowControls.tsx
│   ├── LibraryPanel.tsx
│   ├── MovieDetail.tsx
│   ├── MovieTable.tsx
│   ├── TmdbEditDialog.tsx
│   ├── SettingsPanel.tsx
│   └── ScanPanel.tsx
├── hooks/               # Custom React hooks
│   ├── useApi.ts
│   └── useScan.ts
├── lib/                 # Utility modules
│   ├── api.ts           # API functions
│   ├── config.ts        # Config loading
│   ├── player.ts        # Player utilities
│   └── tauri.ts         # Tauri helpers
└── types/               # TypeScript types
    └── index.ts
```

---

## Entry Points

### main.tsx - App Entry Point

**File:** `ui/src/main.tsx`

**Purpose:** Bootstraps the main React application.

```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import { App } from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

**Key Points:**
- Uses React 18's `createRoot` API
- StrictMode enabled for development checks
- Loads global CSS (Tailwind base styles)

---

### player-main.tsx - Player Entry Point

**File:** `ui/src/player-main.tsx`

**Purpose:** Bootstraps the player window React application.

```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import { PlayerApp } from './PlayerApp'
import { parsePlayerQuery } from './lib/player'
import './index.css'

const route = parsePlayerQuery()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <PlayerApp initial={route} />
  </React.StrictMode>,
)
```

**Key Points:**
- Parses URL query parameters for initial route
- Passes `initial` prop to PlayerApp for immediate stream loading

---

## Core Components

### App.tsx - Main Application

**File:** `ui/src/App.tsx`

**Purpose:** Root component managing navigation state and movie selection.

**State:**
```tsx
const [activeSection, setActiveSection] = useState<Section>('library')
const [selectedMovie, setSelectedMovie] = useState<MovieDetail | null>(null)
const [movieLoading, setMovieLoading] = useState(false)
const { movies, loading, error, refreshMovies } = useApi()
```

**Sections:**
| Section | Component | Description |
|---------|-----------|-------------|
| `library` | `LibraryPanel` | Movie browser and detail view |
| `scan` | `ScanPanel` | Library scan controls |
| `settings` | `SettingsPanel` | Configuration display |

**Key Handlers:**

#### `handleMovieSelect`
```tsx
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
```

#### `handleMovieUpdated`
```tsx
const handleMovieUpdated = (movie: MovieDetail) => {
  setSelectedMovie(movie)
  void refreshMovies()
}
```

---

### PlayerApp.tsx - Video Player

**File:** `ui/src/PlayerApp.tsx`

**Purpose:** Handles video playback in the dedicated player window.

**State Machine:**
```tsx
type PlaybackPhase = 'idle' | 'loading' | 'playing' | 'error'

const [phase, setPhase] = useState<PlaybackPhase>(initial ? 'loading' : 'idle')
const [streamUrl, setStreamUrl] = useState<string | null>(initial?.streamUrl ?? null)
const [error, setError] = useState<string | null>(null)
```

**Key Effects:**

#### 1. Config Loading (when slug provided without URL)
```tsx
useEffect(() => {
  if (streamUrl || !slug) return
  loadDesktopConfig()
    .then((config) => {
      setStreamUrl(`${config.serverUrl}/stream/${encodeURIComponent(slug)}`)
    })
    .catch((err) => setError(err.message))
}, [slug, streamUrl])
```

#### 2. Tauri Event Listener
```tsx
useEffect(() => {
  if (!isTauri()) return
  let unlisten: (() => void) | undefined
  void import('@tauri-apps/api/event').then(({ listen }) => {
    void listen<PlayerLoadPayload>('player:load', (event) => {
      applyLoad(event.payload)
      void getCurrentWindow().setTitle(event.payload.title)
    })
  })
  return () => unlisten?.()
}, [applyLoad])
```

**Event Payload:** `PlayerLoadPayload`
```typescript
interface PlayerLoadPayload {
  slug: string
  title: string
  streamUrl: string
}
```

#### 3. Video Element Loading
```tsx
useEffect(() => {
  const video = videoRef.current
  if (!video || !streamUrl) return
  
  setPhase('loading')
  setError(null)
  video.src = streamUrl
  void video.play().catch(() => { /* autoplay may be blocked */ })
}, [streamUrl])
```

**Video Event Handlers:**
| Event | Handler | Phase Change |
|-------|---------|--------------|
| `onLoadStart` | `setPhase('loading')` | loading |
| `onWaiting` | `setPhase('loading')` | loading |
| `onCanPlay` | `setPhase('playing')` | playing |
| `onPlaying` | `setPhase('playing')` | playing |
| `onError` | `setPhase('error')` | error |

---

## Component Hierarchy

```
App
├── AppShell
│   ├── WindowControls (Tauri only)
│   ├── Nav Sidebar
│   └── Main Content
│       ├── LibraryPanel
│       │   ├── Search Input
│       │   ├── MovieTable
│       │   │   └── Movie rows (clickable)
│       │   └── MovieDetail
│       │       ├── Poster/Backdrop
│       │       ├── Action Buttons
│       │       ├── Cast Grid
│       │       └── File Info
│       │       └── TmdbEditDialog (modal)
│       ├── ScanPanel
│       │   ├── Scan Buttons
│       │   └── Progress Display
│       └── SettingsPanel
│           └── Config Display
```

---

## State Management

### useApi Hook

**File:** `ui/src/hooks/useApi.ts`

**Purpose:** Manages movie data fetching and caching.

**Returns:**
```typescript
{
  movies: MovieSummary[],
  loading: boolean,
  error: string | null,
  refreshMovies: () => Promise<void>,
  serverUrl: string | null
}
```

**Implementation:**
```tsx
export function useApi() {
  const [movies, setMovies] = useState<MovieSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refreshMovies = useCallback(async () => {
    try {
      setLoading(true)
      const config = await loadDesktopConfig()
      setServerUrl(config.serverUrl)
      await fetchHealth()
      const list = await fetchMovies()
      setMovies(list)
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refreshMovies()
  }, [refreshMovies])

  return { movies, loading, error, refreshMovies }
}
```

---

### useScan Hook

**File:** `ui/src/hooks/useScan.ts`

**Purpose:** Manages library scan state and SSE streaming.

**Returns:**
```typescript
{
  status: LibraryStatusResponse | null,
  isScanning: boolean,
  progress: ScanProgress | null,
  error: string | null,
  startScan: (full?: boolean) => Promise<void>,
  refreshStatus: () => Promise<void>,
  clearError: () => void
}
```

**Scan Streaming:**
```tsx
for await (const event of startScanStream(full)) {
  if (event.type === 'progress') {
    setStatus({ ...status, progress: event.progress })
  } else if (event.type === 'complete') {
    setStatus({ ...status, state: 'idle', movies_count: event.movies_count })
  } else if (event.type === 'error') {
    setError(event.message)
  }
}
```

**Event Types:** `ScanStreamEvent`
```typescript
type ScanStreamEvent = 
  | { type: 'started', scan_id?: string }
  | { type: 'progress', progress?: ScanProgress }
  | { type: 'complete', movies_count?: number, duration_secs?: number }
  | { type: 'error', message?: string }
```

---

## Build Configuration

### vite.config.ts

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
  },
  build: {
    outDir: 'dist',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        player: resolve(__dirname, 'player.html'),
      },
    },
  },
})
```

**Key Settings:**
- Dev server: port 5173
- Multi-page app: `main` and `player` entry points
- Output: `dist/` directory

---

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

**Strict Mode:** Enabled for type safety

---

### tailwind.config.ts

**Custom Theme:**
```typescript
theme: {
  extend: {
    colors: {
      'loon-bg': '#090B10',
      'loon-fg': '#E2E8F0',
      'loon-primary': '#7DD3FC',
      'loon-secondary': '#64748B',
      'loon-border': '#1E293B',
      'loon-surface': '#111827',
      'loon-accent': '#38BDF8',
      'loon-muted': '#94A3B8',
      'loon-error': '#EF4444',
    },
    borderRadius: {
      'loon-sm': '4px',
      'loon-md': '8px',
      'loon-lg': '12px',
    }
  }
}
```

See [Component Styling](./04-components.md) for full design system reference.
