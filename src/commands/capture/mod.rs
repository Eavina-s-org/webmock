//! Capture command implementation
//!
//! This module handles the capture command which records web pages and all their
//! network requests for later replay.

mod execution;
mod storage;
mod validation;

pub use execution::*;
pub use storage::*;
pub use validation::*;

use tracing::info;

use crate::capture::CaptureSession;
use crate::error::Result;
use crate::feedback::{chrome_detection::ChromeDetection, ProgressReporter, UserFeedback};

/// Handle the capture command
pub async fn capture_command(
    url: &str,
    name: &str,
    timeout: u64,
    storage_arg: Option<String>,
) -> Result<()> {
    info!(
        "Starting capture command for URL: {} with name: {}",
        url, name
    );

    // Step 0: System requirements check
    UserFeedback::info("Checking system requirements...");
    crate::feedback::ValidationHelper::check_system_requirements()?;

    // Step 1: Validate inputs with enhanced feedback
    UserFeedback::info("Validating inputs...");
    validate_inputs(url, name, timeout)?;
    UserFeedback::success("Input validation passed");

    // Step 1.5: Check Chrome availability
    UserFeedback::info("Checking Chrome browser availability...");
    ChromeDetection::validate_and_guide()?;

    // Step 2: Initialize storage
    let storage = initialize_storage(storage_arg).await?;

    // Step 3: Check if snapshot already exists
    check_snapshot_exists(&storage, name)?;

    // Step 4: Create and run capture session with progress reporting
    let mut progress = ProgressReporter::new();
    let mut session = CaptureSession::new(storage).await?;

    // Step 5: Start capture with comprehensive progress reporting
    run_capture_with_progress(&mut session, &mut progress, url, name, timeout).await?;

    // Success feedback
    UserFeedback::success("Capture completed successfully!");
    println!("📸 Snapshot '{}' has been saved", name);
    println!();
    UserFeedback::tip("Use 'webmock list' to see all snapshots");
    UserFeedback::tip(&format!(
        "Use 'webmock serve {}' to start the mock server",
        name
    ));

    Ok(())
}
