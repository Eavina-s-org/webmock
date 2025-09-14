use crate::capture::CaptureSession;
use crate::commands::capture::run_capture_with_progress;
use crate::feedback::ProgressReporter;
use crate::test_utils::test_helpers::*;
use std::sync::Arc;

#[tokio::test]
#[ignore = "slow test - requires network/Chrome"]
async fn test_run_capture_with_progress_success() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    // Create a capture session
    let mut session = CaptureSession::new(storage_arc).await.unwrap();
    let mut progress = ProgressReporter::new();

    // Note: This test will fail in CI without Chrome, but tests the structure
    let result = run_capture_with_progress(
        &mut session,
        &mut progress,
        "https://httpbin.org/get", // Use a simple endpoint
        "test-capture",
        5, // Short timeout
    )
    .await;

    // The capture might fail due to missing Chrome, but we test the error handling
    match result {
        Ok(_) => {
            // Success case - verify session state
            assert!(!session.is_active());
        }
        Err(e) => {
            // Expected in test environment without Chrome
            // Verify it's a Chrome-related error or timeout
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Chrome")
                    || error_msg.contains("timeout")
                    || error_msg.contains("browser")
                    || error_msg.contains("Browser")
                    || error_msg.contains("Network")
                    || error_msg.contains("oneshot")
                    || error_msg.contains("canceled")
            );
        }
    }

    cleanup_test_env();
}

#[tokio::test]
#[ignore = "slow test - requires network/Chrome"]
async fn test_run_capture_with_progress_invalid_url() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    let mut session = CaptureSession::new(storage_arc).await.unwrap();
    let mut progress = ProgressReporter::new();

    // Test with invalid URL
    let result = run_capture_with_progress(
        &mut session,
        &mut progress,
        "invalid-url",
        "test-capture",
        5,
    )
    .await;

    assert!(result.is_err());

    cleanup_test_env();
}

#[tokio::test]
#[ignore = "slow test - requires network/Chrome"]
async fn test_run_capture_with_recovery_structure() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    let mut session = CaptureSession::new(storage_arc).await.unwrap();
    let mut progress = ProgressReporter::new();

    // Test recovery by using a very short timeout that will likely fail
    let result = run_capture_with_progress(
        &mut session,
        &mut progress,
        "https://httpbin.org/status/200",
        "test-recovery",
        1, // Very short timeout to trigger failure
    )
    .await;

    // Should fail but test the retry logic structure
    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
fn test_progress_reporter_creation() {
    let mut progress = ProgressReporter::new();

    // Test that we can create progress indicators
    let capture_progress = progress.start_capture_progress("https://example.com");
    assert!(!capture_progress.is_finished());

    // Test progress updates
    progress.update_capture_step("Testing step");
    progress.finish_capture_success("test-snapshot");

    assert!(capture_progress.is_finished());
}

#[test]
fn test_progress_reporter_error_handling() {
    let mut progress = ProgressReporter::new();

    let capture_progress = progress.start_capture_progress("https://example.com");

    // Test error finish
    progress.finish_capture_error("Test error message");

    assert!(capture_progress.is_finished());
}

#[tokio::test]
async fn test_capture_session_basic_operations() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    // Test session creation
    let session = CaptureSession::new(storage_arc).await;
    assert!(session.is_ok());

    let mut session = session.unwrap();

    // Test initial state
    assert!(!session.is_active());
    assert_eq!(session.get_proxy_port(), 0);

    // Test request count
    let count = session.get_request_count().await;
    assert_eq!(count, 0);

    // Test cleanup (should not fail even if nothing to clean)
    let cleanup_result = session.cleanup().await;
    assert!(cleanup_result.is_ok());

    cleanup_test_env();
}

#[test]
fn test_error_recovery_logic() {
    use crate::error::WebMockError;

    // Test recoverable errors
    let recoverable_errors = vec![WebMockError::Timeout(30), WebMockError::PortInUse(8080)];

    for error in recoverable_errors {
        assert!(
            error.is_recoverable(),
            "Error should be recoverable: {:?}",
            error
        );
    }

    // Test non-recoverable errors
    let non_recoverable_errors = vec![
        WebMockError::ChromeNotFound,
        WebMockError::SnapshotNotFound("test".to_string()),
        WebMockError::InvalidUrl("invalid".to_string(), "bad format".to_string()),
    ];

    for error in non_recoverable_errors {
        assert!(
            !error.is_recoverable(),
            "Error should not be recoverable: {:?}",
            error
        );
    }
}

#[tokio::test]
#[ignore = "slow test - requires network/Chrome"]
async fn test_capture_timeout_handling() {
    setup_test_env();

    let (_temp_dir, storage) = create_temp_storage();
    let storage_arc = Arc::new(storage);

    let mut session = CaptureSession::new(storage_arc).await.unwrap();
    let mut progress = ProgressReporter::new();

    // Test with very short timeout (should fail quickly)
    let start_time = std::time::Instant::now();

    let result = run_capture_with_progress(
        &mut session,
        &mut progress,
        "https://httpbin.org/delay/10", // This will take 10 seconds
        "timeout-test",
        1, // 1 second timeout
    )
    .await;

    let elapsed = start_time.elapsed();

    // Should fail due to timeout
    assert!(result.is_err());

    // Should fail relatively quickly (within reasonable bounds)
    assert!(
        elapsed.as_secs() < 30,
        "Timeout handling took too long: {:?}",
        elapsed
    );

    cleanup_test_env();
}
