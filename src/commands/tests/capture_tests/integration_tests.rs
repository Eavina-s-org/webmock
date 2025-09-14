use tempfile::TempDir;

use crate::{
    commands::{
        capture::{check_snapshot_exists, initialize_storage},
        capture_command,
    },
    error::WebMockError,
    storage::Storage,
};

#[tokio::test]
async fn test_capture_command_validation_flow() {
    // Test the full validation flow without actually running capture

    // Test 1: Invalid URL should fail early
    let result = capture_command("not-a-url", "test", 30, None).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WebMockError::InvalidUrl(_, _)
    ));

    // Test 2: Invalid name should fail early
    let result = capture_command("https://example.com", "", 30, None).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));

    // Test 3: Invalid timeout should fail early
    let result = capture_command("https://example.com", "test", 0, None).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}

#[tokio::test]
async fn test_capture_command_storage_initialization() {
    // Use direct storage path instead of environment variables
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");

    // Test storage initialization with custom path
    let storage_result = initialize_storage(Some(storage_path.to_string_lossy().to_string())).await;
    assert!(storage_result.is_ok());

    let storage = storage_result.unwrap();

    // Verify snapshots directory is created
    let snapshots_dir = storage.ensure_snapshots_dir();
    assert!(snapshots_dir.is_ok());
    assert!(snapshots_dir.unwrap().exists());
}

#[tokio::test]
async fn test_capture_command_duplicate_snapshot_check() {
    // Set up temporary storage
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage.ensure_snapshots_dir().unwrap();

    // Create a dummy snapshot file
    let snapshot_path = storage.get_snapshot_path("existing-snapshot");
    tokio::fs::create_dir_all(snapshot_path.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&snapshot_path, b"dummy snapshot data")
        .await
        .unwrap();

    // Test that duplicate snapshot check works
    let result = check_snapshot_exists(&storage, "existing-snapshot");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));

    // Test that non-existing snapshot passes
    let result = check_snapshot_exists(&storage, "new-snapshot");
    assert!(result.is_ok());
}
