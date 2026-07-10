# Models Documentation

## Model Modules

```
src/models/
├── mod.rs           # Module exports
├── movie.rs         # Movie DTOs
├── library.rs       # Library/scan DTOs
├── browse.rs        # Browse feed DTOs
├── person.rs        # Person DTOs
└── root.rs          # Root response DTO
```

---

## Movie Models

**File:** `models/movie.rs`

### `HealthResponse`

**Purpose:** Health check response.

```rust
pub struct HealthResponse {
    pub status: &'static str,         // "ok"
    pub service: &'static str,        // "loon-server"
    pub version: &'static str,        // Cargo package version
    pub movies_count: usize,          // Total movies
    pub library_scanned_at: u64,      // Last scan timestamp
}
```

**Example:**
```json
{
  "status": "ok",
  "service": "loon-server",
  "version": "0.1.0",
  "movies_count": 150,
  "library_scanned_at": 1700000000
}
```

---

### `MovieListResponse`

**Purpose:** Paginated movie list response.

```rust
pub struct MovieListResponse {
    pub movies: Vec<MovieSummary>,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<u32>,
}
```

**Unpaginated Example:**
```json
{
  "movies": [...],
  "total": 150
}
```

**Paginated Example:**
```json
{
  "movies": [...],
  "total": 150,
  "page": 1,
  "limit": 50,
  "pages": 3
}
```

---

### `MovieSummary`

**Purpose:** Movie card for list/grid views.

```rust
pub struct MovieSummary {
    pub slug: String,
    pub title: String,
    pub year: Option<u16>,
    pub runtime_minutes: u16,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub summary: String,
    pub relative_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}
```

**Example:**
```json
{
  "slug": "alien-1979",
  "title": "Alien",
  "year": 1979,
  "runtime_minutes": 117,
  "poster_url": "/api/artwork/alien-1979/poster",
  "backdrop_url": "/api/artwork/alien-1979/backdrop",
  "summary": "In space no one can hear you scream.",
  "relative_path": "Movies/Alien (1979)/Alien (1979).mp4",
  "size_bytes": 4589934592
}
```

---

### `MovieDetail`

**Purpose:** Full movie detail for detail screen.

```rust
pub struct MovieDetail {
    pub slug: String,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub runtime_minutes: Option<u16>,
    pub summary: Option<String>,
    pub genres: Vec<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub cast: Vec<CastMemberDto>,
    pub crew: Vec<CrewMemberDto>,
    pub is_favorite: bool,
    pub watch_progress_seconds: Option<u32>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub file: MovieFileInfo,
    pub stream_url: String,
}
```

**Example:**
```json
{
  "slug": "alien-1979",
  "title": "Alien",
  "original_title": null,
  "year": 1979,
  "runtime_minutes": 117,
  "summary": "In space no one can hear you scream.",
  "genres": ["Horror", "Sci-Fi"],
  "poster_url": "/api/artwork/alien-1979/poster",
  "backdrop_url": "/api/artwork/alien-1979/backdrop",
  "cast": [...],
  "crew": [...],
  "is_favorite": false,
  "watch_progress_seconds": null,
  "tmdb_id": "348",
  "imdb_id": "tt0078748",
  "file": {...},
  "stream_url": "/stream/alien-1979"
}
```

---

### `MovieFileInfo`

**Purpose:** On-disk file metadata.

```rust
pub struct MovieFileInfo {
    pub filename: String,
    pub relative_path: String,
    pub extension: Option<String>,
    pub size_bytes: Option<u64>,
    pub content_type: String,
    pub modified_at: Option<u64>,
    pub scanned_at: Option<u64>,
}
```

**Example:**
```json
{
  "filename": "Alien (1979).mp4",
  "relative_path": "Movies/Alien (1979)/Alien (1979).mp4",
  "extension": "mp4",
  "size_bytes": 4589934592,
  "content_type": "video/mp4",
  "modified_at": 1699000000,
  "scanned_at": 1700000000
}
```

---

### `CastMemberDto`

**Purpose:** Cast member in movie detail.

```rust
pub struct CastMemberDto {
    pub name: String,
    pub character: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tmdb_person_id: Option<u32>,
}
```

**Example:**
```json
{
  "name": "Sigourney Weaver",
  "character": "Ripley",
  "profile_url": "https://image.tmdb.org/t/p/w185/...",
  "tmdb_person_id": 10205
}
```

---

### `CrewMemberDto`

