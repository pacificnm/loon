# Pages Documentation

## Page Index

| Page | File | Route | Description |
|------|------|-------|-------------|
| `MoviesPage` | `pages/MoviesPage.tsx` | `/` | Home page with alphabet list |
| `MovieDetailPage` | `pages/MovieDetailPage.tsx` | `/movie/:slug` | Movie details |
| `MovieEditPage` | `pages/MovieEditPage.tsx` | `/movie/:slug/edit` | TMDB match editor |
| `PlayerPage` | `pages/PlayerPage.tsx` | `/play/:slug` | Video player |
| `SearchPage` | `pages/SearchPage.tsx` | `/search` | Search interface |
| `FavoritesPage` | `pages/FavoritesPage.tsx` | `/favorites` | Favorites list |
| `GenresPage` | `pages/GenresPage.tsx` | `/genres` | Genre browser |
| `PersonPage` | `pages/PersonPage.tsx` | `/person/:tmdbId` | Actor details |
| `AdminPage` | `pages/AdminPage.tsx` | `/admin` | Admin panel |

---

## MoviesPage

**File:** `pages/MoviesPage.tsx`

**Route:** `/` or `/genre/:name`

**Purpose:** Home page displaying all movies in alphabetical order.

### Props

```typescript
interface MoviesPageProps {
  focusEpoch?: number;
  genre?: string;  // When viewing genre-specific list
}
```

### State

```typescript
const [movies, setMovies] = useState<MovieSummary[]>([]);
const [loading, setLoading] = useState(true);
const [error, setError] = useState<string | null>(null);
```

### Data Loading

```typescript
const load = useCallback(async () => {
  if (!server) {
    setError('No server configured. Open Admin → Settings.');
    return;
  }
  setLoading(true);
  setError(null);
  try {
    const list = await fetchAllMovies(server, { genre });
    setMovies(list);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load movies');
  } finally {
    setLoading(false);
  }
}, [server, genre]);
```

### Rendering

```tsx
<div className={styles.page}>
  <h1 className={styles.heading}>{genre ? genre : 'Movies'}</h1>
  {loading && <p>Loading movies…</p>}
  {error && (
    <div className={styles.error}>
      <p>{error}</p>
      <button onClick={() => void load()}>Retry</button>
    </div>
  )}
  {!loading && !error && server && (
    <MovieAlphabetList
      movies={movies}
      server={server}
      focusEpoch={focusEpoch}
      onSelect={(movie) => navigate(`/movie/${movie.slug}`)}
    />
  )}
</div>
```

---

## MovieDetailPage

**File:** `pages/MovieDetailPage.tsx`

**Route:** `/movie/:slug`

**Purpose:** Full movie details with backdrop, cast, and actions.

### Props

```typescript
interface MovieDetailPageProps {
  refreshedMovie?: MovieDetail;  // Fresh data from TMDB rematch
  refreshEpoch?: number;          // Cache busting
}
```

### State

```typescript
const [detail, setDetail] = useState<MovieDetail | null>(null);
const [similar, setSimilar] = useState<MovieSummary[]>([]);
const [loading, setLoading] = useState(true);
const [error, setError] = useState<string | null>(null);
const [favoriteBusy, setFavoriteBusy] = useState(false);
```

### Loading Strategy

```typescript
const load = useCallback(async () => {
  // Use refreshedMovie as seed if available
  const seed = refreshedMovie?.slug === slug ? refreshedMovie : undefined;
  if (seed) {
    setDetail(seed);
    setLoading(false);
    // Fetch similar movies in background
    fetchSimilarMovies(server, seed).then(setSimilar);
  } else {
    setLoading(true);
    setDetail(null);
  }

  // Always fetch fresh data
  const movie = await fetchMovie(server, slug, {
    cacheBust: refreshEpoch > 0 ? refreshEpoch : Date.now(),
  });
  setDetail(movie);
  const related = await fetchSimilarMovies(server, movie);
  setSimilar(related);
}, [refreshedMovie, refreshEpoch, server, slug]);
```

### Backdrop Hero Section

