//! Artwork proxy integration tests.

use loon_server::services::catalog::catalog_from_records;
use loon_server::state;
use loon_server::{spawn_test_server_with_config, ServerConfig};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn artwork_proxy_serves_cached_image() {
    let image = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/poster.jpg"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"cached-poster")
                .insert_header("content-type", "image/jpeg"),
        )
        .expect(1..=2)
        .mount(&image)
        .await;

    let temp = tempfile::tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut config = ServerConfig::test_with_data_dir(Some(data_dir));
    config.cache = Some(loon_server::config::LoonCacheConfig {
        enabled: true,
        root: cache_dir,
        max_bytes: None,
    });

    let server = spawn_test_server_with_config(config, Some(temp), false)
        .await
        .unwrap();
    let client = reqwest::Client::new();
    let base = server.base_url();

    let movies = client
        .get(format!("{base}/api/movies"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let slug = movies["movies"][0]["slug"].as_str().unwrap();

    {
        let repo = state::repo();
        let mut record = repo.get_by_slug(slug).unwrap().expect("movie record");
        record.poster_url = Some(format!("{}/poster.jpg", image.uri()));
        repo.upsert_movie("main", &record, 1, 1, None).unwrap();
        state::replace_catalog(catalog_from_records(repo.load_all().unwrap()));
    }

    let response = client
        .get(format!("{base}/api/artwork/{slug}/poster"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/jpeg")
    );
    assert_eq!(&response.bytes().await.unwrap()[..], b"cached-poster");

    let cached = client
        .get(format!("{base}/api/artwork/{slug}/poster"))
        .send()
        .await
        .unwrap();
    assert_eq!(cached.status(), 200);
    assert_eq!(&cached.bytes().await.unwrap()[..], b"cached-poster");

    server.shutdown().await;
}

#[tokio::test]
async fn artwork_invalid_kind_returns_400() {
    let server = loon_server::spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();
    let base = server.base_url();

    let response = client
        .get(format!("{base}/api/artwork/alien-1979/trailer"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);

    server.shutdown().await;
}
