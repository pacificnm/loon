# Services Documentation

## Service Modules

```
src/services/
├── catalog.rs         # In-memory movie catalog
├── scan_service.rs    # Library scan orchestration
├── enrichment.rs      # TMDB metadata enrichment
├── tmdb.rs            # TMDB runtime configuration
├── tmdb_match.rs      # Manual TMDB matching
├── artwork.rs         # Artwork caching
├── ai.rs              # AI runtime (Ollama)
├── filename_guess.rs  # AI filename parsing
├── streaming.rs       # HTTP byte-range streaming
├── browse.rs          # Browse feed builder
├── person.rs          # Person caching and lookup
├── cast_backfill.rs   # Cast person ID backfill
├── slug.rs            # URL slug generation
├── media_file.rs      # File metadata helpers
├── scan_state.rs      # Scan coordination
├── scan_events.rs     # Scan SSE events
└── library.rs         # Library discovery
```

---

## Catalog Service

**File:** `services/catalog.rs`

### Types

#### `LoonMovieRecord`

**Purpose:** Complete movie record in the in-memory catalog.

```rust
pub struct LoonMovieRecord {
    pub media_id: String,              // Stable ID (file:{path})
    pub slug: String,                  // URL-safe identifier
    pub relative_path: String,         // Path relative to media_root
    pub title: String,                 // Display title
    pub original_title: Option<String>,
    pub year: Option<u16>,
    pub runtime_minutes: Option<u16>,
    pub summary: Option<String>,
    pub genres: Vec<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub cast: Vec<CastMemberDto>,
    pub crew: Vec<CrewMemberDto>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_locked: bool,             // Prevent auto-overwrite
    pub scanned_at: u64,               // Last scan timestamp
    pub size_bytes: Option<u64>,
    pub modified_secs: Option<u64>,
    pub is_favorite: bool,
    pub watch_progress_seconds: Option<u32>,
    pub watch_duration_seconds: Option<u32>,
}
```

#### `LoonCatalog`

**Purpose:** In-memory index of all movies.

```rust
pub struct LoonCatalog {
    by_slug: HashMap<String, LoonMovieRecord>,
}
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new()` | `fn new() -> Self` | Creates empty catalog |
| `len()` | `fn len(&self) -> usize` | Movie count |
| `list()` | `fn list(&self) -> Vec<MovieSummary>` | All movies sorted by title |
| `get()` | `fn get(&self, slug: &str) -> Option<&LoonMovieRecord>` | Lookup by slug |
| `get_mut()` | `fn get_mut(&mut self, slug: &str) -> Option<&mut LoonMovieRecord>` | Mutable lookup |
| `insert()` | `fn insert(&mut self, record: LoonMovieRecord)` | Insert/replace |
| `replace()` | `fn replace(&mut self, records: Vec<LoonMovieRecord>)` | Replace all |

### Key Functions

#### `apply_tmdb_fetch`

**Signature:**
```rust
pub fn apply_tmdb_fetch(
    record: &mut LoonMovieRecord,
    tmdb_id: u32,
    metadata: &MovieMetadata,
    artwork: Option<&ScanArtwork>,
)
```

**Purpose:** Updates record with TMDB metadata while preserving slug and path.

**Fields Updated:**
- `title`, `original_title`, `year`
- `runtime_minutes`, `summary`
- `genres`, `cast`, `crew`
- `tmdb_id`, `imdb_id`
- `poster_url`, `backdrop_url`

#### `catalog_from_scan`

**Signature:**
```rust
pub fn catalog_from_scan(result: ScanResult, artwork: &ScanArtworkMap) -> LoonCatalog
```

**Purpose:** Builds catalog from filesystem scan results.

**Behavior:**
- Rejects candidates with path traversal (`..`)
- Applies artwork URLs from enrichment
- Generates slugs from title/year or filename

---

## Scan Service

**File:** `services/scan_service.rs`

### Types

#### `ScanOptions`

```rust
pub struct ScanOptions {
    pub full_metadata: bool,  // Re-fetch all TMDB data
}
```

#### `ScanRunResult`

```rust
pub struct ScanRunResult {
    pub scanned_at: u64,       // Completion timestamp
    pub movies_count: usize,   // Total movies after persist
    pub stats: ScanStats,      // Scan statistics
}
```

### Key Functions

#### `scan_and_persist`

