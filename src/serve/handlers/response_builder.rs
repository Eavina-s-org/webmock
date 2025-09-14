//! Response building utilities for mock server

use crate::capture::proxy::RequestRecord;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use tracing::error;

/// Create an HTTP response from a recorded request
pub fn create_response_from_record(record: &RequestRecord) -> Response<Full<Bytes>> {
    let mut response_builder = Response::builder().status(record.response.status);

    // Add headers from the recorded response
    for (key, value) in &record.response.headers {
        // Skip headers that hyper manages automatically, but allow connection header for CONNECT
        let key_lower = key.to_lowercase();
        if !matches!(key_lower.as_str(), "content-length" | "transfer-encoding") {
            // For CONNECT responses, preserve connection headers for upgrade support
            if key_lower == "connection" && record.method == "CONNECT" {
                response_builder = response_builder.header("Connection", "upgrade");
            } else if key_lower != "connection" {
                response_builder = response_builder.header(key, value);
            }
        }
    }

    // Always set content-length based on actual body size
    response_builder =
        response_builder.header("content-length", record.response.body.len().to_string());

    // Create response body
    let body = Full::new(Bytes::from(record.response.body.clone()));

    response_builder.body(body).unwrap_or_else(|e| {
        error!("Failed to create response: {}", e);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from("Internal server error")))
            .unwrap()
    })
}

/// Create a 404 Not Found response with helpful information
pub fn create_404_response(url: &str) -> Response<Full<Bytes>> {
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found - WebMock</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .container {{ max-width: 600px; margin: 0 auto; }}
        .error {{ color: #d32f2f; }}
        .url {{ background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="error">404 Not Found</h1>
        <p>The requested resource was not found in the recorded snapshot.</p>
        <div class="url">{}</div>
        <p>This means the URL was not captured during the original recording session.</p>
        <p>To capture this resource, re-run the capture command and ensure you navigate to or trigger requests for this URL.</p>
    </div>
</body>
</html>"#,
        html_escape::encode_text(url)
    );

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}