```tsx
<section className={styles.backdropHero}>
  {backdropUrl ? (
    <img key={backdropUrl} className={styles.backdrop} src={backdropUrl} alt="" />
  ) : (
    <div className={styles.backdropFallback} />
  )}
  <div className={styles.backdropScrim} />
  <div className={styles.heroContent}>
    <div className={styles.posterFrame}>
      {posterUrl ? <img src={posterUrl} /> : <div>{detail.title[0]}</div>}
    </div>
    <div className={styles.info}>
      <h1>{detail.title}</h1>
      <p className={styles.meta}>
        {[detail.year, runtime, detail.genres.join(' · ')].filter(Boolean).join(' · ')}
      </p>
      <div className={styles.actions}>
        <FocusButton focusKey="detail-play" label="Play" onPress={...} />
        <FocusButton focusKey="detail-favorite" label={is_favorite ? 'Remove Favorite' : 'Favorite'} />
        <FocusButton focusKey="detail-edit" label="Edit" onPress={...} />
      </div>
    </div>
  </div>
</section>
```

### Favorite Toggle

```typescript
const toggleFavorite = async () => {
  if (!detail || favoriteBusy) return;
  setFavoriteBusy(true);
  try {
    const response = await setFavorite(server, detail.slug);
    setDetail({ ...detail, is_favorite: response.favorite });
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update favorite');
  } finally {
    setFavoriteBusy(false);
  }
};
```

### Cast Row

```tsx
{detail.cast && detail.cast.length > 0 && (
  <CastRow
    cast={detail.cast}
    resolveArtwork={resolveArtwork}
    onSelectPerson={(member) => {
      if (member.tmdb_person_id) {
        navigate(`/person/${member.tmdb_person_id}`);
      } else {
        navigate('/person/lookup', {
          state: { movieSlug: slug, castName: member.name },
        });
      }
    }}
  />
)}
```

---

## MovieEditPage

**File:** `pages/MovieEditPage.tsx`

**Route:** `/movie/:slug/edit`

**Purpose:** Manual TMDB ID matching for misidentified movies.

### Props

```typescript
// No props - reads from URL params
const { slug = '' } = useParams();
```

### State

```typescript
const [tmdbId, setTmdbId] = useState('');
const [loading, setLoading] = useState(true);
const [saving, setSaving] = useState(false);
const [error, setError] = useState<string | null>(null);
const [title, setTitle] = useState('');
```

### Form Submission

```typescript
const submit = useCallback(async () => {
  const trimmed = tmdbId.trim();
  if (!trimmed) {
    setError('Enter a TMDB movie id');
    return;
  }

  setSaving(true);
  setError(null);
  try {
    const updated = await setMovieTmdbMatch(server, slug, trimmed);
    navigate(`/movie/${slug}`, {
      replace: true,
      state: {
        refreshedMovie: updated,
        refreshEpoch: Date.now(),
      },
    });
  } catch (err) {
    if (err instanceof LoonApiError && err.code === 'tmdb_not_configured') {
      setError('TMDB is not configured on the server');
    } else {
      setError(err instanceof Error ? err.message : 'Failed to update TMDB match');
    }
  } finally {
    setSaving(false);
  }
}, [navigate, server, slug, tmdbId]);
```

### TMDB ID Input

```tsx
<TmdbIdInput
  value={tmdbId}
  onChange={setTmdbId}
  disabled={saving}
/>
<p className={styles.hint}>
  Enter the numeric id from themoviedb.org (for example 348 for Alien).
</p>
```

### Navigation After Save

After successful save:
- Navigates back to movie detail
- Passes `refreshedMovie` in state (shows immediately)
- `refreshEpoch` busts artwork cache

---

## PlayerPage

**File:** `pages/PlayerPage.tsx`

**Route:** `/play/:slug`

**Purpose:** Full-screen video playback.

### Props

```typescript
// No props - reads from URL params
const { slug = '' } = useParams();
```

### Implementation

