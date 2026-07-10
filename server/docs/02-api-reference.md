# API Reference

## Handler Modules

```
src/api/
├── health.rs        # Health check
├── movies.rs        # Movie listing and details
├── stream.rs        # Video streaming
├── library.rs       # Library scanning
├── favorites.rs     # Favorite management
├── match_handler.rs # TMDB matching
├── progress.rs      # Watch progress
├── search.rs        # Title search
├── browse.rs        # Browse feed
├── genres.rs        # Genre listing
├── artwork.rs       # Artwork proxy
└── people.rs        # Person details
```

---

## Root Endpoint

### `GET /`

**File:** `api/root.rs`

**Handler:** `root()`

**Response:** `RootResponse`

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

## Health Endpoints

### `GET /api/health`

**File:** `api/health.rs`

**Handler:** `health()`

**Purpose:** Confirms server is running and returns movie count.

**Response:** `HealthResponse`

```json
{
  "status": "ok",
  "service": "loon-server",
  "version": "0.1.0",
  "movies_count": 150,
  "library_scanned_at": 1700000000
}
```

**Implementation:**
```rust
pub async fn health(_ctx: RequestContext) -> HttpResult {
    let app = state::app_state();
    let movies_count = app.repo.movie_count()
        .unwrap_or_else(|_| app.catalog.read().len());
    
    Json(HealthResponse {
        status: "ok",
        service: "loon-server",
        version: env!("CARGO_PKG_VERSION"),
        movies_count,
        library_scanned_at: state::library_scanned_at(),
    }).into_response()
}
```

---

## Movie Endpoints

### `GET /api/movies`

**File:** `api/movies.rs`

**Handler:** `list_movies()`

**Purpose:** Lists movies with optional pagination and filtering.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | `u32` | `1` | Page number (1-indexed) |
| `limit` | `u32` | `50` | Items per page (max 100) |
| `sort` | `string` | `"title"` | Sort: `title`, `year`, `recently_added` |
| `genre` | `string` | — | Filter by genre name |

**Response (unpaginated):**
```json
{
  "movies": [...],
  "total": 150
}
```

**Response (paginated):**
```json
{
  "movies": [...],
  "total": 150,
  "page": 1,
  "limit": 50,
  "pages": 3
}
```

**Implementation:**
```rust
pub async fn list_movies(ctx: RequestContext) -> HttpResult {
    let paginated = ctx.query("page").is_some() || ctx.query("limit").is_some();
    
    if paginated {
        let page = parse_u32(ctx.query("page"), 1)?.max(1);
        let limit = parse_u32(ctx.query("limit"), 50)?.clamp(1, 100);
        let sort = match ctx.query("sort").unwrap_or("title") {
            "year" => MovieSort::Year,
            "recently_added" => MovieSort::RecentlyAdded,
            "title" => MovieSort::Title,
            other => return Err(invalid_request(format!("invalid sort: {other}"))),
        };
        let genre = ctx.query("genre").map(str::to_string);
        
        let query = MovieListQuery { page, limit, sort, genre };
        let total = state::repo().count_movies(&query)?;
        let records = state::repo().list_movies(&query)?;
        // ...
    }
    
    // Unpaginated: return full catalog
    let catalog = state::catalog();
    let movies = catalog.read().list();
    Json(MovieListResponse { movies, total: movies.len(), .. })
}
```

---

### `GET /api/movies/:slug`

**File:** `api/movies.rs`

**Handler:** `get_movie()`

**Purpose:** Returns full movie details including cast, crew, and file info.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | URL-safe movie identifier |

**Response:** `MovieDetail`

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

**Implementation:**
```rust
pub async fn get_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let app = state::app_state();
    
    let mut record = state::repo().get_by_slug(&slug)?
        .or_else(|| catalog.read().get(&slug).cloned())
        .ok_or_else(|| movie_not_found(&slug))?;
    
    // Backfill cast person IDs if TMDB enabled
    if let Some(tmdb) = app.tmdb.as_ref() {
        if backfill_cast_person_ids(&mut record, tmdb).await? {
            persist_cast_backfill(&app, &record)?;
        }
    }
    
    Json(record.to_detail()).into_response()
}
```

---

## Streaming Endpoints

### `GET /stream/:slug`

**File:** `api/stream.rs`

**Handler:** `stream_movie()`

**Purpose:** Streams video file with HTTP byte-range support.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | Movie identifier |

**Headers:**
- `Range: bytes=start-end` (optional)