**Signature:**
```rust
pub async fn scan_and_persist(
    config: &ServerConfig,
    repo: &LibraryRepository,
    tmdb: Option<&TmdbRuntime>,
    ai: Option<&AiRuntime>,
    options: ScanOptions,
    reporter: Option<&ScanReporter>,
    artwork_cache: Option<&ArtworkRuntime>,
) -> NestResult<ScanRunResult>
```

**Phases:**

1. **Discovering** - Filesystem walk via `discover_library()`
2. **Enriching** - TMDB metadata fetch for changed files
3. **Persisting** - SQLite upsert and orphan deletion

**Scan Flow:**
```rust
// 1. Filesystem discovery
let mut result = discover_library(&config)?;

// 2. Determine which files need enrichment
for candidate in &result.candidates {
    if should_enrich_metadata(options, existing_file, existing_movie, candidate) {
        refreshed_paths.insert(path);
    }
}

// 3. Enrich candidates
for candidate in &mut result.candidates {
    if needs_tmdb {
        if let Some(tmdb_id) = stored_tmdb_id(existing_movie) {
            // Use cached TMDB ID for fast lookup
            enrich_candidate_by_tmdb_id(candidate, tmdb_id, tmdb, ...).await;
        } else {
            // AI filename guess if title unknown
            if let Some(ai) = ai {
                apply_ai_filename_guess(ai, candidate, &path).await;
            }
            // Full TMDB search
            enrich_candidate(candidate, tmdb, &mut artwork_map).await;
        }
    }
}

// 4. Persist to SQLite
for record in catalog.records() {
    // Respect tmdb_locked - don't overwrite manual matches
    if existing.tmdb_locked {
        record = existing;
    }
    repo.upsert_movie(&library_id, &record, scanned_at, size_bytes, modified_secs)?;
}

// 5. Remove orphans
repo.delete_orphans(&library_id, &seen_paths)?;
```

#### `should_enrich_metadata`

**Signature:**
```rust
fn should_enrich_metadata(
    options: ScanOptions,
    existing_file: Option<&StoredFile>,
    existing_movie: Option<&LoonMovieRecord>,
    candidate: &MovieScanCandidate,
) -> bool
```

**Logic:**
```rust
// Never enrich manually locked movies
if existing_movie.is_some_and(|m| m.tmdb_locked) {
    return false;
}

// Full scan forces enrichment
if options.full_metadata {
    return true;
}

// Incremental: enrich if file changed
should_refresh_metadata(existing_file, candidate)
```

#### `load_catalog_from_db`

**Signature:**
```rust
pub fn load_catalog_from_db(repo: &LibraryRepository) -> NestResult<LoonCatalog>
```

**Purpose:** Rebuilds in-memory catalog from SQLite on startup.

---

## Enrichment Service

**File:** `services/enrichment.rs`

### Types

#### `ScanArtwork`

```rust
pub struct ScanArtwork {
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
}
```

#### `ScanArtworkMap`

```rust
pub type ScanArtworkMap = HashMap<String, ScanArtwork>;
```

### Key Functions

#### `enrich_with_tmdb`

**Signature:**
```rust
pub async fn enrich_with_tmdb(result: &mut ScanResult, tmdb: &TmdbRuntime) -> ScanArtworkMap
```

**Purpose:** Enriches all candidates with TMDB metadata.

#### `enrich_candidate`

**Signature:**
```rust
pub async fn enrich_candidate(
    candidate: &mut MovieScanCandidate,
    tmdb: &TmdbRuntime,
    artwork: &mut ScanArtworkMap,
)
```

**Process:**
1. Generate title variants (guessed title, spaced, camelCase split)
2. Search TMDB with variants
3. Pick best result using guessed year for disambiguation
4. Fetch full metadata
5. Extract artwork URLs

#### `search_title_variants`

**Signature:**
```rust
pub fn search_title_variants(candidate: &MovieScanCandidate) -> Vec<String>
```

**Generates variants from:**
- Guessed title (from AI or filename)
- Spaced version of concatenated titles ("AngerManagement" → "Anger Management")
- CamelCase split ("BladeRunner" → "Blade Runner")
- Filename stem (fallback)

**Example:**
```rust
// Input: "AngerManagement2003.mp4" with AI guess "AngerManagement"
variants = [
    "angermanagement",       // Normalized guess
    "anger management",      // Spaced alternate
    "anger management 2003", // From filename
]
```

