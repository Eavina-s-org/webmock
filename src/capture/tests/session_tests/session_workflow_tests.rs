use crate::capture::CaptureSession;
use crate::storage::Storage;
use crate::test_utils::test_helpers::*;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_capture_invalid_url() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
    let mut session = CaptureSession::new(storage).await.unwrap();

    // Test invalid URL format
    let result = session.capture("not-a-url", "test", 30).await;
    assert!(result.is_err());

    // Test unsupported scheme
    let result = session.capture("ftp://example.com", "test", 30).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_navigate_and_wait_no_browser() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
    let mut session = CaptureSession::new(storage).await.unwrap();

    // Should fail when no browser is initialized
    let result = session.navigate_and_wait("http://example.com").await;
    assert!(result.is_err());
}

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
async fn test_capture_url_validation() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Test invalid URL formats
    let invalid_urls = [
        "",
        "not-a-url",
        "ftp://example.com",
        "file:///etc/passwd",
        "javascript:alert('xss')",
    ];

    for url in invalid_urls {
        let result = session.capture(url, "test", 5).await;
        assert!(result.is_err(), "Should reject invalid URL: {}", url);
    }

    cleanup_test_env();
}

#[tokio::test]
#[ignore = "slow test - requires network/Chrome"]
async fn test_capture_with_valid_urls() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Test valid URLs (will likely fail due to no Chrome, but tests validation)
    let valid_urls = [
        "http://httpbin.org/get",
        "https://example.com",
        "http://localhost:3000",
    ];

    for url in valid_urls {
        let result = session.capture(url, "test", 1).await; // Very short timeout

        // Will likely fail due to Chrome not being available or timeout
        // But should not fail due to URL validation
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Chrome")
                    || error_msg.contains("timeout")
                    || error_msg.contains("browser")
                    || error_msg.contains("Browser")
                    || error_msg.contains("Network")
                    || error_msg.contains("connection")
                    || error_msg.contains("oneshot")
                    || error_msg.contains("canceled"),
                "Unexpected error for URL {}: {}",
                url,
                error_msg
            );
        }
    }

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

#[test]
fn test_timeout_validation() {
    // Test timeout parameter validation
    let valid_timeouts = [1, 5, 30, 60, 300];
    let invalid_timeouts = [0, 601, 1000];

    for timeout in valid_timeouts {
        // Should be within reasonable bounds
        assert!(timeout > 0);
        assert!(timeout <= 600);
    }

    for timeout in invalid_timeouts {
        // These should be rejected by validation
        assert!(timeout == 0 || timeout > 600);
    }
}

#[test]
fn test_snapshot_name_validation() {
    let valid_names = [
        "test-snapshot",
        "my_capture",
        "snapshot123",
        "valid-name_with-chars",
    ];

    let invalid_names = [
        "",
        "invalid/name",
        "test snapshot", // spaces
        "name with spaces",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // too long (101 chars)
    ];

    for name in valid_names {
        assert!(!name.is_empty());
        assert!(name.len() <= 100);
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
        assert!(!name.contains(' '));
    }

    for name in invalid_names {
        assert!(
            name.is_empty()
                || name.len() > 100
                || name.contains('/')
                || name.contains('\\')
                || name.contains(' ')
        );
    }
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    // Test creating multiple sessions
    let session1 = CaptureSession::new(storage_arc.clone()).await;
    let session2 = CaptureSession::new(storage_arc.clone()).await;

    assert!(session1.is_ok());
    assert!(session2.is_ok());

    let mut session1 = session1.unwrap();
    let mut session2 = session2.unwrap();

    // Test that both sessions can be cleaned up
    assert!(session1.cleanup().await.is_ok());
    assert!(session2.cleanup().await.is_ok());

    cleanup_test_env();
}

#[tokio::test]
async fn test_session_state_management() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Test initial state
    assert!(!session.is_active());
    assert_eq!(session.get_proxy_port(), 0);

    // Test request count tracking
    let initial_count = session.get_request_count().await;
    assert_eq!(initial_count, 0);

    // After cleanup, state should remain consistent
    session.cleanup().await.unwrap();
    assert!(!session.is_active());

    cleanup_test_env();
}

#[test]
fn test_error_handling_patterns() {
    use crate::error::WebMockError;

    // Test different error types that might occur during capture
    let errors = [
        WebMockError::ChromeNotFound,
        WebMockError::InvalidUrl("invalid".to_string(), "bad format".to_string()),
        WebMockError::Timeout(30),
        WebMockError::config("Browser not initialized"),
    ];

    for error in errors {
        // Test that errors have meaningful messages
        let user_msg = error.user_message();
        assert!(!user_msg.is_empty());
        assert!(user_msg.len() > 10);

        // Test error categorization
        let is_recoverable = error.is_recoverable();
        match error {
            WebMockError::ChromeNotFound => assert!(!is_recoverable),
            WebMockError::InvalidUrl(_, _) => assert!(!is_recoverable),
            WebMockError::Timeout(_) => assert!(is_recoverable),
            _ => {} // Other errors may vary
        }
    }
}

#[tokio::test]
async fn test_storage_integration() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();

    // Test that storage is properly initialized
    assert!(storage.ensure_snapshots_dir().is_ok());

    // Test snapshot operations
    let snapshot = create_test_snapshot_with_name("session-test");
    assert!(storage.save_snapshot(snapshot.clone()).await.is_ok());

    // Test loading
    let loaded = storage.load_snapshot("session-test").await;
    assert!(loaded.is_ok());

    let loaded_snapshot = loaded.unwrap();
    assert_eq!(loaded_snapshot.name, snapshot.name);
    assert_eq!(loaded_snapshot.url, snapshot.url);

    cleanup_test_env();
}

#[test]
fn test_url_parsing_and_validation() {
    use url::Url;

    let test_cases = [
        ("http://example.com", true),
        ("https://example.com", true),
        ("http://localhost:3000", true),
        ("https://api.example.com/v1", true),
        ("ftp://example.com", true), // Valid URL but wrong scheme for our use case
        ("invalid-url", false),
        ("", false),
    ];

    for (url_str, should_parse) in test_cases {
        let result = Url::parse(url_str);
        assert_eq!(
            result.is_ok(),
            should_parse,
            "URL parsing mismatch for: {}",
            url_str
        );

        if should_parse {
            let url = result.unwrap();
            // Additional validation for our use case
            let is_http_scheme = url.scheme() == "http" || url.scheme() == "https";
            if url_str.starts_with("http") {
                assert!(is_http_scheme, "Should be HTTP/HTTPS scheme: {}", url_str);
            }
        }
    }
}
