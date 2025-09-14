//! Integration tests for WebMock CLI
//!
//! This module contains end-to-end tests that verify complete workflows:
//! - capture → list → serve
//! - Error scenarios and recovery mechanisms  
//! - Various resource types and API request handling

mod integration;

// Re-export all test modules
pub use integration::*;