```tsx
export function PlayerPage() {
  const { slug = '' } = useParams();
  const server = useServerUrl();

  if (!server) {
    return <p>No server configured.</p>;
  }

  return (
    <VideoPlayer
      src={streamUrl(server, slug)}
      title={slug}
      onBack={() => window.history.back()}
    />
  );
}
```

---

## VideoPlayer

**File:** `player/VideoPlayer.tsx`

**Purpose:** HTML5 video player with webOS integration.

### Props

```typescript
interface VideoPlayerProps {
  src: string;      // Stream URL
  title: string;    // Video title
  onBack: () => void;  // Back button handler
}
```

### Video Loading

```typescript
useEffect(() => {
  const video = videoRef.current;
  if (!video) return;
  video.src = src;
  void video.play().catch(() => {
    /* autoplay policy — user can press play on TV */
  });
}, [src]);
```

### Back Button Handling

```typescript
useEffect(() => {
  const onKeyDown = (event: KeyboardEvent) => {
    if (!isAppBackKey(event)) return;
    event.preventDefault();
    onBack();
  };
  window.addEventListener('keydown', onKeyDown, true);
  return () => window.removeEventListener('keydown', onKeyDown, true);
}, [onBack]);
```

### Visibility Handling

```typescript
useWebOsVisibility(() => {
  videoRef.current?.pause();
});
```

When app is backgrounded (user switches apps), video pauses automatically.

---

## SearchPage

**File:** `pages/SearchPage.tsx`

**Route:** `/search`

**Purpose:** Search movies by title.

### Props

```typescript
interface SearchPageProps {
  focusEpoch?: number;
}
```

### Debounced Search

```typescript
const runSearch = useCallback(async (text: string) => {
  const trimmed = text.trim();
  if (trimmed.length < 2) {
    setMovies([]);
    setError(null);
    return;
  }
  if (!server) {
    setError('No server configured.');
    return;
  }
  setLoading(true);
  setError(null);
  try {
    const response = await searchMovies(server, trimmed);
    setMovies(response.movies);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Search failed');
  } finally {
    setLoading(false);
  }
}, [server]);

// Debounce with 300ms delay
useEffect(() => {
  const timer = window.setTimeout(() => {
    void runSearch(query);
  }, 300);
  return () => window.clearTimeout(timer);
}, [query, runSearch]);
```

### Search Input

```tsx
<SearchInput value={query} onChange={setQuery} />
```

The input is focusable and handles TV remote text entry.

---

## FavoritesPage

**File:** `pages/FavoritesPage.tsx`

**Route:** `/favorites`

**Purpose:** Display favorited movies.

### Data Loading

```typescript
const load = useCallback(async () => {
  if (!server) {
    setError('No server configured.');
    return;
  }
  setLoading(true);
  setError(null);
  try {
    setMovies(await fetchFavorites(server));
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load favorites');
  } finally {
    setLoading(false);
  }
}, [server]);
```

### fetchFavorites Implementation

```typescript
export async function fetchFavorites(baseUrl: string): Promise<MovieSummary[]> {
  const browse = await fetchBrowse(baseUrl);
  const row = browse.rows.find((entry) => entry.row_type === 'favorites');
  return row?.movies ?? [];
}
```

---

## GenresPage

**File:** `pages/GenresPage.tsx`

**Route:** `/genres`

**Purpose:** Browse movies by genre.

### Data Loading

```typescript
const load = useCallback(async () => {
  if (!server) return;
  setLoading(true);
  setError(null);
  try {
    const response = await fetchGenres(server);
    setGenres(response.genres);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to load genres');
  } finally {
    setLoading(false);
  }
}, [server]);
```

### Genre Item

```tsx
function GenreItem({ genre, onSelect }) {
  const { ref, focused } = useFocusable({
    focusKey: `genre-${genre.name}`,
    onEnterPress: () => onSelect(genre.name),
  });

  return (
    <button ref={ref} className={`${styles.genreItem} ${focused ? styles.genreFocused : ''}`}>
      <span>{genre.name}</span>
      <span className={styles.genreCount}>{genre.count}</span>
    </button>
  );
}
```

---

## PersonPage

**File:** `pages/PersonPage.tsx`

**Route:** `/person/:tmdbId` or `/person/lookup`

