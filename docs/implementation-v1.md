# Loon implementation checklist v1

## Status: In progress

Task breakdown for loon-server. **API completion roadmap:** [api-roadmap.md](api-roadmap.md) (finish all routes before webOS).

Plans: [v1.md](v1.md), [data-v1.md](data-v1.md), [webos-v1.md](webos-v1.md) (deferred).

## Phase 0 — Axum spike (optional)

- [x] Skipped — went straight to nest-http-serve

## Phase 1 — Nest HTTP host (M0)

- [x] Create `pacificnm/loon` repo — checkout at `nest/apps/loon/`
- [x] Workspace `Cargo.toml`, `.cargo/config.toml`, `.gitignore`
- [x] `config.example.toml` + local `config.toml` (gitignored)
- [x] `nest-http-serve` bootstrap
- [x] In-memory catalog
- [x] `MovieSummary` + slug helpers + unit tests
- [x] `GET /api/health`, `/api/movies`, `/api/movies/:slug`
- [x] Error envelope JSON
- [x] `GET /` service index

## Phase 2 — Scan + stream (M1)

- [x] `LibraryScanner` + startup scan
- [x] TMDB enrichment (`enrichment.rs`, `MovieFetchResult`)
- [x] `LoonCatalog` + `MovieDetail` DTOs
- [x] `GET /stream/:slug` byte-range
- [x] Range integration tests
- [x] [api.md](api.md) route reference

## Phase 3 — API completion (before UI)

See [api-roadmap.md](api-roadmap.md) for full route map and order.

### 3a — SQLite foundation

- [ ] `server/migrations/001_initial.sql`
- [ ] `LoonLibraryRepository` + `nest-data-sqlite`
- [ ] Upsert after scan; load catalog from DB on startup
- [ ] `--force-scan` CLI flag
- [ ] Orphan cleanup

### 3b — Read routes

- [ ] `GET /api/movies` pagination + sort + genre filter
- [ ] `GET /api/search?q=`
- [ ] `GET /api/genres`

### 3c — User state

- [ ] `PUT /api/movies/:slug/favorite`
- [ ] `PUT /api/movies/:slug/progress`
- [ ] `MovieDetail` reflects favorite + progress from DB

### 3d — Browse

- [ ] `GET /api/browse` (hero + rows)

### 3e — Library ops

- [ ] `POST /api/library/scan` (background)
- [ ] `GET /api/library/status`

### 3f — Admin / optional

- [ ] `PUT /api/movies/:slug/match`
- [ ] `GET /api/artwork/:slug/:kind` (defer until cache)

## Phase W0 — webOS spike

**Blocked** until Phase 3 API checklist complete.

See [webos-v1.md](webos-v1.md).

## Nest prep (can parallelize)

- [x] [nest-tmdb v1.1 artwork fetch](../../../docs/plan/nest-tmdb-v1.1-artwork-fetch.md) — `MovieFetchResult`
- [ ] Extract [nest-stream](../../../docs/plan/nest-stream-v1.md) when stream handler stabilizes
- [ ] [nest-cache-file](../../../docs/plan/nest-cache-file-v1.md) for artwork proxy

## Related

- [API roadmap](api-roadmap.md) — **start here for next work**
- [Server v1 plan](v1.md)
- [API v0.2 spec](api-v0.2.md)
- [Data layer v1](data-v1.md)
- [Setup v1](setup-v1.md)
