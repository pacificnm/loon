# Loon Server - Architecture Overview

## What is Loon Server?

Loon Server is the **backend API service** for the Loon media library system. It provides HTTP endpoints for browsing, searching, and streaming movies, along with library scanning and TMDB metadata enrichment.

## Application Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Loon Server                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                  HTTP Layer                           │  │
│  │  (nest-http-serve)                                    │  │
│  │  GET /api/movies           → movies::list_movies      │  │
│  │  GET /api/movies/:slug     → movies::get_movie        │  │
│  │  GET /stream/:slug         → stream::stream_movie     │  │
│  │  POST /api/library/scan    → library::start_scan      │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                Service Layer                          │  │
│  │  • catalog       • enrichment    • tmdb               │  │
│  │  • scan_service  • artwork       • ai                 │  │
│  │  • browse        • streaming     • person             │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │               Data Layer                              │  │
│  │  • SQLite repository (rusqlite)                       │  │
│  │  • Migrations (nest-data-sqlite)                      │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌──────────┐        ┌──────────┐        ┌──────────┐
   │  Media   │        │  TMDB    │        │  Ollama  │
   │  Files   │        │   API    │        │   (AI)   │
   └──────────┘        └──────────┘        └──────────┘
```

## Key Design Decisions

### 1. Hybrid Catalog Architecture
- **In-memory catalog** (`LoonCatalog`) for fast reads during request handling
- **SQLite persistence** for durability across restarts
- Catalog rebuilt from SQLite on startup or after library scans

### 2. Incremental Library Scanning
- File changes detected via size/mtime comparison
- Only changed files trigger TMDB re-enrichment
- Manual TMDB matches are "locked" to prevent overwrite

### 3. AI-Assisted Filename Guessing
- Optional Ollama integration for ambiguous filenames
- Confidence threshold prevents low-quality guesses
- Falls back to filename parsing when AI unavailable

### 4. Artwork Proxy/Caching
- Local cache for TMDB images (configurable)
- Falls back to direct TMDB URLs when cache disabled
- Cache invalidation on metadata refresh

## Project Structure

```
server/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library exports, route registration
│   ├── error.rs             # API error helpers
│   ├── state.rs             # Shared application state
│   ├── logging.rs           # nest-logging initialization
│   ├── config/
│   │   ├── mod.rs           # CLI, config loading
│   │   └── cache.rs         # Artwork cache config
│   ├── api/                 # HTTP handlers
│   │   ├── mod.rs
│   │   ├── health.rs        # GET /api/health
│   │   ├── movies.rs        # GET /api/movies, /api/movies/:slug
│   │   ├── stream.rs        # GET /stream/:slug
│   │   ├── library.rs       # POST /api/library/scan, GET /api/library/status
│   │   ├── favorites.rs     # PUT /api/movies/:slug/favorite
│   │   ├── match_handler.rs # PUT /api/movies/:slug/match
│   │   ├── progress.rs      # PUT /api/movies/:slug/progress
│   │   ├── search.rs        # GET /api/search
│   │   ├── browse.rs        # GET /api/browse
│   │   ├── genres.rs        # GET /api/genres
│   │   ├── artwork.rs       # GET /api/artwork/:slug/:kind
│   │   └── people.rs        # GET /api/people/:tmdb_id
│   ├── db/                  # SQLite persistence
│   │   ├── mod.rs           # Database opening, migrations
│   │   ├── repository.rs    # LibraryRepository CRUD
│   │   └── migrations.rs    # Embedded SQL migrations
│   ├── models/              # API DTOs
│   │   ├── mod.rs
│   │   ├── movie.rs         # MovieDetail, MovieSummary
│   │   ├── library.rs       # Scan/progress DTOs
│   │   ├── browse.rs        # BrowseResponse
│   │   ├── person.rs        # PersonDetail
│   │   └── root.rs          # RootResponse
│   └── services/            # Domain logic
│       ├── mod.rs
│       ├── catalog.rs       # In-memory catalog
│       ├── scan_service.rs  # Library scan orchestration
│       ├── enrichment.rs    # TMDB metadata fetching
│       ├── tmdb.rs          # TMDB runtime
│       ├── tmdb_match.rs    # Manual TMDB matching
│       ├── artwork.rs       # Artwork caching
│       ├── ai.rs            # AI runtime (Ollama)
│       ├── filename_guess.rs# AI filename parsing
│       ├── streaming.rs     # HTTP byte-range streaming
│       ├── browse.rs        # Netflix-style feed
│       ├── person.rs        # Person caching/lookup
│       ├── cast_backfill.rs # Person ID backfill
│       ├── slug.rs          # URL slug generation
│       └── media_file.rs    # File metadata helpers
├── migrations/              # SQL migration files
│   ├── 001_initial.sql
│   ├── 002_tmdb_locked.sql
│   └── 003_people.sql
├── tests/                   # Integration tests
│   ├── api.rs
│   ├── artwork.rs
│   └── enrichment_e2e.rs
├── Cargo.toml
└── docs/                    # This documentation
```

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **HTTP** | nest-http-serve | Request routing, JSON serialization |
| **Database** | rusqlite + nest-data-sqlite | SQLite persistence, migrations |
| **Config** | nest-config | TOML configuration loading |
| **TMDB** | nest-tmdb | Metadata enrichment |
| **AI** | nest-ai-ollama | Filename guessing |
| **Cache** | nest-cache-file | Artwork disk cache |
| **Media** | nest-media-library | File discovery, scanning |
| **Logging** | nest-logging | Tracing subscriber |

## API Endpoints

### Core Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `GET` | `/` | `root::root` | API index |
| `GET` | `/api/health` | `health::health` | Health check |
| `GET` | `/api/movies` | `movies::list_movies` | List all movies |
| `GET` | `/api/movies/:slug` | `movies::get_movie` | Movie details |
| `GET` | `/stream/:slug` | `stream::stream_movie` | Video playback |

### Library Management

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `POST` | `/api/library/scan` | `library::start_scan` | Start scan (SSE) |
| `GET` | `/api/library/status` | `library::library_status` | Scan status |

### User Actions

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `PUT` | `/api/movies/:slug/favorite` | `favorites::set_favorite` | Toggle favorite |
| `PUT` | `/api/movies/:slug/match` | `match_handler::set_tmdb_match` | Manual TMDB match |
| `PUT` | `/api/movies/:slug/progress` | `progress::save_progress` | Save watch progress |

### Browse & Discovery

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `GET` | `/api/browse` | `browse::browse` | Netflix-style feed |
| `GET` | `/api/search?q=` | `search::search` | Title search |
| `GET` | `/api/genres` | `genres::list_genres` | Genre list |
| `GET` | `/api/artwork/:slug/:kind` | `artwork::artwork` | Poster/backdrop |
| `GET` | `/api/people/:tmdb_id` | `people::get_person` | Person details |

## Data Flow

### Library Scan Flow

```
1. POST /api/library/scan
2. library::start_scan() acquires scan lock
3. spawn background task:
   a. discover_library() → ScanResult (filesystem walk)
   b. For each candidate:
      - Check if file changed (size/mtime)
      - If changed: enrich_candidate() via TMDB
      - Apply AI filename guess if enabled
   c. upsert_movie() for each record
   d. delete_orphans() for removed files
