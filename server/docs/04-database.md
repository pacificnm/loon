# Database Documentation

## Overview

Loon Server uses **SQLite** for persistent storage with migrations managed by `nest-data-sqlite`.

### Database Location

Default: `./data/loon.db` (relative to working directory)  
Configurable: `data_dir` in `[loon]` section of config.toml

---

## Schema

### Migrations

**File:** `src/db/migrations.rs`

```rust
pub fn loon_migrations() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(SqlMigration::new("001_initial", include_str!("../../migrations/001_initial.sql"), ...)),
        Box::new(SqlMigration::new("002_tmdb_locked", include_str!("../../migrations/002_tmdb_locked.sql"), ...)),
        Box::new(SqlMigration::new("003_people", include_str!("../../migrations/003_people.sql"), ...)),
    ]
}
```

---

## Migration 001_initial

**File:** `migrations/001_initial.sql`

### Tables

#### `movies`

Core movie metadata table.

```sql
CREATE TABLE movies (
    id                  TEXT PRIMARY KEY,       -- Stable media ID (file:{path})
    slug                TEXT NOT NULL UNIQUE,   -- URL-safe identifier
    title               TEXT NOT NULL,          -- Display title
    original_title      TEXT,                   -- Original release title
    year                INTEGER,                -- Release year
    runtime_seconds     INTEGER,                -- Runtime in seconds
    summary             TEXT,                   -- Plot summary
    tmdb_id             TEXT,                   -- TMDB movie ID
    imdb_id             TEXT,                   -- IMDb ID
    cast_json           TEXT NOT NULL DEFAULT '[]',  -- JSON array of cast
    crew_json           TEXT NOT NULL DEFAULT '[]',  -- JSON array of crew
    poster_url          TEXT,                   -- Poster image URL
    backdrop_url        TEXT,                   -- Backdrop image URL
    created_at          INTEGER NOT NULL,       -- Creation timestamp
    updated_at          INTEGER NOT NULL        -- Last update timestamp
);
```

**Indexes:**
- `idx_movies_slug` - Lookup by slug
- `idx_movies_title` - Title search
- `idx_movies_year` - Year filtering

---

#### `library_files`

Maps physical files to movies.

```sql
CREATE TABLE library_files (
    id              TEXT PRIMARY KEY,           -- file:{relative_path}
    movie_id        TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    library_id      TEXT NOT NULL,              -- Library identifier
    relative_path   TEXT NOT NULL UNIQUE,       -- Path relative to media_root
    size_bytes      INTEGER NOT NULL,           -- File size in bytes
    modified_secs   INTEGER,                    -- Modification timestamp
    scanned_at      INTEGER NOT NULL            -- Last scan timestamp
);
```

**Indexes:**
- `idx_library_files_path` - Path lookup
- `idx_library_files_scanned_at` - Recently scanned ordering

---

#### `movie_genres`

Many-to-many genre tagging.

```sql
CREATE TABLE movie_genres (
    movie_id    TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    genre       TEXT NOT NULL,
    PRIMARY KEY (movie_id, genre)
);
```

**Indexes:**
- `idx_movie_genres_genre` - Genre filtering

---

#### `favorites`

User-marked favorite movies.

```sql
CREATE TABLE favorites (
    movie_id    TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    added_at    INTEGER NOT NULL,               -- Favorite timestamp
    PRIMARY KEY (movie_id)
);
```

**Indexes:**
- `idx_favorites_added_at` - Ordering by date added

---

#### `watch_progress`

Playback progress for resume.

```sql
CREATE TABLE watch_progress (
    movie_id            TEXT PRIMARY KEY REFERENCES movies(id) ON DELETE CASCADE,
    position_seconds    INTEGER NOT NULL,       -- Current position
    duration_seconds    INTEGER,                -- Total duration (if known)
    updated_at          INTEGER NOT NULL        -- Last update timestamp
);
```

**Indexes:**
- `idx_watch_progress_updated_at` - Recently watched ordering

---

## Migration 002_tmdb_locked

