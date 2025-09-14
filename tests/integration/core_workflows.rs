//! Core session integration tests

use super::helpers::*;
use std::time::Duration;
use webmock_cli::{serve::MockServer, storage::Storage};

#[tokio::test]
async fn test_end_to_end_capture_list_serve_workflow() {
    // This test simulates the complete session without actual browser automation
    let (_temp_dir, storage) = create_test_storage_with_samples().await;

    // Test 1: List snapshots (should show our test snapshots)
    let snapshots = storage
        .list_snapshots()
        .await
        .expect("Failed to list snapshots");
    assert_eq!(snapshots.len(), 2);

    let snapshot_names: Vec<&str> = snapshots.iter().map(|s| s.name.as_str()).collect();
    assert!(snapshot_names.contains(&"html-test-site"));
    assert!(snapshot_names.contains(&"api-test-endpoints"));

    // Test 2: Load and verify HTML snapshot
    let html_snapshot = storage
        .load_snapshot("html-test-site")
        .await
        .expect("Failed to load HTML snapshot");
    assert_eq!(html_snapshot.name, "html-test-site");
    assert_eq!(html_snapshot.url, "https://example.com/");
    assert_eq!(html_snapshot.requests.len(), 4); // HTML, CSS, JS, PNG

    // Verify different content types are present
    let content_types: Vec<&str> = html_snapshot
        .requests
        .iter()
        .map(|r| r.response.content_type.as_str())
        .collect();
    assert!(content_types.contains(&"text/html"));
    assert!(content_types.contains(&"text/css"));
    assert!(content_types.contains(&"application/javascript"));
    assert!(content_types.contains(&"image/png"));

    // Test 3: Create and test mock server
    let mock_server = MockServer::new(html_snapshot);
    let port = find_available_port();

    // Start server in background
    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test HTTP requests to the mock server
    let client = reqwest::Client::new();

    // Test main page
    let response = client
        .get(format!("http://localhost:{}/", port))
        .send()
        .await
        .expect("Failed to request main page");
    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Hello from WebMock!"));

    // Test CSS file
    let css_response = client
        .get(format!("http://localhost:{}/styles.css", port))
        .send()
        .await
        .expect("Failed to request CSS");
    assert_eq!(css_response.status(), 200);
    let css_body = css_response.text().await.expect("Failed to get CSS body");
    assert!(css_body.contains("font-family: Arial"));

    // Test 404 for non-existent resource
    let not_found_response = client
        .get(format!("http://localhost:{}/nonexistent.html", port))
        .send()
        .await
        .expect("Failed to request non-existent resource");
    assert_eq!(not_found_response.status(), 404);

    // Stop the server
    server_handle.abort();
}

#[tokio::test]
async fn test_api_request_capture_and_replay_accuracy() {
    let (_temp_dir, storage) = create_test_storage_with_samples().await;

    // Load API snapshot
    let api_snapshot = storage
        .load_snapshot("api-test-endpoints")
        .await
        .expect("Failed to load API snapshot");
    assert_eq!(api_snapshot.requests.len(), 2); // GET and POST

    // Verify GET request
    let get_request = api_snapshot
        .requests
        .iter()
        .find(|r| r.method == "GET" && r.url.contains("/users"))
        .expect("GET request not found");
    assert_eq!(get_request.response.status, 200);
    assert_eq!(get_request.response.content_type, "application/json");

    let get_body: serde_json::Value = serde_json::from_slice(&get_request.response.body)
        .expect("Failed to parse GET response JSON");
    assert!(get_body["users"].is_array());
    assert_eq!(get_body["users"].as_array().unwrap().len(), 2);

    // Verify POST request
    let post_request = api_snapshot
        .requests
        .iter()
        .find(|r| r.method == "POST" && r.url.contains("/users"))
        .expect("POST request not found");
    assert_eq!(post_request.response.status, 201);
    assert!(post_request.body.is_some());

    let post_body: serde_json::Value = serde_json::from_slice(&post_request.response.body)
        .expect("Failed to parse POST response JSON");
    assert_eq!(post_body["created"], true);

    // Test mock server with API requests
    let mock_server = MockServer::new(api_snapshot);
    let port = find_available_port();

    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Test GET API endpoint
    let get_response = client
        .get(format!("http://localhost:{}/users", port))
        .send()
        .await
        .expect("Failed to request API endpoint");
    assert_eq!(get_response.status(), 200);

    let api_body: serde_json::Value = get_response
        .json()
        .await
        .expect("Failed to parse API response");
    assert!(api_body["users"].is_array());

    // Test POST API endpoint
    let post_response = client
        .post(format!("http://localhost:{}/users", port))
        .json(&serde_json::json!({"name": "New User"}))
        .send()
        .await
        .expect("Failed to POST to API endpoint");
    assert_eq!(post_response.status(), 201);

    server_handle.abort();
}

#[tokio::test]
async fn test_snapshot_persistence_and_loading() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Create and save a snapshot
    let original_snapshot = create_sample_html_snapshot().await;
    storage
        .save_snapshot(original_snapshot.clone())
        .await
        .expect("Failed to save snapshot");

    // Load the snapshot back
    let loaded_snapshot = storage
        .load_snapshot(&original_snapshot.name)
        .await
        .expect("Failed to load snapshot");

    // Verify all data is preserved
    assert_eq!(loaded_snapshot.name, original_snapshot.name);
    assert_eq!(loaded_snapshot.url, original_snapshot.url);
    assert_eq!(
        loaded_snapshot.requests.len(),
        original_snapshot.requests.len()
    );

    // Verify request details are preserved
    for (original, loaded) in original_snapshot
        .requests
        .iter()
        .zip(loaded_snapshot.requests.iter())
    {
        assert_eq!(original.method, loaded.method);
        assert_eq!(original.url, loaded.url);
        assert_eq!(original.response.status, loaded.response.status);
        assert_eq!(original.response.body, loaded.response.body);
        assert_eq!(original.response.content_type, loaded.response.content_type);
    }
}