#### `pick_best_search_result`

**Signature:**
```rust
fn pick_best_search_result(
    results: &[MovieSearchResult],
    guessed_year: Option<u16>,
) -> &MovieSearchResult
```

**Logic:**
- Without year: return first result
- With year: pick result with minimum year distance
- Tie-breaker: preserve TMDB result order

---

## TMDB Service

**File:** `services/tmdb.rs`

### Types

#### `TmdbRuntime`

```rust
pub struct TmdbRuntime {
    pub provider: TmdbMetadataProvider,  // Metadata fetcher
    pub images: TmdbImageService,        // URL builder
}
```

**Methods:**
```rust
pub fn from_config(config: &TmdbConfig) -> NestResult<Self>
```

**Usage:**
```rust
let tmdb = TmdbRuntime::from_config(&tmdb_config)?;
let fetch = tmdb.provider.fetch_movie(external_id).await?;
let poster_url = tmdb.images.poster_url(path, ImageSize::W500).await;
```

---

## TMDB Match Service

**File:** `services/tmdb_match.rs`

### Key Functions

#### `parse_tmdb_id`

**Signature:**
```rust
pub fn parse_tmdb_id(raw: &str) -> Result<u32, ServeError>
```

**Accepted formats:**
- `"348"` → `348`
- `"tmdb:348"` → `348`
- `"  tmdb:348  "` → `348`

**Errors:** Empty or non-numeric input

#### `rematch_movie_by_tmdb_id`

**Signature:**
```rust
pub async fn rematch_movie_by_tmdb_id(
    slug: &str,
    tmdb_id: u32,
    repo: &LibraryRepository,
    tmdb: &TmdbRuntime,
    artwork_cache: Option<&ArtworkRuntime>,
    library_id: &str,
) -> NestResult<LoonMovieRecord>
```

**Process:**
1. Fetch movie metadata from TMDB
2. Apply to existing record
3. Set `tmdb_locked = true` (prevent auto-overwrite)
4. Invalidate artwork cache
5. Persist to SQLite

---

## Artwork Service

**File:** `services/artwork.rs`

### Types

#### `ArtworkKind`

```rust
pub enum ArtworkKind {
    Poster,
    Backdrop,
}
```

#### `ArtworkPayload`

```rust
pub struct ArtworkPayload {
    pub bytes: Vec<u8>,
    pub content_type: String,
}
```

#### `ArtworkRuntime`

```rust
pub struct ArtworkRuntime {
    cache: Cache,
    adapter: Arc<FileCacheAdapter>,
    http: HttpClientService,
}
```

### Key Functions

#### `proxy_url`

**Signature:**
```rust
pub fn proxy_url(slug: &str, kind: ArtworkKind, remote_url: &Option<String>) -> Option<String>
```

**Purpose:** Generates `/api/artwork/:slug/:kind` URL when remote artwork exists.

```rust
// Returns Some("/api/artwork/alien-1979/poster") if remote_url is Some
// Returns None if remote_url is None
```

#### `get_cached`

**Signature:**
```rust
pub fn get_cached(&self, slug: &str, kind: ArtworkKind) -> NestResult<Option<ArtworkPayload>>
```

**Purpose:** Returns cached artwork bytes if present.

#### `fetch_and_cache`

**Signature:**
```rust
pub async fn fetch_and_cache(
    &self,
    slug: &str,
    kind: ArtworkKind,
    source_url: &str,
) -> NestResult<ArtworkPayload>
```

**Process:**
1. HTTP GET from source URL
2. Extract Content-Type header
3. Store in file cache with metadata
4. Return bytes

#### `invalidate_movie`

**Signature:**
```rust
pub fn invalidate_movie(&self, slug: &str) -> NestResult<u64>
```

**Purpose:** Clears cached artwork for a movie (called on metadata refresh).

---

## AI Service

**File:** `services/ai.rs`

### Types

#### `AiRuntime`

```rust
pub struct AiRuntime {
    pub provider: Arc<dyn AiProvider>,  // OllamaProvider
    pub min_confidence: f32,            // Confidence threshold
}
```

**Methods:**
```rust
pub fn from_config(config: &LoonAiConfig) -> NestResult<Self>
```

---

## Filename Guess Service

**File:** `services/filename_guess.rs`

### Types

#### `MovieFilenameGuess`

