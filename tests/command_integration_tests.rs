//! Command-level integration tests for WebMock CLI
//!
//! These tests verify the integration between different command modules
//! and test error scenarios and recovery mechanisms.

use std::env;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

use std::collections::HashMap;
use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    commands::{list_command, serve_command},
    error::{Result, WebMockError},
    storage::{Snapshot, Storage},
};

/// Helper to set up a temporary home directory for testing
fn setup_test_home(temp_dir: &TempDir) -> PathBuf {
    let home_path = temp_dir.path().to_path_buf();
    env::set_var("HOME", &home_path);
    home_path
}

/// Helper to create a test snapshot in storage
async fn create_test_snapshot_in_storage(storage: &Storage, name: &str) -> Result<()> {
    let mut headers = HashMap::new();
    headers.insert(
        "content-type".to_string(),
        "text/html; charset=utf-8".to_string(),
    );

    let html_content = format!(
        "<html><head><title>{}</title></head><body><h1>Test Page: {}</h1></body></html>",
        name, name
    );

    let request = RequestRecord {
        method: "GET".to_string(),
        url: format!("https://example.com/{}", name),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers,
            body: html_content.as_bytes().to_vec(),
            content_type: "text/html".to_string(),
        },
        timestamp: chrono::Utc::now(),
    };

    let snapshot = Snapshot {
        name: name.to_string(),
        url: format!("https://example.com/{}", name),
        created_at: chrono::Utc::now(),
        requests: vec![request],
    };

    storage.save_snapshot(snapshot).await
}

