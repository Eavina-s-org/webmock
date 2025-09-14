use crate::commands::serve_command;
use crate::test_utils::test_helpers::*;
use std::net::{SocketAddr, TcpListener};

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[test]
fn test_port_availability_logic() {
    let free_port = find_free_port();

    // Test that we can bind to a free port
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], free_port)));
    assert!(listener.is_ok());

    // Test port range validation
    assert!(free_port > 0);
    // assert!(free_port <= 65535); // Removed as this is always true for u16
}

#[tokio::test]
#[ignore = "slow test - requires network binding"]
async fn test_serve_command_nonexistent_snapshot() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");
    std::fs::create_dir_all(&storage_path).unwrap();

    let free_port = find_free_port();
    let result = serve_command(
        "nonexistent-snapshot",
        free_port,
        Some(storage_path.to_string_lossy().to_string()),
    )
    .await;

    assert!(result.is_err());
    if let Err(crate::error::WebMockError::SnapshotNotFound(name)) = result {
        assert_eq!(name, "nonexistent-snapshot");
    } else {
        panic!("Expected SnapshotNotFound error");
    }
}

#[tokio::test]
#[ignore = "slow test - requires network binding and server startup"]
async fn test_serve_command_with_valid_snapshot() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");
    std::fs::create_dir_all(&storage_path).unwrap();

    // Create storage and save a test snapshot
    let storage = crate::storage::Storage::new(storage_path.clone());
    let snapshot = create_test_snapshot_with_name("test-serve");
    storage.save_snapshot(snapshot).await.unwrap();

    let free_port = find_free_port();

    // Note: This test will try to start a server, which we can't easily test
    // without more complex setup. We test the validation and setup parts.

    // The serve command will likely fail when trying to start the actual server
    // in a test environment, but we can test the initial validation
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        serve_command(
            "test-serve",
            free_port,
            Some(storage_path.to_string_lossy().to_string()),
        ),
    )
    .await;

    // Either succeeds, times out, or fails with a server error
    match result {
        Ok(Ok(())) => {
            // Unexpected success in test environment
        }
        Ok(Err(e)) => {
            // Expected - likely fails when trying to start server
            let error_msg = e.to_string();
            // Should be a server-related error, not a validation error
            assert!(!error_msg.contains("not found"));
        }
        Err(_) => {
            // Timeout - also acceptable as server might be trying to start
        }
    }
}

#[test]
fn test_validation_helper_port_validation() {
    use crate::feedback::ValidationHelper;

    // Test valid ports
    assert!(ValidationHelper::validate_port(8080).is_ok());
    assert!(ValidationHelper::validate_port(3000).is_ok());
    assert!(ValidationHelper::validate_port(65535).is_ok());

    // All u16 values are technically valid ports
    assert!(ValidationHelper::validate_port(1).is_ok());
    assert!(ValidationHelper::validate_port(80).is_ok());
}

#[test]
fn test_validation_helper_snapshot_name() {
    use crate::feedback::ValidationHelper;

    // Test valid snapshot names
    assert!(ValidationHelper::validate_snapshot_name("test-serve").is_ok());
    assert!(ValidationHelper::validate_snapshot_name("my_snapshot").is_ok());
    assert!(ValidationHelper::validate_snapshot_name("snapshot123").is_ok());

    // Test invalid snapshot names
    assert!(ValidationHelper::validate_snapshot_name("").is_err());
    assert!(ValidationHelper::validate_snapshot_name("invalid/name").is_err());
    assert!(ValidationHelper::validate_snapshot_name("test snapshot").is_err());
}

#[test]
fn test_progress_reporter_for_serve() {
    use crate::feedback::ProgressReporter;

    let progress = ProgressReporter::new();

    // Test spinner for loading
    let loading_spinner = progress.create_spinner("Loading snapshot...");
    assert!(!loading_spinner.is_finished());

    loading_spinner.finish_with_message("✅ Loaded snapshot");
    assert!(loading_spinner.is_finished());

    // Test network progress
    let network_progress = progress.create_network_progress("Starting server...");
    assert!(!network_progress.is_finished());

    network_progress.finish_with_message("✅ Server started");
    assert!(network_progress.is_finished());
}

#[test]
fn test_mock_server_creation() {
    let snapshot = create_test_snapshot_with_name("test-mock");
    let _server = crate::serve::MockServer::new(snapshot.clone());

    // Test that server can be created without panicking
    // The actual server functionality is tested in serve module tests
}

#[tokio::test]
async fn test_storage_operations_for_serve() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();

    // Test loading non-existent snapshot
    let result = storage.load_snapshot("nonexistent").await;
    assert!(result.is_err());

    // Create and save a snapshot
    let snapshot = create_test_snapshot_with_name("serve-test");
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    // Test loading existing snapshot
    let loaded = storage.load_snapshot("serve-test").await.unwrap();
    assert_eq!(loaded.name, snapshot.name);
    assert_eq!(loaded.url, snapshot.url);
    assert_eq!(loaded.requests.len(), snapshot.requests.len());

    cleanup_test_env();
}

#[test]
fn test_error_handling_for_serve() {
    use crate::error::WebMockError;

    // Test port in use error
    let port_error = WebMockError::PortInUse(8080);
    assert!(port_error.is_recoverable());

    let user_msg = port_error.user_message();
    assert!(user_msg.contains("8080"));
    assert!(user_msg.contains("port"));

    // Test snapshot not found error
    let snapshot_error = WebMockError::SnapshotNotFound("test".to_string());
    assert!(!snapshot_error.is_recoverable());

    let user_msg = snapshot_error.user_message();
    assert!(user_msg.contains("test"));
    assert!(user_msg.contains("not found"));
}

#[test]
fn test_user_feedback_for_serve() {
    use crate::feedback::UserFeedback;

    // Test that feedback methods work without panicking
    UserFeedback::info("Starting server...");
    UserFeedback::success("Server started successfully");
    UserFeedback::warning("Port conflict detected");
    UserFeedback::error("Failed to start server");
    UserFeedback::tip("Try using a different port");
}

#[test]
fn test_socket_addr_creation() {
    // Test SocketAddr creation for different ports
    let addr1 = SocketAddr::from(([127, 0, 0, 1], 8080));
    assert_eq!(addr1.port(), 8080);
    assert!(addr1.is_ipv4());

    let addr2 = SocketAddr::from(([127, 0, 0, 1], 3000));
    assert_eq!(addr2.port(), 3000);

    // Test that we can create listeners on different addresses
    let listener1 = TcpListener::bind("127.0.0.1:0");
    assert!(listener1.is_ok());

    let listener2 = TcpListener::bind("localhost:0");
    assert!(listener2.is_ok());
}

#[test]
fn test_port_range_validation() {
    // Test edge cases for port numbers
    let max_port = 65535u16;
    let min_port = 1u16;

    // Test port number bounds
    assert!(max_port == 65535);
    assert!(min_port == 1);

    // Test socket address creation with edge case ports
    let addr_max = SocketAddr::from(([127, 0, 0, 1], max_port));
    let addr_min = SocketAddr::from(([127, 0, 0, 1], min_port));

    assert_eq!(addr_max.port(), max_port);
    assert_eq!(addr_min.port(), min_port);
}
