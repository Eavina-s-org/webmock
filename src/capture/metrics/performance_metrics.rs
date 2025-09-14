use serde::{Deserialize, Serialize};

/// Performance metrics collected during request processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Number of active connections
    pub active_connections: usize,
    /// Number of requests processed
    pub requests_processed: usize,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Peak memory usage during processing
    pub peak_memory_usage: u64,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Total data transferred in bytes
    pub total_data_transferred: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            active_connections: 0,
            requests_processed: 0,
            avg_response_time_ms: 0.0,
            peak_memory_usage: 0,
            failed_requests: 0,
            total_data_transferred: 0,
            cache_hit_ratio: 0.0,
        }
    }
}

impl PerformanceMetrics {
    /// Create a new PerformanceMetrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metrics based on a completed request
    pub fn update_from_request(&mut self, response_time_ms: f64, data_size: u64, success: bool) {
        self.requests_processed += 1;
        if success {
            self.total_data_transferred += data_size;
        } else {
            self.failed_requests += 1;
        }

        // Update average response time
        let total_time = self.avg_response_time_ms * (self.requests_processed as f64 - 1.0);
        self.avg_response_time_ms =
            (total_time + response_time_ms) / self.requests_processed as f64;
    }

    /// Get memory usage in human-readable format
    pub fn memory_usage_human(&self) -> String {
        let bytes = self.memory_usage as f64;
        if bytes < 1024.0 {
            format!("{:.2} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get peak memory usage in human-readable format
    pub fn peak_memory_usage_human(&self) -> String {
        let bytes = self.peak_memory_usage as f64;
        if bytes < 1024.0 {
            format!("{:.2} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get total data transferred in human-readable format
    pub fn total_data_transferred_human(&self) -> String {
        let bytes = self.total_data_transferred as f64;
        if bytes < 1024.0 {
            format!("{:.2} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