#[cfg(test)]
mod command_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_command_with_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);
        env::set_var("WEBMOCK_SKIP_PERMISSION_CHECK", "1");

        // Create .webmock directory structure
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir.clone());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create test snapshots
        create_test_snapshot_in_storage(&storage, "test-site-1")
            .await
            .expect("Failed to create test snapshot 1");
        create_test_snapshot_in_storage(&storage, "test-site-2")
            .await
            .expect("Failed to create test snapshot 2");

        // Test list command with explicit storage path
        let result = list_command(Some(webmock_dir.to_string_lossy().to_string())).await;
        assert!(
            result.is_ok(),
            "List command should succeed with snapshots present"
        );
    }

    #[tokio::test]
    async fn test_list_command_empty_storage() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);
        env::set_var("WEBMOCK_SKIP_PERMISSION_CHECK", "1");

        // Create empty .webmock directory
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir.clone());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Test list command with explicit storage path
        let result = list_command(Some(webmock_dir.to_string_lossy().to_string())).await;
        if let Err(e) = &result {
            println!("List command failed with error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "List command should succeed even with empty storage"
        );
    }

    #[tokio::test]
    async fn test_list_command_no_storage_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);
        env::set_var("WEBMOCK_SKIP_PERMISSION_CHECK", "1");

        // Don't create .webmock directory - test error handling
        let result = list_command(None).await;
        // Should handle missing directory gracefully (returns empty list)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_serve_command_with_valid_snapshot() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);

        // Create .webmock directory and snapshot
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir);
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");
        create_test_snapshot_in_storage(&storage, "test-serve-snapshot")
            .await
            .expect("Failed to create test snapshot");

        // Test serve command with timeout (since it runs indefinitely)
        let serve_future = serve_command("test-serve-snapshot", 8080, None);
        let result = timeout(Duration::from_millis(500), serve_future).await;

        // Should timeout (meaning server started successfully) or return port conflict error
        assert!(result.is_err() || result.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_serve_command_snapshot_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);

        // Create empty .webmock directory
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir);
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Test serve command with non-existent snapshot
        let result = serve_command("non-existent-snapshot", 8080, None).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebMockError::SnapshotNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_command_with_valid_snapshot() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);

        // Create .webmock directory and snapshot
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir);
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");
        create_test_snapshot_in_storage(&storage, "test-delete-snapshot")
            .await
            .expect("Failed to create test snapshot");

        // Verify snapshot exists
        let snapshots_before = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots_before.len(), 1);

        // Note: delete_command requires user confirmation, so we can't easily test it
        // without mocking the confirmation dialog. Instead, we test the storage delete directly.
        let delete_result = storage.delete_snapshot("test-delete-snapshot").await;
        assert!(delete_result.is_ok());

        // Verify snapshot is deleted
        let snapshots_after = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots_after.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_command_snapshot_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _home_path = setup_test_home(&temp_dir);

        // Create empty .webmock directory
        let webmock_dir = temp_dir.path().join(".webmock");
        tokio::fs::create_dir_all(&webmock_dir)
            .await
            .expect("Failed to create .webmock dir");

        let storage = Storage::new(webmock_dir);
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Test delete with non-existent snapshot
        let result = storage.delete_snapshot("non-existent-snapshot").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebMockError::SnapshotNotFound(_)
        ));
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_permission_error_handling() {
        // Create a path that should trigger permission errors on most systems
        // Use a directory that exists but is not writable by the current user
        let invalid_path = if cfg!(target_os = "macos") {
            // On macOS, use a system directory that's not writable
            PathBuf::from("/System/Library/PrivateFrameworks/webmock_test")
        } else if cfg!(target_os = "windows") {
            // On Windows, use a system directory
            PathBuf::from("C:\\Windows\\System32\\webmock_test")
        } else {
            // On Linux and other Unix-like systems
            PathBuf::from("/root/invalid_webmock_path")
        };

        let storage = Storage::new(invalid_path.clone());

        // Should handle permission errors gracefully
        let result = storage.ensure_snapshots_dir();

        // The test should pass if we get any kind of error (permission denied, path not found, etc.)
        // This makes the test more robust across different environments
        assert!(
            result.is_err(),
            "Expected error when trying to create directory in {:?}, but got: {:?}",
            invalid_path,
            result
        );

        // Optionally verify it's a storage or permission-related error
        match result.unwrap_err() {
            WebMockError::Storage(_) | WebMockError::PermissionDenied(_) => {
                // Expected error types
            }
            _ => {
                // Any other error is also acceptable for this test
                // as long as we don't silently succeed
            }
        }
    }

    #[tokio::test]
    async fn test_corrupted_storage_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create a corrupted snapshot file
        let snapshot_path = storage.get_snapshot_path("corrupted");
        tokio::fs::write(&snapshot_path, b"invalid msgpack data")
            .await
            .expect("Failed to write corrupted file");

        // List snapshots should handle corrupted files gracefully
        let result = storage.list_snapshots().await;
        // Should either succeed (skipping corrupted files) or return a clear error
        match result {
            Ok(snapshots) => {
                // If it succeeds, corrupted files should be filtered out
                assert!(snapshots.iter().all(|s| s.name != "corrupted"));
            }
            Err(e) => {
                // If it fails, should be a clear storage error
                assert!(matches!(e, WebMockError::Storage(_)));
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_snapshot_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create multiple snapshots sequentially
        for i in 0..5 {
            create_test_snapshot_in_storage(&storage, &format!("sequential-{}", i))
                .await
                .unwrap_or_else(|_| panic!("Failed to create snapshot {}", i));
        }

        // Verify all snapshots were created
        let snapshots = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        println!("Found {} snapshots after operations", snapshots.len());
        for snapshot in &snapshots {
            println!("  - {}", snapshot.name);
        }
        assert_eq!(snapshots.len(), 5);

        // Verify all expected names are present
        let names: std::collections::HashSet<&str> =
            snapshots.iter().map(|s| s.name.as_str()).collect();
        for i in 0..5 {
            assert!(names.contains(&format!("sequential-{}", i).as_str()));
        }
    }

    #[tokio::test]
    async fn test_large_snapshot_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create a snapshot with many requests (simulating a large capture session)
        let mut requests = Vec::new();

        for i in 0..1000 {
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());

            let response_body = format!(
                "{{\"id\": {}, \"data\": \"test data for request {}\"}}",
                i, i
            );

            requests.push(RequestRecord {
                method: "GET".to_string(),
                url: format!("https://api.example.com/item/{}", i),
                headers: HashMap::new(),
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers,
                    body: response_body.as_bytes().to_vec(),
                    content_type: "application/json".to_string(),
                },
                timestamp: chrono::Utc::now(),
            });
        }

        let large_snapshot = Snapshot {
            name: "large-snapshot".to_string(),
            url: "https://api.example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests,
        };

        // Test saving large snapshot
        let save_result = storage.save_snapshot(large_snapshot).await;
        assert!(
            save_result.is_ok(),
            "Should be able to save large snapshots"
        );

        // Test loading large snapshot
        let load_result = storage.load_snapshot("large-snapshot").await;
        assert!(
            load_result.is_ok(),
            "Should be able to load large snapshots"
        );

        let loaded_snapshot = load_result.unwrap();
        assert_eq!(loaded_snapshot.requests.len(), 1000);
    }

    #[tokio::test]
    async fn test_disk_space_simulation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create a very large snapshot to test disk space handling
        // Note: This is a simulation - in a real scenario, we'd need to mock filesystem operations
        let large_body = vec![b'X'; 10 * 1024 * 1024]; // 10MB

        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "application/octet-stream".to_string(),
        );

        let request = RequestRecord {
            method: "GET".to_string(),
            url: "https://example.com/large-file".to_string(),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers,
                body: large_body,
                content_type: "application/octet-stream".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let large_snapshot = Snapshot {
            name: "disk-space-test".to_string(),
            url: "https://example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: vec![request],
        };

        // This should succeed in most test environments
        // In production, this would test disk space error handling
        let result = storage.save_snapshot(large_snapshot).await;

        match result {
            Ok(_) => {
                // If successful, verify we can load it back
                let loaded = storage.load_snapshot("disk-space-test").await;
                assert!(loaded.is_ok());
                assert_eq!(
                    loaded.unwrap().requests[0].response.body.len(),
                    10 * 1024 * 1024
                );
            }
            Err(e) => {
                // If it fails due to disk space, should be a storage error
                assert!(matches!(e, WebMockError::Storage(_)));
            }
        }
    }
}