```rust
pub struct MovieFilenameGuess {
    pub search_title: String,   // Normalized title for TMDB search
    pub likely_year: Option<u16>,
    pub confidence: f32,
}
```

### Key Functions

#### `guess_movie_from_filename`

**Signature:**
```rust
pub async fn guess_movie_from_filename(
    ai: &AiRuntime,
    relative_path: &str,
) -> Option<MovieFilenameGuess>
```

**System Prompt:**
```
You are helping identify movies from filenames.
Return ONLY valid JSON with this shape:
{
  "search_title": string,
  "likely_year": number | null,
  "likely_genres": string[],
  "confidence": number
}
```

**Example:**
```
Input:  "Movies/Alien.1979.BluRay.x264.mp4"
Output: MovieFilenameGuess {
    search_title: "Alien",
    likely_year: Some(1979),
    confidence: 0.95,
}
```

#### `parse_guess`

**Signature:**
```rust
fn parse_guess(raw: &str, min_confidence: f32) -> Option<MovieFilenameGuess>
```

**Validation:**
- Year must be 1888-2100
- Confidence must meet threshold
- Title must be non-empty

---

## Streaming Service

**File:** `services/streaming.rs`

### Key Functions

#### `stream_movie`

**Signature:**
```rust
pub async fn stream_movie(ctx: RequestContext) -> HttpResult
```

**Features:**
- Byte-range request support (HTTP 206)
- Path traversal prevention
- Content-Type detection from extension

**Range Parsing:**
```rust
fn parse_range(header: Option<&str>, total: u64) -> RangeParseResult {
    // "bytes=0-"     → Full file from start
    // "bytes=100-199" → Bytes 100-199
    // "bytes=-500"    → Last 500 bytes
}
```

**Security:**
```rust
fn resolve_media_path(media_root: &Path, relative_path: &str) -> Result<PathBuf, Error> {
    // Reject path traversal
    if relative_path.contains("..") {
        return Err(movie_not_found("invalid"));
    }
    
    // Verify canonical path is under media_root
    let canonical = std::fs::canonicalize(&path)?;
    let root = std::fs::canonicalize(media_root)?;
    if !canonical.starts_with(&root) {
        return Err(movie_not_found("invalid"));
    }
    
    Ok(canonical)
}
```

---

## Browse Service

**File:** `services/browse.rs`

### Key Functions

#### `build_browse`

**Signature:**
```rust
pub fn build_browse(repo: &LibraryRepository) -> NestResult<BrowseResponse>
```

**Row Order:**
1. Continue Watching (incomplete progress)
2. Recently Added (last 20 by scan date)
3. Favorites (last 20)
4. Genre rows (min 3 movies each)

**Hero Selection:**
- First recently-added with backdrop
- Falls back to first recently-added

---

## Person Service

**File:** `services/person.rs`

### Types

#### `PersonRecord`

```rust
pub struct PersonRecord {
    pub tmdb_person_id: u32,
    pub name: String,
    pub biography: Option<String>,
    pub birthday: Option<String>,
    pub deathday: Option<String>,
    pub place_of_birth: Option<String>,
    pub profile_path: Option<String>,
    pub known_for_department: Option<String>,
    pub gender: Option<i32>,
    pub also_known_as: Vec<String>,
    pub updated_at: u64,
}
```

### Key Functions

#### `get_person_detail`

**Signature:**
```rust
pub async fn get_person_detail(
    tmdb_person_id: u32,
    repo: &LibraryRepository,
    movies: &[LoonMovieRecord],
    tmdb: &TmdbRuntime,
) -> NestResult<PersonDetail>
```

**Process:**
1. Check SQLite cache
2. If not cached: fetch from TMDB and cache
3. Find library movies featuring person
4. Sort by year descending

#### `get_person_for_cast`

**Signature:**
```rust
pub async fn get_person_for_cast(
    movie_slug: &str,
    cast_name: &str,
    repo: &LibraryRepository,
    movies: &[LoonMovieRecord],
    tmdb: &TmdbRuntime,
    library_id: &str,
) -> NestResult<PersonDetail>
```

**Process:**
1. Load movie by slug
2. Backfill cast person IDs (if missing)
3. Find TMDB ID for cast member
4. Call `get_person_detail`

---

## Cast Backfill Service

**File:** `services/cast_backfill.rs`

### Key Functions

#### `backfill_cast_person_ids`

