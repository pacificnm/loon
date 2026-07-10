# UI Components Reference

## Component Index

| Component | File | Purpose |
|-----------|------|---------|
| `AppShell` | `components/AppShell.tsx` | Main layout with sidebar navigation |
| `WindowControls` | `components/WindowControls.tsx` | Tauri window chrome (minimize/maximize/close) |
| `LibraryPanel` | `components/LibraryPanel.tsx` | Movie browser with search and detail view |
| `MovieTable` | `components/MovieTable.tsx` | Sortable movie list table |
| `MovieDetail` | `components/MovieDetail.tsx` | Full movie information display |
| `TmdbEditDialog` | `components/TmdbEditDialog.tsx` | TMDB ID editing modal |
| `SettingsPanel` | `components/SettingsPanel.tsx` | Configuration display |
| `ScanPanel` | `components/ScanPanel.tsx` | Library scan controls and progress |

---

## AppShell

**File:** `components/AppShell.tsx`

**Purpose:** Main application layout with sidebar navigation and title bar.

### Props

```typescript
interface AppShellProps {
  activeSection: Section          // Current navigation section
  onSectionChange: (section: Section) => void  // Navigation handler
  title: string                   // Window title
  subtitle: string                // Window subtitle
  children: React.ReactNode       // Main content
}
```

**Section Type:**
```typescript
type Section = 'library' | 'scan' | 'settings'
```

### Structure

```
┌────────────────────────────────────────────────────┐
│ Title Bar (32px)                                   │
│ [Drag Region] Title/Subtitle [WindowControls]      │
├──────────┬─────────────────────────────────────────┤
│          │                                         │
│ Sidebar  │         Main Content                    │
│ (240px)  │         (scrollable)                    │
│          │                                         │
│ [Nav]    │                                         │
│ Library  │                                         │
│ Scan     │                                         │
│ Settings │                                         │
│          │                                         │
└──────────┴─────────────────────────────────────────┘
```

### Navigation Items

```typescript
const NAV_ITEMS = [
  { id: 'library', label: 'Library', icon: faFilm },
  { id: 'scan', label: 'Scan', icon: faArrowRotateRight },
  { id: 'settings', label: 'Settings', icon: faGear },
]
```

### Tauri-Specific Behavior

**Title Bar:**
- Tauri: Custom chrome with drag region, centered title
- Web: Standard header with left-aligned title

**Window Controls:**
- Tauri: Rendered (minimize/maximize/close)
- Web: Hidden (browser provides controls)

### CSS Classes

| Element | Classes |
|---------|---------|
| Title bar | `border-b border-loon-border bg-loon-surface` |
| Sidebar | `w-60 border-r border-loon-border bg-loon-surface` |
| Nav item (active) | `bg-loon-primary/10 text-loon-primary` |
| Nav item (inactive) | `text-loon-fg hover:bg-loon-border/50` |

---

## WindowControls

**File:** `components/WindowControls.tsx`

**Purpose:** Frameless window controls for Tauri applications.

### Props

```typescript
{
  onClose?: () => void | Promise<void>  // Custom close handler (player uses hide)
}
```

### Controls

| Control | Icon | Action |
|---------|------|--------|
| Minimize | `faMinus` | `minimizeWindow()` |
| Maximize/Restore | `faWindowMaximize` / `faWindowRestore` | `toggleMaximizeWindow()` |
| Close | `faXmark` | `closeWindow()` or `onClose` |

### State Management

**Maximized State:**
```tsx
const [maximized, setMaximized] = useState(false)

useEffect(() => {
  if (!tauri) return
  const win = getCurrentWindow()
  const current = await win.isMaximized()
  setMaximized(current)
  unlisten = await win.onResized(async () => {
    const next = await win.isMaximized()
    setMaximized(next)
  })
}, [tauri])
```

### Dimensions

```typescript
const CONTROL_WIDTH = 46  // Each button width in pixels
```

### Styling

