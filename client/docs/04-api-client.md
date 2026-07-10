# API Client Documentation

## Client Module

**File:** `src/api/client.ts`

## Error Handling

### LoonApiError

```typescript
export class LoonApiError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'LoonApiError';
    this.code = code;
  }
}
```

### Request Helper

```typescript
async function request<T>(
  baseUrl: string,
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, init);
  const text = await response.text();

  if (!response.ok) {
    try {
      const body = JSON.parse(text) as ApiErrorBody;
      throw new LoonApiError(body.error.code, body.error.message);
    } catch (error) {
      if (error instanceof LoonApiError) throw error;
      throw new LoonApiError('http_error', `HTTP ${response.status}`);
    }
  }

  if (!text) {
    throw new LoonApiError('empty_response', 'Empty response from server');
  }

  try {
    return JSON.parse(text) as T;
  } catch {
    throw new LoonApiError('invalid_json', 'Invalid JSON from server');
  }
}
```

---

## Movie Functions

### fetchMovies

**Signature:**
```typescript
export async function fetchMovies(
  baseUrl: string,
  options: ListMoviesOptions = {},
): Promise<MovieListResponse>
```

**Options:**
```typescript
interface ListMoviesOptions {
  page?: number;
  limit?: number;
  sort?: 'title' | 'year' | 'recently_added';
  genre?: string;
}
```

**Example:**
```typescript
// Get first page, 50 items, sorted by title
const response = await fetchMovies(serverUrl, {
  page: 1,
  limit: 50,
  sort: 'title',
});

// Get movies in genre
const actionMovies = await fetchMovies(serverUrl, {
  genre: 'Action',
  page: 1,
  limit: 20,
});
```

**Response:**
```typescript
interface MovieListResponse {
  movies: MovieSummary[];
  total: number;
  page?: number;
  limit?: number;
  pages?: number;
}
```

---

### fetchMovie

**Signature:**
```typescript
export async function fetchMovie(
  baseUrl: string,
  slug: string,
  options: { cacheBust?: number } = {},
): Promise<MovieDetail>
```

**Example:**
```typescript
// Normal fetch
const movie = await fetchMovie(serverUrl, 'alien-1979');

// With cache busting (after TMDB rematch)
const movie = await fetchMovie(serverUrl, 'alien-1979', {
  cacheBust: Date.now(),
});
```

**Normalization:**
```typescript
const detail = await request<MovieDetail>(...);
return normalizeMovieDetail(detail);
```

Ensures `cast`, `crew`, `genres`, and `file` fields are never null/undefined.

---

### fetchAllMovies

**Signature:**
```typescript
export async function fetchAllMovies(
  baseUrl: string,
  options: Omit<ListMoviesOptions, 'page' | 'limit'> = {},
): Promise<MovieSummary[]>
```

**Purpose:** Fetches ALL movies by paginating through results.

**Implementation:**
```typescript
const pageSize = 100;
const first = await fetchMovies(baseUrl, {
  ...options,
  page: 1,
  limit: pageSize,
  sort: 'title',
});

const movies = [...first.movies];
const pages = first.pages ?? 1;

for (let page = 2; page <= pages; page += 1) {
  const next = await fetchMovies(baseUrl, {
    ...options,
    page,
    limit: pageSize,
    sort: 'title',
  });
  movies.push(...next.movies);
}

// Final sort by title (case-insensitive)
return movies.sort((a, b) =>
  a.title.localeCompare(b.title, undefined, { sensitivity: 'base' }),
);
```

---

### fetchSimilarMovies

**Signature:**
```typescript
export async function fetchSimilarMovies(
  baseUrl: string,
  detail: MovieDetail,
  limit = 12,
): Promise<MovieSummary[]>
```

**Purpose:** Finds movies in the same genre (excluding the current movie).

**Implementation:**
```typescript
const genre = detail.genres[0];
if (!genre) return [];

const list = await fetchMovies(baseUrl, {
  genre,
  limit: limit + 1,
  page: 1,
  sort: 'title',
});

return list.movies
  .filter((movie) => movie.slug !== detail.slug)
  .slice(0, limit);
```

---

## Person Functions

### fetchPerson

**Signature:**
```typescript
export async function fetchPerson(
  baseUrl: string,
  tmdbPersonId: number,
): Promise<PersonDetail>
```

**Example:**
```typescript
const person = await fetchPerson(serverUrl, 85);  // Johnny Depp
```

---

### fetchPersonForCast

