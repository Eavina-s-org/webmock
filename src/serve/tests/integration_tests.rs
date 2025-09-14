use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use crate::serve::MockServer;
use crate::storage::{Snapshot, Storage};
use chrono::Utc;
use std::collections::HashMap;
use std::net::TcpListener;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

async fn create_test_storage_with_snapshot() -> (TempDir, Storage, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Create a test snapshot
    let mut headers = HashMap::new();
    headers.insert(
        "content-type".to_string(),
        "text/html; charset=utf-8".to_string(),
    );

    let response = ResponseRecord {
        status: 200,
        headers: headers.clone(),
        body: b"<html><head><title>Test</title></head><body><h1>Hello from WebMock!</h1></body></html>".to_vec(),
        content_type: "text/html".to_string(),
    };

    let request = RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/".to_string(),
        headers: HashMap::new(),
        body: None,
        response,
        timestamp: Utc::now(),
    };

    let snapshot = Snapshot {
        name: "integration-test-snapshot".to_string(),
        url: "https://example.com/".to_string(),
        created_at: Utc::now(),
        requests: vec![request],
    };

    // Save the snapshot
    storage
        .save_snapshot(snapshot)
        .await
        .expect("Failed to save test snapshot");

    (temp_dir, storage, "integration-test-snapshot".to_string())
}

fn find_available_port() -> u16 {
    // Find an available port for testing
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener
        .local_addr()
        .expect("Failed to get local addr")
        .port();
    drop(listener); // Release the port
    port
}

fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

fn find_available_port_in_range(start_port: u16) -> Option<u16> {
    (start_port..start_port + 100).find(|&port| is_port_available(port))
}

#[tokio::test]
#[ignore = "slow test - need socket binding"]
async fn test_mock_server_startup_and_shutdown() {
    let (_temp_dir, storage, snapshot_name) = create_test_storage_with_snapshot().await;

    // Load the snapshot
    let snapshot = storage
        .load_snapshot(&snapshot_name)
        .await
        .expect("Failed to load snapshot");

    // Create mock server
    let mock_server = MockServer::new(snapshot);
    let port = find_available_port();

    // Start server with timeout to ensure it doesn't run forever
    let server_future = mock_server.start(port);
    let result = timeout(Duration::from_millis(50), server_future).await;

    // The server should timeout (meaning it started successfully and was running)
    assert!(
        result.is_err(),
        "Server should have been running and timed out"
    );
}

#[test]
fn test_port_availability_check() {
    // Test with a port that should be available
    let available_port = find_available_port();
    assert!(is_port_available(available_port));

    // Bind to a port to make it unavailable
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", available_port)).expect("Failed to bind to port");

    // Now the port should not be available
    assert!(!is_port_available(available_port));

    drop(listener); // Release the port

    // Port should be available again
    assert!(is_port_available(available_port));
}

#[test]
fn test_find_available_port_function() {
    // Test finding available port starting from a high number
    let result = find_available_port_in_range(9000);
    assert!(result.is_some());
    let port = result.unwrap();
    assert!(port >= 9000);
    assert!(port < 9100); // Should find one within the range

    // Verify the returned port is actually available
    assert!(is_port_available(port));
}

#[tokio::test]
async fn test_snapshot_loading_and_server_creation() {
    let (_temp_dir, storage, snapshot_name) = create_test_storage_with_snapshot().await;

    // Test loading snapshot
    let snapshot = storage
        .load_snapshot(&snapshot_name)
        .await
        .expect("Failed to load snapshot");

    // Verify snapshot contents
    assert_eq!(snapshot.name, "integration-test-snapshot");
    assert_eq!(snapshot.url, "https://example.com/");
    assert_eq!(snapshot.requests.len(), 1);

    // Test creating mock server
    let mock_server = MockServer::new(snapshot);

    // Verify server was created successfully (no panics)
    // We can't easily test the actual HTTP serving without more complex setup,
    // but we can verify the server object was created properly
    assert_eq!(mock_server.snapshot.name, "integration-test-snapshot");
    assert_eq!(mock_server.snapshot.requests.len(), 1);
}
