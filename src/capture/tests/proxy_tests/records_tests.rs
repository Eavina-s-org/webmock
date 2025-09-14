use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use std::collections::HashMap;

fn create_test_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "test-agent".to_string());
    headers
}

fn create_test_response(status: u16, content_type: &str, body: Vec<u8>) -> ResponseRecord {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());

    ResponseRecord {
        status,
        headers,
        body,
        content_type: content_type.to_string(),
    }
}

#[test]
fn test_request_record_new() {
    let headers = create_test_headers();
    let body = Some(b"test body".to_vec());
    let response = create_test_response(200, "text/html", b"<html></html>".to_vec());

    let request = RequestRecord::new(
        "GET".to_string(),
        "https://example.com".to_string(),
        headers.clone(),
        body.clone(),
        response,
    );

    assert_eq!(request.method, "GET");
    assert_eq!(request.url, "https://example.com");
    assert_eq!(request.headers, headers);
    assert_eq!(request.body, body);
    assert_eq!(request.response.status, 200);
}

#[test]
fn test_request_record_content_type_detection() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let response = create_test_response(200, "text/html", b"<html></html>".to_vec());
    let request = RequestRecord::new(
        "POST".to_string(),
        "https://api.example.com".to_string(),
        headers,
        Some(b"{}".to_vec()),
        response,
    );

    let content_type = request.get_request_content_type().unwrap();
    assert_eq!(content_type, mime::APPLICATION_JSON);
    assert!(request.is_request_body_text());
}

#[test]
fn test_request_record_body_as_string() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let response = create_test_response(200, "text/html", b"<html></html>".to_vec());
    let request = RequestRecord::new(
        "POST".to_string(),
        "https://api.example.com".to_string(),
        headers,
        Some(b"{\"key\": \"value\"}".to_vec()),
        response,
    );

    let body_string = request.get_request_body_as_string().unwrap();
    assert_eq!(body_string, "{\"key\": \"value\"}");
    assert_eq!(request.get_request_body_size(), 16);
}

#[test]
fn test_response_record_new_with_detection() {
    let headers = HashMap::new();
    let body = b"<html><body>Test</body></html>".to_vec();
    let url = Some("https://example.com/index.html");

    let response = ResponseRecord::new(200, headers, body.clone(), url);

    assert_eq!(response.status, 200);
    assert_eq!(response.body, body);
    // Should detect HTML from URL extension
    assert!(response.content_type.contains("html"));
}

#[test]
fn test_response_record_content_type_from_headers() {
    let mut headers = HashMap::new();
    headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    let body = b"{\"message\": \"hello\"}".to_vec();
    let response = ResponseRecord::new(200, headers, body, None);

    assert_eq!(response.content_type, "application/json; charset=utf-8");
    assert!(response.is_json());
    assert!(response.is_text_content());
    assert!(!response.is_image());
    assert!(!response.is_html());
}

#[test]
fn test_response_record_status_checks() {
    let response_200 = create_test_response(200, "text/html", b"<html></html>".to_vec());
    let response_301 = create_test_response(301, "text/html", b"".to_vec());
    let response_404 = create_test_response(404, "text/html", b"Not Found".to_vec());
    let response_500 = create_test_response(500, "text/html", b"Server Error".to_vec());

    assert!(response_200.is_success());
    assert!(!response_200.is_redirect());
    assert!(!response_200.is_client_error());
    assert!(!response_200.is_server_error());

    assert!(!response_301.is_success());
    assert!(response_301.is_redirect());
    assert!(!response_301.is_client_error());
    assert!(!response_301.is_server_error());

    assert!(!response_404.is_success());
    assert!(!response_404.is_redirect());
    assert!(response_404.is_client_error());
    assert!(!response_404.is_server_error());

    assert!(!response_500.is_success());
    assert!(!response_500.is_redirect());
    assert!(!response_500.is_client_error());
    assert!(response_500.is_server_error());
}

#[test]
fn test_response_record_body_as_string() {
    let response = create_test_response(200, "text/plain", b"Hello, World!".to_vec());

    let body_string = response.get_body_as_string().unwrap();
    assert_eq!(body_string, "Hello, World!");
    assert_eq!(response.get_body_size(), 13);
}

#[test]
fn test_serialization_deserialization() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let response = ResponseRecord {
        status: 200,
        headers: headers.clone(),
        body: b"binary\x00\x01\x02data".to_vec(),
        content_type: "application/json".to_string(),
    };

    let request = RequestRecord {
        method: "POST".to_string(),
        url: "https://example.com/api".to_string(),
        headers: headers.clone(),
        body: Some(b"request\x00\x01body".to_vec()),
        response,
        timestamp: chrono::Utc::now(),
    };

    // Test serialization to JSON (which uses base64 for binary data)
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("POST"));
    assert!(serialized.contains("https://example.com/api"));

    // Test deserialization
    let deserialized: RequestRecord = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.method, request.method);
    assert_eq!(deserialized.url, request.url);
    assert_eq!(deserialized.body, request.body);
    assert_eq!(deserialized.response.body, request.response.body);
    assert_eq!(deserialized.response.status, request.response.status);
}
