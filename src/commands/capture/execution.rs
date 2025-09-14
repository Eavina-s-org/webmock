//! Capture execution logic with progress reporting and error recovery

use std::time::Duration;
use tracing::{error, warn};

use crate::capture::CaptureSession;
use crate::error::Result;
use crate::feedback::ProgressReporter;

/// Run capture with comprehensive progress reporting and error handling
pub async fn run_capture_with_progress(
    session: &mut CaptureSession,
    progress: &mut ProgressReporter,
    url: &str,
    name: &str,
    timeout: u64,
) -> Result<()> {
    // Start main progress indicator
    let main_progress = progress.start_capture_progress(url);

    // Run capture with comprehensive error handling and progress updates
    let capture_result = run_capture_with_recovery(session, progress, url, name, timeout).await;

    match capture_result {
        Ok(_) => {
            progress.finish_capture_success(name);
            Ok(())
        }
        Err(e) => {
            progress.finish_capture_error(&e.user_message());

            // Attempt cleanup on failure
            main_progress.set_message("🧹 Cleaning up...");
            if let Err(cleanup_err) = session.cleanup().await {
                warn!("Failed to cleanup after capture failure: {}", cleanup_err);
            }

            Err(e)
        }
    }
}

/// Run capture with automatic recovery for certain errors
async fn run_capture_with_recovery(
    session: &mut CaptureSession,
    progress: &mut ProgressReporter,
    url: &str,
    name: &str,
    timeout: u64,
) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    loop {
        // Update progress with current attempt
        progress.update_capture_step(&format!(
            "Starting attempt {} of {}",
            retry_count + 1,
            MAX_RETRIES + 1
        ));

        match session.capture(url, name, timeout).await {
            Ok(_) => {
                progress.update_capture_step("Saving snapshot...");

                // Capture successful, now stop and save
                session.stop(name, url).await?;
                return Ok(());
            }
            Err(e) if e.is_recoverable() && retry_count < MAX_RETRIES => {
                retry_count += 1;
                warn!(
                    "Capture attempt {} failed ({}), retrying... ({}/{})",
                    retry_count, e, retry_count, MAX_RETRIES
                );

                progress.update_capture_step(&format!(
                    "Attempt {} failed, retrying in 2s... ({}/{})",
                    retry_count, retry_count, MAX_RETRIES
                ));

                // Wait before retry with progress update
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Cleanup before retry
                progress.update_capture_step("Cleaning up before retry...");
                if let Err(cleanup_err) = session.cleanup().await {
                    warn!("Cleanup failed before retry: {}", cleanup_err);
                }

                continue;
            }
            Err(e) => {
                error!("Capture failed after {} attempts: {}", retry_count + 1, e);
                return Err(e);
            }
        }
    }
}