**File:** `migrations/002_tmdb_locked.sql`

Adds manual TMDB match protection:

```sql
ALTER TABLE movies ADD COLUMN tmdb_locked INTEGER NOT NULL DEFAULT 0;
```

**Purpose:** When `tmdb_locked = 1`, library scans skip TMDB re-enrichment for this movie.

---

## Migration 003_people

**File:** `migrations/003_people.sql`

Adds person caching table:

```sql
CREATE TABLE people (
    tmdb_person_id       INTEGER PRIMARY KEY,   -- TMDB person ID
    name                 TEXT NOT NULL,         -- Display name
    biography            TEXT,                  -- Biography text
    birthday             TEXT,                  -- ISO date
    deathday             TEXT,                  -- ISO date
    place_of_birth       TEXT,                  -- Birth place
    profile_path         TEXT,                  -- TMDB profile path
    known_for_department TEXT,                  -- Primary department
    gender               INTEGER,               -- TMDB gender code
    also_known_as_json   TEXT NOT NULL DEFAULT '[]',  -- Alternate names JSON
    updated_at           INTEGER NOT NULL       -- Last update timestamp
);

CREATE INDEX idx_people_name ON people(name);
```

---

## Repository

**File:** `src/db/repository.rs`

### `LibraryRepository`

Main database access layer.

```rust
pub struct LibraryRepository {
    conn: SqliteConnection,
}
```

---

### Movie Queries

#### `movie_count`

**Signature:** `pub fn movie_count(&self) -> NestResult<usize>`

**SQL:** `SELECT COUNT(*) FROM movies`

---

#### `get_by_slug`

**Signature:** `pub fn get_by_slug(&self, slug: &str) -> NestResult<Option<LoonMovieRecord>>`

**SQL:**
```sql
SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
       m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
       m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked,
       lf.scanned_at, lf.size_bytes, lf.modified_secs,
       EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
       wp.position_seconds, wp.duration_seconds,
       (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
FROM movies m
JOIN library_files lf ON lf.movie_id = m.id
LEFT JOIN watch_progress wp ON wp.movie_id = m.id
WHERE m.slug = ?1
```

---

#### `get_by_media_id`

**Signature:** `pub fn get_by_media_id(&self, media_id: &str) -> NestResult<Option<LoonMovieRecord>>`

**Purpose:** Lookup by stable media ID (used for incremental scan).

---

#### `load_all`

**Signature:** `pub fn load_all(&self) -> NestResult<Vec<LoonMovieRecord>>`

**Purpose:** Load all movies for catalog rebuild on startup.

**Ordering:** `ORDER BY m.title COLLATE NOCASE ASC`

---

#### `upsert_movie`

**Signature:**
```rust
pub fn upsert_movie(
    &self,
    library_id: &str,
    record: &LoonMovieRecord,
    scanned_at: u64,
    size_bytes: u64,
    modified_secs: Option<u64>,
) -> NestResult<()>
```

**Operations (transactional):**
1. INSERT/UPDATE `movies`
2. INSERT/UPDATE `library_files`
3. DELETE/INSERT `movie_genres`

**SQL (movies):**
```sql
INSERT INTO movies (id, slug, title, original_title, year, runtime_seconds, summary,
                    tmdb_id, imdb_id, cast_json, crew_json, poster_url, backdrop_url,
                    tmdb_locked, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
ON CONFLICT(id) DO UPDATE SET
    slug = excluded.slug,
    title = excluded.title,
    original_title = excluded.original_title,
    year = excluded.year,
    runtime_seconds = excluded.runtime_seconds,
    summary = excluded.summary,
    tmdb_id = excluded.tmdb_id,
    imdb_id = excluded.imdb_id,
    cast_json = excluded.cast_json,
    crew_json = excluded.crew_json,
    poster_url = excluded.poster_url,
    backdrop_url = excluded.backdrop_url,
    tmdb_locked = excluded.tmdb_locked,
    updated_at = excluded.updated_at
```

---

#### `delete_orphans`

