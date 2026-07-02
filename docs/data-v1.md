# Loon data layer v1 Plan

## Status: Planned

Persistence and repository design for **loon-server** Phase 3 (v0.2). In-memory catalog for v0.1 is defined in [v1.md](v1.md#data-model--persistence).

## Context

Loon needs a **richer schema** than `nest_media::MediaLibraryRepository` provides (file paths, slugs, artwork paths, favorites, progress). The repository lives entirely in the Loon repo; Nest supplies `nest-data-sqlite` primitives only.

## Repository trait

```rust
/// Loon-owned persistence — not a nest-media trait.
#[async_trait]
pub trait LoonLibraryRepository: Send + Sync {
    /// Insert or update full movie record + related rows (transaction).
    async fn upsert_movie(&self, record: &LoonMovieRecord) -> LoonResult<()>;

    async fn get_by_slug(&self, slug: &str) -> LoonResult<Option<LoonMovieRecord>>;
    async fn list_movies(&self, query: MovieListQuery) -> LoonResult<Vec<LoonMovieRecord>>;
    async fn list_by_genre(&self, genre: &str, limit: u32) -> LoonResult<Vec<LoonMovieRecord>>;

    async fn set_favorite(&self, slug: &str, favorite: bool) -> LoonResult<()>;
    async fn save_progress(&self, slug: &str, progress: WatchProgress) -> LoonResult<()>;

    async fn delete_orphans(&self, library_id: &str, seen_paths: &[String]) -> LoonResult<u32>;
}
```

`MovieListQuery`: `{ limit, offset, sort: Title | RecentlyAdded }` — v0.2 pagination.

## Upsert transaction

One `upsert_movie` writes atomically:

```text
BEGIN
  INSERT OR REPLACE INTO movies (...)
  INSERT OR REPLACE INTO library_files (...)
  DELETE FROM movie_genres WHERE movie_id = ?
  INSERT movie_genres for each genre
  INSERT OR REPLACE INTO movie_artwork (poster, backdrop)
COMMIT
```

Favorites and watch_progress are **not** overwritten on scan upsert.

## Migrations

```text
server/migrations/
├── 001_initial.sql      # schema from v1.md
└── 002_browse_rows.sql  # seed default browse_rows (optional)
```

Wire via `nest-data-sqlite`:

```rust
SqliteDataModule::primary(&config.db_path())
    .with_migrations(loon_migrations())
```

Migration runner applies on startup when `LOON_RUN_MIGRATIONS=1` (default true).

## Incremental scan

After `LibraryIndexer` returns `ScanResult`:

```rust
let seen_paths: Vec<String> = result.candidates.iter()
    .map(|c| c.file.relative_path.clone())
    .collect();

for candidate in &result.candidates {
    let existing = repo.get_file_by_path(&candidate.file.relative_path).await?;
    let needs_tmdb = should_refresh_metadata(existing.as_ref(), candidate);
    let record = catalog.build_record(candidate, enrichment, needs_tmdb).await?;
    repo.upsert_movie(&record).await?;
}

repo.delete_orphans(&config.library_id, &seen_paths).await?;
```

### `should_refresh_metadata`

| Condition | Re-fetch TMDB |
|-----------|---------------|
| New file | Yes |
| `modified_secs` or `size_bytes` changed | Yes (metadata may still apply; always re-FFprobe) |
| Unchanged file, metadata present | No |
| Manual `tmdb_id` override set | No (until override cleared) |

## Load on startup (v0.2)

```text
if db exists and not --force-scan:
    catalog = LoonCatalog::from_repository(repo.list_movies(default)).await?
else:
    catalog = scan_and_persist().await?
```

`--force-scan` CLI flag for admin rescan.

## Search (v0.2)

SQLite FTS5 optional:

```sql
CREATE VIRTUAL TABLE movies_fts USING fts5(title, summary, content='movies', content_rowid='rowid');
```

v0.1 search fallback: `LIKE %query%` on title (good enough for <500 movies).

## File layout

```text
server/src/db/
├── mod.rs
├── schema.sql           # reference copy
├── migrations.rs        # embed or load 001_initial.sql
├── repository.rs        # LoonLibraryRepository impl
└── sqlite.rs            # nest-data-sqlite wiring
```

## Testing

| Test | Type |
|------|------|
| upsert + get_by_slug round-trip | Integration (temp db) |
| upsert preserves favorite flag | Integration |
| delete_orphans removes stale files | Integration |
| migration 001 applies cleanly | Integration |

Use `:memory:` SQLite or `tempfile` for tests.

## Related

- [Server v1 plan](v1.md) — full schema SQL
- [nest-data-sqlite](../../../docs/nest-data-sqlite/README.md)
- [nest-data v1 plan](../../../docs/plan/nest-data-v1.md)
