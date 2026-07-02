# Loon API roadmap

## Status: Active

Server-side plan to finish the HTTP API **before** any webOS UI work. Spec details live in [api.md](api.md) (shipped) and [api-v0.2.md](api-v0.2.md) (planned).

**Principle:** Every route gets a handler, integration test, and curl example before the client exists.

---

## Current state (v0.1 — done)

| Method | Path | Status |
|--------|------|--------|
| `GET` | `/` | ✅ Service index |
| `GET` | `/api/health` | ✅ Liveness + catalog stats |
| `GET` | `/api/movies` | ✅ Full list (no pagination) |
| `GET` | `/api/movies/:slug` | ✅ Detail + `stream_url` |
| `GET` | `/stream/:slug` | ✅ Byte-range video |

**Also done:** TMDB enrichment at startup, `config.toml` loading, error envelope on 404.

**Not done:** SQLite, background rescan, favorites, progress, browse/search/genres, pagination, admin match, artwork proxy.

---

## Full route map (target)

| # | Method | Path | Purpose | Needs |
|---|--------|------|---------|-------|
| 1 | `GET` | `/api/movies` | Paginated list + sort + genre filter | SQLite or in-memory extension |
| 2 | `GET` | `/api/search` | Title search | Catalog query |
| 3 | `GET` | `/api/genres` | Genre list + counts | Catalog query |
| 4 | `GET` | `/api/browse` | Hero + home rows | SQLite (progress, favorites, `scanned_at`) |
| 5 | `PUT` | `/api/movies/:slug/favorite` | Set/toggle favorite | SQLite |
| 6 | `PUT` | `/api/movies/:slug/progress` | Save watch position | SQLite |
| 7 | `PUT` | `/api/movies/:slug/match` | Fix wrong TMDB match | SQLite + TMDB |
| 8 | `POST` | `/api/library/scan` | Trigger background rescan | Scan task + status |
| 9 | `GET` | `/api/library/status` | Scan progress / last run | Scan task + status |
| 10 | `GET` | `/api/artwork/:slug/:kind` | Poster/backdrop proxy | Optional — defer to cache phase |

v0.1 routes (`/`, `/api/health`, `/api/movies/:slug`, `/stream/:slug`) stay unchanged.

---

## Implementation phases

Build in this order. Each phase ends with `./build check` and new tests in `server/tests/api.rs`.

### Phase 3a — SQLite foundation

**Goal:** Persist catalog across restarts; unlock user state tables.

**Work:**

| Task | Files |
|------|-------|
| `001_initial.sql` migration | `server/migrations/` |
| `LoonLibraryRepository` trait + SQLite impl | `server/src/db/` |
| Wire `nest-data-sqlite`, `data_dir` from config | `config.rs`, `lib.rs` |
| After scan: upsert movies + `library_files` | `services/library.rs` |
| On startup: load catalog from DB (skip full scan when DB warm) | `init_app` |
| `--force-scan` CLI flag | `config.rs`, `main.rs` |
| Orphan cleanup after scan | `db/repository.rs` |

