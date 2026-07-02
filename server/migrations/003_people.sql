CREATE TABLE people (
    tmdb_person_id       INTEGER PRIMARY KEY,
    name                 TEXT NOT NULL,
    biography            TEXT,
    birthday             TEXT,
    deathday             TEXT,
    place_of_birth       TEXT,
    profile_path         TEXT,
    known_for_department TEXT,
    gender               INTEGER,
    also_known_as_json   TEXT NOT NULL DEFAULT '[]',
    updated_at           INTEGER NOT NULL
);

CREATE INDEX idx_people_name ON people(name);
