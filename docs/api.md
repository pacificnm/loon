# Loon HTTP API

## Status: Current (v0.2)

Reference for routes **implemented today** in `loon-server`. Browse, search, favorites, progress, and library scan routes are live. Deferred items (artwork proxy, manual TMDB match) remain in [api-roadmap.md](api-roadmap.md).

**Base URL:** `http://{host}:3000` (default bind from `[loon].bind` in `config.toml`)

**Content types:**

| Route pattern | `Content-Type` |
|---------------|----------------|
| `/`, `/api/*` | `application/json` |
| `/stream/:slug` | `video/*` (from file extension) |

There is **no web UI** at `/` yet — only JSON (and video on stream routes). The webOS client is a separate app.

---

## Quick reference

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/` | Service index — links to API entry points |
| `GET` | `/api/health` | Liveness, version, catalog stats |
| `GET` | `/api/browse` | Netflix-style home feed (hero + rows) |
| `GET` | `/api/movies` | List movies (full or paginated) |
| `GET` | `/api/movies/:slug` | Single movie detail + stream URL |
| `PUT` | `/api/movies/:slug/favorite` | Set or toggle favorite |
| `PUT` | `/api/movies/:slug/progress` | Save watch position |
| `GET` | `/api/search?q=` | Search by title |
| `GET` | `/api/genres` | Genre list with counts |
| `POST` | `/api/library/scan` | Trigger background library rescan |
| `GET` | `/api/library/status` | Scan state and catalog stats |
| `GET` | `/stream/:slug` | Play the video file (byte-range supported) |

---

## `GET /`

Browser-friendly index when you open the server IP in a tab.

### Response `200`

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

### Example

```bash
curl http://192.168.88.205:3000/
```

---

## `GET /api/health`

Health check and catalog summary. Use this to confirm the server is up and how many movies were found on the last startup scan.

### Response `200`

```json
{
  "status": "ok",
  "service": "loon-server",
  "version": "0.1.0",
  "movies_count": 1,
  "library_scanned_at": 1782951875
}
```

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Always `"ok"` when the handler succeeds |
| `service` | string | Service name (`loon-server`) |
| `version` | string | Crate / server version |
| `movies_count` | number | Movies in the in-memory catalog after last scan |
| `library_scanned_at` | number | Unix timestamp (seconds) when the startup scan finished |

### Example

```bash
curl http://192.168.88.205:3000/api/health
```

---

## `GET /api/movies`

Returns movies from the SQLite catalog.

- **Without** `page` or `limit` query params: returns the full in-memory list (same as v0.1).
- **With** `page` and/or `limit`: paginated query against SQLite with optional `sort` and `genre`.

### Query parameters (pagination mode)

| Param | Default | Description |
|-------|---------|-------------|
| `page` | `1` | Page number (1-based) |
| `limit` | `50` | Page size (max 100) |
| `sort` | `title` | `title`, `year`, or `recently_added` |
| `genre` | — | Filter to a genre name |

### Response `200` (paginated)

```json
{
  "movies": [ /* MovieSummary[] */ ],
  "total": 42,
  "page": 1,
  "limit": 20,
  "pages": 3
}
```

### Response `200` (full list)

```json
{
  "movies": [ /* MovieSummary[] */ ],
  "total": 2
}
```

### `MovieSummary`

| Field | Type | Description |
|-------|------|-------------|
| `slug` | string | Stable URL id for API and stream routes |
| `title` | string | Display title |
| `year` | number \| null | Release year when known |
| `runtime_minutes` | number | Runtime in minutes (`0` if unknown) |
| `poster_url` | string \| null | Poster URL (TMDB — not wired yet) |
| `backdrop_url` | string \| null | Backdrop URL (TMDB — not wired yet) |
| `summary` | string | Plot text (`""` if unknown) |

### Example

```bash
curl http://192.168.88.205:3000/api/movies
```

---

## `GET /api/movies/:slug`

Single movie for the detail screen. Same catalog as the list route; 404 if the slug is unknown.

### Path parameters

| Param | Description |
|-------|-------------|
| `slug` | Movie slug from `GET /api/movies` (e.g. `alien-1979`) |

### Response `200`

```json
{
  "slug": "the-chronicles-of-narnia-the-lion-the-witch-and-the-wardrobe",
  "title": "the chronicles of narnia the lion the witch and the wardrobe",
  "original_title": null,
  "year": null,
  "runtime_minutes": null,
  "summary": null,
  "genres": [],
  "poster_url": null,
  "backdrop_url": null,
  "cast": [],
  "crew": [],
  "is_favorite": false,
  "watch_progress_seconds": null,
  "stream_url": "/stream/the-chronicles-of-narnia-the-lion-the-witch-and-the-wardrobe"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `slug` | string | Movie identifier |
| `title` | string | Display title |
| `original_title` | string \| null | Original title (TMDB — not wired yet) |
| `year` | number \| null | Release year when known |
| `runtime_minutes` | number \| null | Runtime when known |
| `summary` | string \| null | Plot when known |
| `genres` | string[] | Genre names (empty until TMDB) |
| `poster_url` | string \| null | Poster image URL |
| `backdrop_url` | string \| null | Backdrop image URL |
| `cast` | array | `{ "name", "character"? }[]` |
| `crew` | array | `{ "name", "job"? }[]` |
| `is_favorite` | boolean | User favorite state (persisted in SQLite) |
| `watch_progress_seconds` | number \| null | Last saved position |
| `stream_url` | string | Relative path for playback — prepend server base URL for webOS |

### Errors

| HTTP | `error.code` | When |
|------|--------------|------|
| `404` | `movie_not_found` | No movie with that slug |

### Example

```bash
curl http://192.168.88.205:3000/api/movies/alien-1979
```

---

## `GET /stream/:slug`

Streams the video file for a catalog movie. Used by webOS, VLC, or curl for playback.

**Security:** The slug is resolved through the catalog only — raw filesystem paths are never accepted from the client.

### Path parameters

| Param | Description |
|-------|-------------|
| `slug` | Movie slug from the catalog |

### Request headers

| Header | Required | Description |
|--------|----------|-------------|
| `Range` | No | Byte range for seeking, e.g. `bytes=0-1048575` |

### Responses

| Case | HTTP | Headers |
|------|------|---------|
| Full file | `200` | `Content-Type`, `Accept-Ranges: bytes`, `Content-Length` |
| Range satisfied | `206` | `Content-Range`, `Content-Length`, `Accept-Ranges: bytes` |
| Invalid range | `416` | `Content-Range: bytes */{total}` |
| Unknown slug | `404` | JSON error envelope |

### Content-Type by extension

| Extension | `Content-Type` |
|-----------|----------------|
| `.mp4`, `.m4v`, `.mov` | `video/mp4` |
| `.mkv` | `video/x-matroska` |
| `.webm` | `video/webm` |
| `.avi` | `video/x-msvideo` |
| other | `application/octet-stream` |

v0.1 is **direct play only** — no transcoding.

### Errors

| HTTP | `error.code` | When |
|------|--------------|------|
| `404` | `movie_not_found` | Unknown slug or missing file |

### Examples

```bash
# Full file
curl -O http://192.168.88.205:3000/stream/alien-1979

# First 1 MiB (seeking test)
curl -H "Range: bytes=0-1048575" http://192.168.88.205:3000/stream/alien-1979
```

---

## Error envelope

All JSON error responses use this shape:

```json
{
  "error": {
    "code": "movie_not_found",
    "message": "No movie with slug 'alien-2099'"
  }
}
```

### Error codes

| HTTP | `code` | When |
|------|--------|------|
| `400` | `invalid_request` | Malformed body or query param |
| `404` | `movie_not_found` | Unknown movie slug |
| `409` | `scan_already_running` | `POST /api/library/scan` while scan active |
| `503` | `library_scanning` | `GET /api/browse` while scan in progress |
| `400` | `NEST_HTTP_SERVE_PARAM_MISSING` | Missing path parameter (framework) |

---

## `GET /api/browse`

Netflix-style home feed — hero movie plus horizontal rows (continue watching, recently added, favorites, genre rows).

Returns `503` with `library_scanning` while a background scan is running.

See [api-v0.2.md](api-v0.2.md#get-apibrowse) for the full response shape.

---

## `GET /api/search`

Search movies by title substring. Requires `q` (minimum 2 characters). Optional `limit` (default 20, max 50).

---

## `GET /api/genres`

Returns `{ "genres": [{ "name", "count" }] }` from persisted movie metadata.

---

## `PUT /api/movies/:slug/favorite`

Set favorite state. Body `{ "favorite": true }` or omit body to toggle. Persists to SQLite.

---

## `PUT /api/movies/:slug/progress`

Save watch position. Body `{ "position_seconds", "duration_seconds"? }`. Clears progress when >90% watched.

---

## `POST /api/library/scan`

Triggers a **background** rescan of the media library. Returns `202 Accepted` immediately.

Body (optional): `{ "full": true }` to force TMDB re-fetch for all files. Default incremental scan skips TMDB for unchanged files.

---

## `GET /api/library/status`

Returns scan state (`idle` / `scanning`), `movies_count`, `last_scan_at`, and progress info.

---

## Catalog behavior (v0.2)

- **Storage:** SQLite at `{data_dir}/loon.db` — metadata, favorites, and watch progress survive restarts
- **Startup:** Opens DB, migrates schema; scans disk only when DB is empty or `--force-scan` is passed; otherwise loads catalog from DB
- **TMDB:** Called at **scan time only** when `TMDB_API_KEY` is set — playback and browse read from SQLite, not TMDB
- **Rescan:** `POST /api/library/scan` updates the DB and in-memory catalog in the background

Configure library location and TMDB in `config.toml`:

```toml
[loon]
media_root = "/media/movies"
data_dir = "/var/lib/loon"

[media-library]
roots = ["."]   # scan directly under media_root

[tmdb]
api_key_env = "TMDB_API_KEY"
language = "en-US"
```

Set the key before starting the server:

```bash
export TMDB_API_KEY="your-key-here"
./build run
```

---

## Planned routes

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/api/artwork/:slug/:kind` | Cached poster/backdrop proxy |
| `PUT` | `/api/movies/:slug/match` | Manual TMDB match override |

Full build order: [api-roadmap.md](api-roadmap.md). Spec details: [api-v0.2.md](api-v0.2.md).

---

## Slugs

Slugs identify movies in URLs. Generated at scan time from title and optional year:

| Title | Year | Slug |
|-------|------|------|
| Alien | 1979 | `alien-1979` |
| Blade Runner | 1982 | `blade-runner-1982` |
| The Chronicles of Narnia… | *(unknown)* | `the-chronicles-of-narnia-the-lion-the-witch-and-the-wardrobe` |

Rules:

- Lowercase; non-alphanumeric characters become `-`
- With year: `{title}-{year}`
- Without year: `{title}` slug only
- Collisions get a unique suffix (filename stem or `-2`, `-3`, …)

---

## Related

- [v1 plan](v1.md) — architecture, stream spec, phases
- [api-v0.2.md](api-v0.2.md) — browse, search, persistence API
- [setup-v1.md](setup-v1.md) — install and systemd
- [config.example.toml](../config.example.toml)