**Purpose:** Display actor/person details with filmography.

### Route Parsing

```typescript
function parsePersonRouteId(raw: string): number | null {
  const trimmed = raw.trim();
  if (!trimmed || trimmed === 'lookup') return null;
  const numeric = trimmed.startsWith('tmdb:') ? trimmed.slice('tmdb:'.length) : trimmed;
  const personId = Number.parseInt(numeric, 10);
  return Number.isFinite(personId) && personId > 0 ? personId : null;
}
```

### Lookup Mode

When navigating from cast member without TMDB ID:

```typescript
navigate('/person/lookup', {
  state: { movieSlug: slug, castName: member.name },
});
```

Then `fetchPersonForCast` resolves the person:

```typescript
const load = isLookup
  ? fetchPersonForCast(server, castLookup.movieSlug, castLookup.castName)
  : fetchPerson(server, personId);
```

### Gender Formatting

```typescript
function formatGender(code?: number): string | null {
  switch (code) {
    case 1: return 'Male';
    case 2: return 'Female';
    case 3: return 'Non-binary';
    default: return null;
  }
}
```

### Known For Row

Displays movies in the user's library featuring this person:

```tsx
{detail.known_for.length > 0 && (
  <KnownForRow
    movies={detail.known_for}
    resolveArtwork={resolveArtwork}
    onSelect={(movie) => navigate(`/movie/${movie.slug}`)}
  />
)}
```

---

## AdminPage

**File:** `pages/AdminPage.tsx`

**Route:** `/admin`

**Purpose:** Library scanning and server configuration.

### Tabs

```typescript
type AdminTab = 'scan' | 'settings';

const tab = parseTab(searchParams.get('tab'));
// 'scan' (default) or 'settings' (via ?tab=settings)
```

### Scan Functionality

```typescript
const runScan = useCallback(async (full: boolean) => {
  if (streaming || !server) return;

  const controller = new AbortController();
  abortRef.current = controller;
  setStreaming(true);
  appendLog(full ? '--- Starting metadata refresh ---' : '--- Starting library scan ---');

  try {
    await streamLibraryScan(
      server,
      { full, signal: controller.signal },
      (event) => appendLog(formatScanEvent(event)),
    );
    await refreshStatus();
  } catch (error) {
    if (error instanceof LoonApiError && error.code === 'scan_already_running') {
      appendLog('Scan already running on server');
    } else {
      appendLog(error instanceof Error ? error.message : 'Scan failed');
    }
  } finally {
    setStreaming(false);
    abortRef.current = null;
  }
}, [server, streaming]);
```

### Status Polling

```typescript
useEffect(() => {
  if (!status?.scan_in_progress || streaming || tab !== 'scan' || !server) {
    return;
  }

  const intervalId = window.setInterval(() => {
    void refreshStatus().catch(() => { /* polling errors are non-fatal */ });
  }, 2000);

  return () => window.clearInterval(intervalId);
}, [refreshStatus, server, status?.scan_in_progress, streaming, tab]);
```

### Log Display

```tsx
<div ref={logRef} className={adminStyles.log} aria-live="polite">
  {logLines.length === 0 ? (
    <p className={adminStyles.logEmpty}>
      Scan output will appear here.
    </p>
  ) : (
    logLines.map((line, index) => (
      <div key={`${index}-${line}`} className={adminStyles.logLine}>
        {line}
      </div>
    ))
  )}
</div>
```

Auto-scrolls to bottom on new log lines.

---

## AdminSettingsTab

**File:** `pages/AdminSettingsTab.tsx`

**Purpose:** Server URL configuration.

### Features

- Display current server URL
- Input for new URL
- Save/Clear buttons
- Validation (must start with http:// or https://)

---

## Page Styling

All pages use shared page styles:

**File:** `pages/page.module.css`

```css
.page {
  padding: 24px;
  min-height: 100vh;
  background: var(--loon-bg);
}

.heading {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.content {
  /* Page-specific content styling */
}

.status {
  color: var(--loon-muted);
}

.errorText {
  color: var(--loon-error);
}
```
