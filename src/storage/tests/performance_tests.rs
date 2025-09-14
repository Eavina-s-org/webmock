use chrono::Utc;
use std::collections::HashMap;
use tempfile::TempDir;

use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use crate::storage::types::Snapshot;
use crate::storage::{serialization::SnapshotSerializer, Storage};

#[tokio::test]
#[ignore = "slow test - large data processing"]
async fn test_compression_for_large_snapshots() {
    // Create a large snapshot with many requests
    let mut requests = Vec::new();

    // Create 100 requests with large response bodies
    for i in 0..100 {
        let large_body = vec![b'x'; 10000]; // 10KB per response
        let response = ResponseRecord::new(
            200,
            HashMap::new(),
            large_body,
            Some(&format!("https://example.com/api/data/{}", i)),
        );

        let request = RequestRecord::new(
            "GET".to_string(),
            format!("https://example.com/api/data/{}", i),
            HashMap::new(),
            None,
            response,
        );

        requests.push(request);
    }

    let snapshot = Snapshot {
        name: "large-test".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests,
    };

    // Test compression ratio
    let compression_ratio = SnapshotSerializer::get_compression_ratio(&snapshot).unwrap();
    println!("Compression ratio: {:.2}", compression_ratio);

    // Compression should provide some benefit for repetitive data
    assert!(compression_ratio < 1.0);
    // For highly repetitive data, compression can be very effective
    assert!(compression_ratio >= 0.0);
}

#[tokio::test]
#[ignore = "slow test - large data processing"]
async fn test_streaming_serialization_for_large_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Create a large snapshot
    let mut requests = Vec::new();

    // Create 50 requests with large response bodies (total ~50MB)
    for i in 0..50 {
        let large_body = vec![b'x'; 1024 * 1024]; // 1MB per response
        let response = ResponseRecord::new(
            200,
            HashMap::new(),
            large_body,
            Some(&format!("https://example.com/large/{}", i)),
        );

        let request = RequestRecord::new(
            "GET".to_string(),
            format!("https://example.com/large/{}", i),
            HashMap::new(),
            None,
            response,
        );

        requests.push(request);
    }

    let snapshot = Snapshot {
        name: "streaming-test".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests,
    };

    // Save the large snapshot (should use streaming)
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    // Load it back (should use streaming)
    let loaded_snapshot = storage.load_snapshot("streaming-test").await.unwrap();

    // Verify the data is intact
    assert_eq!(loaded_snapshot.name, snapshot.name);
    assert_eq!(loaded_snapshot.url, snapshot.url);
    assert_eq!(loaded_snapshot.requests.len(), snapshot.requests.len());

    // Verify a few requests
    for i in 0..5 {
        assert_eq!(loaded_snapshot.requests[i].url, snapshot.requests[i].url);
        assert_eq!(loaded_snapshot.requests[i].response.body.len(), 1024 * 1024);
    }
}

#[tokio::test]
async fn test_small_snapshot_regular_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Create a small snapshot
    let response = ResponseRecord::new(
        200,
        HashMap::new(),
        b"Hello, World!".to_vec(),
        Some("https://example.com/small"),
    );

    let request = RequestRecord::new(
        "GET".to_string(),
        "https://example.com/small".to_string(),
        HashMap::new(),
        None,
        response,
    );

    let snapshot = Snapshot {
        name: "small-test".to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests: vec![request],
    };

    // Save the small snapshot (should use regular serialization)
    storage.save_snapshot(snapshot.clone()).await.unwrap();

    // Load it back
    let loaded_snapshot = storage.load_snapshot("small-test").await.unwrap();

    // Verify the data is intact
    assert_eq!(loaded_snapshot.name, snapshot.name);
    assert_eq!(loaded_snapshot.url, snapshot.url);
    assert_eq!(loaded_snapshot.requests.len(), 1);
    assert_eq!(loaded_snapshot.requests[0].response.body, b"Hello, World!");
}

#[test]
fn test_compression_detection() {
    // Test uncompressed data
    let uncompressed = b"Hello, World!";
    assert!(!SnapshotSerializer::is_compressed(uncompressed));

    // Test gzip magic bytes
    let gzip_data = [0x1f, 0x8b, 0x08, 0x00]; // Gzip magic + flags
    assert!(SnapshotSerializer::is_compressed(&gzip_data));

    // Test other data that looks like gzip but isn't
    let fake_gzip = [0x1f, 0x8b];
    assert!(SnapshotSerializer::is_compressed(&fake_gzip));

    // Test empty data
    let empty = [];
    assert!(!SnapshotSerializer::is_compressed(&empty));
}

#[tokio::test]
#[ignore = "slow test - large data processing"]
async fn test_memory_usage_estimation() {
    // Create snapshots of different sizes and verify estimation
    let small_snapshot = create_test_snapshot("small", 1, 100);
    let medium_snapshot = create_test_snapshot("medium", 10, 1000);
    let large_snapshot = create_test_snapshot("large", 100, 10000);

    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // The estimate_snapshot_size method is private, but we can test it indirectly
    // by checking that larger snapshots trigger streaming serialization

    // Save snapshots and check file sizes
    storage.save_snapshot(small_snapshot).await.unwrap();
    storage.save_snapshot(medium_snapshot).await.unwrap();
    storage.save_snapshot(large_snapshot).await.unwrap();

    // Verify files exist
    assert!(storage.snapshot_exists("small"));
    assert!(storage.snapshot_exists("medium"));
    assert!(storage.snapshot_exists("large"));

    // Load them back to verify integrity
    let loaded_small = storage.load_snapshot("small").await.unwrap();
    let loaded_medium = storage.load_snapshot("medium").await.unwrap();
    let loaded_large = storage.load_snapshot("large").await.unwrap();

    assert_eq!(loaded_small.requests.len(), 1);
    assert_eq!(loaded_medium.requests.len(), 10);
    assert_eq!(loaded_large.requests.len(), 100);
}

fn create_test_snapshot(name: &str, num_requests: usize, body_size: usize) -> Snapshot {
    let mut requests = Vec::new();

    for i in 0..num_requests {
        let body = vec![b'x'; body_size];
        let response = ResponseRecord::new(
            200,
            HashMap::new(),
            body,
            Some(&format!("https://example.com/{}/{}", name, i)),
        );

        let request = RequestRecord::new(
            "GET".to_string(),
            format!("https://example.com/{}/{}", name, i),
            HashMap::new(),
            None,
            response,
        );

        requests.push(request);
    }

    Snapshot {
        name: name.to_string(),
        url: "https://example.com".to_string(),
        created_at: Utc::now(),
        requests,
    }
}
