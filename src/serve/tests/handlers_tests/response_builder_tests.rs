use crate::{
    serve::handlers::response_builder::*,
    test_utils::test_helpers::{create_multi_request_snapshot, create_test_snapshot_with_name},
};
use http_body_util::BodyExt;

#[test]
fn test_create_response_from_record() {
    let snapshot = create_multi_request_snapshot("test");
    let record = &snapshot.requests[0]; // HTML page

    let response = create_response_from_record(record);

    assert_eq!(response.status(), hyper::StatusCode::OK);

    // Check that content-type header is preserved
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "text/html; charset=utf-8");
}

#[test]
fn test_create_response_from_record_css() {
    let snapshot = create_multi_request_snapshot("test");
    let record = &snapshot.requests[1]; // CSS file

    let response = create_response_from_record(record);

    assert_eq!(response.status(), hyper::StatusCode::OK);

    // Check CSS content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "text/css");
}

#[test]
fn test_create_response_from_record_api() {
    let snapshot = create_multi_request_snapshot("test");
    let record = &snapshot.requests[2]; // API endpoint

    let response = create_response_from_record(record);

    assert_eq!(response.status(), hyper::StatusCode::OK);

    // Check JSON content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "application/json");
}

#[test]
fn test_create_404_response() {
    let response = create_404_response("/nonexistent/path");

    assert_eq!(response.status(), hyper::StatusCode::NOT_FOUND);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "text/html; charset=utf-8");
}

#[tokio::test]
async fn test_create_404_response_body_content() {
    let response = create_404_response("/test/path");
    let body_bytes = {
        let body = response.into_body();
        let collected = body.collect().await.unwrap();
        collected.to_bytes().to_vec()
    };
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert!(body_str.contains("404 Not Found"));
    assert!(body_str.contains("/test/path"));
    assert!(body_str.contains("WebMock"));
}

#[tokio::test]
async fn test_response_body_content() {
    let snapshot = create_multi_request_snapshot("test");
    let record = &snapshot.requests[2]; // API endpoint

    let response = create_response_from_record(record);
    let body_bytes = {
        let body = response.into_body();
        let collected = body.collect().await.unwrap();
        collected.to_bytes().to_vec()
    };
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert!(body_str.contains("Hello from API"));
    assert!(body_str.contains("success"));
}

#[test]
fn test_response_headers_filtering() {
    let mut snapshot = create_test_snapshot_with_name("test");

    // Add headers that should be filtered out
    snapshot.requests[0]
        .response
        .headers
        .insert("content-length".to_string(), "123".to_string());
    snapshot.requests[0]
        .response
        .headers
        .insert("transfer-encoding".to_string(), "chunked".to_string());
    snapshot.requests[0]
        .response
        .headers
        .insert("connection".to_string(), "keep-alive".to_string());
    snapshot.requests[0]
        .response
        .headers
        .insert("custom-header".to_string(), "should-be-kept".to_string());

    let response = create_response_from_record(&snapshot.requests[0]);

    // content-length should be recalculated based on actual body size
    let expected_length = snapshot.requests[0].response.body.len().to_string();
    let content_length_header = response
        .headers()
        .get("content-length")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(content_length_header, expected_length);

    // Filtered headers should not be present
    assert!(response.headers().get("transfer-encoding").is_none());
    assert!(response.headers().get("connection").is_none());

    // Custom headers should be preserved
    assert_eq!(
        response.headers().get("custom-header").unwrap(),
        "should-be-kept"
    );
}
