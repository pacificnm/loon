//! AI-assisted movie title guessing from filenames.

use std::path::Path;

use nest_ai::{ChatMessage, CompletionRequest, ResponseFormat};
use tracing::warn;

use crate::services::ai::AiRuntime;

/// Parsed movie guess from an AI provider.
#[derive(Debug, Clone, PartialEq)]
pub struct MovieFilenameGuess {
    /// Title to use for TMDB search.
    pub search_title: String,
    /// Likely release year when known.
    pub likely_year: Option<u16>,
    /// Minimum confidence threshold applied by the caller.
    pub confidence: f32,
}

const SYSTEM_PROMPT: &str = "You are helping identify movies from filenames.
Return ONLY valid JSON with this shape:
{
  \"search_title\": string,
  \"likely_year\": number | null,
  \"likely_genres\": string[],
  \"confidence\": number
}
Rules:
- search_title must be the theatrical movie title with normal spacing and capitalization (e.g. \"Anger Management\", not \"Angermanagement\").
- likely_year must be the original theatrical release year. Use null if unsure — do not guess from unrelated adaptations or re-releases.";

/// Guesses a movie title and year from a media filename using AI.
pub async fn guess_movie_from_filename(
    ai: &AiRuntime,
    relative_path: &str,
) -> Option<MovieFilenameGuess> {
    let filename = Path::new(relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(relative_path);

    let mut request = CompletionRequest::user_message(format!("Filename: {filename}"));
    request.messages.insert(0, ChatMessage::system(SYSTEM_PROMPT));
    request.format = Some(ResponseFormat::Json);

    let response = match ai.provider.complete(request).await {
        Ok(response) => response,
        Err(error) => {
            warn!(
                path = %relative_path,
                error = %error.message(),
                "AI filename guess failed"
            );
            return None;
        }
    };

    parse_guess(&response.content, ai.min_confidence)
}

fn parse_guess(raw: &str, min_confidence: f32) -> Option<MovieFilenameGuess> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    let search_title = value
        .get("search_title")
        .and_then(|field| field.as_str())
        .map(str::trim)
        .filter(|title| !title.is_empty())?
        .to_string();

    let likely_year = value
        .get("likely_year")
        .and_then(|field| {
            if field.is_null() {
                None
            } else {
                field.as_u64().map(|year| year as u16)
            }
        })
        .filter(|year| (1888..=2100).contains(year));

    let confidence = value
        .get("confidence")
        .and_then(|field| field.as_f64())
        .map(|value| value as f32)
        .unwrap_or(0.0);

    if confidence < min_confidence {
        return None;
    }

    Some(MovieFilenameGuess {
        search_title,
        likely_year,
        confidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_guess_json() {
        let guess = parse_guess(
            r#"{
              "search_title": "The Chronicles of Narnia: The Lion, the Witch and the Wardrobe",
              "likely_year": 2005,
              "likely_genres": ["Fantasy"],
              "confidence": 1.0
            }"#,
            0.5,
        )
        .unwrap();

        assert_eq!(
            guess.search_title,
            "The Chronicles of Narnia: The Lion, the Witch and the Wardrobe"
        );
        assert_eq!(guess.likely_year, Some(2005));
    }

    #[test]
    fn rejects_low_confidence() {
        assert!(parse_guess(
            r#"{"search_title":"Alien","likely_year":1979,"likely_genres":[],"confidence":0.2}"#,
            0.5
        )
        .is_none());
    }
}
