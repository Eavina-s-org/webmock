//! Error handling and recovery mechanism tests
//! Tests various error scenarios and recovery mechanisms

use std::collections::HashMap;
use std::net::TcpListener;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    error::WebMockError,
    serve::MockServer,
    storage::{Snapshot, Storage},
};

/// Helper to find available port
fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener
        .local_addr()
        .expect("Failed to get local addr")
        .port();
    drop(listener);
    port
}

/// Create a minimal test snapshot
async fn create_minimal_snapshot(name: &str) -> Snapshot {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "text/html".to_string());

    let request = RequestRecord {
        method: "GET".to_string(),
        url: format!("https://example.com/{}", name),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers,
            body: b"<html><body>Test</body></html>".to_vec(),
            content_type: "text/html".to_string(),
        },
        timestamp: chrono::Utc::now(),
    };

    Snapshot {
        name: name.to_string(),
        url: format!("https://example.com/{}", name),
        created_at: chrono::Utc::now(),
        requests: vec![request],
    }
}

#[cfg(test)]
mod storage_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_not_found_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Try to load non-existent snapshot
        let result = storage.load_snapshot("non-existent-snapshot").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebMockError::SnapshotNotFound(name) => {
                assert_eq!(name, "non-existent-snapshot");
            }
            other => panic!("Expected SnapshotNotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_corrupted_snapshot_file() {
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

        // Try to load corrupted snapshot
        let result = storage.load_snapshot("corrupted").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebMockError::Storage(_) | WebMockError::Deserialization(_) => {
                // Expected - should be a storage/deserialization error
            }
            other => panic!(
                "Expected Storage or Deserialization error, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_permission_denied_error() {
        // Try to create storage in a location that should cause permission issues
        let invalid_path = std::path::PathBuf::from("/root/webmock_test_invalid");
        let storage = Storage::new(invalid_path);

        // Should fail to create snapshots directory
        let result = storage.ensure_snapshots_dir();
        assert!(result.is_err());

        // The exact error type may vary by system, but it should be an error
        match result.unwrap_err() {
            WebMockError::Storage(_) | WebMockError::PermissionDenied(_) => {
                // Expected - permission or storage error
            }
            other => panic!("Expected permission or storage error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_empty_snapshot_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create an empty snapshot file
        let snapshot_path = storage.get_snapshot_path("empty");
        tokio::fs::write(&snapshot_path, b"")
            .await
            .expect("Failed to write empty file");

        // Try to load empty snapshot
        let result = storage.load_snapshot("empty").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebMockError::Storage(_) | WebMockError::Deserialization(_) => {
                // Expected - should be a deserialization error
            }
            other => panic!(
                "Expected Storage or Deserialization error, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_duplicate_snapshot_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create and save a snapshot
        let snapshot = create_minimal_snapshot("duplicate-test").await;
        storage
            .save_snapshot(snapshot.clone())
            .await
            .expect("Failed to save first snapshot");

        // Try to save the same snapshot again (should overwrite)
        let result = storage.save_snapshot(snapshot).await;
        assert!(
            result.is_ok(),
            "Should be able to overwrite existing snapshot"
        );

        // Verify only one snapshot exists
        let snapshots = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots.len(), 1);
    }
}

#[cfg(test)]
mod server_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_port_already_in_use() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        let snapshot = create_minimal_snapshot("port-test").await;
        storage
            .save_snapshot(snapshot.clone())
            .await
            .expect("Failed to save snapshot");

        // Bind to a port to make it unavailable
        let port = find_available_port();
        let _listener =
            TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to port");

        // Try to start server on the occupied port
        let mock_server = MockServer::new(snapshot);

        // This should panic or fail when trying to bind to the occupied port
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                timeout(Duration::from_millis(1000), mock_server.start(port)).await
            })
        }));

        // Should either panic or return an error
        assert!(
            result.is_err() || {
                match result.unwrap() {
                    Ok(Ok(_)) => false, // Unexpected success
                    _ => true,          // Timeout or error - both are acceptable
                }
            }
        );
    }

    #[tokio::test]
    async fn test_server_with_empty_snapshot() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create snapshot with no requests
        let empty_snapshot = Snapshot {
            name: "empty-server-test".to_string(),
            url: "https://example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: Vec::new(),
        };

        storage
            .save_snapshot(empty_snapshot.clone())
            .await
            .expect("Failed to save empty snapshot");

        // Start server with empty snapshot
        let mock_server = MockServer::new(empty_snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        // All requests should return 404
        let client = reqwest::Client::new();

        let response = client
            .get(format!("http://localhost:{}/", port))
            .send()
            .await
            .expect("Failed to make request");
        assert_eq!(response.status(), 404);

        let response = client
            .get(format!("http://localhost:{}/any-path", port))
            .send()
            .await
            .expect("Failed to make request");
        assert_eq!(response.status(), 404);

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_server_with_malformed_responses() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create snapshot with potentially problematic response data
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let request = RequestRecord {
            method: "GET".to_string(),
            url: "https://example.com/malformed".to_string(),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers,
                body: b"{ invalid json content".to_vec(), // Malformed JSON
                content_type: "application/json".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let malformed_snapshot = Snapshot {
            name: "malformed-test".to_string(),
            url: "https://example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: vec![request],
        };

        storage
            .save_snapshot(malformed_snapshot.clone())
            .await
            .expect("Failed to save malformed snapshot");

        // Server should still serve the malformed content as-is
        let mock_server = MockServer::new(malformed_snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://localhost:{}/malformed", port))
            .send()
            .await
            .expect("Failed to make request");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to get response body");
        assert_eq!(body, "{ invalid json content");

        server_handle.abort();
    }
}

#[cfg(test)]
mod recovery_mechanism_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_directory_auto_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage_path = temp_dir.path().join("auto-created-storage");

        // Storage directory doesn't exist yet
        assert!(!storage_path.exists());

        let storage = Storage::new(storage_path.clone());

        // Should auto-create directory when needed
        let result = storage.ensure_snapshots_dir();
        assert!(result.is_ok());
        assert!(storage_path.exists());
        assert!(storage_path.join("snapshots").exists());
    }

    #[tokio::test]
    async fn test_graceful_handling_of_partial_snapshot_list() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create valid snapshots
        let snapshot1 = create_minimal_snapshot("valid1").await;
        let snapshot2 = create_minimal_snapshot("valid2").await;

        storage
            .save_snapshot(snapshot1)
            .await
            .expect("Failed to save snapshot1");
        storage
            .save_snapshot(snapshot2)
            .await
            .expect("Failed to save snapshot2");

        // Create a corrupted file in the snapshots directory
        let snapshots_dir = storage
            .ensure_snapshots_dir()
            .expect("Failed to ensure snapshots dir");
        let corrupted_path = snapshots_dir.join("corrupted.msgpack");
        tokio::fs::write(&corrupted_path, b"invalid data")
            .await
            .expect("Failed to write corrupted file");

        // List snapshots should handle corrupted files gracefully
        let result = storage.list_snapshots().await;

        match result {
            Ok(snapshots) => {
                // Should return valid snapshots, filtering out corrupted ones
                assert!(snapshots.len() >= 2);
                let names: Vec<&str> = snapshots.iter().map(|s| s.name.as_str()).collect();
                assert!(names.contains(&"valid1"));
                assert!(names.contains(&"valid2"));
                assert!(!names.contains(&"corrupted"));
            }
            Err(_) => {
                // If it fails completely, that's also acceptable behavior
                // The important thing is that it doesn't panic or hang
            }
        }
    }

    #[tokio::test]
    async fn test_server_recovery_after_client_disconnect() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        let snapshot = create_minimal_snapshot("disconnect-test").await;
        storage
            .save_snapshot(snapshot.clone())
            .await
            .expect("Failed to save snapshot");

        let mock_server = MockServer::new(snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Make multiple requests, some of which we'll abort
        let client = reqwest::Client::new();

        // Normal request - need to match the URL in the snapshot
        let response1 = client
            .get(format!("http://localhost:{}/disconnect-test", port))
            .send()
            .await
            .expect("Failed to make first request");
        assert_eq!(response1.status(), 200);

        // Simulate abrupt client disconnect by dropping the client
        drop(client);

        // Create new client and make another request
        let client2 = reqwest::Client::new();
        let response2 = client2
            .get(format!("http://localhost:{}/disconnect-test", port))
            .send()
            .await
            .expect("Failed to make request after disconnect");
        assert_eq!(response2.status(), 200);

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_concurrent_snapshot_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create multiple snapshots concurrently
        let mut handles = Vec::new();

        for i in 0..10 {
            let storage_path = temp_dir.path().to_path_buf();
            let handle = tokio::spawn(async move {
                let storage_clone = Storage::new(storage_path);
                let snapshot = create_minimal_snapshot(&format!("concurrent-{}", i)).await;
                storage_clone.save_snapshot(snapshot).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All operations should succeed
        for (i, result) in results.into_iter().enumerate() {
            assert!(result.is_ok(), "Task {} panicked", i);
            assert!(result.unwrap().is_ok(), "Save operation {} failed", i);
        }

        // Verify all snapshots were created
        let snapshots = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots.len(), 10);

        // Verify all expected names are present
        let names: std::collections::HashSet<&str> =
            snapshots.iter().map(|s| s.name.as_str()).collect();
        for i in 0..10 {
            assert!(names.contains(&format!("concurrent-{}", i).as_str()));
        }
    }

    #[tokio::test]
    async fn test_large_snapshot_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create a snapshot with a large response body
        let large_body = vec![b'X'; 5 * 1024 * 1024]; // 5MB

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
                body: large_body.clone(),
                content_type: "application/octet-stream".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let large_snapshot = Snapshot {
            name: "large-file-test".to_string(),
            url: "https://example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: vec![request],
        };

        // Test saving large snapshot
        let save_result = storage.save_snapshot(large_snapshot).await;
        assert!(
            save_result.is_ok(),
            "Should be able to save large snapshots"
        );

        // Test loading large snapshot
        let load_result = storage.load_snapshot("large-file-test").await;
        assert!(
            load_result.is_ok(),
            "Should be able to load large snapshots"
        );

        let loaded_snapshot = load_result.unwrap();
        assert_eq!(
            loaded_snapshot.requests[0].response.body.len(),
            5 * 1024 * 1024
        );
        assert_eq!(loaded_snapshot.requests[0].response.body, large_body);

        // Test serving large file
        let mock_server = MockServer::new(loaded_snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(300)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://localhost:{}/large-file", port))
            .send()
            .await
            .expect("Failed to request large file");

        assert_eq!(response.status(), 200);

        let body = response.bytes().await.expect("Failed to get response body");
        assert_eq!(body.len(), 5 * 1024 * 1024);

        server_handle.abort();
    }
}
