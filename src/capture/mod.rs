pub mod browser;
pub mod metrics;
pub mod network;
pub mod performance;
pub mod proxy;
pub mod resource_manager;
pub mod session;
pub mod validation;

#[cfg(test)]
mod tests;

// Re-export the main types for convenience
pub use metrics::{PerformanceMetrics, PerformanceMonitor, RequestTimer};
pub use resource_manager::ResourceManager;
pub use session::CaptureSession;
