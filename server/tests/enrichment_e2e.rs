//! End-to-end enrichment test: API scan → AI filename guess → TMDB → persisted metadata.

use std::path::PathBuf;

use loon_server::{spawn_test_server_with_config, ServerConfig};
use nest_ai_ollama::OllamaConfig;
use nest_tmdb::TmdbConfig;
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const NARNIA_FILENAME: &str = "the_chronicles_of_narnia_the_lion_the_witch_and_the_wardrobe.mp4";
const NARNIA_SEARCH_TITLE: &str = "The Chronicles of Narnia: The Lion, the Witch and the Wardrobe";
const NARNIA_TMDB_ID: u32 = 411;

#[tokio::test]
async fn scan_enriches_narnia_via_ai_then_tmdb() {
    let ollama = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains(NARNIA_FILENAME))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "smollm2:360m",
            "message": {
                "role": "assistant",
                "content": format!(
                    r#"{{
                      "search_title": "{NARNIA_SEARCH_TITLE}",
                      "likely_year": 2005,
                      "likely_genres": ["Fantasy", "Adventure"],
                      "confidence": 1.0
                    }}"#
                )
            },
            "done": true
        })))
        .expect(1)
        .mount(&ollama)
        .await;

    let tmdb = MockServer::start().await;
    mount_tmdb_narnia_mocks(&tmdb).await;

    let media_root = isolated_narnia_media_root();
    let data_dir = tempfile::tempdir().unwrap().keep();
    let config = ServerConfig::enrichment_test(
        media_root,
        data_dir,
        OllamaConfig::new(ollama.uri(), "smollm2:360m"),
        TmdbConfig::builder()
            .api_key("test-key")
            .base_url(tmdb.uri())
            .build()
            .unwrap(),
    );

    let server = spawn_test_server_with_config(config, None, true)
        .await
        .unwrap();
    let client = reqwest::Client::new();
    let base = server.base_url();

    let scan = client
        .post(format!("{base}/api/library/scan"))
        .json(&json!({ "full": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(scan.status(), 200);
    let body = scan.text().await.unwrap();
    assert!(body.contains("event: complete"));

    let movies = client
        .get(format!("{base}/api/movies"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(movies["total"], 1);

    let slug = movies["movies"][0]["slug"].as_str().unwrap();
    let detail = client
        .get(format!("{base}/api/movies/{slug}"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(detail["title"], NARNIA_SEARCH_TITLE);
    assert_eq!(detail["year"], 2005);
    assert_eq!(
        detail["summary"],
        "Four kids travel through a wardrobe to the land of Narnia."
    );
    assert!(detail["poster_url"]
        .as_str()
        .unwrap()
        .starts_with("/api/artwork/"));
    assert!(detail["genres"]
        .as_array()
        .unwrap()
        .iter()
        .any(|genre| genre == "Fantasy"));
    assert_eq!(detail["cast"][0]["name"], "Georgie Henley");
    assert!(detail["cast"][0]["profile_url"]
        .as_str()
        .unwrap()
        .contains("georgie.jpg"));

    server.shutdown().await;
}

fn isolated_narnia_media_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = manifest.join("tests/fixtures/narnia").join(NARNIA_FILENAME);
    let temp = tempfile::tempdir().unwrap();
    std::fs::copy(source, temp.path().join(NARNIA_FILENAME)).unwrap();
    temp.keep()
}

async fn mount_tmdb_narnia_mocks(tmdb: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "images": {
                "base_url": "https://image.tmdb.org/t/p/",
                "poster_sizes": ["w500"],
                "backdrop_sizes": ["w1280"]
            }
        })))
        .mount(tmdb)
        .await;

    Mock::given(method("GET"))
        .and(path("/search/movie"))
        .and(query_param("api_key", "test-key"))
        .and(query_param("query", NARNIA_SEARCH_TITLE))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "id": NARNIA_TMDB_ID,
                "title": NARNIA_SEARCH_TITLE,
                "overview": "Four kids travel through a wardrobe to the land of Narnia.",
                "release_date": "2005-12-09"
            }]
        })))
        .expect(1)
        .mount(tmdb)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/movie/{NARNIA_TMDB_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": NARNIA_TMDB_ID,
            "title": NARNIA_SEARCH_TITLE,
            "original_title": NARNIA_SEARCH_TITLE,
            "overview": "Four kids travel through a wardrobe to the land of Narnia.",
            "release_date": "2005-12-09",
            "runtime": 143,
            "poster_path": "/narnia-poster.jpg",
            "backdrop_path": "/narnia-backdrop.jpg",
            "genres": [
                { "id": 14, "name": "Fantasy" },
                { "id": 12, "name": "Adventure" }
            ]
        })))
        .mount(tmdb)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/movie/{NARNIA_TMDB_ID}/credits")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "cast": [{
                "id": 123,
                "name": "Georgie Henley",
                "character": "Lucy Pevensie",
                "order": 0,
                "profile_path": "/georgie.jpg"
            }],
            "crew": [{
                "name": "Andrew Adamson",
                "job": "Director"
            }]
        })))
        .mount(tmdb)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/movie/{NARNIA_TMDB_ID}/external_ids")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "imdb_id": "tt0363771",
            "tmdb_id": NARNIA_TMDB_ID
        })))
        .mount(tmdb)
        .await;
}
