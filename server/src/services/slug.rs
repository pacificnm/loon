//! Slug generation for Loon movies.

use std::path::Path;

/// Slugifies text: lowercase, non-alphanumeric collapsed to `-`.
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_was_dash = true;

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

/// Builds a URL slug from title and optional year.
///
/// With year: `{title}-{year}` (e.g. `alien-1979`).
/// Without year: `{title}` only (e.g. `the-chronicles-of-narnia-...`).
pub fn movie_slug(title: &str, year: Option<u16>) -> String {
    let base = slugify(title);
    match year {
        Some(year) => format!("{base}-{year}"),
        None => base,
    }
}

/// Returns a slug unique within `existing`, using the relative path if needed.
pub fn unique_movie_slug(
    title: &str,
    year: Option<u16>,
    relative_path: &str,
    existing: &std::collections::HashMap<String, ()>,
) -> String {
    let base = movie_slug(title, year);
    if !existing.contains_key(&base) {
        return base;
    }

    if let Some(stem) = Path::new(relative_path)
        .file_stem()
        .and_then(|name| name.to_str())
    {
        let path_slug = slugify(stem);
        if !path_slug.is_empty() && !existing.contains_key(&path_slug) {
            return path_slug;
        }
    }

    let mut counter = 2_u16;
    loop {
        let candidate = format!("{base}-{counter}");
        if !existing.contains_key(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn alien_slug() {
        assert_eq!(movie_slug("Alien", Some(1979)), "alien-1979");
    }

    #[test]
    fn slug_without_year() {
        assert_eq!(
            movie_slug("The Chronicles of Narnia", None),
            "the-chronicles-of-narnia"
        );
    }

    #[test]
    fn blade_runner_slug() {
        assert_eq!(movie_slug("Blade Runner", Some(1982)), "blade-runner-1982");
    }

    #[test]
    fn collapses_punctuation() {
        assert_eq!(
            movie_slug("Star Wars: A New Hope", Some(1977)),
            "star-wars-a-new-hope-1977"
        );
    }

    #[test]
    fn unique_slug_uses_path_stem_on_collision() {
        let mut existing = HashMap::new();
        existing.insert("alien-1979".to_string(), ());
        assert_eq!(
            unique_movie_slug(
                "Alien",
                Some(1979),
                "Movies/Alien (1979)/copy.mp4",
                &existing
            ),
            "copy"
        );
    }
}
