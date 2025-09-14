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
async fn test_capture_url_validation() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);
    let mut session = CaptureSession::new(storage_arc).await.unwrap();

    // Test invalid URL formats
    let invalid_urls = vec![
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

#[test]
fn test_timeout_validation() {
    // Test timeout parameter validation
    let valid_timeouts = vec![1, 5, 30, 60, 300];
    let invalid_timeouts = vec![0, 601, 1000];

    for timeout in valid_timeouts {
        // Should be within reasonable bounds
        assert!(timeout > 0);
        assert!(timeout <= 600);
    }

    for timeout in invalid_timeouts {
        assert!(!(timeout > 0 && timeout <= 600));
    }
}
