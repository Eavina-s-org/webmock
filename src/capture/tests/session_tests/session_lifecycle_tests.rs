use crate::capture::CaptureSession;

use crate::test_utils::test_helpers::*;
use std::sync::Arc;

#[tokio::test]
async fn test_capture_session_lifecycle() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    // Test session creation
    let session = CaptureSession::new(storage_arc).await;
    assert!(session.is_ok());

    let session = session.unwrap();

    // Test initial state
    assert!(!session.is_active());
    assert_eq!(session.get_proxy_port(), 0);

    // Test request count
    let count = session.get_request_count().await;
    assert_eq!(count, 0);

    cleanup_test_env();
}

#[tokio::test]
async fn test_session_cleanup() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Test cleanup without any resources
    let result = session.cleanup().await;
    assert!(result.is_ok());

    // Test multiple cleanups (should be safe)
    let result = session.cleanup().await;
    assert!(result.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_browser_initialization_logic() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let session = CaptureSession::new(storage_arc).await.unwrap();

    // Test that session is created without browser initially
    assert!(!session.is_active());

    // Browser initialization would happen during capture
    // We test the session state management instead

    cleanup_test_env();
}

#[tokio::test]
async fn test_stop_session_without_start() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Try to stop session without starting capture
    let result = session.stop("test-snapshot", "http://example.com").await;

    // Should handle gracefully (might succeed with empty snapshot or fail appropriately)
    match result {
        Ok(snapshot) => {
            assert_eq!(snapshot.name, "test-snapshot".to_string());
            assert_eq!(snapshot.url, "http://example.com".to_string());
            // Should have no requests since no capture was performed
            assert_eq!(snapshot.requests.len(), 0);
        }
        Err(e) => {
            // Also acceptable - stopping without starting might be an error
            assert!(!e.to_string().is_empty());
        }
    }

    cleanup_test_env();
}