**Close Button (danger):**
- Hover: `bg-red-600 hover:text-white`
- Normal: `text-loon-fg/90`

**Other Buttons:**
- Hover: `bg-loon-border/60 hover:text-loon-fg`
- Normal: `text-loon-fg/85`

---

## LibraryPanel

**File:** `components/LibraryPanel.tsx`

**Purpose:** Movie library browser with search, list view, and detail view.

### Props

```typescript
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
```

### States

| State | Display |
|-------|---------|
| `loading` | "Loading movies..." centered |
| `error` | Error message centered |
| `movieLoading` | "Loading movie details..." centered |
| `selectedMovie` | MovieDetail component |
| Default | MovieTable with search |

### Search Functionality

```tsx
const [searchQuery, setSearchQuery] = useState('')

const filteredMovies = useMemo(() => {
  if (!searchQuery.trim()) return movies
  const query = searchQuery.toLowerCase().trim()
  return movies.filter(
    (movie) =>
      movie.title.toLowerCase().includes(query) ||
      movie.relative_path.toLowerCase().includes(query) ||
      (movie.year && movie.year.toString().includes(query))
  )
}, [movies, searchQuery])
```

**Search Fields:**
- Title (case-insensitive)
- Relative path (case-insensitive)
- Year (exact match)

**Result Counter:**
```tsx
{searchQuery && (
  <span className="text-xs text-loon-muted">
    {filteredMovies.length} of {movies.length}
  </span>
)}
```

---

## MovieTable

**File:** `components/MovieTable.tsx`

**Purpose:** Sortable table displaying movie summaries.

### Props

```typescript
interface MovieTableProps {
  movies: MovieSummary[]
  onMovieSelect: (slug: string) => void
}
```

### Columns

| Column | Field | Format |
|--------|-------|--------|
| Title | `movie.title` | Plain text, sortable |
| Year | `movie.year` | Number or "—" |
| File | `movie.relative_path` | Monospace, truncated |
| Size | `movie.size_bytes` | Human-readable (GB/MB/KB) |

### Sorting

```tsx
const [sortTitleAsc, setSortTitleAsc] = useState(true)

const sortedMovies = [...movies].sort((a, b) => {
  const order = a.title.toLowerCase().localeCompare(b.title.toLowerCase())
  return sortTitleAsc ? order : -order
})
```

### Size Formatting

```tsx
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
```

### Row Selection

```tsx
const handleRowClick = (slug: string) => {
  setSelectedSlug(slug)
  onMovieSelect(slug)
}
```

**Active Row Styling:**
- Selected: `bg-loon-primary/10`
- Hover: `bg-loon-border/30`

---

## MovieDetail

**File:** `components/MovieDetail.tsx`

**Purpose:** Full movie information display with actions.

### Props

```typescript
interface MovieDetailProps {
  movie: MovieDetail
  onBack: () => void
  onMovieUpdated?: (movie: MovieDetail) => void
}
```

### Sections

#### 1. Hero Section
- Backdrop image (full width with gradient overlay)
- Poster image (left side)
- Title and original title
- Meta line: Year · Runtime · Genres
- Action buttons: Play, Favorite, Edit

#### 2. Overview Section
- Summary text
- Director(s)
- Producer(s)

#### 3. Cast Section
- Horizontal scrollable grid
- Up to 12 cast members
- Profile images or initial placeholder

#### 4. File Info Section
- File name
- Relative path
- Format (extension)
- IMDb ID
- Favorite status

### Action Handlers

#### Play
```tsx
const handlePlay = async () => {
  try {
    await playStream(movie.slug, movie.title)
  } catch (err) {
    setError(`Failed to start playback: ${err.message}`)
  }
}
```

#### Toggle Favorite
```tsx
const handleToggleFavorite = async () => {
  setFavoriteLoading(true)
  try {
    await setFavorite(movie.slug, !isFavorite)
    setIsFavorite(!isFavorite)
  } catch (err) {
    setError(`Failed to toggle favorite: ${err.message}`)
  } finally {
    setFavoriteLoading(false)
  }
}
```