**Purpose:** Crew member in movie detail.

```rust
pub struct CrewMemberDto {
    pub name: String,
    pub job: Option<String>,
}
```

**Example:**
```json
{
  "name": "Ridley Scott",
  "job": "Director"
}
```

---

## Library Models

**File:** `models/library.rs`

### `ScanStartRequest`

**Purpose:** Scan initiation request body.

```rust
pub struct ScanStartRequest {
    pub full: bool,  // Re-fetch all TMDB metadata
}
```

**Example:**
```json
{
  "full": false
}
```

---

### `LibraryStatusResponse`

**Purpose:** Current scan state.

```rust
pub struct LibraryStatusResponse {
    pub state: String,                // "idle" or "scanning"
    pub last_scan_at: Option<String>, // ISO timestamp
    pub last_scan_duration_secs: u64,
    pub movies_count: usize,
    pub scan_in_progress: bool,
    pub progress: Option<ScanProgress>,
}
```

**Idle Example:**
```json
{
  "state": "idle",
  "last_scan_at": "1700000000",
  "last_scan_duration_secs": 120,
  "movies_count": 150,
  "scan_in_progress": false,
  "progress": null
}
```

**Scanning Example:**
```json
{
  "state": "scanning",
  "last_scan_at": "1700000000",
  "last_scan_duration_secs": 120,
  "movies_count": 150,
  "scan_in_progress": true,
  "progress": {
    "phase": "enriching",
    "files_seen": 100,
    "candidates": 50,
    "errors": 0,
    "enriched": 25,
    "total_to_enrich": 50,
    "current_path": "Movies/Blade Runner (1982)/movie.mp4"
  }
}
```

---

### `ScanProgress`

**Purpose:** Live scan progress (defined in `services/scan_state.rs`).

```rust
pub struct ScanProgress {
    pub phase: Option<ScanPhase>,
    pub files_seen: u32,
    pub candidates: u32,
    pub errors: u32,
    pub enriched: u32,
    pub total_to_enrich: u32,
    pub current_path: Option<String>,
}
```

---

### `ScanPhase`

```rust
pub enum ScanPhase {
    Discovering,   // Walking filesystem
    Enriching,     // Fetching TMDB metadata
    Persisting,    // Writing to SQLite
}
```

---

### `SearchResponse`

**Purpose:** Search results.

```rust
pub struct SearchResponse {
    pub query: String,
    pub movies: Vec<MovieSummary>,
    pub total: usize,
}
```

**Example:**
```json
{
  "query": "alien",
  "movies": [...],
  "total": 3
}
```

---

### `GenresResponse`

**Purpose:** Genre list with counts.

```rust
pub struct GenresResponse {
    pub genres: Vec<GenreEntry>,
}
```

---

### `GenreEntry`

```rust
pub struct GenreEntry {
    pub name: String,
    pub count: u32,
}
```

**Example:**
```json
{
  "name": "Action",
  "count": 45
}
```

---

### `FavoriteRequest`

**Purpose:** Toggle/set favorite request body.

```rust
pub struct FavoriteRequest {
    pub favorite: Option<bool>,  // Omit to toggle
}
```

**Examples:**
```json
{ "favorite": true }   // Set favorite
{ "favorite": false }  // Remove favorite
{}                     // Toggle
```

---

### `FavoriteResponse`

```rust
pub struct FavoriteResponse {
    pub slug: String,
    pub favorite: bool,
}
```

**Example:**
```json
{
  "slug": "alien-1979",
  "favorite": true
}
```

---

### `MatchRequest`

**Purpose:** Manual TMDB match request.

```rust
pub struct MatchRequest {
    pub tmdb_id: String,  // "348" or "tmdb:348"
}
```

---

### `ProgressRequest`

**Purpose:** Watch progress update.

```rust
pub struct ProgressRequest {
    pub position_seconds: u32,
    pub duration_seconds: Option<u32>,
}
```

**Example:**
```json
{
  "position_seconds": 1200,
  "duration_seconds": 7020
}
```

---

### `ProgressResponse`

```rust
pub struct ProgressResponse {
    pub slug: String,
    pub position_seconds: u32,
    pub duration_seconds: Option<u32>,
    pub updated_at: String,
}
```

---

## Browse Models

**File:** `models/browse.rs`

### `BrowseResponse`

**Purpose:** Netflix-style home feed.

```rust
pub struct BrowseResponse {
    pub hero: Option<MovieSummary>,
    pub rows: Vec<BrowseRow>,
}
```

