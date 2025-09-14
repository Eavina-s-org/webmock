use crate::capture::CaptureSession;
use crate::storage::Storage;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_capture_session_creation() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));

    let session = CaptureSession::new(storage).await;
    assert!(session.is_ok());

    let session = session.unwrap();
    assert!(!session.is_active());
    assert_eq!(session.get_proxy_port(), 0);
}

#[tokio::test]
async fn test_get_request_count_empty() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
    let session = CaptureSession::new(storage).await.unwrap();

    let count = session.get_request_count().await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_cleanup_without_resources() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
    let mut session = CaptureSession::new(storage).await.unwrap();

    // Should not fail even if no resources are allocated
    let result = session.cleanup().await;
    assert!(result.is_ok());
}
