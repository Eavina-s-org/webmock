use crate::capture::proxy::{RequestRecord, ResponseRecord};
use crate::storage::{Snapshot, SnapshotData, SnapshotSerializer};
use chrono::Utc;
use std::collections::HashMap;

fn create_test_snapshot() -> Snapshot {
    Snapshot {
        name: "test-snapshot".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests: vec![RequestRecord {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
            body: None,
            response: ResponseRecord {
                status: 200,
                headers: HashMap::new(),
                body: b"<html><body>Test</body></html>".to_vec(),
                content_type: "text/html".to_string(),
            },
            timestamp: Utc::now(),
        }],
    }
}

#[test]
fn test_serialize_deserialize_snapshot() {
    let snapshot = create_test_snapshot();

    // Test serialization
    let serialized = SnapshotSerializer::serialize(&snapshot).unwrap();
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized = SnapshotSerializer::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.name, snapshot.name);
    assert_eq!(deserialized.url, snapshot.url);
    assert_eq!(deserialized.requests.len(), snapshot.requests.len());

    // Check first request
    let original_req = &snapshot.requests[0];
    let deserialized_req = &deserialized.requests[0];
    assert_eq!(deserialized_req.method, original_req.method);
    assert_eq!(deserialized_req.url, original_req.url);
    assert_eq!(
        deserialized_req.response.status,
        original_req.response.status
    );
    assert_eq!(deserialized_req.response.body, original_req.response.body);
}

#[test]
fn test_serialize_includes_metadata() {
    let snapshot = create_test_snapshot();

    let serialized = SnapshotSerializer::serialize(&snapshot).unwrap();
    let snapshot_data: SnapshotData = rmp_serde::from_slice(&serialized).unwrap();

    assert_eq!(snapshot_data.metadata.name, snapshot.name);
    assert_eq!(snapshot_data.metadata.url, snapshot.url);
    assert_eq!(snapshot_data.metadata.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(snapshot_data.requests.len(), snapshot.requests.len());
}

#[test]
fn test_deserialize_metadata_only() {
    let snapshot = create_test_snapshot();

    let serialized = SnapshotSerializer::serialize(&snapshot).unwrap();
    let metadata = SnapshotSerializer::deserialize_metadata(&serialized).unwrap();

    assert_eq!(metadata.name, snapshot.name);
    assert_eq!(metadata.url, snapshot.url);
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_serialize_empty_requests() {
    let mut snapshot = create_test_snapshot();
    snapshot.requests.clear();

    let serialized = SnapshotSerializer::serialize(&snapshot).unwrap();
    let deserialized = SnapshotSerializer::deserialize(&serialized).unwrap();

    assert_eq!(deserialized.name, snapshot.name);
    assert_eq!(deserialized.url, snapshot.url);
    assert!(deserialized.requests.is_empty());
}

#[test]
fn test_serialize_multiple_requests() {
    let mut snapshot = create_test_snapshot();

    // Add another request
    snapshot.requests.push(RequestRecord {
        method: "POST".to_string(),
        url: "https://example.com/api".to_string(),
        headers: {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers
        },
        body: Some(b"{\"test\": true}".to_vec()),
        response: ResponseRecord {
            status: 201,
            headers: HashMap::new(),
            body: b"{\"id\": 123}".to_vec(),
            content_type: "application/json".to_string(),
        },
        timestamp: Utc::now(),
    });

    let serialized = SnapshotSerializer::serialize(&snapshot).unwrap();
    let deserialized = SnapshotSerializer::deserialize(&serialized).unwrap();

    assert_eq!(deserialized.requests.len(), 2);

    // Check first request (GET)
    assert_eq!(deserialized.requests[0].method, "GET");
    assert_eq!(deserialized.requests[0].response.status, 200);

    // Check second request (POST)
    assert_eq!(deserialized.requests[1].method, "POST");
    assert_eq!(deserialized.requests[1].response.status, 201);
    assert_eq!(
        deserialized.requests[1].body,
        Some(b"{\"test\": true}".to_vec())
    );
}

#[test]
fn test_deserialize_invalid_data() {
    let invalid_data = b"invalid msgpack data";
    let result = SnapshotSerializer::deserialize(invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_metadata_invalid_data() {
    let invalid_data = b"invalid msgpack data";
    let result = SnapshotSerializer::deserialize_metadata(invalid_data);
    assert!(result.is_err());
}
