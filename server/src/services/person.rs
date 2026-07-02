//! TMDB person fetch, cache, and known-for lookup.

use nest_error::{NestError, NestResult};
use nest_tmdb::{ImageSize, TmdbImageService, DEFAULT_IMAGE_BASE_URL};

use crate::db::LibraryRepository;
use crate::models::{KnownForMovie, PersonDetail};
use crate::services::catalog::LoonMovieRecord;
use crate::services::tmdb::TmdbRuntime;

/// Cached person row stored in SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonRecord {
    /// TMDB person id.
    pub tmdb_person_id: u32,
    /// Display name.
    pub name: String,
    /// Biography text.
    pub biography: Option<String>,
    /// Birthday ISO date.
    pub birthday: Option<String>,
    /// Deathday ISO date.
    pub deathday: Option<String>,
    /// Birth place label.
    pub place_of_birth: Option<String>,
    /// TMDB profile path token.
    pub profile_path: Option<String>,
    /// Primary department.
    pub known_for_department: Option<String>,
    /// TMDB gender code.
    pub gender: Option<i32>,
    /// Alternate names.
    pub also_known_as: Vec<String>,
    /// Last update unix timestamp.
    pub updated_at: u64,
}

/// Loads a person from cache or TMDB, then attaches library known-for movies.
pub async fn get_person_detail(
    tmdb_person_id: u32,
    repo: &LibraryRepository,
    movies: &[LoonMovieRecord],
    tmdb: &TmdbRuntime,
) -> NestResult<PersonDetail> {
    let record = get_or_fetch_person(tmdb_person_id, repo, tmdb).await?;
    let known_for = known_for_movies(movies, tmdb_person_id, &record.name);
    Ok(record_to_detail(&record, known_for))
}

async fn get_or_fetch_person(
    tmdb_person_id: u32,
    repo: &LibraryRepository,
    tmdb: &TmdbRuntime,
) -> NestResult<PersonRecord> {
    if let Some(record) = repo.get_person(tmdb_person_id)? {
        return Ok(record);
    }
    fetch_and_store_person(tmdb_person_id, repo, tmdb).await
}

async fn fetch_and_store_person(
    tmdb_person_id: u32,
    repo: &LibraryRepository,
    tmdb: &TmdbRuntime,
) -> NestResult<PersonRecord> {
    let person = tmdb
        .provider
        .fetch_person(tmdb_person_id)
        .await
        .map_err(|error| {
            NestError::service(format!("TMDB person fetch failed: {}", error.message()))
        })?;

    let record = PersonRecord {
        tmdb_person_id: person.id,
        name: person.name,
        biography: person.biography,
        birthday: person.birthday,
        deathday: person.deathday,
        place_of_birth: person.place_of_birth,
        profile_path: person.profile_path,
        known_for_department: person.known_for_department,
        gender: person.gender,
        also_known_as: person.also_known_as,
        updated_at: now_secs(),
    };

    repo.upsert_person(&record)?;
    Ok(record)
}

fn known_for_movies(
    movies: &[LoonMovieRecord],
    tmdb_person_id: u32,
    name: &str,
) -> Vec<KnownForMovie> {
    let mut known_for: Vec<KnownForMovie> = movies
        .iter()
        .filter_map(|record| {
            let cast_entry = record.cast.iter().find(|member| {
                member.tmdb_person_id == Some(tmdb_person_id)
                    || (member.tmdb_person_id.is_none()
                        && member.name.eq_ignore_ascii_case(name))
            })?;
            let summary = record.to_summary();
            Some(KnownForMovie::from_summary(summary, cast_entry.character.clone()))
        })
        .collect();

    known_for.sort_by(|left, right| {
        right
            .year
            .cmp(&left.year)
            .then_with(|| left.title.cmp(&right.title))
    });
    known_for
}

fn record_to_detail(record: &PersonRecord, known_for: Vec<KnownForMovie>) -> PersonDetail {
    PersonDetail {
        tmdb_person_id: record.tmdb_person_id,
        name: record.name.clone(),
        biography: record.biography.clone(),
        birthday: record.birthday.clone(),
        deathday: record.deathday.clone(),
        place_of_birth: record.place_of_birth.clone(),
        profile_url: profile_url_from_path(record.profile_path.as_deref()),
        known_for_department: record.known_for_department.clone(),
        gender: record.gender,
        also_known_as: record.also_known_as.clone(),
        known_for,
    }
}

fn profile_url_from_path(path: Option<&str>) -> Option<String> {
    let path = path.filter(|value| !value.is_empty())?;
    Some(TmdbImageService::profile_url_with_base(
        DEFAULT_IMAGE_BASE_URL,
        path,
        ImageSize::W185,
    ))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use crate::models::CastMemberDto;
    use crate::services::catalog::LoonMovieRecord;

    use super::*;

    fn movie_with_cast(slug: &str, title: &str, year: u16, cast: Vec<CastMemberDto>) -> LoonMovieRecord {
        LoonMovieRecord {
            media_id: format!("file:{slug}.mp4"),
            slug: slug.into(),
            relative_path: format!("{slug}.mp4"),
            title: title.into(),
            original_title: None,
            year: Some(year),
            runtime_minutes: Some(120),
            summary: None,
            genres: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            cast,
            crew: Vec::new(),
            tmdb_id: None,
            imdb_id: None,
            tmdb_locked: false,
            scanned_at: 1,
            size_bytes: Some(1),
            modified_secs: None,
            is_favorite: false,
            watch_progress_seconds: None,
            watch_duration_seconds: None,
        }
    }

    #[test]
    fn known_for_matches_person_id_and_sorts_by_year() {
        let records = vec![
            movie_with_cast(
                "older",
                "Older Film",
                1990,
                vec![CastMemberDto {
                    name: "Johnny Depp".into(),
                    character: Some("Guy".into()),
                    profile_url: None,
                    tmdb_person_id: Some(85),
                }],
            ),
            movie_with_cast(
                "newer",
                "Newer Film",
                2003,
                vec![CastMemberDto {
                    name: "Johnny Depp".into(),
                    character: Some("Jack".into()),
                    profile_url: None,
                    tmdb_person_id: Some(85),
                }],
            ),
        ];

        let known_for = known_for_movies(&records, 85, "Johnny Depp");
        assert_eq!(known_for.len(), 2);
        assert_eq!(known_for[0].slug, "newer");
        assert_eq!(known_for[1].slug, "older");
    }
}
