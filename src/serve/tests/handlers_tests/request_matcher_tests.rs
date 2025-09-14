use hyper::Method;

use crate::{
    serve::handlers::request_matcher::*, test_utils::test_helpers::create_multi_request_snapshot,
};

#[test]
fn test_find_matching_record_exact_match() {
    let snapshot = create_multi_request_snapshot("test");

    // Test exact URL and method match
    let record = find_matching_record(&snapshot, &Method::GET, "https://example.com/");

    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.method, "GET");
    assert_eq!(record.url, "https://example.com/");
}

#[test]
fn test_find_matching_record_path_match() {
    let snapshot = create_multi_request_snapshot("test");

    // Test path-based matching with different domain
    let record = find_matching_record(&snapshot, &Method::GET, "http://localhost:8080/style.css");

    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.method, "GET");
    assert!(record.url.contains("/style.css"));
}

#[test]
fn test_find_matching_record_api_endpoint() {
    let snapshot = create_multi_request_snapshot("test");

    // Test API endpoint matching
    let record = find_matching_record(&snapshot, &Method::GET, "https://example.com/api/data");

    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.method, "GET");
    assert!(record.url.contains("/api/data"));
}

#[test]
fn test_find_matching_record_no_match() {
    let snapshot = create_multi_request_snapshot("test");

    // Test non-existent path
    let record = find_matching_record(&snapshot, &Method::GET, "https://example.com/nonexistent");

    assert!(record.is_none());
}

#[test]
fn test_find_matching_record_wrong_method() {
    let snapshot = create_multi_request_snapshot("test");

    // Test wrong method for existing path
    let record = find_matching_record(&snapshot, &Method::POST, "https://example.com/");

    assert!(record.is_none());
}
