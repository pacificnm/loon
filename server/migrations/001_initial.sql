CREATE TABLE movies (
    id                  TEXT PRIMARY KEY,
    slug                TEXT NOT NULL UNIQUE,
    title               TEXT NOT NULL,
    original_title      TEXT,
    year                INTEGER,
    runtime_seconds     INTEGER,
    summary             TEXT,
    tmdb_id             TEXT,
    imdb_id             TEXT,
    cast_json           TEXT NOT NULL DEFAULT '[]',
    crew_json           TEXT NOT NULL DEFAULT '[]',
    poster_url          TEXT,
    backdrop_url        TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

CREATE TABLE library_files (
    id              TEXT PRIMARY KEY,
    movie_id        TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    library_id      TEXT NOT NULL,
    relative_path   TEXT NOT NULL UNIQUE,
    size_bytes      INTEGER NOT NULL,
    modified_secs   INTEGER,
    scanned_at      INTEGER NOT NULL
);

CREATE TABLE movie_genres (
    movie_id    TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    genre       TEXT NOT NULL,
    PRIMARY KEY (movie_id, genre)
);

CREATE TABLE favorites (
    movie_id    TEXT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    added_at    INTEGER NOT NULL,
    PRIMARY KEY (movie_id)
);

CREATE TABLE watch_progress (
    movie_id            TEXT PRIMARY KEY REFERENCES movies(id) ON DELETE CASCADE,
    position_seconds    INTEGER NOT NULL,
    duration_seconds    INTEGER,
    updated_at          INTEGER NOT NULL
);

CREATE INDEX idx_movies_slug ON movies(slug);
CREATE INDEX idx_movies_title ON movies(title);
CREATE INDEX idx_movies_year ON movies(year);
CREATE INDEX idx_movie_genres_genre ON movie_genres(genre);
CREATE INDEX idx_library_files_path ON library_files(relative_path);
CREATE INDEX idx_library_files_scanned_at ON library_files(scanned_at DESC);
CREATE INDEX idx_favorites_added_at ON favorites(added_at DESC);
CREATE INDEX idx_watch_progress_updated_at ON watch_progress(updated_at DESC);
