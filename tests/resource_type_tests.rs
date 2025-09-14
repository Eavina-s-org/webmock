//! Resource type and content handling tests
//! Tests various content types, HTTP methods, and resource handling

use std::collections::HashMap;
use std::net::TcpListener;
use std::time::Duration;
use tempfile::TempDir;

use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
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

/// Create snapshot with various content types
async fn create_content_types_snapshot() -> Snapshot {
    let mut requests = Vec::new();

    // Test cases: (path, content_type, body, expected_content)
    let test_cases = vec![
        (
            "/html",
            "text/html",
            b"<html><body><h1>HTML Content</h1></body></html>".to_vec(),
        ),
        (
            "/json",
            "application/json",
            b"{\"message\": \"Hello JSON\", \"status\": \"success\"}".to_vec(),
        ),
        (
            "/css",
            "text/css",
            b"body { background: #fff; color: #333; }".to_vec(),
        ),
        (
            "/js",
            "application/javascript",
            b"console.log('Hello JavaScript');".to_vec(),
        ),
        (
            "/xml",
            "text/xml",
            b"<?xml version=\"1.0\"?><root><message>Hello XML</message></root>".to_vec(),
        ),
        ("/plain", "text/plain", b"Hello plain text content".to_vec()),
        (
            "/jpeg",
            "image/jpeg",
            vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46],
        ), // JPEG header
        (
            "/png",
            "image/png",
            vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        ), // PNG header
        (
            "/pdf",
            "application/pdf",
            b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj".to_vec(),
        ),
        (
            "/binary",
            "application/octet-stream",
            vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD],
        ),
    ];

    for (path, content_type, body) in test_cases {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), content_type.to_string());
        headers.insert("content-length".to_string(), body.len().to_string());

        requests.push(RequestRecord {
            method: "GET".to_string(),
            url: format!("https://example.com{}", path),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers,
                body,
                content_type: content_type.to_string(),
            },
            timestamp: chrono::Utc::now(),
        });
    }

    Snapshot {
        name: "content-types-test".to_string(),
        url: "https://example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    }
}

