# Loon Media Server Documentation

Loon is a personal media streaming system with three applications: a backend server, a webOS TV client, and a desktop admin UI.

---

## Applications

| Application | Path | Description | Documentation |
|-------------|------|-------------|---------------|
| **Server** | `apps/loon/server` | Rust backend API for movie streaming | [Server Docs](../server/docs/) |
| **Client** | `apps/loon/client` | LG webOS TV application | [Client Docs](../client/docs/) |
| **Desktop** | `apps/loon/desktop` | Tauri desktop admin UI | [Desktop Docs](../desktop/docs/) |

---

## Architecture Overview

```
┌─────────────────────┐         ┌─────────────────────┐
│   Loon Client       │         │   Loon Desktop      │
│   (webOS TV)        │         │   (Tauri Admin)     │
│   React + Vite      │         │   React + Tauri     │
└──────────┬──────────┘         └──────────┬──────────┘
           │                               │
           │         HTTP API              │
           └───────────────┬───────────────┘
                           │
                           ▼
                 ┌─────────────────────┐
                 │    Loon Server      │
                 │    Rust + Axum      │
                 │    SQLite + TMDB    │
                 └──────────┬──────────┘
                            │
                            │ File Access
                            ▼
                 ┌─────────────────────┐
                 │   Media Library     │
                 │   (Movies/TV)       │
                 └─────────────────────┘
```

---

## Quick Links

### Getting Started

- [Server Setup](../server/docs/06-configuration.md) - Configure and run the backend
- [Client Setup](../client/README.md) - Build and deploy to webOS
- [Desktop Setup](../desktop/docs/06-configuration.md) - Install admin UI

### Technical Reference

- [Server API Endpoints](../server/docs/02-api-reference.md) - HTTP routes and handlers
- [Client Components](../client/docs/02-components.md) - UI component reference
- [Desktop Components](../desktop/docs/04-components.md) - Admin UI reference

### Deep Dives

- [Server Services](../server/docs/03-services.md) - Domain logic and scanning
- [Client Platform](../client/docs/05-platform.md) - webOS integration
- [Desktop Rust Backend](../desktop/docs/02-rust-backend.md) - Tauri commands

---

## Common Tasks

### Server URL Configuration

All apps need the server URL configured:

| App | Configuration Method |
|-----|---------------------|
| **Server** | `~/.config/loon/config.toml` - `[loon].bind` |
| **Client** | Admin → Settings (stored in localStorage) |
| **Desktop** | `~/.config/loon/config.toml` - `[loon-admin].server_url` |

### Library Scanning

1. Open Desktop Admin → Scan tab
2. Click "Scan library" for new files
3. Click "Refresh metadata" to re-fetch TMDB data

Or via API:
```bash
curl -X POST http://localhost:3000/api/library/scan \
  -H "Content-Type: application/json" \
  -d '{"full": false}'
```

### Video Playback

| App | Method |
|-----|--------|
| **Client** | HTML5 video with Magic Remote controls |
| **Desktop** | Built-in player window or external (ffplay/mpv) |

---

## Development

### Build Commands

```bash
# Server
cd apps/loon/server
./build dev

# Client
cd apps/loon/client
npm run dev

# Desktop
cd apps/loon/desktop
./build dev
```

### Testing

```bash
# Server tests
cd apps/loon/server
./build test

# Client tests
cd apps/loon/client
npm test

# Desktop tests
cd apps/loon/desktop
./build test
```

---

## API Summary

### Core Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/movies` | List movies |
| `GET` | `/api/movies/:slug` | Movie details |
| `GET` | `/stream/:slug` | Video stream |
| `POST` | `/api/library/scan` | Start scan (SSE) |
| `GET` | `/api/library/status` | Scan status |

### User Actions

| Method | Path | Description |
|--------|------|-------------|
| `PUT` | `/api/movies/:slug/favorite` | Toggle favorite |
| `PUT` | `/api/movies/:slug/match` | Manual TMDB match |
| `PUT` | `/api/movies/:slug/progress` | Save watch progress |

### Discovery

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/browse` | Netflix-style feed |
| `GET` | `/api/search?q=` | Title search |
| `GET` | `/api/genres` | Genre list |
| `GET` | `/api/people/:tmdb_id` | Person details |

---

## Configuration Files

### Server Config

**Location:** `~/.config/loon/config.toml`

```toml
[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"
data_dir = "./data"

[media-library]
id = "main"
roots = ["Movies"]

[tmdb]
api_key = "${TMDB_API_KEY}"
```

### Desktop Config

**Location:** `~/.config/loon/config.toml`

```toml
[loon-admin]
server_url = "http://192.168.88.10:3000"
```

### Client Config

**Method:** Admin → Settings in the TV app

Or via environment:
```bash
VITE_LOON_SERVER=http://192.168.88.10:3000
```

---

## Technology Stack

| Layer | Server | Client | Desktop |
|-------|--------|--------|---------|
| **Language** | Rust | TypeScript | Rust + TypeScript |
| **Framework** | nest-http-serve | React 18 | Tauri 2 + React |
| **Database** | SQLite | — | — |
| **Navigation** | — | Spatial Navigation | — |
| **Video** | Byte-range streaming | HTML5 video | HTML5 + ffplay/mpv |

---

## Related Documentation

- [Server Documentation](../server/docs/) - Complete server reference
- [Client Documentation](../client/docs/) - webOS app reference
- [Desktop Documentation](../desktop/docs/) - Admin UI reference

---

## Support

- **GitHub**: https://github.com/pacificnm/loon
- **Issues**: File on the main Nest repository

---

## License

MIT OR Apache-2.0