**Signature:** `pub fn delete_orphans(&self, library_id: &str, seen_paths: &[String]) -> NestResult<u32>`

**Purpose:** Removes files no longer present in filesystem.

**SQL:**
```sql
DELETE FROM library_files
WHERE library_id = ?1 AND relative_path NOT IN (?)

DELETE FROM movies
WHERE id NOT IN (SELECT movie_id FROM library_files)
```

---

### List Queries

#### `list_movies`

**Signature:** `pub fn list_movies(&self, query: &MovieListQuery) -> NestResult<Vec<LoonMovieRecord>>`

**Parameters:**
```rust
pub struct MovieListQuery {
    pub page: u32,
    pub limit: u32,
    pub sort: MovieSort,
    pub genre: Option<String>,
}
```

**Sort Options:**
- `Title` - `ORDER BY m.title COLLATE NOCASE ASC`
- `Year` - `ORDER BY m.year DESC, m.title COLLATE NOCASE ASC`
- `RecentlyAdded` - `ORDER BY lf.scanned_at DESC, m.title COLLATE NOCASE ASC`

---

#### `count_movies`

**Signature:** `pub fn count_movies(&self, query: &MovieListQuery) -> NestResult<usize>`

**Purpose:** Count for pagination.

---

#### `search_movies`

**Signature:** `pub fn search_movies(&self, query: &str, limit: u32) -> NestResult<Vec<LoonMovieRecord>>`

**SQL:**
```sql
SELECT ... FROM movies m
JOIN library_files lf ON lf.movie_id = m.id
LEFT JOIN watch_progress wp ON wp.movie_id = m.id
WHERE m.title LIKE ?1 COLLATE NOCASE
ORDER BY m.title COLLATE NOCASE ASC
LIMIT ?2
```

---

#### `list_genres`

**Signature:** `pub fn list_genres(&self) -> NestResult<Vec<GenreCount>>`

**SQL:**
```sql
SELECT genre, COUNT(*) AS count
FROM movie_genres
GROUP BY genre
ORDER BY count DESC, genre COLLATE NOCASE ASC
```

---

#### `list_continue_watching`

**Signature:** `pub fn list_continue_watching(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>>`

**SQL:**
```sql
SELECT ... FROM watch_progress wp
JOIN movies m ON m.id = wp.movie_id
JOIN library_files lf ON lf.movie_id = m.id
WHERE wp.duration_seconds IS NULL
   OR wp.position_seconds < CAST(wp.duration_seconds * 9 / 10 AS INTEGER)
ORDER BY wp.updated_at DESC
LIMIT ?1
```

**Logic:** Returns movies where position < 90% of duration (not finished).

---

#### `list_recently_added`

**Signature:** `pub fn list_recently_added(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>>`

**Implementation:** Calls `list_movies` with `MovieSort::RecentlyAdded`.

---

#### `list_favorites`

**Signature:** `pub fn list_favorites(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>>`

**SQL:**
```sql
SELECT ... FROM favorites f
JOIN movies m ON m.id = f.movie_id
JOIN library_files lf ON lf.movie_id = m.id
LEFT JOIN watch_progress wp ON wp.movie_id = m.id
ORDER BY f.added_at DESC
LIMIT ?1
```

---

#### `list_by_genre`

**Signature:** `pub fn list_by_genre(&self, genre: &str, limit: u32) -> NestResult<Vec<LoonMovieRecord>>`

**Implementation:** Calls `list_movies` with genre filter.

---

### Favorite Operations

#### `set_favorite`

**Signature:** `pub fn set_favorite(&self, slug: &str, favorite: bool) -> NestResult<bool>`

**SQL (add):**
```sql
INSERT OR REPLACE INTO favorites (movie_id, added_at)
VALUES (?1, ?2)
```

**SQL (remove):**
```sql
DELETE FROM favorites WHERE movie_id = ?1
```

---

#### `is_favorite`

**Signature:** `pub fn is_favorite(&self, slug: &str) -> NestResult<bool>`