#### Edit TMDB
```tsx
const handleEdit = () => {
  if (!serverUrl) {
    setError('Server URL not loaded yet')
    return
  }
  setEditOpen(true)
}
```

### Artwork Caching

```tsx
const [artworkVersion, setArtworkVersion] = useState(0)

const artworkCacheBust = movie.tmdb_id ?? artworkVersion

const posterUrl = movie.poster_url && serverUrl
  ? `${serverUrl}${movie.poster_url}?v=${encodeURIComponent(String(artworkCacheBust))}`
  : null
```

**Cache Invalidation:**
- Initial: Uses movie's TMDB ID
- After edit: Uses timestamp (`Date.now()`)

### Crew Formatting

```tsx
const crewLines = (): Array<{ label: string; names: string }> => {
  const lines = []
  
  const directors = movie.crew
    .filter(m => m.job?.toLowerCase() === 'director')
    .map(m => m.name)
  
  if (directors.length > 0) {
    lines.push({ label: 'Director', names: directors.join(', ') })
  }
  
  // Similar for producers...
  return lines
}
```

---

## TmdbEditDialog

**File:** `components/TmdbEditDialog.tsx`

**Purpose:** Modal dialog for editing TMDB movie ID match.

### Props

```typescript
interface TmdbEditDialogProps {
  open: boolean
  movie: MovieDetail
  onClose: () => void
  onSaved: (updated: MovieDetail) => void
}
```

### TMDB ID Parsing

```typescript
function parseTmdbNumericId(raw: string): number | null {
  const trimmed = raw.trim()
  if (!trimmed) return null
  const numeric = trimmed.startsWith('tmdb:') 
    ? trimmed.slice('tmdb:'.length) 
    : trimmed
  const id = Number.parseInt(numeric.trim(), 10)
  return Number.isFinite(id) && id > 0 ? id : null
}
```

**Accepted Formats:**
- `348` (numeric only)
- `tmdb:348` (with prefix)
- Any whitespace trimmed

### TMDB URL

```typescript
function tmdbMovieUrl(id: number): string {
  return `https://www.themoviedb.org/movie/${id}`
}
```

### Validation

```tsx
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
  const updated = await setMovieTmdbMatch(movie.slug, trimmed)
  onSaved(updated)
  onClose()
}
```

### Keyboard Handling

```tsx
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
```

### Modal Structure

```
┌─────────────────────────────────────┐
│ [Edit icon] Edit TMDB match  [X]    │ ← Header
├─────────────────────────────────────┤
│ Movie Title                         │
│                                     │
│ TMDB movie id                       │
│ Enter the numeric id...             │
│ [Input field]                       │
│                                     │
│ [Open on TMDB]                      │ ← Link (if valid)
│                                     │
│ [Error message]                     │ ← If error
│                                     │
├─────────────────────────────────────┤
│                    [Cancel] [Save]  │ ← Footer
└─────────────────────────────────────┘
```

---

## SettingsPanel

**File:** `components/SettingsPanel.tsx`

**Purpose:** Displays current configuration and backend status.

### State

```tsx
const [serverUrl, setServerUrl] = useState<string | null>(null)
const [configPath, setConfigPath] = useState<string | null>(null)
const [health, setHealth] = useState<string | null>(null)
const [playerPath, setPlayerPath] = useState<string | null>(null)
const [error, setError] = useState<string | null>(null)
```

### Data Loading

```tsx
useEffect(() => {
  loadDesktopConfig()
    .then(async (config) => {
      setServerUrl(config.serverUrl)
      setConfigPath(config.configPath)
      setPlayerPath(config.playerPath ?? 'Built-in player window')
      const status = await fetchHealth()
      setHealth(`${status.status} (${status.movies_count} movies)`)
    })
    .catch((err) => setError(err.message))
}, [])
```

### Display Sections

| Section | Content |
|---------|---------|
| Config file | Path to `~/.config/loon/config.toml` |
| Backend API | Server URL from config |
| Health | API status and movie count |
| Video player | Player path or "Built-in player window" |

---

## ScanPanel

**File:** `components/ScanPanel.tsx`

**Purpose:** Library scan controls and real-time progress display.

### Hook Usage

```tsx
const {
  status,
  isScanning,
  progress,
  error,
  startScan,
  refreshStatus,
  clearError,
} = useScan()
```

### Scan Types

| Type | Parameter | Description |
|------|-----------|-------------|
| Incremental | `full = false` | Scan for new/changed files only |
| Full Refresh | `full = true` | Re-enrich all metadata |

### Progress Display

**Phase Labels:**
```tsx
const formatPhase = (phase: string | null): string => {
  switch (phase) {
    case 'discovering': return 'Discovering files'
    case 'enriching': return 'Enriching metadata'
    case 'persisting': return 'Saving results'
    default: return 'Idle'
  }
}
```

**Progress Bar:**
```tsx
<div className="h-2 w-full overflow-hidden rounded-full bg-loon-border">
  <div
    className="h-full bg-loon-primary transition-all"
    style={{
      width: `${(progress.enriched / progress.total_to_enrich) * 100}%`,
    }}
  />
