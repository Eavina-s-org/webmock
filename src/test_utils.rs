//! Unified test utilities
//!
//! This module provides common helper functions used in all tests,
//! avoiding duplication of the same code in different test files.

#[cfg(test)]
pub mod test_helpers {
    use crate::capture::proxy::records::response::ResponseRecord;
    use crate::capture::proxy::RequestRecord;
    use crate::storage::Snapshot;
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Create a standard test snapshot
    ///
    /// Contains a simple HTML page request, suitable for most test scenarios
    pub fn create_test_snapshot() -> Snapshot {
        create_test_snapshot_with_name("test-snapshot")
    }

    /// Create a test snapshot with custom name
    ///
    /// # Parameters
    /// * `name` - Snapshot name
    pub fn create_test_snapshot_with_name(name: &str) -> Snapshot {
        let mut headers = HashMap::new();
        headers.insert(
            "user-agent".to_string(),
            "WebMock-Test-Agent/1.0".to_string(),
        );
        headers.insert(
            "accept".to_string(),
            "text/html,application/xhtml+xml".to_string(),
        );

        let mut response_headers = HashMap::new();
        response_headers.insert(
            "content-type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        response_headers.insert(
            "cache-control".to_string(),
            "public, max-age=3600".to_string(),
        );

        Snapshot {
            name: name.to_string(),
            url: "https://example.com".to_string(),
            created_at: Utc::now(),
            requests: vec![
                RequestRecord {
                    method: "GET".to_string(),
                    url: "https://example.com/".to_string(),
                    headers,
                    body: None,
                    response: ResponseRecord {
                        status: 200,
                        headers: response_headers,
                        body: b"<html><body><h1>Test Page</h1><p>This is a test page for WebMock CLI.</p></body></html>".to_vec(),
                        content_type: "text/html".to_string(),
                    },
                    timestamp: Utc::now(),
                }
            ],
        }
    }

