//! Example: Creating Test Snapshots Programmatically
//!
//! This example demonstrates how to create test snapshots programmatically
//! using the WebMock CLI storage API. This is useful for:
//! - Setting up test data for development
//! - Creating reproducible test scenarios
//! - Generating sample snapshots for demonstrations
//!
//! ## Usage
//!
//! ```bash
//! # Run this example
//! cargo run --example create_test_snapshot
//!
//! # Then serve the created snapshot
//! webmock serve test-snapshot --port 8080
//!
//! # Visit http://localhost:8080 to see the test page
//! ```

use chrono::Utc;
use std::collections::HashMap;
use webmock_cli::capture::proxy::records::response::ResponseRecord;
use webmock_cli::capture::proxy::RequestRecord;
use webmock_cli::storage::{Snapshot, Storage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Creating test snapshot...");

    // Create storage in home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let storage_path = home_dir.join(".webmock");
    let storage = Storage::new(storage_path);

    // Ensure storage directory exists
    storage.ensure_snapshots_dir()?;

    // Create a comprehensive test snapshot with multiple request types
    let test_snapshot = Snapshot {
        name: "test-snapshot".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests: vec![
            // Main HTML page
            RequestRecord {
                method: "GET".to_string(),
                url: "https://example.com/".to_string(),
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("user-agent".to_string(), "WebMock-CLI/1.0".to_string());
                    headers.insert("accept".to_string(), "text/html,application/xhtml+xml".to_string());
                    headers
                },
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
                        headers.insert("cache-control".to_string(), "public, max-age=3600".to_string());
                        headers
                    },
                    body: create_test_html().into_bytes(),
                    content_type: "text/html".to_string(),
                },
                timestamp: Utc::now(),
            },

            // API endpoint
            RequestRecord {
                method: "GET".to_string(),
                url: "https://example.com/api/data".to_string(),
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("accept".to_string(), "application/json".to_string());
                    headers
                },
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "application/json".to_string());
                        headers.insert("access-control-allow-origin".to_string(), "*".to_string());
                        headers
                    },
                    body: create_test_json().into_bytes(),
                    content_type: "application/json".to_string(),
                },
                timestamp: Utc::now(),
            },

            // CSS file
            RequestRecord {
                method: "GET".to_string(),
                url: "https://example.com/styles.css".to_string(),
                headers: HashMap::new(),
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "text/css".to_string());
                        headers.insert("cache-control".to_string(), "public, max-age=86400".to_string());
                        headers
                    },
                    body: create_test_css().into_bytes(),
                    content_type: "text/css".to_string(),
                },
                timestamp: Utc::now(),
            },

            // JavaScript file
            RequestRecord {
                method: "GET".to_string(),
                url: "https://example.com/app.js".to_string(),
                headers: HashMap::new(),
                body: None,
                response: ResponseRecord {
                    status: 200,
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "application/javascript".to_string());
                        headers.insert("cache-control".to_string(), "public, max-age=86400".to_string());
                        headers
                    },
                    body: create_test_js().into_bytes(),
                    content_type: "application/javascript".to_string(),
                },
                timestamp: Utc::now(),
            },

            // POST API request example
            RequestRecord {
                method: "POST".to_string(),
                url: "https://example.com/api/submit".to_string(),
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("content-type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(b"{\"name\": \"test\", \"email\": \"test@example.com\"}".to_vec()),
                response: ResponseRecord {
                    status: 201,
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("content-type".to_string(), "application/json".to_string());
                        headers.insert("location".to_string(), "/api/users/123".to_string());
                        headers
                    },
                    body: b"{\"id\": 123, \"status\": \"created\", \"message\": \"User created successfully\"}".to_vec(),
                    content_type: "application/json".to_string(),
                },
                timestamp: Utc::now(),
            }
        ],
    };

    // Save the snapshot
    storage.save_snapshot(test_snapshot).await?;

    println!("✅ Created test snapshot 'test-snapshot'");
    println!();
    println!("📋 Snapshot contains:");
    println!("   • Main HTML page (/)");
    println!("   • API endpoint (/api/data)");
    println!("   • CSS stylesheet (/styles.css)");
    println!("   • JavaScript file (/app.js)");
    println!("   • POST API example (/api/submit)");
    println!();
    println!("🚀 Next steps:");
    println!("   1. Start the mock server:");
    println!("      webmock serve test-snapshot --port 8080");
    println!();
    println!("   2. Visit the test page:");
    println!("      http://localhost:8080");
    println!();
    println!("   3. Test API endpoints:");
    println!("      curl http://localhost:8080/api/data");
    println!("      curl -X POST http://localhost:8080/api/submit \\");
    println!("           -H 'Content-Type: application/json' \\");
    println!("           -d '{{\"name\":\"test\",\"email\":\"test@example.com\"}}'");

    Ok(())
}