**Response (full):**
```
HTTP/1.1 200 OK
Content-Type: video/mp4
Accept-Ranges: bytes
Content-Length: 4589934592
```

**Response (partial):**
```
HTTP/1.1 206 Partial Content
Content-Type: video/mp4
Accept-Ranges: bytes
Content-Length: 8388608
Content-Range: bytes 0-8388607/4589934592
```

**Implementation:**
```rust
pub async fn stream_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    let state = state::app_state();
    
    // Lookup relative_path from catalog
    let relative_path = catalog.read().get(slug)
        .map(|r| r.relative_path.clone())
        .or_else(|| repo().get_by_slug(slug).ok().flatten().map(|r| r.relative_path))
        .ok_or_else(|| movie_not_found(slug))?;
    
    // Resolve and validate path (prevent traversal)
    let file_path = resolve_media_path(&state.media_root, &relative_path)?;
    let metadata = tokio::fs::metadata(&file_path).await?;
    let total = metadata.len();
    
    // Parse Range header
    let range_header = ctx.header("range");
    match parse_range(range_header, total) {
        RangeParseResult::Full => stream_full_file(...).await,
        RangeParseResult::Partial { start, end } => stream_partial_file(...).await,
        RangeParseResult::Invalid => Ok(invalid_range_response(total)),
    }
}
```

**Range Parsing:**
```rust
fn parse_range(header: Option<&str>, total: u64) -> RangeParseResult {
    let Some(header) = header else {
        return RangeParseResult::Full;
    };
    
    let Some(spec) = header.strip_prefix("bytes=") else {
        return RangeParseResult::Invalid;
    };
    
    // Parse "start-end", "start-", or "-suffix"
    // ...
}
```

**Security:**
- Path traversal check (`..` rejected)
- Canonical path validation (must be under media_root)

---

## Library Management

### `POST /api/library/scan`

**File:** `api/library.rs`

**Handler:** `start_scan()`

**Purpose:** Initiates library scan with Server-Sent Events progress stream.

**Request Body (optional):**
```json
{
  "full": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `full` | `boolean` | `false` | Re-fetch all TMDB metadata |

**Response:** Server-Sent Events stream

**Event Format:**
```
event: started
data: {"scan_id":"scan-1700000000"}

event: progress
data: {"phase":"enriching","files_seen":100,"candidates":50,"enriched":25,"total_to_enrich":50}

event: complete
data: {"scan_id":"scan-1700000000","movies_count":150,"duration_secs":120,"stats":{...}}

