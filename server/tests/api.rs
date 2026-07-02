//! HTTP API integration tests.
//!
//! Tests share process-global app state; `./build check` runs them with
//! `--test-threads=1`.

use loon_server::spawn_test_server;

#[tokio::test]
async fn root_returns_service_index() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["service"], "loon-server");
    assert_eq!(body["endpoints"]["health"], "/api/health");
    assert_eq!(body["endpoints"]["movies"], "/api/movies");

    server.shutdown().await;
}

#[tokio::test]
async fn health_returns_ok() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/health", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "loon-server");
    assert_eq!(body["movies_count"], 2);

    server.shutdown().await;
}

#[tokio::test]
async fn movies_list_returns_scanned_catalog() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/movies", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    let movies = body["movies"].as_array().unwrap();
    assert_eq!(movies.len(), 2);
    assert_eq!(body["total"], 2);

    let slugs: Vec<_> = movies
        .iter()
        .map(|movie| movie["slug"].as_str().unwrap())
        .collect();
    assert!(slugs.contains(&"alien-1979"));
    assert!(slugs.contains(&"blade-runner-1982"));

    server.shutdown().await;
}

#[tokio::test]
async fn movie_detail_by_slug() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/movies/alien-1979", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["slug"], "alien-1979");
    assert_eq!(body["title"], "Alien");
    assert_eq!(body["year"], 1979);
    assert_eq!(body["stream_url"], "/stream/alien-1979");

    server.shutdown().await;
}

#[tokio::test]
async fn movie_not_found_returns_error_envelope() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/movies/alien-2099", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], "movie_not_found");

    server.shutdown().await;
}

#[tokio::test]
async fn stream_returns_byte_range() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/stream/alien-1979", server.base_url()))
        .header("Range", "bytes=0-15")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 206);
    let content_range = response
        .headers()
        .get("content-range")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_range.starts_with("bytes 0-15/"));

    let body = response.bytes().await.unwrap();
    assert_eq!(body.len(), 16);
    assert!(body.starts_with(b"LOON_ALIEN_"));

    server.shutdown().await;
}

#[tokio::test]
async fn stream_unknown_slug_returns_404() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/stream/unknown-movie-1999", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);

    server.shutdown().await;
}

#[tokio::test]
async fn browse_returns_home_feed() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/browse", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert!(!rows.is_empty());

    let row_slugs: Vec<_> = rows
        .iter()
        .map(|row| row["slug"].as_str().unwrap())
        .collect();
    assert!(row_slugs.contains(&"recently-added"));

    let recently_added = rows
        .iter()
        .find(|row| row["slug"] == "recently-added")
        .unwrap();
    assert_eq!(recently_added["movies"].as_array().unwrap().len(), 2);

    server.shutdown().await;
}

#[tokio::test]
async fn search_finds_movies_by_title() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/search?q=alien", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["query"], "alien");
    assert_eq!(body["total"], 1);
    assert_eq!(body["movies"][0]["slug"], "alien-1979");

    server.shutdown().await;
}

#[tokio::test]
async fn genres_returns_list() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/genres", server.base_url()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["genres"].is_array());

    server.shutdown().await;
}

#[tokio::test]
async fn favorite_and_progress_persist() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();
    let base = server.base_url();

    let favorite = client
        .put(format!("{base}/api/movies/alien-1979/favorite"))
        .json(&serde_json::json!({ "favorite": true }))
        .send()
        .await
        .unwrap();
    assert!(favorite.status().is_success());
    let body: serde_json::Value = favorite.json().await.unwrap();
    assert_eq!(body["slug"], "alien-1979");
    assert_eq!(body["favorite"], true);

    let progress = client
        .put(format!("{base}/api/movies/alien-1979/progress"))
        .json(&serde_json::json!({
            "position_seconds": 120,
            "duration_seconds": 6000
        }))
        .send()
        .await
        .unwrap();
    assert!(progress.status().is_success());
    let body: serde_json::Value = progress.json().await.unwrap();
    assert_eq!(body["position_seconds"], 120);
    assert_eq!(body["duration_seconds"], 6000);

    let browse = client
        .get(format!("{base}/api/browse"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = browse.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    let row_slugs: Vec<_> = rows
        .iter()
        .map(|row| row["slug"].as_str().unwrap())
        .collect();
    assert!(row_slugs.contains(&"continue-watching"));
    assert!(row_slugs.contains(&"favorites"));

    server.shutdown().await;
}

#[tokio::test]
async fn paginated_movies_list() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "{}/api/movies?page=1&limit=1&sort=title",
            server.base_url()
        ))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["movies"].as_array().unwrap().len(), 1);
    assert_eq!(body["total"], 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["limit"], 1);
    assert_eq!(body["pages"], 2);

    server.shutdown().await;
}

#[tokio::test]
async fn library_scan_and_status() {
    let server = spawn_test_server().await.unwrap();
    let client = reqwest::Client::new();
    let base = server.base_url();

    let status = client
        .get(format!("{base}/api/library/status"))
        .send()
        .await
        .unwrap();
    assert!(status.status().is_success());
    let body: serde_json::Value = status.json().await.unwrap();
    assert_eq!(body["state"], "idle");
    assert_eq!(body["movies_count"], 2);
    assert_eq!(body["scan_in_progress"], false);

    let scan = client
        .post(format!("{base}/api/library/scan"))
        .json(&serde_json::json!({ "full": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(scan.status(), 200);
    assert!(scan
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("text/event-stream")));
    let body = scan.text().await.unwrap();
    assert!(body.contains("event: started"));
    assert!(body.contains("event: complete"));

    let status = client
        .get(format!("{base}/api/library/status"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = status.json().await.unwrap();
    assert_eq!(body["state"], "idle");
    assert_eq!(body["movies_count"], 2);
    assert_eq!(body["scan_in_progress"], false);

    server.shutdown().await;
}
