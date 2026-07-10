# Loon

> A lightweight, self-hosted movie streaming system for LG webOS TVs.

## Overview

Loon is a personal movie streaming system with three components:

| Application | Description | Docs |
|-------------|-------------|------|
| **Server** | Rust backend API for movie streaming | [Server Docs](server/docs/) |
| **Client** | LG webOS TV application (React) | [Client Docs](client/docs/) |
| **Desktop** | Tauri admin UI for library management | [Desktop Docs](desktop/docs/) |

```
┌─────────────────────┐         ┌─────────────────────┐
│   Loon Client       │         │   Loon Desktop      │
│   (webOS TV)        │         │   (Tauri Admin)     │
└──────────┬──────────┘         └──────────┬──────────┘
           │                               │
           └───────────────┬───────────────┘
                           ▼
                 ┌─────────────────────┐
                 │    Loon Server      │
                 │    (Rust + SQLite)  │
                 └──────────┬──────────┘
                            ▼
                 ┌─────────────────────┐
                 │   Media Library     │
                 └─────────────────────┘
```

---

## Goals

- Beautiful Netflix-inspired interface for LG webOS
- Extremely fast browsing and instant playback
- Lightweight Rust server with SQLite storage
- Self-hosted, API-first architecture
- Minimal configuration required

---

## Non-Goals

Loon is **not** trying to be Plex, Kodi, or Jellyfin. It excludes:

- Live TV, DVR, IPTV
- Music, Photos, Plugins
- Mobile or browser clients
- Multiple TV platforms
- Docker deployment

**The only supported video client is LG webOS.**

The Desktop app is for **administration only** (library management, TMDB matching, settings) — not for watching movies.

---

## Quick Start

### 1. Install Server

```bash
cd apps/loon/server
./build dev
```

Configure `~/.config/loon/config.toml`:
```toml
[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"

[tmdb]
api_key = "${TMDB_API_KEY}"
```

### 2. Build Client

```bash
cd apps/loon/client
npm install
npm run build
npm run package:webos
```

Deploy `package/` to your LG webOS TV.

### 3. Configure

On the TV:
1. Open Loon app
2. Go to Admin → Settings
3. Enter server URL (e.g., `http://192.168.1.100:3000`)

---

## Documentation

| Category | Link |
|----------|------|
| **Main Docs** | [docs/README.md](docs/) |
| **Server** | [server/docs/](server/docs/) |
| **Client** | [client/docs/](client/docs/) |
| **Desktop** | [desktop/docs/](desktop/docs/) |

### Key Guides

- [Server Configuration](server/docs/06-configuration.md)
- [Server API Reference](server/docs/02-api-reference.md)
- [Client Components](client/docs/02-components.md)
- [Client Platform (webOS)](client/docs/05-platform.md)
- [Desktop Setup](desktop/docs/06-configuration.md)

---

## API Summary

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/movies` | List movies |
| `GET` | `/api/movies/:slug` | Movie details |
| `GET` | `/stream/:slug` | Video stream |
| `POST` | `/api/library/scan` | Start scan (SSE) |
| `PUT` | `/api/movies/:slug/favorite` | Toggle favorite |
| `GET` | `/api/browse` | Netflix-style feed |
| `GET` | `/api/search?q=` | Title search |

See [Server API Reference](server/docs/02-api-reference.md) for full documentation.

---

## Technology Stack

| Layer | Server | Client | Desktop |
|-------|--------|--------|---------|
| **Language** | Rust | TypeScript | Rust + TypeScript |
| **Framework** | nest-http-serve | React 18 | Tauri 2 + React |
| **Database** | SQLite | — | — |
| **Video** | Byte-range streaming | HTML5 | HTML5 + ffplay/mpv |

---

## Development

```bash
# Server
cd apps/loon/server && ./build dev

# Client
cd apps/loon/client && npm run dev

# Desktop
cd apps/loon/desktop && ./build dev

# Run tests
./build test  # in each app directory
```

---

## License

MIT OR Apache-2.0
