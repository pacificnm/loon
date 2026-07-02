# Loon API v0.2 Plan

## Status: Implemented (v0.2)

Full HTTP API for **home screen UX + persistence** (SQLite). Builds on [v0.1 routes in v1.md](v1.md#api-reference).

Base URL: `http://{server}:3000` — JSON unless noted.

## Versioning

- v0.1 routes remain unchanged
- v0.2 adds routes below; no `/api/v2` prefix in v0.2 (single household server)
- Breaking changes require explicit version bump in `HealthResponse.version`

## Shared types

### `MovieSummary` (unchanged from v0.1)

```json
{
  "slug": "alien-1979",
  "title": "Alien",
  "year": 1979,
  "runtime_minutes": 117,
  "poster_url": "https://image.tmdb.org/t/p/w500/...",
  "backdrop_url": "https://image.tmdb.org/t/p/w1280/...",
  "summary": "In space no one can hear you scream."
}
```

### Error envelope (all JSON error responses)

```json
{
  "error": {
    "code": "movie_not_found",
    "message": "No movie with slug 'alien-2099'"
  }
}
```

### Error code registry

| HTTP | `code` | When |
|------|--------|------|
| `400` | `invalid_request` | Malformed body, bad query param |
| `400` | `invalid_slug` | Slug format rejected |
| `404` | `movie_not_found` | Unknown movie slug |
| `404` | `genre_not_found` | Unknown genre row |
| `409` | `scan_already_running` | `POST /api/library/scan` while scan active |
| `503` | `library_scanning` | Catalog empty / refresh in progress |
| `500` | `internal_error` | Unexpected server error |

---

## `GET /api/browse`

Netflix-style home feed — hero + horizontal rows.

### Response `200`

```json
{
  "hero": {
    "slug": "blade-runner-1982",
    "title": "Blade Runner",
    "year": 1982,
    "runtime_minutes": 117,
    "poster_url": "https://...",
    "backdrop_url": "https://...",
    "summary": "..."
  },
  "rows": [
    {
      "slug": "continue-watching",
      "title": "Continue Watching",
      "row_type": "continue_watching",
      "movies": [ /* MovieSummary[], max 20 */ ]
    },
    {
      "slug": "recently-added",
      "title": "Recently Added",
      "row_type": "recently_added",
      "movies": [ /* ... */ ]
    },
    {
      "slug": "favorites",
      "title": "Favorites",
      "row_type": "favorites",
      "movies": [ /* ... */ ]
    },
    {
      "slug": "genre-science-fiction",
      "title": "Science Fiction",
      "row_type": "genre",
      "movies": [ /* ... */ ]
    }
  ]
}
```

### Row selection rules

| `row_type` | Query | Max items |
|------------|-------|-----------|
| `continue_watching` | `watch_progress` incomplete, ordered by `updated_at` desc | 20 |
| `recently_added` | `library_files.scanned_at` desc | 20 |
| `favorites` | join `favorites`, `added_at` desc | 20 |
| `genre` | one row per genre with ≥3 movies; title = genre name | 20 |
| `static` | `all-movies` omitted from browse (use list endpoint) | — |

**Hero:** first movie in `recently_added` with non-null `backdrop_url`; else first with `poster_url`; else omit `hero`.

### Errors

| HTTP | Code |
|------|------|
| `503` | `library_scanning` |

---

## `GET /api/movies` (v0.2 pagination)

Extends v0.1 list.

### Query

| Param | Default | Description |
|-------|---------|-------------|
| `page` | `1` | 1-based page |
| `limit` | `50` | max 100 |
| `sort` | `title` | `title` \| `year` \| `recently_added` |
| `genre` | — | filter by genre name |

### Response `200`

```json
{
  "movies": [ /* MovieSummary[] */ ],
  "total": 142,
  "page": 1,
  "limit": 50,
  "pages": 3
}
```

---

## `GET /api/search`

Instant title search (SQLite `LIKE` v0.2; FTS5 optional later).

### Query

| Param | Required | Description |
|-------|----------|-------------|
| `q` | yes | min 2 chars |
| `limit` | no | default 20, max 50 |

### Response `200`

```json
{
  "query": "alien",
  "movies": [ /* MovieSummary[] */ ],
  "total": 2
}
```

### Errors

| HTTP | Code | When |
|------|------|------|
| `400` | `invalid_request` | `q` missing or < 2 chars |

---

## `GET /api/genres`

Distinct genres for filter UI.

### Response `200`

```json
{
  "genres": [
    { "name": "Horror", "count": 12 },
    { "name": "Science Fiction", "count": 8 }
  ]
}
```

Sorted by `count` desc, then name.

---

## `PUT /api/movies/:slug/favorite`

Toggle or set favorite state.

### Request

```json
{
  "favorite": true
}
```

Omit body → toggle current state.

### Response `200`

```json
{
  "slug": "alien-1979",
  "favorite": true
}
```

---

## `PUT /api/movies/:slug/progress`

Save watch position (continue watching).

### Request

```json
{
  "position_seconds": 3600,
  "duration_seconds": 7200
}
```

| Field | Required | Notes |
|-------|----------|-------|
| `position_seconds` | yes | ≥ 0 |
| `duration_seconds` | no | from player; used to detect "finished" |

**Finished rule:** if `position_seconds / duration_seconds > 0.9`, delete progress row (movie leaves continue watching).

### Response `200`

```json
{
  "slug": "alien-1979",
  "position_seconds": 3600,
  "duration_seconds": 7200,
  "updated_at": "2026-07-01T20:15:00Z"
}
```

webOS: send on pause, back, app exit, and every **30s** during playback.

---

## `PUT /api/movies/:slug/match` (v0.2 admin)

Fix wrong TMDB match manually.

### Request

```json
{
  "tmdb_id": 348
}
```

### Behavior

1. Fetch metadata + artwork via `nest-tmdb` for `tmdb:{id}`
2. Rebuild slug if title/year changed (handle collision)
3. Upsert SQLite; invalidate browse cache if any

### Response `200`

Full `MovieDetail`.

---

## `POST /api/library/scan`

Trigger library rescan (background).

### Request

Empty body or:

```json
{
  "full": true
}
```

`full: true` — re-fetch all TMDB metadata; default incremental.

### Response `202`

```json
{
  "status": "started",
  "scan_id": "scan-20260701-201500"
}
```

### Errors

| HTTP | Code |
|------|------|
| `409` | `scan_already_running` |

---

## `GET /api/library/status`

Poll scan progress.

### Response `200`

```json
{
  "state": "idle",
  "last_scan_at": "2026-07-01T12:00:00Z",
  "last_scan_duration_secs": 45,
  "movies_count": 142,
  "scan_in_progress": false,
  "progress": null
}
```

When scanning:

```json
{
  "state": "scanning",
  "scan_in_progress": true,
  "progress": {
    "files_seen": 120,
    "candidates": 98,
    "errors": 1
  }
}
```

---

## `GET /api/artwork/:slug/:kind` (optional v0.2)

Proxy cached poster/backdrop when [nest-cache-file](../../../docs/plan/nest-cache-file-v1.md) enabled.

| Param | Values |
|-------|--------|
| `kind` | `poster` \| `backdrop` |

### Response

- `200` — image bytes, `Content-Type: image/jpeg`
- `302` — redirect to TMDB URL when not cached
- `404` — unknown slug/kind

---

## UX feature map

| README UX | API |
|-----------|-----|
| Hero banner | `GET /api/browse` → `hero` |
| Genre rows | `GET /api/browse` → genre rows |
| Continue Watching | `GET /api/browse` + `PUT /progress` |
| Recently Added | `GET /api/browse` → `recently-added` |
| Instant search | `GET /api/search` |
| Movie details | `GET /api/movies/:slug` |
| Resume playback | `PUT /progress` + `MovieDetail.watch_progress_seconds` |
| Favorites | `PUT /favorite` + favorites row |

## Related

- [v1.md](v1.md) — v0.1 API + stream
- [data-v1.md](data-v1.md) — persistence
- [webos-v1.md](webos-v1.md) — client consumption
