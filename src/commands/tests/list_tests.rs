use tempfile::TempDir;

use crate::{
    commands::list, storage::Storage, test_utils::test_helpers::create_test_snapshot_with_name,
};

#[tokio::test]
async fn test_list_empty_snapshots() {
    // Create temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");

    // This should not fail even with no snapshots
    let result = list::list_command(Some(storage_path.to_string_lossy().to_string())).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_with_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");
    std::fs::create_dir_all(&storage_path).unwrap();

    // Create storage and add test snapshots
    let storage = Storage::new(storage_path.clone());
    let snapshot = create_test_snapshot_with_name("test-list-snapshot");
    storage.save_snapshot(snapshot).await.unwrap();

    let result = list::list_command(Some(storage_path.to_string_lossy().to_string())).await;
    assert!(result.is_ok());
}