**Signature:**
```typescript
export async function fetchPersonForCast(
  baseUrl: string,
  movieSlug: string,
  castName: string,
): Promise<PersonDetail>
```

**Purpose:** Resolves cast member to person details when TMDB ID is missing.

**Query Parameters:**
- `movie_slug` - Movie containing the cast member
- `name` - Cast member name

**Example:**
```typescript
const person = await fetchPersonForCast(
  serverUrl,
  'pirates-of-the-caribbean-2003',
  'Johnny Depp',
);
```

---

## Search Functions

### searchMovies

**Signature:**
```typescript
export async function searchMovies(
  baseUrl: string,
  query: string,
  limit = 20,
): Promise<SearchResponse>
```

**Example:**
```typescript
const response = await searchMovies(serverUrl, 'alien', 20);
console.log(response.movies);  // Array of matching movies
console.log(response.total);   // Total matches
```

**Response:**
```typescript
interface SearchResponse {
  query: string;
  movies: MovieSummary[];
  total: number;
}
```

---

### fetchGenres

**Signature:**
```typescript
export async function fetchGenres(baseUrl: string): Promise<GenresResponse>
```

**Response:**
```typescript
interface GenresResponse {
  genres: GenreEntry[];
}

interface GenreEntry {
  name: string;
  count: number;
}
```

---

## Browse Functions

### fetchBrowse

**Signature:**
```typescript
export async function fetchBrowse(baseUrl: string): Promise<BrowseResponse>
```

**Response:**
```typescript
interface BrowseResponse {
  hero?: MovieSummary;
  rows: BrowseRow[];
}

interface BrowseRow {
  slug: string;
  title: string;
  row_type: string;
  movies: MovieSummary[];
}
```

**Row Types:**
- `continue_watching` - Incomplete watch progress
- `recently_added` - Most recently scanned
- `favorites` - User favorites
- `genre-{name}` - Genre-based rows

---

### fetchFavorites

**Signature:**
```typescript
export async function fetchFavorites(baseUrl: string): Promise<MovieSummary[]>
```

**Implementation:**
```typescript
const browse = await fetchBrowse(baseUrl);
const row = browse.rows.find((entry) => entry.row_type === 'favorites');
return row?.movies ?? [];
```

---

## Favorite Functions

### setFavorite

**Signature:**
```typescript
export async function setFavorite(
  baseUrl: string,
  slug: string,
  favorite?: boolean,
): Promise<FavoriteResponse>
```

**Usage:**
```typescript
// Toggle favorite (no body)
const response = await setFavorite(serverUrl, 'alien-1979');

// Set explicitly
await setFavorite(serverUrl, 'alien-1979', true);   // Add
await setFavorite(serverUrl, 'alien-1979', false);  // Remove
```

**Request Body (when favorite specified):**
```json
{ "favorite": true }
```

**Response:**
```typescript
interface FavoriteResponse {
  slug: string;
  favorite: boolean;
}
```

---

## TMDB Match Functions

### setMovieTmdbMatch

**Signature:**
```typescript
export async function setMovieTmdbMatch(
  baseUrl: string,
  slug: string,
  tmdbId: string,
): Promise<MovieDetail>
```

**Example:**
```typescript
const updated = await setMovieTmdbMatch(serverUrl, 'alien-1979', '348');
```

**Request:**
```typescript
{
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ tmdb_id: tmdbId }),
}
```

**Response:** Updated `MovieDetail` with new TMDB metadata.

---

## Library Functions

### fetchLibraryStatus

**Signature:**
```typescript
export async function fetchLibraryStatus(
  baseUrl: string,
): Promise<LibraryStatusResponse>
```

**Response:**
```typescript
interface LibraryStatusResponse {
  state: string;              // "idle" or "scanning"
  last_scan_at?: string;      // ISO timestamp
  last_scan_duration_secs: number;
  movies_count: number;
  scan_in_progress: boolean;
  progress?: ScanProgress;
}
```

---

### fetchHealth

**Signature:**
```typescript
export async function fetchHealth(baseUrl: string): Promise<HealthResponse>
```

**Response:**
```typescript
interface HealthResponse {
  status: string;
  service?: string;
  version?: string;
  movies_count: number;
  library_scanned_at?: number;
}
```

---

### streamLibraryScan

**Signature:**
```typescript
export async function streamLibraryScan(
  baseUrl: string,
  options: StreamLibraryScanOptions,
  onEvent: (event: ScanStreamEvent) => void,
): Promise<void>
```

