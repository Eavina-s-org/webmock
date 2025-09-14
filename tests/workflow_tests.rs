//! Core session integration tests
//! Tests the capture → list → serve session end-to-end

use std::collections::HashMap;
use std::net::TcpListener;
use std::time::Duration;
use tempfile::TempDir;

use webmock_cli::{
    capture::proxy::records::{RequestRecord, ResponseRecord},
    serve::MockServer,
    storage::{Snapshot, Storage},
};

/// Helper function to find an available port for testing
fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener
        .local_addr()
        .expect("Failed to get local addr")
        .port();
    drop(listener);
    port
}

/// Create a comprehensive test snapshot with multiple resource types
async fn create_comprehensive_test_snapshot() -> Snapshot {
    let mut requests = Vec::new();

    // Main HTML page
    let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>WebMock Test Site</title>
    <link rel="stylesheet" href="/assets/styles.css">
    <link rel="icon" href="/favicon.ico">
</head>
<body>
    <header>
        <h1>Welcome to WebMock Test Site</h1>
        <nav>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
        </nav>
    </header>
    <main>
        <img src="/images/hero.jpg" alt="Hero Image">
        <p>This is a test page for WebMock CLI integration testing.</p>
        <div id="api-data"></div>
    </main>
    <script src="/js/app.js"></script>
</body>
</html>
"#;

    let mut html_headers = HashMap::new();
    html_headers.insert(
        "content-type".to_string(),
        "text/html; charset=utf-8".to_string(),
    );
    html_headers.insert("cache-control".to_string(), "max-age=3600".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://testsite.example.com/".to_string(),
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

    // CSS stylesheet
    let css_content = r#"
body {
    font-family: 'Arial', sans-serif;
    margin: 0;
    padding: 20px;
    background-color: #f5f5f5;
}

header {
    background: #333;
    color: white;
    padding: 1rem;
    margin-bottom: 2rem;
}

nav a {
    color: white;
    text-decoration: none;
    margin-right: 1rem;
}

main {
    max-width: 800px;
    margin: 0 auto;
}

img {
    max-width: 100%;
    height: auto;
}
"#;

    let mut css_headers = HashMap::new();
    css_headers.insert("content-type".to_string(), "text/css".to_string());
    css_headers.insert("cache-control".to_string(), "max-age=86400".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://testsite.example.com/assets/styles.css".to_string(),
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
    let js_content = r#"
document.addEventListener('DOMContentLoaded', function() {
    console.log('WebMock test site loaded');
    
    // Simulate API call
    fetch('/api/data')
        .then(response => response.json())
        .then(data => {
            const container = document.getElementById('api-data');
            container.innerHTML = '<h3>API Data:</h3><pre>' + JSON.stringify(data, null, 2) + '</pre>';
        })
        .catch(error => {
            console.error('API call failed:', error);
        });
});
"#;

    let mut js_headers = HashMap::new();
    js_headers.insert(
        "content-type".to_string(),
        "application/javascript".to_string(),
    );
    js_headers.insert("cache-control".to_string(), "max-age=86400".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://testsite.example.com/js/app.js".to_string(),
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

    // API endpoint
    let api_response = r#"{
    "status": "success",
    "data": {
        "users": [
            {"id": 1, "name": "Alice Johnson", "role": "admin"},
            {"id": 2, "name": "Bob Smith", "role": "user"},
            {"id": 3, "name": "Carol Davis", "role": "moderator"}
        ],
        "metadata": {
            "total": 3,
            "page": 1,
            "timestamp": "2024-01-15T10:30:00Z"
        }
    }
}"#;

    let mut api_headers = HashMap::new();
    api_headers.insert("content-type".to_string(), "application/json".to_string());
    api_headers.insert("access-control-allow-origin".to_string(), "*".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://testsite.example.com/api/data".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: api_headers,
            body: api_response.as_bytes().to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    // Favicon
    let favicon_data = vec![
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x10, 0x10, 0x00, 0x00, 0x01, 0x00, 0x20, 0x00, 0x68,
        0x04, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, // ICO header
    ];

    let mut favicon_headers = HashMap::new();
    favicon_headers.insert("content-type".to_string(), "image/x-icon".to_string());
    favicon_headers.insert("cache-control".to_string(), "max-age=604800".to_string());

    requests.push(RequestRecord {
        method: "GET".to_string(),
        url: "https://testsite.example.com/favicon.ico".to_string(),
        headers: HashMap::new(),
        body: None,
        response: ResponseRecord {
            status: 200,
            headers: favicon_headers,
            body: favicon_data,
            content_type: "image/x-icon".to_string(),
        },
        timestamp: chrono::Utc::now(),
    });

    Snapshot {
        name: "comprehensive-test-site".to_string(),
        url: "https://testsite.example.com/".to_string(),
        created_at: chrono::Utc::now(),
        requests,
    }
}

#[cfg(test)]
mod workflow_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_capture_list_serve_workflow() {
        // Setup test storage
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Step 1: Simulate capture by creating a comprehensive snapshot
        let test_snapshot = create_comprehensive_test_snapshot().await;
        let snapshot_name = test_snapshot.name.clone();

        storage
            .save_snapshot(test_snapshot)
            .await
            .expect("Failed to save test snapshot");

        // Step 2: Test list functionality
        let snapshots = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].name, snapshot_name);
        assert_eq!(snapshots[0].url, "https://testsite.example.com/");

        // Step 3: Test serve functionality
        let loaded_snapshot = storage
            .load_snapshot(&snapshot_name)
            .await
            .expect("Failed to load snapshot");
        assert_eq!(loaded_snapshot.requests.len(), 5); // HTML, CSS, JS, API, favicon
        let mock_server = MockServer::new(loaded_snapshot);
        let port = find_available_port();

        // Start server in background
        let server_handle = tokio::spawn(async move { mock_server.start(port).await });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Step 4: Test all captured resources
        let client = reqwest::Client::new();

        // Test main HTML page
        let html_response = client
            .get(format!("http://localhost:{}/", port))
            .send()
            .await
            .expect("Failed to request main page");
        assert_eq!(html_response.status(), 200);
        let html_body = html_response.text().await.expect("Failed to get HTML body");
        assert!(html_body.contains("Welcome to WebMock Test Site"));
        assert!(html_body.contains("/assets/styles.css"));
        assert!(html_body.contains("/js/app.js"));

        // Test CSS file
        let css_response = client
            .get(format!("http://localhost:{}/assets/styles.css", port))
            .send()
            .await
            .expect("Failed to request CSS");
        assert_eq!(css_response.status(), 200);
        let css_body = css_response.text().await.expect("Failed to get CSS body");
        assert!(css_body.contains("font-family: 'Arial'"));
        assert!(css_body.contains("background-color: #f5f5f5"));

        // Test JavaScript file
        let js_response = client
            .get(format!("http://localhost:{}/js/app.js", port))
            .send()
            .await
            .expect("Failed to request JS");
        assert_eq!(js_response.status(), 200);
        let js_body = js_response.text().await.expect("Failed to get JS body");
        assert!(js_body.contains("WebMock test site loaded"));
        assert!(js_body.contains("fetch('/api/data')"));

        // Test API endpoint
        let api_response = client
            .get(format!("http://localhost:{}/api/data", port))
            .send()
            .await
            .expect("Failed to request API");
        assert_eq!(api_response.status(), 200);
        let api_data: serde_json::Value =
            api_response.json().await.expect("Failed to parse API JSON");
        assert_eq!(api_data["status"], "success");
        assert!(api_data["data"]["users"].is_array());
        assert_eq!(api_data["data"]["users"].as_array().unwrap().len(), 3);

        // Test favicon
        let favicon_response = client
            .get(format!("http://localhost:{}/favicon.ico", port))
            .send()
            .await
            .expect("Failed to request favicon");
        assert_eq!(favicon_response.status(), 200);

        // Test 404 for non-existent resource
        let not_found_response = client
            .get(format!("http://localhost:{}/nonexistent.html", port))
            .send()
            .await
            .expect("Failed to request non-existent resource");
        assert_eq!(not_found_response.status(), 404);

        // Cleanup
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_workflow_with_multiple_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Storage::new(temp_dir.path().to_path_buf());
        storage
            .ensure_snapshots_dir()
            .expect("Failed to create snapshots dir");

        // Create multiple test snapshots
        let snapshot1 = create_comprehensive_test_snapshot().await;

        let mut snapshot2 = create_comprehensive_test_snapshot().await;
        snapshot2.name = "second-test-site".to_string();
        snapshot2.url = "https://second.example.com/".to_string();

        let mut snapshot3 = create_comprehensive_test_snapshot().await;
        snapshot3.name = "third-test-site".to_string();
        snapshot3.url = "https://third.example.com/".to_string();

        // Save all snapshots
        storage
            .save_snapshot(snapshot1)
            .await
            .expect("Failed to save snapshot 1");
        storage
            .save_snapshot(snapshot2)
            .await
            .expect("Failed to save snapshot 2");
        storage
            .save_snapshot(snapshot3)
            .await
            .expect("Failed to save snapshot 3");

        // Test listing multiple snapshots
        let snapshots = storage
            .list_snapshots()
            .await
            .expect("Failed to list snapshots");
        assert_eq!(snapshots.len(), 3);

        let snapshot_names: Vec<&str> = snapshots.iter().map(|s| s.name.as_str()).collect();
        assert!(snapshot_names.contains(&"comprehensive-test-site"));
        assert!(snapshot_names.contains(&"second-test-site"));
        assert!(snapshot_names.contains(&"third-test-site"));

        // Test serving different snapshots on different ports
        let ports: Vec<u16> = (0..3).map(|_| find_available_port()).collect();
        let mut server_handles = Vec::new();

        for (i, snapshot_name) in snapshot_names.iter().enumerate() {
            let loaded_snapshot = storage
                .load_snapshot(snapshot_name)
                .await
                .expect("Failed to load snapshot");
            let mock_server = MockServer::new(loaded_snapshot);
            let port = ports[i];

            let handle = tokio::spawn(async move { mock_server.start(port).await });
            server_handles.push(handle);
        }

        tokio::time::sleep(Duration::from_millis(300)).await;

        // Test that each server serves its content correctly
        let client = reqwest::Client::new();

        for port in &ports {
            let response = client
                .get(format!("http://localhost:{}/", port))
                .send()
                .await
                .expect("Failed to request from server");
            assert_eq!(response.status(), 200);

            let body = response.text().await.expect("Failed to get response body");
            assert!(body.contains("Welcome to WebMock Test Site"));
        }

        // Cleanup all servers
        for handle in server_handles {
            handle.abort();
        }
    }

    #[tokio::test]
    async fn test_workflow_persistence_across_restarts() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Phase 1: Create and save snapshot
        {
            let storage = Storage::new(temp_dir.path().to_path_buf());
            storage
                .ensure_snapshots_dir()
                .expect("Failed to create snapshots dir");

            let test_snapshot = create_comprehensive_test_snapshot().await;
            storage
                .save_snapshot(test_snapshot)
                .await
                .expect("Failed to save snapshot");
        }

        // Phase 2: Simulate restart by creating new storage instance
        {
            let storage = Storage::new(temp_dir.path().to_path_buf());

            // Should be able to list snapshots after restart
            let snapshots = storage
                .list_snapshots()
                .await
                .expect("Failed to list snapshots after restart");
            assert_eq!(snapshots.len(), 1);
            assert_eq!(snapshots[0].name, "comprehensive-test-site");

            // Should be able to load and serve snapshot after restart
            let loaded_snapshot = storage
                .load_snapshot("comprehensive-test-site")
                .await
                .expect("Failed to load snapshot after restart");
            assert_eq!(loaded_snapshot.requests.len(), 5);

            let mock_server = MockServer::new(loaded_snapshot);
            let port = find_available_port();

            let server_handle = tokio::spawn(async move { mock_server.start(port).await });

            tokio::time::sleep(Duration::from_millis(200)).await;

            // Test that server works after restart
            let client = reqwest::Client::new();
            let response = client
                .get(format!("http://localhost:{}/", port))
                .send()
                .await
                .expect("Failed to request after restart");
            assert_eq!(response.status(), 200);

            server_handle.abort();
        }
    }
}