**SQL:**
```sql
SELECT COUNT(*) FROM favorites f
JOIN movies m ON m.id = f.movie_id
WHERE m.slug = ?1
```

---

### Watch Progress Operations

#### `save_progress`

**Signature:** `pub fn save_progress(&self, slug: &str, progress: &WatchProgress) -> NestResult<bool>`

**Logic:**
- If position > 90% of duration: DELETE (mark finished)
- Otherwise: INSERT/UPDATE

**SQL (update):**
```sql
INSERT INTO watch_progress (movie_id, position_seconds, duration_seconds, updated_at)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(movie_id) DO UPDATE SET
    position_seconds = excluded.position_seconds,
    duration_seconds = excluded.duration_seconds,
    updated_at = excluded.updated_at
```

---

### File Operations

#### `get_file_by_path`

**Signature:** `pub fn get_file_by_path(&self, relative_path: &str) -> NestResult<Option<StoredFile>>`

**Purpose:** Get stored file metadata for incremental scan comparison.

**SQL:**
```sql
SELECT movie_id, relative_path, size_bytes, modified_secs, scanned_at
FROM library_files
WHERE relative_path = ?1
```

---

### Person Operations

#### `get_person`

**Signature:** `pub fn get_person(&self, tmdb_person_id: u32) -> NestResult<Option<PersonRecord>>`

**SQL:**
```sql
SELECT tmdb_person_id, name, biography, birthday, deathday, place_of_birth,
       profile_path, known_for_department, gender, also_known_as_json, updated_at
FROM people
WHERE tmdb_person_id = ?1
```

---

#### `upsert_person`

**Signature:** `pub fn upsert_person(&self, record: &PersonRecord) -> NestResult<()>`

**SQL:**
```sql
INSERT INTO people (tmdb_person_id, name, biography, birthday, deathday, place_of_birth,
                    profile_path, known_for_department, gender, also_known_as_json, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT(tmdb_person_id) DO UPDATE SET
    name = excluded.name,
    biography = excluded.biography,
    birthday = excluded.birthday,
    deathday = excluded.deathday,
    place_of_birth = excluded.place_of_birth,
    profile_path = excluded.profile_path,
    known_for_department = excluded.known_for_department,
    gender = excluded.gender,
    also_known_as_json = excluded.also_known_as_json,
    updated_at = excluded.updated_at
```

---

## Helper Types

### `StoredFile`

```rust
pub struct StoredFile {
    pub movie_id: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub modified_secs: Option<u64>,
    pub scanned_at: u64,
}
```

**Purpose:** Represents existing file for incremental scan comparison.

---

### `WatchProgress`

```rust
pub struct WatchProgress {
    pub position_seconds: u32,
    pub duration_seconds: Option<u32>,
}
```

---

### `MovieListQuery`

```rust
pub struct MovieListQuery {
    pub page: u32,
    pub limit: u32,
    pub sort: MovieSort,
    pub genre: Option<String>,
}
```

**Default:**
```rust
pub fn default_list() -> Self {
    Self {
        page: 1,
        limit: 50,
        sort: MovieSort::Title,
        genre: None,
    }
}
```

---

### `MovieSort`

```rust
pub enum MovieSort {
    Title,
    Year,
    RecentlyAdded,
}
```

---

### `GenreCount`

```rust
pub struct GenreCount {
    pub name: String,
    pub count: u32,
}
```

---

## Database Opening

**File:** `src/db/mod.rs`

### `open_database`

**Signature:** `pub fn open_database(path: &Path) -> NestResult<LibraryRepository>`

**Process:**
1. Create parent directory if needed
2. Open SQLite connection
3. Apply pending migrations
4. Return repository

```rust
pub fn open_database(path: &Path) -> NestResult<LibraryRepository> {
    // Create directory
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Open connection
    let conn = SqliteConnection::open(&SqliteConfig::file(path))?;
    
    // Apply migrations
    apply_migrations(&conn, &loon_migrations())?;
    
    Ok(LibraryRepository::new(conn))
}
```