**Example:**
```json
{
  "hero": {
    "slug": "dune-2021",
    "title": "Dune",
    "backdrop_url": "/api/artwork/dune-2021/backdrop",
    ...
  },
  "rows": [
    {
      "slug": "continue-watching",
      "title": "Continue Watching",
      "row_type": "continue_watching",
      "movies": [...]
    },
    {
      "slug": "recently-added",
      "title": "Recently Added",
      "row_type": "recently_added",
      "movies": [...]
    }
  ]
}
```

---

### `BrowseRow`

```rust
pub struct BrowseRow {
    pub slug: String,
    pub title: String,
    pub row_type: String,
    pub movies: Vec<MovieSummary>,
}
```

**Row Types:**
- `continue_watching`
- `recently_added`
- `favorites`
- `genre-{name}`

---

## Person Models

**File:** `models/person.rs`

### `KnownForMovie`

**Purpose:** Movie entry in person's filmography.

```rust
pub struct KnownForMovie {
    pub slug: String,
    pub title: String,
    pub year: Option<u16>,
    pub poster_url: Option<String>,
    pub character: Option<String>,
}
```

**Example:**
```json
{
  "slug": "pirates-of-the-caribbean-2003",
  "title": "Pirates of the Caribbean: The Curse of the Black Pearl",
  "year": 2003,
  "poster_url": "/api/artwork/pirates-of-the-caribbean-2003/poster",
  "character": "Jack Sparrow"
}
```

---

### `PersonDetail`

**Purpose:** Full person detail response.

```rust
pub struct PersonDetail {
    pub tmdb_person_id: u32,
    pub name: String,
    pub biography: Option<String>,
    pub birthday: Option<String>,
    pub deathday: Option<String>,
    pub place_of_birth: Option<String>,
    pub profile_url: Option<String>,
    pub known_for_department: Option<String>,
    pub gender: Option<i32>,
    pub also_known_as: Vec<String>,
    pub known_for: Vec<KnownForMovie>,
}
```

**Example:**
```json
{
  "tmdb_person_id": 85,
  "name": "Johnny Depp",
  "biography": "John Christopher Depp II is an American actor and musician...",
  "birthday": "1963-06-09",
  "deathday": null,
  "place_of_birth": "Owensboro, Kentucky, USA",
  "profile_url": "https://image.tmdb.org/t/p/w185/...",
  "known_for_department": "Acting",
  "gender": 2,
  "also_known_as": ["John Depp", "Johnny"],
  "known_for": [...]
}
```

---

### `KnownForMovie::from_summary`

**Constructor:**
```rust
pub fn from_summary(summary: MovieSummary, character: Option<String>) -> Self
```

**Purpose:** Converts movie summary to known-for entry with character name.

---

## Root Models

**File:** `models/root.rs`

### `RootResponse`

**Purpose:** API index at `GET /`.

```rust
pub struct RootResponse {
    pub service: &'static str,
    pub message: &'static str,
    pub endpoints: RootEndpoints,
}
```

**Default Implementation:**
```rust
impl Default for RootResponse {
    fn default() -> Self {
        Self {
            service: "loon-server",
            message: "Loon API — use the endpoints below (no web UI at / yet)",
            endpoints: RootEndpoints {
                health: "/api/health",
                movies: "/api/movies",
            },
        }
    }
}
```

**Example:**
```json
{
  "service": "loon-server",
  "message": "Loon API — use the endpoints below (no web UI at / yet)",
  "endpoints": {
    "health": "/api/health",
    "movies": "/api/movies"
  }
}
```

---

### `RootEndpoints`

```rust
pub struct RootEndpoints {
    pub health: &'static str,
    pub movies: &'static str,
}
```

---

## Type Mappings

### Internal to DTO

| Internal Type | DTO Type | Conversion |
|--------------|----------|------------|
| `LoonMovieRecord` | `MovieSummary` | `to_summary()` |
| `LoonMovieRecord` | `MovieDetail` | `to_detail()` |
| `LoonMovieRecord` | `KnownForMovie` | `KnownForMovie::from_summary()` |
| `PersonRecord` | `PersonDetail` | `record_to_detail()` |

---

### Serialization Notes

**Conditional Fields:**
- `#[serde(skip_serializing_if = "Option::is_none")]` - Omits null values
- `#[serde(default)]` - Uses default for missing values

**Example:**
```rust
pub struct MovieSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,  // Omitted if None
}
```
