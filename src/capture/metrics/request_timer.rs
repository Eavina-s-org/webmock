use std::time::{Duration, Instant};
use tracing::debug;

/// Timer for measuring request processing time
#[derive(Debug)]
pub struct RequestTimer {
    start_time: Instant,
    label: String,
}

impl RequestTimer {
    /// Create a new request timer
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            label: label.into(),
        }
    }

    /// Start a new request timer with default label
    pub fn start() -> Self {
        Self::new("request")
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Print elapsed time
    pub fn print_elapsed(&self) {
        let elapsed = self.elapsed();
        debug!("{}: {:?}", self.label, elapsed);
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        debug!("Request '{}' completed in {:?}", self.label, elapsed);
    }
}
