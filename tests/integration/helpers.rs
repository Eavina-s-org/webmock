//! Helper functions for integration tests

use std::collections::HashMap;
use std::net::TcpListener;
use tempfile::TempDir;

use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    storage::{Snapshot, Storage},
};

/// Helper function to find an available port for testing
pub fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener
        .local_addr()
        .expect("Failed to get local addr")
        .port();
    drop(listener);
    port
}

/// Helper function to create a test storage with sample snapshots
pub async fn create_test_storage_with_samples() -> (TempDir, Storage) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Ensure snapshots directory exists
    storage
        .ensure_snapshots_dir()
        .expect("Failed to create snapshots dir");

    // Create sample HTML snapshot
    let html_snapshot = create_sample_html_snapshot().await;
    storage
        .save_snapshot(html_snapshot)
        .await
        .expect("Failed to save HTML snapshot");

    // Create sample API snapshot
    let api_snapshot = create_sample_api_snapshot().await;
    storage
        .save_snapshot(api_snapshot)
        .await
        .expect("Failed to save API snapshot");

    (temp_dir, storage)
}

/// Create a sample HTML snapshot with various resource types
pub async fn create_sample_html_snapshot() -> Snapshot {
    let mut requests = Vec::new();

    // Main HTML page
    let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <h1>Hello from WebMock!</h1>
    <img src="/logo.png" alt="Logo">
    <script src="/app.js"></script>
</body>
</html>
"#;

    let mut html_headers = HashMap::new();
    html_headers.insert(
        "content-type".to_string(),
        "text/html; charset=utf-8".to_string(),
    );
    html_headers.insert("content-length".to_string(), html_content.len().to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: html_headers,
            body: html_content.as_bytes().to_vec(),
            content_type: "text/html".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // CSS file
    let css_content = "body { font-family: Arial, sans-serif; }";
    let mut css_headers = HashMap::new();
    css_headers.insert("content-type".to_string(), "text/css".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/styles.css".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: css_headers,
            body: css_content.as_bytes().to_vec(),
            content_type: "text/css".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // JavaScript file
    let js_content = "console.log('Hello from WebMock!');";
    let mut js_headers = HashMap::new();
    js_headers.insert(
        "content-type".to_string(),
        "application/javascript".to_string(),
    );

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/app.js".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: js_headers,
            body: js_content.as_bytes().to_vec(),
            content_type: "application/javascript".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // Image file (mock PNG)
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let mut png_headers = HashMap::new();
    png_headers.insert("content-type".to_string(), "image/png".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://example.com/logo.png".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: png_headers,
            body: png_data,
            content_type: "image/png".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    Snapshot {
        name: "html-test-site".to_string(),
        url: "https://example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    }
}

/// Create a sample API snapshot with JSON responses
pub async fn create_sample_api_snapshot() -> Snapshot {
    let mut requests = Vec::new();

    // GET API endpoint
    let api_response =
        r#"{"users": [{"id": 1, "name": "John Doe"}, {"id": 2, "name": "Jane Smith"}]}"#;
    let mut api_headers = HashMap::new();
    api_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: api_headers.clone(),
            body: api_response.as_bytes().to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // POST API endpoint
    let post_response = r#"{"id": 3, "name": "New User", "created": true}"#;
    let mut post_request_headers = HashMap::new();
    post_request_headers.insert("content-type".to_string(), "application/json".to_string());

    requests.push(RequestRecord {
        method: "POST".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: post_request_headers,
        body: Some(r#"{"name": "New User"}"#.as_bytes().to_vec()),
        response: ResponseRecord {
            status: 201,
            headers: api_headers,
            body: post_response.as_bytes().to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    Snapshot {
        name: "api-test-endpoints".to_string(),
        url: "https://api.example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    }
}