**Options:**
```typescript
interface StreamLibraryScanOptions {
  full?: boolean;
  signal?: AbortSignal;
}
```

**Event Types:**
```typescript
type ScanStreamEvent =
  | { type: 'started'; scan_id: string }
  | { type: 'progress'; progress: ScanProgress }
  | { type: 'complete'; scan_id: string; movies_count: number; duration_secs: number; stats: ScanStats }
  | { type: 'error'; scan_id: string; message: string };
```

**Usage:**
```typescript
const controller = new AbortController();

await streamLibraryScan(
  serverUrl,
  { full: false, signal: controller.signal },
  (event) => {
    console.log(formatScanEvent(event));
  },
);
```

**SSE Parsing:**
```typescript
await readSseStream(response, (message) => {
  const event = JSON.parse(message.data) as ScanStreamEvent;
  onEvent(event);
}, signal);
```

---

## SSE Parser

**File:** `src/api/sse.ts`

### parseSseMessages

**Signature:**
```typescript
export function parseSseMessages(chunk: string): {
  messages: SseMessage[];
  remainder: string;
}
```

**Parses SSE format:**
```
event: progress
data: {"phase":"enriching","files_seen":100,...}

event: complete
data: {"movies_count":50,"duration_secs":120}
```

### readSseStream

**Signature:**
```typescript
export async function readSseStream(
  response: Response,
  onMessage: (message: SseMessage) => void,
  signal?: AbortSignal,
): Promise<void>
```

**Process:**
1. Get response body reader
2. Read chunks until done or cancelled
3. Decode UTF-8 text
4. Parse SSE messages
5. Buffer incomplete messages for next chunk

---

## Normalization

**File:** `src/api/normalize.ts`

### normalizeMovieDetail

**Signature:**
```typescript
export function normalizeMovieDetail(raw: MovieDetail): MovieDetail
```

**Purpose:** Ensures API responses work with older servers that may omit fields.

**Normalization:**
```typescript
return {
  ...raw,
  cast: raw.cast ?? [],
  crew: raw.crew ?? [],
  genres: raw.genres ?? [],
  file: raw.file ?? fallbackFileInfo(raw.slug),
};
```

### fallbackFileInfo

**Signature:**
```typescript
export function fallbackFileInfo(slug: string): MovieFileInfo
```

**Creates minimal file info from slug:**
```typescript
const filename = slug;
const extension = extensionFromPath(filename);
return {
  filename,
  relative_path: filename,
  extension: extension ?? null,
  size_bytes: null,
  content_type: contentTypeForExtension(extension),
  modified_at: null,
  scanned_at: null,
};
```

---

## Type Definitions

**File:** `src/api/types.ts`

### Core Types

```typescript
interface MovieSummary {
  slug: string;
  title: string;
  year?: number;
  runtime_minutes: number;
  poster_url?: string;
  backdrop_url?: string;
  summary: string;
}

interface MovieDetail {
  slug: string;
  title: string;
  original_title?: string;
  year?: number;
  runtime_minutes?: number;
  summary?: string;
  genres: string[];
  poster_url?: string;
  backdrop_url?: string;
  cast: CastMember[];
  crew: CrewMember[];
  is_favorite: boolean;
  watch_progress_seconds?: number;
  tmdb_id?: string | null;
  imdb_id?: string | null;
  file?: MovieFileInfo | null;
  stream_url: string;
}

interface CastMember {
  name: string;
  character?: string;
  profile_url?: string;
  tmdb_person_id?: number;
}

interface CrewMember {
  name: string;
  job?: string;
}
```

### Scan Types

```typescript
type ScanPhase = 'discovering' | 'enriching' | 'persisting';

interface ScanProgress {
  phase?: ScanPhase | null;
  files_seen: number;
  candidates: number;
  errors: number;
  enriched: number;
  total_to_enrich: number;
  current_path?: string | null;
}

interface ScanStats {
  files_seen: number;
  candidates: number;
  errors: number;
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `movie_not_found` | 404 | Movie doesn't exist |
| `invalid_request` | 400 | Bad input |
| `scan_already_running` | 409 | Scan in progress |
| `library_scanning` | 503 | Service busy |
| `tmdb_not_configured` | 503 | TMDB disabled |
| `artwork_not_found` | 404 | No artwork available |
| `http_error` | - | HTTP request failed |
| `empty_response` | - | Empty response body |
| `invalid_json` | - | Invalid JSON response |
