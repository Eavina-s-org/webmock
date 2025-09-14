use crate::capture::proxy::{RequestRecord, ResponseRecord};
use crate::storage::{Snapshot, Storage};
use chrono::Utc;
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_snapshot() -> Snapshot {
    Snapshot {
        name: "test-snapshot".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests: vec![RequestRecord {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers: HashMap::new(),
                body: b"<html><body>Test</body></html>".to_vec(),
                content_type: "text/html".to_string(),
            },
            timestamp: Utc::now(),
        }],
    }
}

#[test]
fn test_storage_new() {
    let temp_dir = TempDir::new().unwrap();
    let _storage = Storage::new(temp_dir.path().to_path_buf());
    // Storage is created successfully - we can't access private fields but that's OK
}

#[test]
fn test_ensure_snapshots_dir() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    let snapshots_dir = storage.ensure_snapshots_dir().unwrap();
    assert!(snapshots_dir.exists());
    assert!(snapshots_dir.is_dir());
    assert_eq!(snapshots_dir, temp_dir.path().join("snapshots"));
}

#[test]
fn test_get_snapshot_path() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    let path = storage.get_snapshot_path("test-snapshot");
    let expected = temp_dir
        .path()
        .join("snapshots")
        .join("test-snapshot.msgpack");
    assert_eq!(path, expected);
}

#[test]
fn test_snapshot_exists() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Should not exist initially
    assert!(!storage.snapshot_exists("test-snapshot"));

    // Create the snapshots directory and a test file
    storage.ensure_snapshots_dir().unwrap();
    let snapshot_path = storage.get_snapshot_path("test-snapshot");
    std::fs::write(&snapshot_path, b"test data").unwrap();

    // Should exist now
    assert!(storage.snapshot_exists("test-snapshot"));
}

#[tokio::test]
async fn test_save_and_load_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());
    let snapshot = create_test_snapshot();

    // Save snapshot
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    // Verify file exists
    assert!(storage.snapshot_exists(&snapshot.name));

    // Load snapshot
    let loaded_snapshot = storage.load_snapshot(&snapshot.name).await.unwrap();

    // Verify loaded data matches original
    assert_eq!(loaded_snapshot.name, snapshot.name);
    assert_eq!(loaded_snapshot.url, snapshot.url);
    assert_eq!(loaded_snapshot.requests.len(), snapshot.requests.len());

    // Check first request details
    let original_req = &snapshot.requests[0];
    let loaded_req = &loaded_snapshot.requests[0];
    assert_eq!(loaded_req.method, original_req.method);
    assert_eq!(loaded_req.url, original_req.url);
    assert_eq!(loaded_req.response.status, original_req.response.status);
    assert_eq!(loaded_req.response.body, original_req.response.body);
}

#[tokio::test]
async fn test_load_nonexistent_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    let result = storage.load_snapshot("nonexistent").await;
    assert!(result.is_err());

    if let Err(crate::error::WebMockError::SnapshotNotFound(name)) = result {
        assert_eq!(name, "nonexistent");
    } else {
        panic!("Expected SnapshotNotFound error");
    }
}

#[tokio::test]
async fn test_list_snapshots_empty() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    let snapshots = storage.list_snapshots().await.unwrap();
    assert!(snapshots.is_empty());
}

#[tokio::test]
async fn test_list_snapshots_with_data() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Create and save multiple snapshots
    let mut snapshot1 = create_test_snapshot();
    snapshot1.name = "snapshot1".to_string();
    snapshot1.url = "https://example1.com".to_string();

    let mut snapshot2 = create_test_snapshot();
    snapshot2.name = "snapshot2".to_string();
    snapshot2.url = "https://example2.com".to_string();
    snapshot2.created_at = Utc::now() + chrono::Duration::hours(1); // Make it newer

    storage.save_snapshot(snapshot1.clone()).await.unwrap();
    storage.save_snapshot(snapshot2.clone()).await.unwrap();

    // List snapshots
    let snapshots = storage.list_snapshots().await.unwrap();
    assert_eq!(snapshots.len(), 2);

    // Should be sorted by creation date (newest first)
    assert_eq!(snapshots[0].name, "snapshot2");
    assert_eq!(snapshots[1].name, "snapshot1");

    // Verify snapshot info
    assert_eq!(snapshots[0].url, "https://example2.com");
    assert_eq!(snapshots[1].url, "https://example1.com");
}

#[tokio::test]
async fn test_delete_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());
    let snapshot = create_test_snapshot();

    // Save snapshot
    storage.save_snapshot(snapshot.clone()).await.unwrap();
    assert!(storage.snapshot_exists(&snapshot.name));

    // Delete snapshot
    storage.delete_snapshot(&snapshot.name).await.unwrap();
    assert!(!storage.snapshot_exists(&snapshot.name));

    // Verify it's gone from list
    let snapshots = storage.list_snapshots().await.unwrap();
    assert!(snapshots.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    let result = storage.delete_snapshot("nonexistent").await;
    assert!(result.is_err());

    if let Err(crate::error::WebMockError::SnapshotNotFound(name)) = result {
        assert_eq!(name, "nonexistent");
    } else {
        panic!("Expected SnapshotNotFound error");
    }
}

#[tokio::test]
async fn test_save_snapshot_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());
    let snapshot = create_test_snapshot();

    // Ensure snapshots directory doesn't exist initially
    let snapshots_dir = temp_dir.path().join("snapshots");
    assert!(!snapshots_dir.exists());

    // Save snapshot should create the directory
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    assert!(snapshots_dir.exists());
    assert!(snapshots_dir.is_dir());
    assert!(storage.snapshot_exists(&snapshot.name));
}

#[tokio::test]
// #[ignore = "slow test - large data processing"]
async fn test_list_snapshots_ignores_non_msgpack_files() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Create snapshots directory
    storage.ensure_snapshots_dir().unwrap();
    let snapshots_dir = temp_dir.path().join("snapshots");

    // Create a valid snapshot
    let snapshot = create_test_snapshot();
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    // Create some non-msgpack files that should be ignored
    tokio::fs::write(snapshots_dir.join("readme.txt"), b"some text")
        .await
        .unwrap();
    tokio::fs::write(snapshots_dir.join("config.json"), b"{}")
        .await
        .unwrap();
    tokio::fs::write(snapshots_dir.join("invalid.msgpack"), b"invalid data")
        .await
        .unwrap();

    // List snapshots should only return the valid one
    let snapshots = storage.list_snapshots().await.unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].name, snapshot.name);
}