4. Replace in-memory catalog
5. Stream SSE events to client
```

### Movie Playback Flow

```
1. GET /stream/:slug
2. Lookup slug in catalog → relative_path
3. Resolve media_root + relative_path (security check)
4. Parse Range header for byte-range request
5. Stream file with appropriate Content-Type
6. Return 206 Partial Content for range requests
```

### Request Handling

```rust
// lib.rs registers routes
pub fn api_routes() -> RouteGroup {
    RouteGroup::new("/api")
        .get("/health", api::health::health)
        .get("/movies", api::movies::list_movies)
        .get("/movies/:slug", api::movies::get_movie)
        // ...
}

// Handlers access shared state
pub async fn get_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    let record = state::repo().get_by_slug(&slug)?;
    Json(record.to_detail()).into_response()
}
```

## Build & Run

```bash
cd apps/loon/server

# Run with config
./build dev

# Force full scan on startup
./build dev -- --force-scan

# Production build
./build build

# Run tests
./build test
```

## Configuration

Configuration loaded from `~/.config/loon/config.toml`:

```toml
[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"
data_dir = "./data"

[media-library]
id = "main"
roots = ["Movies"]
video_extensions = ["mp4", "mkv", "avi"]

[tmdb]
api_key = "${TMDB_API_KEY}"

[ai]
enabled = true
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama3.1"
min_confidence = 0.5

[cache]
enabled = true
root = "./cache"
```

See [Configuration Guide](./06-configuration.md) for details.

## Related Documentation

- [Rust API Reference](./02-api-reference.md) - Handler documentation
- [Services](./03-services.md) - Domain service documentation
- [Database](./04-database.md) - Schema and repository
- [Models](./05-models.md) - DTO and type reference
