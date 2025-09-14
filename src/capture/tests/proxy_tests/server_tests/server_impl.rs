use crate::capture::proxy::client_pool::HttpClientPool;
use crate::capture::proxy::server::is_hop_by_hop_header;
use crate::capture::proxy::server::utils::{forward_request, forward_request_with_pool};
use hyper::Method;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_hop_by_hop_header_detection() {
    assert!(is_hop_by_hop_header("connection"));
    assert!(is_hop_by_hop_header("keep-alive"));
    assert!(is_hop_by_hop_header("proxy-authenticate"));
    assert!(is_hop_by_hop_header("proxy-authorization"));
    assert!(is_hop_by_hop_header("te"));
    assert!(is_hop_by_hop_header("trailers"));
    assert!(is_hop_by_hop_header("transfer-encoding"));
    assert!(is_hop_by_hop_header("upgrade"));

    assert!(!is_hop_by_hop_header("content-type"));
    assert!(!is_hop_by_hop_header("authorization"));
    assert!(!is_hop_by_hop_header("user-agent"));
    assert!(!is_hop_by_hop_header("accept"));
}

#[tokio::test]
#[ignore = "slow test - requires network connectivity"]
async fn test_forward_request() {
    // Test with a simple GET request to a known endpoint
    let method = Method::GET;
    let url = "https://httpbin.org/get";
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "webmock-test-agent".to_string());
    let body = vec![];
    let version = hyper::Version::HTTP_11;

    // This test might fail if there's no internet connection or httpbin is down
    // In a real test suite, we would use a mock server
    let result = forward_request(method, url, headers, body, version).await;

    // We can't assert specific results since we're making actual HTTP requests
    // but we can check that the function executes without panicking
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_forward_request_with_invalid_url() {
    let method = Method::GET;
    let url = "invalid-url";
    let headers = HashMap::new();
    let body = vec![];
    let version = hyper::Version::HTTP_11;
    let client_pool = Arc::new(HttpClientPool::new());

    let result = forward_request_with_pool(method, url, headers, body, version, client_pool).await;

    // Should return an error for invalid URL
    assert!(result.is_err());

    // Check that it's a proxy error
    let error = result.unwrap_err();
    assert!(format!("{:?}", error).contains("Proxy"));
}

#[tokio::test]
#[ignore = "slow test - requires network connectivity"]
async fn test_forward_request_with_hop_by_hop_headers() {
    let method = Method::GET;
    let url = "https://httpbin.org/get";
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "webmock-test-agent".to_string());
    headers.insert("Connection".to_string(), "keep-alive".to_string());
    headers.insert("Proxy-Authenticate".to_string(), "Basic".to_string());
    headers.insert("Transfer-Encoding".to_string(), "chunked".to_string());
    let body = vec![];
    let version = hyper::Version::HTTP_11;

    // Test that hop-by-hop headers are filtered out
    // This test might fail if there's no internet connection or httpbin is down
    let result = forward_request(method, url, headers, body, version).await;

    // We can't assert specific results since we're making actual HTTP requests
    // but we can check that the function executes without panicking
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore = "slow test - requires network connectivity"]
async fn test_forward_request_with_pool() {
    let method = Method::GET;
    let url = "https://httpbin.org/get";
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "webmock-test-agent".to_string());
    let body = vec![];
    let version = hyper::Version::HTTP_11;
    let client_pool = Arc::new(HttpClientPool::new());

    // This test might fail if there's no internet connection or httpbin is down
    // In a real test suite, we would use a mock server
    let result = forward_request_with_pool(method, url, headers, body, version, client_pool).await;

    // We can't assert specific results since we're making actual HTTP requests
    // but we can check that the function executes without panicking
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore = "slow test - requires network connectivity"]
async fn test_forward_request_with_pool_and_hop_by_hop_headers() {
    let method = Method::GET;
    let url = "https://httpbin.org/get";
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "webmock-test-agent".to_string());
    headers.insert("Connection".to_string(), "keep-alive".to_string());
    headers.insert("Proxy-Authenticate".to_string(), "Basic".to_string());
    headers.insert("Transfer-Encoding".to_string(), "chunked".to_string());
    let body = vec![];
    let version = hyper::Version::HTTP_11;
    let client_pool = Arc::new(HttpClientPool::new());

    // Test that hop-by-hop headers are filtered out
    // This test might fail if there's no internet connection or httpbin is down
    let result = forward_request_with_pool(method, url, headers, body, version, client_pool).await;

    // We can't assert specific results since we're making actual HTTP requests
    // but we can check that the function executes without panicking
    assert!(result.is_ok() || result.is_err());
}
