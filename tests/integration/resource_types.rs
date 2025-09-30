//! Resource type handling integration tests

use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

use super::helpers::*;
use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    serve::MockServer,
    storage::{Snapshot, Storage},
};

#[tokio::test]
async fn test_various_content_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    let mut requests = Vec::new();

    // Test different content types
    let test_cases = [
        (
            "text/html",
            b"<html><body>HTML content</body></html>".to_vec(),
        ),
        ("application/json", b"{\"key\": \"value\"}".to_vec()),
        ("text/css", b"body { color: red; }".to_vec()),
        ("application/javascript", b"console.log('test');".to_vec()),
        ("image/jpeg", vec![0xFF, 0xD8, 0xFF, 0xE0]), // JPEG header
        ("application/pdf", b"%PDF-1.4".to_vec()),
        ("text/xml", b"<?xml version=\"1.0\"?><root></root>".to_vec()),
    ];

    for (i, (content_type, body)) in test_cases.iter().enumerate() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), content_type.to_string());

        requests.push(RequestRecord {
            method: "GET".to_string(),
            url: format!("https://example.com/file{}", i),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers,
                body: body.clone(),
                content_type: content_type.to_string(),
            },
            timestamp: chrono::Utc::now(),
        });
    }

    let content_types_snapshot = Snapshot {
        name: "content-types-test".to_string(),
        url: "https://example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    };

    storage
        .save_snapshot(content_types_snapshot)
        .await
        .expect("Failed to save content types snapshot");
    let loaded_snapshot = storage
        .load_snapshot("content-types-test")
        .await
        .expect("Failed to load content types snapshot");

    // Verify all content types are preserved
    assert_eq!(loaded_snapshot.requests.len(), test_cases.len());

    for (request, (expected_content_type, expected_body)) in
        loaded_snapshot.requests.iter().zip(test_cases.iter())
    {
        assert_eq!(request.response.content_type, *expected_content_type);
        assert_eq!(request.response.body, *expected_body);
    }

    // Test serving different content types
    let mock_server = MockServer::new(loaded_snapshot);
    let port = find_available_port();

    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Test each content type
    for (i, (expected_content_type, expected_body)) in test_cases.iter().enumerate() {
        let response = client
            .get(format!("http://localhost:{}/file{}", port, i))
            .send()
            .await
            .expect("Failed to request file");

        assert_eq!(response.status(), 200);

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.starts_with(expected_content_type));

        let body = response.bytes().await.expect("Failed to get response body");
        assert_eq!(body.as_ref(), expected_body.as_slice());
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_http_methods_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    let mut requests = Vec::new();
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in &methods {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let response_body = format!("{{\"method\": \"{}\", \"success\": true}}", method);

        requests.push(RequestRecord {
            method: method.to_string(),
            url: "https://api.example.com/endpoint".to_string(),
            headers: HashMap::new(),
            body: if *method != "GET" {
                Some(b"{\"data\": \"test\"}".to_vec())
            } else {
                None
            },
            response: ResponseRecord {
                status: if *method == "POST" { 201 } else { 200 },
                headers,
                body: response_body.as_bytes().to_vec(),
                content_type: "application/json".to_string(),
            },
            timestamp: chrono::Utc::now(),
        });
    }

    let methods_snapshot = Snapshot {
        name: "http-methods-test".to_string(),
        url: "https://api.example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    };

    storage
        .save_snapshot(methods_snapshot)
        .await
        .expect("Failed to save methods snapshot");
    let loaded_snapshot = storage
        .load_snapshot("http-methods-test")
        .await
        .expect("Failed to load methods snapshot");

    // Test serving different HTTP methods
    let mock_server = MockServer::new(loaded_snapshot);
    let port = find_available_port();

    let server_handle = tokio::spawn(async move { mock_server.start(port).await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Test GET
    let get_response = client
        .get(format!("http://localhost:{}/endpoint", port))
        .send()
        .await
        .expect("Failed to GET");
    assert_eq!(get_response.status(), 200);

    // Test POST
    let post_response = client
        .post(format!("http://localhost:{}/endpoint", port))
        .json(&serde_json::json!({"data": "test"}))
        .send()
        .await
        .expect("Failed to POST");
    assert_eq!(post_response.status(), 201);

    // Test PUT
    let put_response = client
        .put(format!("http://localhost:{}/endpoint", port))
        .json(&serde_json::json!({"data": "test"}))
        .send()
        .await
        .expect("Failed to PUT");
    assert_eq!(put_response.status(), 200);

    server_handle.abort();
}
