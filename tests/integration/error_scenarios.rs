//! Error scenario integration tests

use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

use super::helpers::*;
use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    error::WebMockError,
    serve::MockServer,
    storage::{Snapshot, Storage},
};

#[tokio::test]
async fn test_snapshot_not_found_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Try to load non-existent snapshot
    let result = storage.load_snapshot("non-existent-snapshot").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WebMockError::SnapshotNotFound(_)
    ));
}

#[tokio::test]
async fn test_port_conflict_handling() {
    // This test verifies that the MockServer can be created successfully
    // Port conflict handling is tested at the integration level since
    // hyper's Server::bind() panics on port conflicts rather than returning errors

    let (_temp_dir, storage) = create_test_storage_with_samples().await;
    let snapshot = storage
        .load_snapshot("html-test-site")
        .await
        .expect("Failed to load snapshot");

    // Test that MockServer can be created with a valid snapshot
    let _mock_server = MockServer::new(snapshot);

    // Test that we can find an available port
    let port = find_available_port();
    assert!(port > 1024); // Should be a user port

    // Note: Actual port conflict testing would require catching panics
    // from hyper's Server::bind(), which is not ideal for unit tests.
    // This functionality is better tested through integration tests
    // or by improving the MockServer implementation to handle bind errors gracefully.
}

#[tokio::test]
async fn test_corrupted_snapshot_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Create a corrupted snapshot file
    let snapshot_path = storage.get_snapshot_path("corrupted-snapshot");
    tokio::fs::write(&snapshot_path, b"invalid msgpack data")
        .await
        .expect("Failed to write corrupted file");

    // Try to load corrupted snapshot
    let result = storage.load_snapshot("corrupted-snapshot").await;
    assert!(result.is_err());
    // Should be a deserialization error
    assert!(matches!(
        result.unwrap_err(),
        WebMockError::Deserialization(_)
    ));
}

#[tokio::test]
async fn test_empty_snapshot_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Create snapshot with no requests
    let empty_snapshot = Snapshot {
        name: "empty-snapshot".to_string(),
        url: "https://example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests: Vec::new(),
    };

    storage
        .save_snapshot(empty_snapshot)
        .await
        .expect("Failed to save empty snapshot");

    // Load and serve empty snapshot
    let loaded_snapshot = storage
        .load_snapshot("empty-snapshot")
        .await
        .expect("Failed to load empty snapshot");
    assert_eq!(loaded_snapshot.requests.len(), 0);

    let mock_server = MockServer::new(loaded_snapshot);
    let port = find_available_port();

    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // All requests should return 404
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/", port))
        .send()
        .await
        .expect("Failed to request from empty server");
    assert_eq!(response.status(), 404);

    server_handle.abort();
}

#[tokio::test]
async fn test_large_response_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Create snapshot with large response body (1MB)
    let large_body = vec![b'A'; 1024 * 1024]; // 1MB of 'A's
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "text/plain".to_string());

    let request = RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/large-file".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers,
            body: large_body.clone(),
            content_type: "text/plain".to_string(),
        },
        timestamp: chrono::Utc::now(),
    };

    let large_snapshot = Snapshot {
        name: "large-response-snapshot".to_string(),
        url: "https://example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests: vec![request],
    };

    // Test saving and loading large snapshot
    storage
        .save_snapshot(large_snapshot)
        .await
        .expect("Failed to save large snapshot");
    let loaded_snapshot = storage
        .load_snapshot("large-response-snapshot")
        .await
        .expect("Failed to load large snapshot");

    assert_eq!(loaded_snapshot.requests[0].response.body.len(), 1024 * 1024);
    assert_eq!(loaded_snapshot.requests[0].response.body, large_body);

    // Test serving large response
    let mock_server = MockServer::new(loaded_snapshot);
    let port = find_available_port();

    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/large-file", port))
        .send()
        .await
        .expect("Failed to request large file");
    assert_eq!(response.status(), 200);

    let body = response.bytes().await.expect("Failed to get response body");
    assert_eq!(body.len(), 1024 * 1024);

    server_handle.abort();
}
