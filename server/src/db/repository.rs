//! SQLite repository for Loon library data.

use std::time::{SystemTime, UNIX_EPOCH};

use nest_data::DataError;
use nest_data_sqlite::SqliteConnection;
use nest_error::{NestError, NestResult};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::models::{CastMemberDto, CrewMemberDto};
use crate::services::catalog::LoonMovieRecord;
use crate::services::person::PersonRecord;

/// Existing file row used for incremental scan decisions.
#[derive(Debug, Clone)]
pub struct StoredFile {
    /// Movie id.
    pub movie_id: String,
    /// Path relative to media root.
    pub relative_path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modification time.
    pub modified_secs: Option<u64>,
    /// Last scan timestamp.
    pub scanned_at: u64,
}

/// Watch progress payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchProgress {
    /// Playback position in seconds.
    pub position_seconds: u32,
    /// Total duration in seconds when known.
    pub duration_seconds: Option<u32>,
}

/// Sort order for movie lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MovieSort {
    /// Alphabetical by title.
    #[default]
    Title,
    /// Release year descending.
    Year,
    /// Recently scanned files first.
    RecentlyAdded,
}

/// Query parameters for paginated movie lists.
#[derive(Debug, Clone)]
pub struct MovieListQuery {
    /// 1-based page number.
    pub page: u32,
    /// Page size.
    pub limit: u32,
    /// Sort order.
    pub sort: MovieSort,
    /// Optional genre filter.
    pub genre: Option<String>,
}

impl MovieListQuery {
    /// Default list query (page 1, 50 items, title sort).
    pub fn default_list() -> Self {
        Self {
            page: 1,
            limit: 50,
            sort: MovieSort::Title,
            genre: None,
        }
    }

    fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }
}

/// Genre name with movie count.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GenreCount {
    /// Genre label.
    pub name: String,
    /// Number of movies tagged with this genre.
    pub count: u32,
}

/// SQLite-backed library repository.
#[derive(Clone)]
pub struct LibraryRepository {
    conn: SqliteConnection,
}

