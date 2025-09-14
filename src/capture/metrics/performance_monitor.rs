use std::sync::Arc;
use std::time::{Duration, Instant};
use sys_info::mem_info;
use tokio::sync::RwLock;
use tracing::info;

use super::performance_metrics::PerformanceMetrics;

/// Performance monitor for tracking system metrics
#[derive(Debug)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
    request_times: Arc<RwLock<Vec<Duration>>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            start_time: Instant::now(),
            request_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Record the start of a request
    pub async fn record_request_start(&self) -> Instant {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections += 1;
        Instant::now()
    }

    /// Record the completion of a request
    pub async fn record_request_complete(&self, start_time: Instant, success: bool) {
        let duration = start_time.elapsed();

        let mut metrics = self.metrics.write().await;
        metrics.active_connections = metrics.active_connections.saturating_sub(1);

        let response_time_ms = duration.as_secs_f64() * 1000.0;
        metrics.update_from_request(response_time_ms, 0, success);

        let mut request_times = self.request_times.write().await;
        request_times.push(duration);

        // Keep only last 1000 requests for statistics
        if request_times.len() > 1000 {
            request_times.remove(0);
        }
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self) {
        let mut metrics = self.metrics.write().await;

        // Get current memory usage (simplified)
        if let Ok(memory_info) = mem_info() {
            let memory_usage = (memory_info.total - memory_info.free) * 1024; // Convert to bytes
            metrics.memory_usage = memory_usage;

            if memory_usage > metrics.peak_memory_usage {
                metrics.peak_memory_usage = memory_usage;
            }
        }
    }

    /// Calculate average response time
    pub async fn calculate_average_response_time(&self) -> f64 {
        let request_times = self.request_times.read().await;
        if request_times.is_empty() {
            0.0
        } else {
            let total: Duration = request_times.iter().sum();
            total.as_secs_f64() * 1000.0 / request_times.len() as f64
        }
    }

    /// Update all metrics
    pub async fn update_metrics(&self) {
        self.update_memory_usage().await;

        let avg_response_time = self.calculate_average_response_time().await;
        let mut metrics = self.metrics.write().await;
        metrics.avg_response_time_ms = avg_response_time;
    }

    /// Print performance summary
    pub async fn print_summary(&self) {
        let metrics = self.get_metrics().await;
        let uptime = self.start_time.elapsed();

        info!("=== Performance Summary ===");
        info!("Uptime: {:?}", uptime);
        info!("Active Connections: {}", metrics.active_connections);
        info!("Total Requests: {}", metrics.requests_processed);
        info!("Failed Requests: {}", metrics.failed_requests);
        info!(
            "Average Response Time: {:.2}ms",
            metrics.avg_response_time_ms
        );
        info!("Memory Usage: {}", metrics.memory_usage_human());
        info!("Peak Memory Usage: {}", metrics.peak_memory_usage_human());
        info!(
            "Total Data Transferred: {}",
            metrics.total_data_transferred_human()
        );
        info!("Cache Hit Ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
        info!("=========================");
    }

    /// Start periodic metrics collection
    pub async fn start_metrics_collection(self: Arc<Self>) {
        let monitor = Arc::clone(&self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                monitor.update_metrics().await;

                // Print summary every 5 minutes
                if monitor.start_time.elapsed().as_secs().is_multiple_of(300) {
                    monitor.print_summary().await;
                }
            }
        });
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
