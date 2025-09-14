use crate::capture::metrics::PerformanceMonitor;
use crate::capture::proxy::client_pool::HttpClientPool;
use crate::capture::proxy::streaming::{ResponseCollector, StreamingWriter};
use bytes::Bytes;
use http_body_util::Full;
use std::sync::Arc;

#[tokio::test]
async fn test_http_client_pool() {
    let pool = HttpClientPool::new();

    // Test getting clients for different hosts
    let client1 = pool.get_client("example.com").await;
    let client2 = pool.get_client("example.com").await;
    let client3 = pool.get_client("google.com").await;

    // Should reuse client for same host
    assert!(Arc::ptr_eq(&client1, &client2));

    // Should create different client for different host
    assert!(!Arc::ptr_eq(&client1, &client3));

    // Check client count
    assert_eq!(pool.client_count().await, 2);

    // Test cleanup
    pool.clear().await;
    assert_eq!(pool.client_count().await, 0);
}

#[tokio::test]
async fn test_response_collector_small_response() {
    let collector = ResponseCollector::default();
    let small_data = b"Hello, World!";
    let body = Full::new(Bytes::from(small_data.to_vec()));

    let result = collector.collect_response(body).await.unwrap();
    assert_eq!(result, small_data);
}

#[tokio::test]
async fn test_response_collector_large_response() {
    let collector = ResponseCollector::new(1024); // 1KB threshold for testing
    let large_data = vec![b'x'; 2048]; // 2KB data
    let body = Full::new(Bytes::from(large_data.clone()));

    let result = collector.collect_response(body).await.unwrap();
    assert_eq!(result, large_data);
}

#[tokio::test]
async fn test_streaming_writer_small_data() {
    let mut writer = StreamingWriter::new(1024);
    let small_data = b"Hello, World!";

    writer.write(small_data).await.unwrap();
    let result = writer.finalize().await.unwrap();

    assert_eq!(result, small_data);
}

#[tokio::test]
async fn test_streaming_writer_large_data() {
    let mut writer = StreamingWriter::new(1024); // 1KB threshold
    let large_data = vec![b'x'; 2048]; // 2KB data

    writer.write(&large_data).await.unwrap();
    let result = writer.finalize().await.unwrap();

    assert_eq!(result, large_data);
}

#[tokio::test]
async fn test_performance_monitor() {
    let monitor = PerformanceMonitor::new();

    // Record some requests
    let start1 = monitor.record_request_start().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    monitor.record_request_complete(start1, true).await;

    let start2 = monitor.record_request_start().await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    monitor.record_request_complete(start2, true).await;

    let metrics = monitor.get_metrics().await;

    assert_eq!(metrics.requests_processed, 2);
    assert!(metrics.avg_response_time_ms > 0.0);

    // Update metrics to get current state
    monitor.update_metrics().await;
    let updated_metrics = monitor.get_metrics().await;
    assert!(updated_metrics.avg_response_time_ms > 0.0);
}

#[tokio::test]
#[ignore = "slow test - simulates 30 seconds of high response times"]
async fn test_performance_monitor_unhealthy_conditions() {
    let monitor = PerformanceMonitor::new();

    // Simulate high response times
    for _ in 0..10 {
        let start = monitor.record_request_start().await;
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        monitor.record_request_complete(start, true).await;
    }

    let metrics = monitor.get_metrics().await;
    assert!(metrics.requests_processed >= 10);

    // Update metrics to get accurate response times
    monitor.update_metrics().await;
    let updated_metrics = monitor.get_metrics().await;
    assert!(updated_metrics.avg_response_time_ms > 2000.0);
}
