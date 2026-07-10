# Loon Server Documentation

Complete documentation for the Loon Server backend API - a Rust-based media library server.

---

## Documentation Index

### Getting Started

| Document | Description |
|----------|-------------|
| [Overview](./01-overview.md) | Architecture, project structure, and key concepts |
| [Configuration](./06-configuration.md) | Setup and configuration guide |

### Technical Reference

| Document | Description |
|----------|-------------|
| [API Reference](./02-api-reference.md) | HTTP endpoint handlers and routes |
| [Services](./03-services.md) | Domain services and business logic |
| [Database](./04-database.md) | SQLite schema and repository |
| [Models](./05-models.md) | API DTOs and type definitions |

---

## Quick Links

### For Developers

- **New to the project?** Start with [Overview](./01-overview.md)
- **Setting up locally?** See [Configuration](./06-configuration.md)
- **Adding API endpoints?** Review [API Reference](./02-api-reference.md)
- **Implementing features?** Check [Services](./03-services.md)

### For Maintainers

- [Database](./04-database.md) - Schema migrations and repository
- [Models](./05-models.md) - Type definitions and serialization

---

## Application Overview

**Loon Server** is a Rust HTTP server that provides:

- 📚 **Media Library** - Scan and catalog video files
- 🎬 **Metadata Enrichment** - TMDB integration for movie details
- 🎥 **Video Streaming** - HTTP byte-range video playback
- 🔍 **Search & Browse** - Netflix-style discovery interface
- 🤖 **AI Assistance** - Ollama-based filename guessing
- 💾 **Artwork Caching** - Local poster/backdrop cache

### Technology Stack

| Layer | Technology |
|-------|------------|
| **HTTP** | nest-http-serve |
| **Database** | rusqlite + nest-data-sqlite |
| **Config** | nest-config (TOML) |
| **TMDB** | nest-tmdb |
| **AI** | nest-ai-ollama |
| **Cache** | nest-cache-file |
| **Media** | nest-media-library |

---

## Project Structure

```
server/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library exports, routes
│   ├── error.rs             # Error helpers
│   ├── state.rs             # Shared application state
│   ├── logging.rs           # Logging setup
│   ├── config/              # Configuration
│   ├── api/                 # HTTP handlers
│   ├── db/                  # SQLite persistence
│   ├── models/              # API DTOs
│   └── services/            # Domain logic
├── migrations/              # SQL migrations
├── tests/                   # Integration tests
└── docs/                    # This documentation
```

---

## API Endpoints

### Core

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | API index |
| `GET` | `/api/health` | Health check |
| `GET` | `/api/movies` | List movies |
| `GET` | `/api/movies/:slug` | Movie details |
| `GET` | `/stream/:slug` | Video playback |

### Library

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/library/scan` | Start scan (SSE) |
| `GET` | `/api/library/status` | Scan status |

### User Actions

| Method | Path | Description |
|--------|------|-------------|
| `PUT` | `/api/movies/:slug/favorite` | Toggle favorite |
| `PUT` | `/api/movies/:slug/match` | Manual TMDB match |
| `PUT` | `/api/movies/:slug/progress` | Save progress |

### Discovery

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/browse` | Home feed |
| `GET` | `/api/search?q=` | Title search |
| `GET` | `/api/genres` | Genre list |
| `GET` | `/api/artwork/:slug/:kind` | Poster/backdrop |
| `GET` | `/api/people/:tmdb_id` | Person details |

---

## Build & Run

```bash
cd apps/loon/server

# Development mode
./build dev

# Force full scan on startup
./build dev -- --force-scan

# Custom config path
./build dev -- --config /path/to/config.toml

# Production build
./build build

# Run tests
./build test

# Clean
./build clean
```

---

## Configuration

Required configuration in `~/.config/loon/config.toml`:

```toml
[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"
data_dir = "./data"

[media-library]
id = "main"
roots = ["Movies"]
video_extensions = ["mp4", "mkv"]

[tmdb]
api_key = "${TMDB_API_KEY}"
```

See [Configuration Guide](./06-configuration.md) for all options.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Loon Server                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  HTTP Layer (nest-http-serve)                         │  │
│  │  Routes: /api/*, /stream/*                            │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Service Layer                                        │  │
│  │  catalog | scan | enrichment | tmdb | artwork | ai    │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Data Layer (SQLite)                                  │  │
│  │  movies | library_files | favorites | watch_progress  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌──────────┐        ┌──────────┐        ┌──────────┐
   │  Media   │        │  TMDB    │        │  Ollama  │
   │  Files   │        │   API    │        │   (AI)   │
   └──────────┘        └──────────┘        └──────────┘
```

---

## Key Concepts

### Hybrid Catalog
- **In-memory** (`LoonCatalog`) for fast reads
- **SQLite** for persistence across restarts
- Rebuilt on startup or after scans

### Incremental Scanning
- File changes detected via size/mtime
- Only changed files trigger TMDB re-enrichment
- Manual TMDB matches are "locked"

### AI Filename Guessing
- Optional Ollama integration
- Confidence threshold prevents bad guesses
- Falls back to filename parsing

### Artwork Proxy
- Local cache for TMDB images
- Falls back to direct URLs when disabled
- Cache invalidation on refresh

---

## Related Projects

| App | Path | Role |
|-----|------|------|
| **Server** | `apps/loon/server` | This backend API |
| **Client** | `apps/loon/client` | LG webOS TV app |
| **Desktop** | `apps/loon/desktop` | Admin UI |

---

## Support

- **GitHub**: https://github.com/pacificnm/loon
- **Issues**: File on the main Nest repository

---

## License

MIT OR Apache-2.0 (see workspace root)
