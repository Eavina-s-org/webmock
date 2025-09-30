use crate::capture::CaptureSession;
use crate::storage::Storage;
use crate::test_utils::test_helpers::*;
use std::sync::Arc;
use tempfile::TempDir;

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