event: error
data: {"scan_id":"scan-1700000000","message":"TMDB API error"}
```

**Event Types:** `ScanStreamEvent`
```rust
pub enum ScanStreamEvent {
    Started { scan_id: String },
    Progress { progress: ScanProgress },
    Complete { scan_id: String, movies_count: usize, duration_secs: u64, stats: ScanStats },
    Error { scan_id: String, message: String },
}
```

**Implementation:**
```rust
pub async fn start_scan(ctx: RequestContext) -> HttpResult {
    let app = state::app_state();
    
    // Single-flight enforcement
    if !app.scan.try_start() {
        return Err(scan_already_running());
    }
    
    let full = ctx.body().is_empty()
        .then(|| ctx.json::<ScanStartRequest>().map(|b| b.full))
        .transpose()?
        .unwrap_or(false);
    
    let (events_tx, events_rx) = mpsc::channel(64);
    let reporter = ScanReporter::new(scan_id, Some(app.scan.clone()), Some(events_tx));
    
    // Spawn background scan task
    tokio::spawn(async move {
        reporter.started().await;
        let result = scan_and_persist(...).await;
        match result {
            Ok(result) => {
                state::replace_catalog(catalog);
                reporter.complete(...).await;
            }
            Err(err) => reporter.error(err.to_string()).await,
        }
    });
    
    // Stream SSE events
    let stream = ReceiverStream::new(events_rx).map(|event| {
        Ok(Bytes::from(format!("event: {}\ndata: {}\n\n", event.event_name(), payload)))
    });
    HttpResponse::event_stream(stream)
}
```

---

### `GET /api/library/status`

**File:** `api/library.rs`

**Handler:** `library_status()`

**Purpose:** Returns current scan state and last scan info.

**Response:** `LibraryStatusResponse`

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

**Progress during scan:**
```json
{
  "state": "scanning",
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

## User Actions

### `PUT /api/movies/:slug/favorite`

**File:** `api/favorites.rs`

**Handler:** `set_favorite()`

**Purpose:** Toggles or sets favorite status.

**Request Body (optional):**
```json
{
  "favorite": true
}
```

**Response:** `FavoriteResponse`

```json
{
  "slug": "alien-1979",
  "favorite": true
}
```

**Implementation:**
```rust
pub async fn set_favorite(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    
    // Toggle if no body, otherwise use provided value
    let favorite = if ctx.body().is_empty() {
        !repo().is_favorite(&slug)?
    } else {
        let body: FavoriteRequest = ctx.json()?;
        body.favorite.ok_or_else(|| invalid_request("favorite required"))?
    };
    
    repo().set_favorite(&slug, favorite)?;
    
    // Update in-memory catalog
    catalog.write().get_mut(&slug).map(|r| r.is_favorite = favorite);
    
    Json(FavoriteResponse { slug, favorite }).into_response()
}
```

---

### `PUT /api/movies/:slug/match`

**File:** `api/match_handler.rs`

**Handler:** `set_tmdb_match()`

**Purpose:** Manually associates movie with TMDB ID.

**Request Body:**
```json
{
  "tmdb_id": "348"
}
```

**Response:** `MovieDetail` (updated record)

**Implementation:**
```rust
pub async fn set_tmdb_match(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let body: MatchRequest = ctx.json()?;
    let tmdb_id = parse_tmdb_id(&body.tmdb_id)?;
    
    let app = state::app_state();
    let Some(tmdb) = app.tmdb.as_ref() else {
        return Err(tmdb_not_configured());
    };
    
    // Fetch and apply TMDB metadata
    let record = rematch_movie_by_tmdb_id(
        &slug, tmdb_id, &app.repo, tmdb, app.artwork.as_ref(), ...
    ).await?;
    
    // Lock to prevent auto-overwrite
    record.tmdb_locked = true;
    
    catalog.write().insert(record.clone());
    Json(record.to_detail()).into_response()
}
```

---

### `PUT /api/movies/:slug/progress`

**File:** `api/progress.rs`

**Handler:** `save_progress()`

**Purpose:** Saves watch progress for resume playback.

**Request Body:**
```json
{
  "position_seconds": 1200,
  "duration_seconds": 7020
}
```

**Response:** `ProgressResponse`

```json
{
  "slug": "alien-1979",
  "position_seconds": 1200,
  "duration_seconds": 7020,
  "updated_at": "1700000000"
}
```

**Implementation:**
```rust
pub async fn save_progress(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?.to_string();
    let body: ProgressRequest = ctx.json()?;
    
    let progress = WatchProgress {
        position_seconds: body.position_seconds,
        duration_seconds: body.duration_seconds,
    };
    
    // Delete if >90% complete (finished)
    let finished = body.duration_seconds.is_some_and(|d| {
        body.position_seconds as f64 / d as f64 > 0.9
    });
    
    repo().save_progress(&slug, &progress)?;
    
    // Update catalog
    if let Some(record) = catalog.write().get_mut(&slug) {
        if finished {
            record.watch_progress_seconds = None;
            record.watch_duration_seconds = None;
        } else {
            record.watch_progress_seconds = Some(body.position_seconds);
            record.watch_duration_seconds = body.duration_seconds;
        }
    }
    
    Json(ProgressResponse { ... }).into_response()
}
```

---

## Browse & Discovery

### `GET /api/browse`

**File:** `api/browse.rs`

**Handler:** `browse()`

**Purpose:** Returns Netflix-style home feed with curated rows.

**Response:** `BrowseResponse`

```json
{
  "hero": {...},
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
    },
    {
      "slug": "genre-action",
      "title": "Action",
      "row_type": "genre",
      "movies": [...]
    }
  ]
}
```

**Row Types:**
- `continue_watching` - Incomplete watch progress
- `recently_added` - Most recently scanned
- `favorites` - User favorites
- `genre-{name}` - Genre-based rows (min 3 movies)

**Implementation:**
```rust
pub async fn browse(_ctx: RequestContext) -> HttpResult {
    if state::app_state().scan.is_running() {
        return Err(library_scanning());
    }
    
    let response = build_browse(&state::repo())?;
    Json(response).into_response()
}
```

---

### `GET /api/search?q=`

**File:** `api/search.rs`

**Handler:** `search()`

**Purpose:** Searches movies by title substring.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | `string` | Yes | Search query (min 2 chars) |
| `limit` | `u32` | No | Max results (default 20, max 50) |

**Response:** `SearchResponse`

```json
{
  "query": "alien",
  "movies": [...],
  "total": 3
}
```

---

### `GET /api/genres`

**File:** `api/genres.rs`

**Handler:** `list_genres()`

**Purpose:** Returns distinct genres with movie counts.

**Response:** `GenresResponse`

```json
{
  "genres": [
    {"name": "Action", "count": 45},
    {"name": "Drama", "count": 32}
  ]
}
```

---

### `GET /api/artwork/:slug/:kind`

**File:** `api/artwork.rs`

**Handler:** `artwork()`

**Purpose:** Proxies or caches movie artwork.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `slug` | `string` | Movie identifier |
| `kind` | `string` | `poster` or `backdrop` |

**Behavior:**
1. Check local cache (if enabled)
2. If cached: return image directly
3. If not cached: fetch from source URL, cache, return
4. If cache disabled: 302 redirect to source URL

**Response (cached):**
```
HTTP/1.1 200 OK
Content-Type: image/jpeg
Content-Length: 123456
```

**Response (redirect):**
```
HTTP/1.1 302 Found
Location: https://image.tmdb.org/t/p/w500/...
```

**Implementation:**
```rust
pub async fn artwork(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    let kind = ArtworkKind::parse(ctx.param("kind")?)
        .ok_or_else(|| invalid_request("kind must be poster or backdrop"))?;
    
    let record = lookup_record(slug).ok_or_else(|| movie_not_found(slug))?;
    let source_url = match kind {
        ArtworkKind::Poster => record.poster_url,
        ArtworkKind::Backdrop => record.backdrop_url,
    }.ok_or_else(|| artwork_not_found(slug, kind))?;
    
    let app = state::app_state();
    
    // Try cache first
    if let Some(artwork) = app.artwork.as_ref() {
        if let Ok(Some(cached)) = artwork.get_cached(slug, kind) {
            return Ok(image_response(cached.bytes, &cached.content_type));
        }
        
        // Fetch and cache
        match artwork.fetch_and_cache(slug, kind, &source_url).await {
            Ok(payload) => return Ok(image_response(payload.bytes, &payload.content_type)),
            Err(error) => warn!("cache fetch failed; redirecting"),
        }
    }
    
    // Fallback redirect
    Ok(HttpResponse::empty(302).with_header("location", source_url))
}
```

---

### `GET /api/people/:tmdb_id`

**File:** `api/people.rs`

**Handler:** `get_person()`

**Purpose:** Returns person details with library movies they appear in.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `tmdb_id` | `string` | TMDB person ID (numeric or `tmdb:` prefix) |

**Response:** `PersonDetail`

```json
{
  "tmdb_person_id": 85,
  "name": "Johnny Depp",
  "biography": "...",
  "birthday": "1963-06-09",
  "deathday": null,
  "place_of_birth": "Owensboro, Kentucky, USA",
  "profile_url": "https://...",
  "known_for_department": "Acting",
  "gender": 2,
  "also_known_as": ["..."],
  "known_for": [
    {
      "slug": "pirates-of-the-caribbean-2003",
      "title": "Pirates of the Caribbean",
      "year": 2003,
      "poster_url": "...",
      "character": "Jack Sparrow"
    }
  ]
}
```

---

### `GET /api/people/resolve`

**File:** `api/people.rs`

**Handler:** `resolve_person()`

**Purpose:** Resolves cast member to person details.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `movie_slug` | `string` | Yes | Movie containing cast member |
| `name` | `string` | Yes | Cast member name |

**Response:** `PersonDetail` (same as above)

---

## Error Handling

### Error Types

**File:** `error.rs`

| Function | Status | Code | Description |
|----------|--------|------|-------------|
| `movie_not_found(slug)` | 404 | `movie_not_found` | Movie doesn't exist |
| `invalid_request(msg)` | 400 | `invalid_request` | Bad input |
| `scan_already_running()` | 409 | `scan_already_running` | Scan in progress |
| `library_scanning()` | 503 | `library_scanning` | Service busy |
| `tmdb_not_configured()` | 503 | `tmdb_not_configured` | TMDB disabled |
| `artwork_not_found(slug, kind)` | 404 | `artwork_not_found` | No artwork available |

### Error Response Format

```json
{
  "code": "movie_not_found",
  "message": "No movie with slug 'alien-1979'"
}
```