/// Create snapshot with various HTTP methods
async fn create_http_methods_snapshot() -> Snapshot {
    let mut requests = Vec::new();

    // GET request
    let mut get_headers = HashMap::new();
    get_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: get_headers,
            body: b"{\"users\": [{\"id\": 1, \"name\": \"John\"}]}".to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // POST request
    let mut post_headers = HashMap::new();
    post_headers.insert("content-type".to_string(), "application/json".to_string());
    post_headers.insert("location".to_string(), "/users/2".to_string());

    let mut post_request_headers = HashMap::new();
    post_request_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "POST".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: post_request_headers,
        body: Some(b"{\"name\": \"Jane\", \"email\": \"jane@example.com\"}".to_vec()),
        response: ResponseRecord {
            status: 201,
            headers: post_headers,
            body: b"{\"id\": 2, \"name\": \"Jane\", \"email\": \"jane@example.com\"}".to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // PUT request
    let mut put_headers = HashMap::new();
    put_headers.insert("content-type".to_string(), "application/json".to_string());

    let mut put_request_headers = HashMap::new();
    put_request_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "PUT".to_string(),
        url: "https://api.example.com/users/1".to_string(),
        headers: put_request_headers,
        body: Some(
            b"{\"name\": \"John Updated\", \"email\": \"john.updated@example.com\"}".to_vec(),
        ),
        response: ResponseRecord {
            status: 200,
            headers: put_headers,
            body:
                b"{\"id\": 1, \"name\": \"John Updated\", \"email\": \"john.updated@example.com\"}"
                    .to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // DELETE request
    let mut delete_headers = HashMap::new();
    delete_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "DELETE".to_string(),
        url: "https://api.example.com/users/1".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 204,
            headers: delete_headers,
            body: Vec::new(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // PATCH request
    let mut patch_headers = HashMap::new();
    patch_headers.insert("content-type".to_string(), "application/json".to_string());

    let mut patch_request_headers = HashMap::new();
    patch_request_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "PATCH".to_string(),
        url: "https://api.example.com/users/2".to_string(),
        headers: patch_request_headers,
        body: Some(b"{\"email\": \"jane.updated@example.com\"}".to_vec()),
        response: ResponseRecord {
            status: 200,
            headers: patch_headers,
            body: b"{\"id\": 2, \"name\": \"Jane\", \"email\": \"jane.updated@example.com\"}"
                .to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    Snapshot {
        name: "http-methods-test".to_string(),
        url: "https://api.example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    }
}

#[cfg(test)]
mod content_type_tests {
    use super::*;

    #[tokio::test]
    async fn test_various_content_types_storage_and_retrieval() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create and save snapshot with various content types
        let snapshot = create_content_types_snapshot().await;
        storage
            .save_snapshot(snapshot)
            .await
            .expect("Failed to save snapshot");

        // Load snapshot back
        let loaded_snapshot = storage
            .load_snapshot("content-types-test")
            .await
            .expect("Failed to load snapshot");

        // Verify all content types are preserved
        assert_eq!(loaded_snapshot.requests.len(), 10);

        let expected_types = [
            "text/html",
            "application/json",
            "text/css",
            "application/javascript",
            "text/xml",
            "text/plain",
            "image/jpeg",
            "image/png",
            "application/pdf",
            "application/octet-stream",
        ];

        for (request, expected_type) in loaded_snapshot.requests.iter().zip(expected_types.iter()) {
            assert_eq!(request.response.content_type, *expected_type);
        }
    }

    #[tokio::test]
    async fn test_content_types_served_correctly() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        let snapshot = create_content_types_snapshot().await;
        storage
            .save_snapshot(snapshot.clone())
            .await
            .expect("Failed to save snapshot");

        // Start mock server
        let mock_server = MockServer::new(snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Test HTML content
        let html_response = client
            .get(format!("http://localhost:{}/html", port))
            .send()
            .await
            .expect("Failed to request HTML");
        assert_eq!(html_response.status(), 200);
        assert!(html_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("text/html"));
        let html_body = html_response.text().await.expect("Failed to get HTML body");
        assert!(html_body.contains("<h1>HTML Content</h1>"));

        // Test JSON content
        let json_response = client
            .get(format!("http://localhost:{}/json", port))
            .send()
            .await
            .expect("Failed to request JSON");
        assert_eq!(json_response.status(), 200);
        assert!(json_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("application/json"));
        let json_data: serde_json::Value =
            json_response.json().await.expect("Failed to parse JSON");
        assert_eq!(json_data["message"], "Hello JSON");
        assert_eq!(json_data["status"], "success");

        // Test CSS content
        let css_response = client
            .get(format!("http://localhost:{}/css", port))
            .send()
            .await
            .expect("Failed to request CSS");
        assert_eq!(css_response.status(), 200);
        assert!(css_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("text/css"));
        let css_body = css_response.text().await.expect("Failed to get CSS body");
        assert!(css_body.contains("background: #fff"));

        // Test JavaScript content
        let js_response = client
            .get(format!("http://localhost:{}/js", port))
            .send()
            .await
            .expect("Failed to request JS");
        assert_eq!(js_response.status(), 200);
        assert!(js_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("application/javascript"));
        let js_body = js_response.text().await.expect("Failed to get JS body");
        assert!(js_body.contains("console.log"));

        // Test binary content (PNG)
        let png_response = client
            .get(format!("http://localhost:{}/png", port))
            .send()
            .await
            .expect("Failed to request PNG");
        assert_eq!(png_response.status(), 200);
        assert!(png_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("image/png"));
        let png_body = png_response.bytes().await.expect("Failed to get PNG body");
        assert_eq!(
            &png_body[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        ); // PNG signature

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_large_content_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create snapshot with large content
        let large_content = "A".repeat(1024 * 1024); // 1MB of 'A's

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let request = RequestRecord {
            method: "GET".to_string(),
            url: "https://example.com/large".to_string(),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers,
                body: large_content.as_bytes().to_vec(),
                content_type: "text/plain".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let large_snapshot = Snapshot {
            name: "large-content-test".to_string(),
            url: "https://example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: vec![request],
        };

        storage
            .save_snapshot(large_snapshot.clone())
            .await
            .expect("Failed to save large snapshot");

        // Test serving large content
        let mock_server = MockServer::new(large_snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://localhost:{}/large", port))
            .send()
            .await
            .expect("Failed to request large content");

        assert_eq!(response.status(), 200);
        let body = response
            .text()
            .await
            .expect("Failed to get large content body");
        assert_eq!(body.len(), 1024 * 1024);
        assert!(body.chars().all(|c| c == 'A'));

        server_handle.abort();
    }
}

#[cfg(test)]
mod http_method_tests {
    use super::*;

    #[tokio::test]
    async fn test_various_http_methods_storage() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        let snapshot = create_http_methods_snapshot().await;
        storage
            .save_snapshot(snapshot)
            .await
            .expect("Failed to save snapshot");

        let loaded_snapshot = storage
            .load_snapshot("http-methods-test")
            .await
            .expect("Failed to load snapshot");

        // Verify all HTTP methods are preserved
        assert_eq!(loaded_snapshot.requests.len(), 5);

        let methods: Vec<&str> = loaded_snapshot
            .requests
            .iter()
            .map(|r| r.method.as_str())
            .collect();
        assert!(methods.contains(&"GET"));
        assert!(methods.contains(&"POST"));
        assert!(methods.contains(&"PUT"));
        assert!(methods.contains(&"DELETE"));
        assert!(methods.contains(&"PATCH"));

        // Verify request bodies are preserved for methods that have them
        let post_request = loaded_snapshot
            .requests
            .iter()
            .find(|r| r.method == "POST")
            .unwrap();
        assert!(post_request.body.is_some());
        let post_body = String::from_utf8(post_request.body.as_ref().unwrap().clone()).unwrap();
        assert!(post_body.contains("Jane"));

        let put_request = loaded_snapshot
            .requests
            .iter()
            .find(|r| r.method == "PUT")
            .unwrap();
        assert!(put_request.body.is_some());
        let put_body = String::from_utf8(put_request.body.as_ref().unwrap().clone()).unwrap();
        assert!(put_body.contains("John Updated"));
    }

    #[tokio::test]
    async fn test_http_methods_served_correctly() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        let snapshot = create_http_methods_snapshot().await;
        storage
            .save_snapshot(snapshot.clone())
            .await
            .expect("Failed to save snapshot");

        let mock_server = MockServer::new(snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let client = reqwest::Client::new();

        // Test GET request
        let get_response = client
            .get(format!("http://localhost:{}/users", port))
            .send()
            .await
            .expect("Failed to GET");
        assert_eq!(get_response.status(), 200);
        let get_data: serde_json::Value =
            get_response.json().await.expect("Failed to parse GET JSON");
        assert!(get_data["users"].is_array());

        // Test POST request
        let post_response = client
            .post(format!("http://localhost:{}/users", port))
            .json(&serde_json::json!({"name": "Jane", "email": "jane@example.com"}))
            .send()
            .await
            .expect("Failed to POST");
        assert_eq!(post_response.status(), 201);
        let post_data: serde_json::Value = post_response
            .json()
            .await
            .expect("Failed to parse POST JSON");
        assert_eq!(post_data["name"], "Jane");

        // Test PUT request
        let put_response = client
            .put(format!("http://localhost:{}/users/1", port))
            .json(&serde_json::json!({"name": "John Updated", "email": "john.updated@example.com"}))
            .send()
            .await
            .expect("Failed to PUT");
        assert_eq!(put_response.status(), 200);
        let put_data: serde_json::Value =
            put_response.json().await.expect("Failed to parse PUT JSON");
        assert_eq!(put_data["name"], "John Updated");

        // Test DELETE request
        let delete_response = client
            .delete(format!("http://localhost:{}/users/1", port))
            .send()
            .await
            .expect("Failed to DELETE");
        assert_eq!(delete_response.status(), 204);

        // Test PATCH request
        let patch_response = client
            .patch(format!("http://localhost:{}/users/2", port))
            .json(&serde_json::json!({"email": "jane.updated@example.com"}))
            .send()
            .await
            .expect("Failed to PATCH");
        assert_eq!(patch_response.status(), 200);
        let patch_data: serde_json::Value = patch_response
            .json()
            .await
            .expect("Failed to parse PATCH JSON");
        assert_eq!(patch_data["email"], "jane.updated@example.com");

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_request_headers_preservation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create snapshot with custom headers
        let mut request_headers = HashMap::new();
        request_headers.insert("authorization".to_string(), "Bearer token123".to_string());
        request_headers.insert("x-custom-header".to_string(), "custom-value".to_string());
        request_headers.insert("user-agent".to_string(), "WebMock-Test/1.0".to_string());

        let mut response_headers = HashMap::new();
        response_headers.insert("content-type".to_string(), "application/json".to_string());
        response_headers.insert("x-response-id".to_string(), "resp-123".to_string());
        response_headers.insert("cache-control".to_string(), "no-cache".to_string());

        let request = RequestRecord {
            method: "GET".to_string(),
            url: "https://api.example.com/protected".to_string(),
            headers: request_headers.clone(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers: response_headers.clone(),
                body: b"{\"message\": \"Protected resource\"}".to_vec(),
                content_type: "application/json".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let headers_snapshot = Snapshot {
            name: "headers-test".to_string(),
            url: "https://api.example.com/".to_string(),
            created_at: chrono::Utc::now(),
            requests: vec![request],
        };

        storage
            .save_snapshot(headers_snapshot.clone())
            .await
            .expect("Failed to save headers snapshot");

        // Load and verify headers are preserved
        let loaded_snapshot = storage
            .load_snapshot("headers-test")
            .await
            .expect("Failed to load headers snapshot");
        let loaded_request = &loaded_snapshot.requests[0];

        // Verify request headers
        assert_eq!(
            loaded_request.headers.get("authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            loaded_request.headers.get("x-custom-header"),
            Some(&"custom-value".to_string())
        );
        assert_eq!(
            loaded_request.headers.get("user-agent"),
            Some(&"WebMock-Test/1.0".to_string())
        );

        // Verify response headers
        assert_eq!(
            loaded_request.response.headers.get("x-response-id"),
            Some(&"resp-123".to_string())
        );
        assert_eq!(
            loaded_request.response.headers.get("cache-control"),
            Some(&"no-cache".to_string())
        );

        // Test serving with headers
        let mock_server = MockServer::new(loaded_snapshot);
        let port = find_available_port();

        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://localhost:{}/protected", port))
            .header("authorization", "Bearer token123")
            .header("x-custom-header", "custom-value")
            .header("user-agent", "WebMock-Test/1.0")
            .send()
            .await
            .expect("Failed to request with headers");

        assert_eq!(response.status(), 200);

        // Verify response headers are served
        assert_eq!(
            response
                .headers()
                .get("x-response-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "resp-123"
        );
        assert_eq!(
            response
                .headers()
                .get("cache-control")
                .unwrap()
                .to_str()
                .unwrap(),
            "no-cache"
        );

        server_handle.abort();
    }
}
