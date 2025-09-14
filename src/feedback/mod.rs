pub mod chrome_detection;
pub mod core;
pub mod error_display;
pub mod progress;
pub mod user_feedback;
pub mod validation;

// Re-export main types for convenience
pub use crate::feedback::core::UserFeedback;
pub use error_display::ErrorDisplay;
pub use progress::ProgressReporter;
pub use validation::ValidationHelper;

#[cfg(test)]
mod tests;