</div>
```

**Stats Grid:**
| Stat | Field |
|------|-------|
| Files seen | `progress.files_seen` |
| Candidates | `progress.candidates` |
| Errors | `progress.errors` |
| Enriched | `progress.enriched / progress.total_to_enrich` |

### Duration Formatting

```tsx
const formatDuration = (secs: number): string => {
  const mins = Math.floor(secs / 60)
  const remainingSecs = secs % 60
  if (mins > 0) return `${mins}m ${remainingSecs}s`
  return `${secs}s`
}
```

### Error Handling

```tsx
{error && (
  <div className="mb-4 rounded-loon-md border border-loon-error/20 bg-loon-error/10 p-3">
    <p className="text-sm text-loon-error">{error}</p>
    <button onClick={clearError} className="mt-2 text-xs text-loon-error hover:underline">
      Dismiss
    </button>
  </div>
)}
```

---

## Design System

### Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `loon-bg` | `#090B10` | Main background |
| `loon-fg` | `#E2E8F0` | Primary text |
| `loon-primary` | `#7DD3FC` | Primary actions |
| `loon-secondary` | `#64748B` | Secondary elements |
| `loon-border` | `#1E293B` | Borders and dividers |
| `loon-surface` | `#111827` | Card backgrounds |
| `loon-accent` | `#38BDF8` | Highlights |
| `loon-muted` | `#94A3B8` | Secondary text |
| `loon-success` | `#22C55E` | Success states |
| `loon-warning` | `#F59E0B` | Warning states |
| `loon-error` | `#EF4444` | Error states |
| `loon-info` | `#38BDF8` | Info states |

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `loon-sm` | `4px` | Small buttons, inputs |
| `loon-md` | `8px` | Cards, tables |
| `loon-lg` | `12px` | Dialogs, panels |
| `loon-full` | `9999px` | Pills, avatars |

### Typography

| Element | Size | Weight | Color |
|---------|------|--------|-------|
| Window title | `12px` | medium | `loon-fg` |
| Window subtitle | `10px` | normal | `loon-muted` |
| Section heading | `14px` | medium | `loon-muted` |
| Body text | `14px` | normal | `loon-fg` |
| Small text | `12px` | normal | `loon-muted` |
| Mono (paths) | `14px` | normal | `loon-fg` |

### Spacing

| Token | Value | Usage |
|-------|-------|-------|
| `loon-xs` | `4px` | Tight spacing |
| `loon-sm` | `8px` | Button padding |
| `loon-md` | `16px` | Section padding |
| `loon-lg` | `24px` | Panel padding |
| `loon-xl` | `32px` | Large gaps |
| `loon-xxl` | `48px` | Section gaps |