/// Create a test HTML page with embedded CSS and JavaScript references
fn create_test_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WebMock CLI Test Page</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <header>
        <h1>🎭 WebMock CLI Test Page</h1>
        <p>This is a test snapshot created programmatically</p>
    </header>
    
    <main>
        <section class="info">
            <h2>📋 What's This?</h2>
            <p>This page demonstrates WebMock CLI's ability to capture and serve complete web applications, including:</p>
            <ul>
                <li>HTML pages with proper structure</li>
                <li>CSS stylesheets for styling</li>
                <li>JavaScript for interactivity</li>
                <li>API endpoints for data</li>
            </ul>
        </section>

        <section class="demo">
            <h2>🚀 Interactive Demo</h2>
            <button id="loadData" onclick="loadApiData()">Load API Data</button>
            <div id="apiResult" class="result"></div>
        </section>

        <section class="features">
            <h2>✨ WebMock CLI Features</h2>
            <div class="feature-grid">
                <div class="feature">
                    <h3>📸 Complete Capture</h3>
                    <p>Captures all network requests including HTML, CSS, JS, images, and API calls</p>
                </div>
                <div class="feature">
                    <h3>🌐 Local Serving</h3>
                    <p>Serves captured content locally for offline development and testing</p>
                </div>
                <div class="feature">
                    <h3>🔧 Developer Friendly</h3>
                    <p>Simple CLI with helpful error messages and progress indicators</p>
                </div>
            </div>
        </section>
    </main>

    <footer>
        <p>Generated by WebMock CLI • <a href="/api/data">Test API</a></p>
    </footer>

    <script src="/app.js"></script>
</body>
</html>"#.to_string()
}

/// Create test CSS for styling the HTML page
fn create_test_css() -> String {
    r#"/* WebMock CLI Test Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
}

header {
    background: rgba(255, 255, 255, 0.95);
    padding: 2rem;
    text-align: center;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

header h1 {
    color: #4a5568;
    margin-bottom: 0.5rem;
    font-size: 2.5rem;
}

header p {
    color: #718096;
    font-size: 1.1rem;
}

main {
    max-width: 1200px;
    margin: 2rem auto;
    padding: 0 2rem;
}

section {
    background: white;
    margin: 2rem 0;
    padding: 2rem;
    border-radius: 10px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

h2 {
    color: #2d3748;
    margin-bottom: 1rem;
    font-size: 1.8rem;
}

h3 {
    color: #4a5568;
    margin-bottom: 0.5rem;
}

ul {
    margin-left: 2rem;
    margin-bottom: 1rem;
}

li {
    margin-bottom: 0.5rem;
}

.feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
    margin-top: 2rem;
}

.feature {
    padding: 1.5rem;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    transition: transform 0.2s, box-shadow 0.2s;
}

.feature:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

button {
    background: #667eea;
    color: white;
    border: none;
    padding: 1rem 2rem;
    border-radius: 5px;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
}

button:hover {
    background: #5a67d8;
}

.result {
    margin-top: 1rem;
    padding: 1rem;
    background: #f7fafc;
    border-radius: 5px;
    border-left: 4px solid #667eea;
    min-height: 50px;
}

footer {
    text-align: center;
    padding: 2rem;
    color: white;
    background: rgba(0, 0, 0, 0.2);
}

footer a {
    color: #90cdf4;
    text-decoration: none;
}

footer a:hover {
    text-decoration: underline;
}

@media (max-width: 768px) {
    header h1 {
        font-size: 2rem;
    }
    
    main {
        padding: 0 1rem;
    }
    
    section {
        padding: 1.5rem;
    }
    
    .feature-grid {
        grid-template-columns: 1fr;
    }
}"#
    .to_string()
}

/// Create test JavaScript for interactivity
fn create_test_js() -> String {
    r#"// WebMock CLI Test JavaScript

console.log('🎭 WebMock CLI Test Page Loaded');

// Function to load data from the API endpoint
async function loadApiData() {
    const button = document.getElementById('loadData');
    const resultDiv = document.getElementById('apiResult');
    
    // Show loading state
    button.disabled = true;
    button.textContent = 'Loading...';
    resultDiv.innerHTML = '<p>🔄 Loading API data...</p>';
    
    try {
        // Fetch data from the captured API endpoint
        const response = await fetch('/api/data');
        const data = await response.json();
        
        // Display the result
        resultDiv.innerHTML = `
            <h4>✅ API Response:</h4>
            <pre>${JSON.stringify(data, null, 2)}</pre>
            <p><strong>Status:</strong> ${response.status}</p>
            <p><strong>Content-Type:</strong> ${response.headers.get('content-type')}</p>
        `;
        
        console.log('API data loaded:', data);
        
    } catch (error) {
        resultDiv.innerHTML = `
            <h4>❌ Error:</h4>
            <p>${error.message}</p>
        `;
        console.error('Failed to load API data:', error);
    } finally {
        // Reset button
        button.disabled = false;
        button.textContent = 'Load API Data';
    }
}

// Add some interactive features when the page loads
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Page ready for interaction');
    
    // Add click animations to feature cards
    const features = document.querySelectorAll('.feature');
    features.forEach(feature => {
        feature.addEventListener('click', function() {
            this.style.transform = 'scale(0.98)';
            setTimeout(() => {
                this.style.transform = '';
            }, 150);
        });
    });
    
    // Show a welcome message
    setTimeout(() => {
        console.log('💡 Try clicking "Load API Data" to see the captured API response!');
    }, 1000);
});

// Export for potential use in other scripts
window.WebMockTest = {
    loadApiData: loadApiData,
    version: '1.0.0'
};"#
    .to_string()
}

/// Create test JSON API response
fn create_test_json() -> String {
    r#"{
  "message": "Hello from WebMock CLI!",
  "status": "success",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "users": [
      {
        "id": 1,
        "name": "Alice Johnson",
        "email": "alice@example.com",
        "role": "developer"
      },
      {
        "id": 2,
        "name": "Bob Smith",
        "email": "bob@example.com",
        "role": "designer"
      }
    ],
    "stats": {
      "total_requests": 1337,
      "uptime": "99.9%",
      "response_time": "45ms"
    }
  },
  "meta": {
    "version": "1.0.0",
    "served_by": "WebMock CLI",
    "documentation": "https://github.com/your-org/webmock-cli"
  }
}"#
    .to_string()
}