**Schema:** [v1.md § SQLite schema](v1.md#sqlite-schema-phase-3--v02), [data-v1.md](data-v1.md).

**Exit criteria:**

```bash
./build run
# restart server — movies still listed without rescanning disk
curl http://localhost:3000/api/health   # movies_count unchanged
```

---

### Phase 3b — Read routes (no writes yet)

**Goal:** Query endpoints the UI will need; all read-only.

| Route | Handler | Logic |
|-------|---------|-------|
| `GET /api/movies?page&limit&sort&genre` | `api/movies.rs` | Repository paginated query |
| `GET /api/search?q&limit` | `api/search.rs` | `LIKE` on title (FTS5 later) |
| `GET /api/genres` | `api/genres.rs` | Aggregate `movie_genres` |

**Response changes:** Extend `MovieListResponse` with `page`, `limit`, `pages` (backward compatible when query omitted → v0.1 full list or default page 1).

**Exit criteria:**

```bash
curl 'http://localhost:3000/api/movies?page=1&limit=10&sort=title'
curl 'http://localhost:3000/api/search?q=alien'
curl http://localhost:3000/api/genres
```

---

### Phase 3c — User state writes

**Goal:** Favorites and continue-watching data survive restart.

| Route | Handler | Logic |
|-------|---------|-------|
| `PUT /api/movies/:slug/favorite` | `api/favorites.rs` | Body `{ "favorite": true }` or toggle |
| `PUT /api/movies/:slug/progress` | `api/progress.rs` | Upsert; delete if >90% watched |

**Also:** Populate `MovieDetail.is_favorite` and `watch_progress_seconds` from DB on `GET /api/movies/:slug`.

**Exit criteria:**

```bash
curl -X PUT http://localhost:3000/api/movies/alien-1979/favorite \
  -H 'Content-Type: application/json' -d '{"favorite":true}'
curl -X PUT http://localhost:3000/api/movies/alien-1979/progress \
  -H 'Content-Type: application/json' \
  -d '{"position_seconds":3600,"duration_seconds":7200}'
# restart — favorite + progress still present
```

---

### Phase 3d — Browse feed

**Goal:** Single home-screen payload.

| Route | Handler | Logic |
|-------|---------|-------|
| `GET /api/browse` | `api/browse.rs` | Hero + rows per [api-v0.2.md](api-v0.2.md) rules |

**Rows:**

| `row_type` | Source |
|------------|--------|
| `continue_watching` | `watch_progress` incomplete, by `updated_at` |
| `recently_added` | `library_files.scanned_at` desc |
| `favorites` | join `favorites` |
| `genre` | genres with ≥3 movies |

**Exit criteria:**

```bash
curl http://localhost:3000/api/browse | jq '.rows[].slug'
```

---

### Phase 3e — Library operations

**Goal:** Rescan without restart; observable progress.

| Route | Handler | Logic |
|-------|---------|-------|
| `POST /api/library/scan` | `api/library.rs` | Spawn background scan task; `202` + `scan_id` |
| `GET /api/library/status` | `api/library.rs` | `idle` / `scanning` + progress stats |

**State:** `AppState` holds scan mutex + last result; reload catalog + DB on completion.

**Errors:** `409 scan_already_running`, `503 library_scanning` on browse during active scan (optional).

**CLI:** `./build run -- --force-scan` for blocking full rescan at startup.

**Exit criteria:**

```bash
curl -X POST http://localhost:3000/api/library/scan
curl http://localhost:3000/api/library/status
```

---

### Phase 3f — Admin + optional (defer if needed)

| Route | When |
|-------|------|
| `PUT /api/movies/:slug/match` | Wrong TMDB match — fetch by `tmdb_id`, re-slug, upsert |
| `GET /api/artwork/:slug/:kind` | After [nest-cache-file](../../../docs/plan/nest-cache-file-v1.md) — redirect to TMDB URL until then |

---

## Suggested file layout (server)

```text
server/src/
├── api/
│   ├── mod.rs
│   ├── root.rs          ✅
│   ├── health.rs        ✅
│   ├── movies.rs        ✅ → extend pagination
│   ├── search.rs        Phase 3b
│   ├── genres.rs        Phase 3b
│   ├── browse.rs        Phase 3d
│   ├── favorites.rs     Phase 3c
│   ├── progress.rs      Phase 3c
│   ├── library.rs       Phase 3e
│   └── match.rs         Phase 3f (admin)
├── db/
│   ├── mod.rs           Phase 3a
│   ├── repository.rs    Phase 3a
│   └── migrations.rs    Phase 3a
├── models/
│   ├── movie.rs         ✅ → browse/search DTOs
│   ├── browse.rs        Phase 3d
│   └── library.rs       Phase 3e
└── services/
    ├── scan_task.rs     Phase 3e — background rescan
    └── ...              ✅ existing
```

---

## Error codes (full registry)

Implement handlers using the envelope from [api-v0.2.md](api-v0.2.md):

| HTTP | `code` | Routes |
|------|--------|--------|
| `400` | `invalid_request` | search, progress, pagination |
| `400` | `invalid_slug` | all `:slug` routes |
| `404` | `movie_not_found` | movies, stream, favorite, progress |
| `404` | `genre_not_found` | genre filter (if strict) |
| `409` | `scan_already_running` | `POST /api/library/scan` |
| `503` | `library_scanning` | `GET /api/browse` during scan |
| `500` | `internal_error` | unexpected |

---

## What we are **not** building yet

| Item | Why wait |
|------|----------|
| webOS app | API must be complete + tested first |
| Poster disk cache / artwork proxy | TMDB URLs work; cache is optimization |
| FFprobe runtime | TMDB covers most cases; add when scan pipeline matures |
| FTS5 search | `LIKE` sufficient for household libraries |
| Auth / API keys | Single household LAN server |

---

## Verification checklist (API complete)

Before starting UI:

- [ ] All 10 v0.2 routes implemented (artwork proxy optional)
- [ ] `./build check` green
- [ ] Integration test per route in `server/tests/api.rs`
- [ ] [api.md](api.md) updated — v0.2 section marked implemented
- [ ] Restart survival: catalog + favorites + progress persist
- [ ] Background rescan updates catalog without process restart
- [ ] curl script or doc section exercises every route

---

## Related

- [api.md](api.md) — current route reference
- [api-v0.2.md](api-v0.2.md) — request/response specs
- [data-v1.md](data-v1.md) — repository + migrations
- [implementation-v1.md](implementation-v1.md) — task checklist
- [webos-v1.md](webos-v1.md) — **blocked until this roadmap is done**