impl LibraryRepository {
    /// Creates a repository over an open connection.
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }

    fn with_db<T, F>(&self, f: F) -> NestResult<T>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
    {
        self.conn
            .with_connection(|db| f(db).map_err(|error| DataError::query(error.to_string())))
            .map_err(map_sqlite_error)
    }

    /// Returns the number of movies in the library.
    pub fn movie_count(&self) -> NestResult<usize> {
        self.with_db(|db| {
            let count: i64 = db.query_row("SELECT COUNT(*) FROM movies", [], |row| row.get(0))?;
            Ok(count as usize)
        })
    }

    /// Returns file metadata for a relative path when present.
    pub fn get_file_by_path(&self, relative_path: &str) -> NestResult<Option<StoredFile>> {
        self.with_db(|db| {
            db.query_row(
                "SELECT movie_id, relative_path, size_bytes, modified_secs, scanned_at
                 FROM library_files WHERE relative_path = ?1",
                [relative_path],
                |row| {
                    Ok(StoredFile {
                        movie_id: row.get(0)?,
                        relative_path: row.get(1)?,
                        size_bytes: row.get::<_, i64>(2)? as u64,
                        modified_secs: row.get(3)?,
                        scanned_at: row.get::<_, i64>(4)? as u64,
                    })
                },
            )
            .optional()
        })
    }

    /// Loads a movie by stable media id.
    pub fn get_by_media_id(&self, media_id: &str) -> NestResult<Option<LoonMovieRecord>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 WHERE m.id = ?1",
            )?;
            let mut rows = stmt.query([media_id])?;
            if let Some(row) = rows.next()? {
                Ok(Some(row_to_record(row)?))
            } else {
                Ok(None)
            }
        })
    }

    /// Loads a movie by slug.
    pub fn get_by_slug(&self, slug: &str) -> NestResult<Option<LoonMovieRecord>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 WHERE m.slug = ?1",
            )?;
            let mut rows = stmt.query([slug])?;
            if let Some(row) = rows.next()? {
                Ok(Some(row_to_record(row)?))
            } else {
                Ok(None)
            }
        })
    }

    /// Loads all movies for in-memory catalog rebuild.
    pub fn load_all(&self) -> NestResult<Vec<LoonMovieRecord>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 ORDER BY m.title COLLATE NOCASE ASC",
            )?;
            let mut rows = stmt.query([])?;
            let mut records = Vec::new();
            while let Some(row) = rows.next()? {
                records.push(row_to_record(row)?);
            }
            Ok(records)
        })
    }

    /// Inserts or updates a movie and related rows in one transaction.
    pub fn upsert_movie(
        &self,
        library_id: &str,
        record: &LoonMovieRecord,
        scanned_at: u64,
        size_bytes: u64,
        modified_secs: Option<u64>,
    ) -> NestResult<()> {
        let cast_json = serde_json::to_string(&record.cast).map_err(json_error)?;
        let crew_json = serde_json::to_string(&record.crew).map_err(json_error)?;
        let now = now_secs();
        let runtime_seconds = record.runtime_minutes.map(|minutes| minutes as u32 * 60);

        self.with_db(|db| {
            db.execute("BEGIN IMMEDIATE", [])?;

            let result = (|| -> Result<(), rusqlite::Error> {
                db.execute(
                    "INSERT INTO movies (
                        id, slug, title, original_title, year, runtime_seconds, summary,
                        tmdb_id, imdb_id, cast_json, crew_json, poster_url, backdrop_url,
                        tmdb_locked, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
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
                        updated_at = excluded.updated_at",
                    params![
                        record.media_id,
                        record.slug,
                        record.title,
                        record.original_title,
                        record.year,
                        runtime_seconds,
                        record.summary,
                        record.tmdb_id,
                        record.imdb_id,
                        cast_json,
                        crew_json,
                        record.poster_url,
                        record.backdrop_url,
                        i64::from(record.tmdb_locked),
                        now,
                        now,
                    ],
                )?;

                db.execute(
                    "INSERT INTO library_files (
                        id, movie_id, library_id, relative_path, size_bytes, modified_secs, scanned_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(relative_path) DO UPDATE SET
                        movie_id = excluded.movie_id,
                        size_bytes = excluded.size_bytes,
                        modified_secs = excluded.modified_secs,
                        scanned_at = excluded.scanned_at",
                    params![
                        format!("file:{}", record.relative_path),
                        record.media_id,
                        library_id,
                        record.relative_path,
                        size_bytes as i64,
                        modified_secs,
                        scanned_at as i64,
                    ],
                )?;

                db.execute(
                    "DELETE FROM movie_genres WHERE movie_id = ?1",
                    [&record.media_id],
                )?;
                for genre in &record.genres {
                    db.execute(
                        "INSERT INTO movie_genres (movie_id, genre) VALUES (?1, ?2)",
                        params![record.media_id, genre],
                    )?;
                }

                Ok(())
            })();

            match result {
                Ok(()) => {
                    db.execute("COMMIT", [])?;
                    Ok(())
                }
                Err(error) => {
                    let _ = db.execute("ROLLBACK", []);
                    Err(error)
                }
            }
        })
    }

    /// Removes library entries whose paths were not seen in the latest scan.
    pub fn delete_orphans(&self, library_id: &str, seen_paths: &[String]) -> NestResult<u32> {
        self.with_db(|db| {
            db.execute("BEGIN IMMEDIATE", [])?;
            let deleted = if seen_paths.is_empty() {
                let count = db.execute(
                    "DELETE FROM library_files WHERE library_id = ?1",
                    [library_id],
                )?;
                db.execute(
                    "DELETE FROM movies WHERE id NOT IN (SELECT movie_id FROM library_files)",
                    [],
                )?;
                count
            } else {
                let placeholders = seen_paths
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(", ");
                let sql = format!(
                    "DELETE FROM library_files
                     WHERE library_id = ?1 AND relative_path NOT IN ({placeholders})"
                );
                let mut param_values: Vec<Box<dyn rusqlite::ToSql>> =
                    vec![Box::new(library_id.to_string())];
                for path in seen_paths {
                    param_values.push(Box::new(path.clone()));
                }
                let params_ref: Vec<&dyn rusqlite::ToSql> =
                    param_values.iter().map(|value| value.as_ref()).collect();
                let count = db.execute(&sql, params_ref.as_slice())?;
                db.execute(
                    "DELETE FROM movies WHERE id NOT IN (SELECT movie_id FROM library_files)",
                    [],
                )?;
                count
            };
            db.execute("COMMIT", [])?;
            Ok(deleted as u32)
        })
    }

    /// Returns a paginated movie list.
    pub fn list_movies(&self, query: &MovieListQuery) -> NestResult<Vec<LoonMovieRecord>> {
        let order = match query.sort {
            MovieSort::Title => "m.title COLLATE NOCASE ASC",
            MovieSort::Year => "m.year DESC, m.title COLLATE NOCASE ASC",
            MovieSort::RecentlyAdded => "lf.scanned_at DESC, m.title COLLATE NOCASE ASC",
        };

        let sql = if query.genre.is_some() {
            format!(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 JOIN movie_genres mg ON mg.movie_id = m.id AND mg.genre = ?1
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 ORDER BY {order}
                 LIMIT ?2 OFFSET ?3"
            )
        } else {
            format!(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 ORDER BY {order}
                 LIMIT ?1 OFFSET ?2"
            )
        };

        self.with_db(|db| {
            let mut stmt = db.prepare(&sql)?;
            if let Some(genre) = &query.genre {
                collect_records(&mut stmt, params![genre, query.limit, query.offset()])
            } else {
                collect_records(&mut stmt, params![query.limit, query.offset()])
            }
        })
    }

    /// Counts movies matching a list query.
    pub fn count_movies(&self, query: &MovieListQuery) -> NestResult<usize> {
        self.with_db(|db| {
            let count: i64 = if let Some(genre) = &query.genre {
                db.query_row(
                    "SELECT COUNT(*) FROM movies m
                     JOIN movie_genres mg ON mg.movie_id = m.id AND mg.genre = ?1",
                    [genre],
                    |row| row.get(0),
                )?
            } else {
                db.query_row("SELECT COUNT(*) FROM movies", [], |row| row.get(0))?
            };
            Ok(count as usize)
        })
    }

    /// Searches movies by title substring.
    pub fn search_movies(&self, query: &str, limit: u32) -> NestResult<Vec<LoonMovieRecord>> {
        let pattern = format!("%{query}%");
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM movies m
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 WHERE m.title LIKE ?1 COLLATE NOCASE
                 ORDER BY m.title COLLATE NOCASE ASC
                 LIMIT ?2",
            )?;
            collect_records(&mut stmt, params![pattern, limit])
        })
    }

    /// Returns distinct genres with counts.
    pub fn list_genres(&self) -> NestResult<Vec<GenreCount>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT genre, COUNT(*) AS count
                 FROM movie_genres
                 GROUP BY genre
                 ORDER BY count DESC, genre COLLATE NOCASE ASC",
            )?;
            let mut rows = stmt.query([])?;
            let mut genres = Vec::new();
            while let Some(row) = rows.next()? {
                genres.push(GenreCount {
                    name: row.get(0)?,
                    count: row.get::<_, i64>(1)? as u32,
                });
            }
            Ok(genres)
        })
    }

    /// Returns movies with incomplete watch progress.
    pub fn list_continue_watching(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM watch_progress wp
                 JOIN movies m ON m.id = wp.movie_id
                 JOIN library_files lf ON lf.movie_id = m.id
                 WHERE wp.duration_seconds IS NULL
                    OR wp.position_seconds < CAST(wp.duration_seconds * 9 / 10 AS INTEGER)
                 ORDER BY wp.updated_at DESC
                 LIMIT ?1",
            )?;
            collect_records(&mut stmt, [limit])
        })
    }

    /// Returns recently scanned movies.
    pub fn list_recently_added(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>> {
        let query = MovieListQuery {
            page: 1,
            limit,
            sort: MovieSort::RecentlyAdded,
            genre: None,
        };
        self.list_movies(&query)
    }

    /// Returns favorite movies.
    pub fn list_favorites(&self, limit: u32) -> NestResult<Vec<LoonMovieRecord>> {
        self.with_db(|db| {
            let mut stmt = db.prepare(
                "SELECT m.id, lf.relative_path, m.slug, m.title, m.original_title, m.year,
                        m.runtime_seconds, m.summary, m.poster_url, m.backdrop_url,
                        m.cast_json, m.crew_json, m.tmdb_id, m.imdb_id, m.tmdb_locked, lf.scanned_at,
                        lf.size_bytes, lf.modified_secs,
                        EXISTS(SELECT 1 FROM favorites f WHERE f.movie_id = m.id) AS is_favorite,
                        wp.position_seconds, wp.duration_seconds,
                        (SELECT GROUP_CONCAT(mg.genre) FROM movie_genres mg WHERE mg.movie_id = m.id) AS genres
                 FROM favorites f
                 JOIN movies m ON m.id = f.movie_id
                 JOIN library_files lf ON lf.movie_id = m.id
                 LEFT JOIN watch_progress wp ON wp.movie_id = m.id
                 ORDER BY f.added_at DESC
                 LIMIT ?1",
            )?;
            collect_records(&mut stmt, [limit])
        })
    }

    /// Returns movies in a genre.
    pub fn list_by_genre(&self, genre: &str, limit: u32) -> NestResult<Vec<LoonMovieRecord>> {
        let query = MovieListQuery {
            page: 1,
            limit,
            sort: MovieSort::Title,
            genre: Some(genre.to_string()),
        };
        self.list_movies(&query)
    }

    /// Sets favorite state for a movie slug.
    pub fn set_favorite(&self, slug: &str, favorite: bool) -> NestResult<bool> {
        self.with_db(|db| {
            let movie_id: Option<String> = db
                .query_row("SELECT id FROM movies WHERE slug = ?1", [slug], |row| {
                    row.get(0)
                })
                .optional()?;
            let Some(movie_id) = movie_id else {
                return Ok(false);
            };

            if favorite {
                db.execute(
                    "INSERT OR REPLACE INTO favorites (movie_id, added_at) VALUES (?1, ?2)",
                    params![movie_id, now_secs() as i64],
                )?;
            } else {
                db.execute("DELETE FROM favorites WHERE movie_id = ?1", [&movie_id])?;
            }
            Ok(true)
        })
    }

    /// Returns whether a movie is favorited.
    pub fn is_favorite(&self, slug: &str) -> NestResult<bool> {
        self.with_db(|db| {
            let count: i64 = db.query_row(
                "SELECT COUNT(*) FROM favorites f
                 JOIN movies m ON m.id = f.movie_id
                 WHERE m.slug = ?1",
                [slug],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        })
    }

    /// Saves watch progress for a movie slug.
    pub fn save_progress(&self, slug: &str, progress: &WatchProgress) -> NestResult<bool> {
        self.with_db(|db| {
            let movie_id: Option<String> = db
                .query_row("SELECT id FROM movies WHERE slug = ?1", [slug], |row| row.get(0))
                .optional()?;
            let Some(movie_id) = movie_id else {
                return Ok(false);
            };

            let finished = progress.duration_seconds.is_some_and(|duration| {
                duration > 0 && progress.position_seconds as f64 / duration as f64 > 0.9
            });

            if finished {
                db.execute(
                    "DELETE FROM watch_progress WHERE movie_id = ?1",
                    [&movie_id],
                )?;
            } else {
                db.execute(
                    "INSERT INTO watch_progress (movie_id, position_seconds, duration_seconds, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(movie_id) DO UPDATE SET
                        position_seconds = excluded.position_seconds,
                        duration_seconds = excluded.duration_seconds,
                        updated_at = excluded.updated_at",
                    params![
                        movie_id,
                        progress.position_seconds,
                        progress.duration_seconds,
                        now_secs() as i64,
                    ],
                )?;
            }
            Ok(true)
        })
    }

    /// Loads a cached person by TMDB id.
    pub fn get_person(&self, tmdb_person_id: u32) -> NestResult<Option<PersonRecord>> {
        self.with_db(|db| {
            db.query_row(
                "SELECT tmdb_person_id, name, biography, birthday, deathday, place_of_birth,
                        profile_path, known_for_department, gender, also_known_as_json, updated_at
                 FROM people WHERE tmdb_person_id = ?1",
                [tmdb_person_id],
                |row| row_to_person(row),
            )
            .optional()
        })
    }

    /// Inserts or updates a cached person row.
    pub fn upsert_person(&self, record: &PersonRecord) -> NestResult<()> {
        let also_known_as_json =
            serde_json::to_string(&record.also_known_as).map_err(json_error)?;

        self.with_db(|db| {
            db.execute(
                "INSERT INTO people (
                    tmdb_person_id, name, biography, birthday, deathday, place_of_birth,
                    profile_path, known_for_department, gender, also_known_as_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
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
                    updated_at = excluded.updated_at",
                params![
                    record.tmdb_person_id,
                    record.name,
                    record.biography,
                    record.birthday,
                    record.deathday,
                    record.place_of_birth,
                    record.profile_path,
                    record.known_for_department,
                    record.gender,
                    also_known_as_json,
                    record.updated_at as i64,
                ],
            )?;
            Ok(())
        })
    }
}

fn collect_records(
    stmt: &mut rusqlite::Statement<'_>,
    params: impl rusqlite::Params,
) -> Result<Vec<LoonMovieRecord>, rusqlite::Error> {
    let mut rows = stmt.query(params)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next()? {
        records.push(row_to_record(row)?);
    }
    Ok(records)
}

fn row_to_record(row: &rusqlite::Row<'_>) -> Result<LoonMovieRecord, rusqlite::Error> {
    let cast_json: String = row.get(10)?;
    let crew_json: String = row.get(11)?;
    let cast: Vec<CastMemberDto> = serde_json::from_str(&cast_json).unwrap_or_default();
    let crew: Vec<CrewMemberDto> = serde_json::from_str(&crew_json).unwrap_or_default();
    let runtime_seconds: Option<i64> = row.get(6)?;
    let is_favorite: i64 = row.get(18)?;
    let position: Option<i64> = row.get(19)?;
    let duration: Option<i64> = row.get(20)?;
    let genres = parse_genres(row.get(21)?);

    Ok(LoonMovieRecord {
        media_id: row.get(0)?,
        relative_path: row.get(1)?,
        slug: row.get(2)?,
        title: row.get(3)?,
        original_title: row.get(4)?,
        year: row.get(5)?,
        runtime_minutes: runtime_seconds.map(|seconds| (seconds / 60) as u16),
        summary: row.get(7)?,
        genres,
        poster_url: row.get(8)?,
        backdrop_url: row.get(9)?,
        cast,
        crew,
        tmdb_id: row.get(12)?,
        imdb_id: row.get(13)?,
        tmdb_locked: row.get::<_, i64>(14)? > 0,
        scanned_at: row.get::<_, i64>(15)? as u64,
        size_bytes: Some(row.get::<_, i64>(16)? as u64),
        modified_secs: row.get(17)?,
        is_favorite: is_favorite > 0,
        watch_progress_seconds: position.map(|value| value as u32),
        watch_duration_seconds: duration.map(|value| value as u32),
    })
}

fn row_to_person(row: &rusqlite::Row<'_>) -> Result<PersonRecord, rusqlite::Error> {
    let also_known_as_json: String = row.get(9)?;
    let also_known_as: Vec<String> =
        serde_json::from_str(&also_known_as_json).unwrap_or_default();

    Ok(PersonRecord {
        tmdb_person_id: row.get::<_, i64>(0)? as u32,
        name: row.get(1)?,
        biography: row.get(2)?,
        birthday: row.get(3)?,
        deathday: row.get(4)?,
        place_of_birth: row.get(5)?,
        profile_path: row.get(6)?,
        known_for_department: row.get(7)?,
        gender: row.get(8)?,
        also_known_as,
        updated_at: row.get::<_, i64>(10)? as u64,
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_genres(raw: Option<String>) -> Vec<String> {
    raw.map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|genre| !genre.is_empty())
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

fn json_error(error: serde_json::Error) -> NestError {
    NestError::data(format!("json encode failed: {error}"))
}

fn map_sqlite_error(error: DataError) -> NestError {
    NestError::data(error.message()).with_source(error)
}