**Signature:**
```rust
pub async fn backfill_cast_person_ids(
    record: &mut LoonMovieRecord,
    tmdb: &TmdbRuntime,
) -> NestResult<bool>
```

**Purpose:** Fills missing `tmdb_person_id` values from TMDB movie credits.

**Behavior:**
- Only fetches if cast has missing person IDs
- Matches cast members by name (case-insensitive)
- Also backfills profile URLs

---

## Slug Service

**File:** `services/slug.rs`

### Key Functions

#### `slugify`

**Signature:**
```rust
pub fn slugify(text: &str) -> String
```

**Rules:**
- Lowercase
- Non-alphanumeric → single `-`
- Trim leading/trailing `-`

**Examples:**
```
"Alien" → "alien"
"Blade Runner" → "blade-runner"
"Star Wars: A New Hope" → "star-wars-a-new-hope"
```

#### `movie_slug`

**Signature:**
```rust
pub fn movie_slug(title: &str, year: Option<u16>) -> String
```

**Format:**
- With year: `{title}-{year}` → `"alien-1979"`
- Without year: `{title}` → `"the-matrix"`

#### `unique_movie_slug`

**Signature:**
```rust
pub fn unique_movie_slug(
    title: &str,
    year: Option<u16>,
    relative_path: &str,
    existing: &HashMap<String, ()>,
) -> String
```

**Collision Handling:**
1. Try `{title}-{year}` (e.g., `"alien-1979"`)
2. If collision: try filename stem (e.g., `"bluray-remux"`)
3. If still collision: append counter (e.g., `"alien-1979-2"`)

---

## Media File Service

**File:** `services/media_file.rs`

### Key Functions

#### `content_type_for_extension`

**Signature:**
```rust
pub fn content_type_for_extension(ext: Option<&str>) -> &'static str
```

**Mappings:**
| Extension | Content-Type |
|-----------|--------------|
| `mp4`, `m4v`, `mov` | `video/mp4` |
| `mkv` | `video/x-matroska` |
| `webm` | `video/webm` |
| `avi` | `video/x-msvideo` |
| other | `application/octet-stream` |

#### `file_info_from_record`

**Signature:**
```rust
pub fn file_info_from_record(record: &LoonMovieRecord) -> MovieFileInfo
```

**Extracts:**
- Filename from path
- Extension (lowercase)
- Content-Type
- Size, modified, scanned timestamps

---

## Scan State Service

**File:** `services/scan_state.rs`

### Types

#### `ScanPhase`

```rust
pub enum ScanPhase {
    Discovering,   // Filesystem walk
    Enriching,     // TMDB fetch
    Persisting,    // SQLite upsert
}
```

#### `ScanProgress`

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

#### `ScanCoordinator`

```rust
pub struct ScanCoordinator {
    running: Mutex<bool>,
    last_scan_at: AtomicU64,
    last_duration_secs: AtomicU64,
    progress: Mutex<Option<ScanProgress>>,
}
```

**Methods:**
| Method | Purpose |
|--------|---------|
| `try_start()` | Acquire scan lock (returns false if already running) |
| `finish()` | Release lock, store timing |
| `is_running()` | Check scan state |
| `set_progress()` | Update progress snapshot |
| `progress()` | Get current progress |

---

## Scan Events Service

**File:** `services/scan_events.rs`

### Types

#### `ScanStreamEvent`

```rust
pub enum ScanStreamEvent {
    Started { scan_id: String },
    Progress { progress: ScanProgress },
    Complete { scan_id: String, movies_count: usize, duration_secs: u64, stats: ScanStats },
    Error { scan_id: String, message: String },
}
```

#### `ScanReporter`

```rust
pub struct ScanReporter {
    scan_id: String,
    coordinator: Option<Arc<ScanCoordinator>>,
    events: Option<mpsc::Sender<ScanStreamEvent>>,
}
```

**Methods:**
- `started()` - Emit started event
- `progress()` - Update coordinator and emit
- `complete()` - Emit completion with stats
- `error()` - Emit failure

---

## Library Service

**File:** `services/library.rs`

### Key Functions

#### `discover_library`

**Signature:**
```rust
pub fn discover_library(config: &ServerConfig) -> NestResult<ScanResult>
```

**Process:**
1. Create scoped `FileService` with `media_root`
2. Run `LibraryScanner::discover()`
3. Return `ScanResult` with candidates and errors
