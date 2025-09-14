use crate::capture::proxy::recorder::RequestRecorder;
use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use crate::capture::proxy::server::HttpProxy;
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

#[tokio::test]
async fn test_request_recorder() {
    let recorder = RequestRecorder::new();

    // Test initial state
    let records = recorder.get_records().await;
    assert!(records.is_empty());

    // Test recording a request
    let response = create_test_response(200, "application/json", b"{\"test\": true}".to_vec());
    let request = RequestRecord::new(
        "GET".to_string(),
        "https://api.example.com/test".to_string(),
        create_test_headers(),
        None,
        response,
    );

    recorder.record_request(request.clone()).await;

    let records = recorder.get_records().await;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].method, "GET");
    assert_eq!(records[0].url, "https://api.example.com/test");

    // Test clearing records
    recorder.clear_records().await;
    let records = recorder.get_records().await;
    assert!(records.is_empty());
}

#[tokio::test]
#[ignore = "slow test - requires network binding"]
async fn test_http_proxy_lifecycle() {
    // Test proxy creation and basic functionality
    let test_port = 18080; // Use a specific test port
    let proxy = HttpProxy::start(test_port).await.unwrap();
    let port = proxy.get_port();
    assert_eq!(port, test_port);

    // Test initial state
    let records = proxy.get_records().await;
    assert!(records.is_empty());

    // Test stopping the proxy
    proxy.stop().await.unwrap();
}