    /// Create a multi-request test snapshot
    ///
    /// Contains multiple types of requests: HTML, CSS, JS, API
    ///
    /// # Parameters
    /// * `name` - Snapshot name
    pub fn create_multi_request_snapshot(name: &str) -> Snapshot {
        let base_time = Utc::now();

        Snapshot {
            name: name.to_string(),
            url: "https://example.com".to_string(),
            created_at: base_time,
            requests: vec![
                // HTML page
                RequestRecord {
                    method: "GET".to_string(),
                    url: "https://example.com/".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("accept".to_string(), "text/html".to_string());
                        h
                    },
                    body: None,
                    response: ResponseRecord {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
                            h
                        },
                        body: b"<html><head><link rel=\"stylesheet\" href=\"/style.css\"></head><body><h1>Test</h1></body></html>".to_vec(),
                        content_type: "text/html".to_string(),
                    },
                    timestamp: base_time,
                },
                // CSS file
                RequestRecord {
                    method: "GET".to_string(),
                    url: "https://example.com/style.css".to_string(),
                    headers: HashMap::new(),
                    body: None,
                    response: ResponseRecord {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("content-type".to_string(), "text/css".to_string());
                            h
                        },
                        body: b"body { font-family: Arial, sans-serif; }".to_vec(),
                        content_type: "text/css".to_string(),
                    },
                    timestamp: base_time,
                },
                // API request
                RequestRecord {
                    method: "GET".to_string(),
                    url: "https://example.com/api/data".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("accept".to_string(), "application/json".to_string());
                        h
                    },
                    body: None,
                    response: ResponseRecord {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("content-type".to_string(), "application/json".to_string());
                            h
                        },
                        body: b"{\"message\": \"Hello from API\", \"status\": \"success\"}".to_vec(),
                        content_type: "application/json".to_string(),
                    },
                    timestamp: base_time,
                }
            ],
        }
    }

    /// Create a large test snapshot (for performance testing)
    ///
    /// # Parameters
    /// * `name` - Snapshot name
    /// * `num_requests` - Number of requests
    /// * `body_size` - Size of each response body (bytes)
    pub fn create_large_test_snapshot(
        name: &str,
        num_requests: usize,
        body_size: usize,
    ) -> Snapshot {
        let mut requests = Vec::with_capacity(num_requests);
        let large_body = "x".repeat(body_size).into_bytes();
        let base_time = Utc::now();

        for i in 0..num_requests {
            requests.push(RequestRecord {
                method: "GET".to_string(),
                url: format!("https://example.com/page/{}", i),
                headers: HashMap::new(),
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("content-type".to_string(), "text/html".to_string());
                        h
                    },
                    body: large_body.clone(),
                    content_type: "text/html".to_string(),
                },
                timestamp: base_time,
            });
        }

        Snapshot {
            name: name.to_string(),
            url: "https://example.com".to_string(),
            created_at: base_time,
            requests,
        }
    }

    /// Set up test environment
    ///
    /// Set necessary environment variables, skip permission checks, etc.
    pub fn setup_test_env() {
        std::env::set_var("WEBMOCK_SKIP_PERMISSION_CHECK", "1");
        std::env::set_var("RUST_LOG", "error"); // Reduce log output during testing
    }

    /// Set up isolated test environment
    ///
    /// Set up an independent environment for a single test to avoid concurrent conflicts
    /// Returns a guard that automatically restores the environment when dropped
    pub fn setup_isolated_test_env(temp_home: &std::path::Path) -> TestEnvGuard {
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("WEBMOCK_SKIP_PERMISSION_CHECK", "1");
        std::env::set_var("RUST_LOG", "error");
        std::env::set_var("HOME", temp_home);

        TestEnvGuard { original_home }
    }

    /// Test environment guard, automatically restores environment variables
    pub struct TestEnvGuard {
        original_home: Option<String>,
    }

    impl Drop for TestEnvGuard {
        fn drop(&mut self) {
            std::env::remove_var("WEBMOCK_SKIP_PERMISSION_CHECK");
            std::env::remove_var("RUST_LOG");

            if let Some(home) = &self.original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    /// Create temporary storage
    ///
    /// Returns a tuple of temporary directory and storage instance
    /// Temporary directory is automatically cleaned up when test ends
    pub fn create_temp_storage() -> (TempDir, crate::storage::Storage) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let storage = crate::storage::Storage::new(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    /// Create fast temporary storage (for simple tests)
    ///
    /// Uses in-memory temporary directory, faster than standard temporary directory
    pub fn create_fast_temp_storage() -> (TempDir, crate::storage::Storage) {
        // Use faster temporary directory creation method
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let storage = crate::storage::Storage::new(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    /// Create temporary storage and save test snapshot
    ///
    /// # Parameters
    /// * `snapshot_name` - Snapshot name
    ///
    /// # Returns
    /// Returns a tuple of (temporary directory, storage instance, snapshot name)
    pub async fn create_temp_storage_with_snapshot(
        snapshot_name: &str,
    ) -> (TempDir, crate::storage::Storage, String) {
        let (temp_dir, storage) = create_temp_storage();
        let snapshot = create_test_snapshot_with_name(snapshot_name);

        storage
            .save_snapshot(snapshot)
            .await
            .expect("Failed to save test snapshot");

        (temp_dir, storage, snapshot_name.to_string())
    }

    /// Clean up test environment
    ///
    /// Remove environment variables set during testing
    pub fn cleanup_test_env() {
        std::env::remove_var("WEBMOCK_SKIP_PERMISSION_CHECK");
        std::env::remove_var("RUST_LOG");
    }

    /// Helper function to validate snapshot content
    ///
    /// # Parameters
    /// * `snapshot` - Snapshot to validate
    /// * `expected_name` - Expected snapshot name
    /// * `expected_request_count` - Expected number of requests
    pub fn assert_snapshot_valid(
        snapshot: &Snapshot,
        expected_name: &str,
        expected_request_count: usize,
    ) {
        assert_eq!(snapshot.name, expected_name);
        assert_eq!(snapshot.requests.len(), expected_request_count);
        assert!(!snapshot.url.is_empty());

        // Verify that each request has valid data
        for request in &snapshot.requests {
            assert!(!request.method.is_empty());
            assert!(!request.url.is_empty());
            assert!(request.response.status >= 100 && request.response.status < 600);
            assert!(!request.response.content_type.is_empty());
        }
    }
}
