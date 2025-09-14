use tracing::{info, warn};

use crate::error::{Result, WebMockError};
use crate::feedback::{ProgressReporter, UserFeedback};
use crate::storage::Storage;

/// Handle the delete command with enhanced feedback
pub async fn delete_command(snapshot_name: &str, storage_arg: Option<String>) -> Result<()> {
    info!("Starting delete command for snapshot: {}", snapshot_name);

    // Step 0: Validate inputs
    UserFeedback::info("Validating inputs...");
    crate::feedback::ValidationHelper::validate_snapshot_name(snapshot_name)?;
    UserFeedback::success("Input validation passed");

    // Initialize storage
    UserFeedback::info("Initializing storage...");
    let storage_path = crate::commands::get_storage_path(storage_arg)?;
    let storage = Storage::new(storage_path);

    // Check if snapshot exists before asking for confirmation
    UserFeedback::info(&format!(
        "Checking if snapshot '{}' exists...",
        snapshot_name
    ));
    if !storage.snapshot_exists(snapshot_name) {
        UserFeedback::error(&format!("Snapshot '{}' not found", snapshot_name));
        UserFeedback::tip("Use 'webmock list' to see available snapshots");
        return Err(WebMockError::SnapshotNotFound(snapshot_name.to_string()));
    }

    UserFeedback::success(&format!("Found snapshot '{}'", snapshot_name));

    // Ask for user confirmation with enhanced dialog
    if !confirm_deletion(snapshot_name)? {
        UserFeedback::info("Deletion cancelled by user");
        return Ok(());
    }

    // Perform the deletion with progress indicator
    let progress = ProgressReporter::new();
    let deletion_spinner =
        progress.create_spinner(&format!("Deleting snapshot '{}'...", snapshot_name));

    match storage.delete_snapshot(snapshot_name).await {
        Ok(()) => {
            deletion_spinner
                .finish_with_message(format!("✅ Deleted snapshot '{}'", snapshot_name));
            UserFeedback::success(&format!(
                "Successfully deleted snapshot '{}'",
                snapshot_name
            ));
            info!("Snapshot '{}' deleted successfully", snapshot_name);

            // Show helpful next steps
            UserFeedback::tip("Use 'webmock list' to see remaining snapshots");
        }
        Err(e) => {
            deletion_spinner.finish_with_message("❌ Deletion failed");
            UserFeedback::error(&format!(
                "Failed to delete snapshot '{}': {}",
                snapshot_name,
                e.user_message()
            ));
            warn!("Failed to delete snapshot '{}': {}", snapshot_name, e);
            return Err(e);
        }
    }

    Ok(())
}

/// Ask user for confirmation before deleting the snapshot with enhanced dialog
fn confirm_deletion(snapshot_name: &str) -> Result<bool> {
    UserFeedback::warning("This action cannot be undone!");

    let message = format!(
        "Are you sure you want to delete snapshot '{}'?",
        snapshot_name
    );
    Ok(UserFeedback::confirm(&message).unwrap_or(false))
}
