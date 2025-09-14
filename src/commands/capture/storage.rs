//! Storage initialization and management for capture command

use std::sync::Arc;
use tracing::info;

use crate::error::{Result, WebMockError};
use crate::feedback::UserFeedback;
use crate::storage::Storage;

/// Initialize storage in the specified or default directory
pub async fn initialize_storage(storage_arg: Option<String>) -> Result<Arc<Storage>> {
    UserFeedback::info("Initializing storage...");
    let storage_path = crate::commands::get_storage_path(storage_arg)?;

    // Create storage directory if it doesn't exist
    if !storage_path.exists() {
        UserFeedback::info(&format!(
            "Creating storage directory: {}",
            storage_path.display()
        ));
        tokio::fs::create_dir_all(&storage_path)
            .await
            .map_err(|e| {
                WebMockError::permission_denied(format!(
                    "Failed to create storage directory: {}",
                    e
                ))
            })?;
    }

    let storage = Arc::new(Storage::new(storage_path.clone()));

    // Ensure snapshots subdirectory exists
    storage.ensure_snapshots_dir()?;

    UserFeedback::success("Storage initialized");
    info!("Storage initialized at: {}", storage_path.display());
    Ok(storage)
}

/// Check if snapshot already exists and provide helpful feedback
pub fn check_snapshot_exists(storage: &Storage, name: &str) -> Result<()> {
    if storage.snapshot_exists(name) {
        UserFeedback::error(&format!("Snapshot '{}' already exists", name));
        UserFeedback::tip("Use a different name or delete the existing snapshot first");
        UserFeedback::tip(&format!("Delete with: webmock delete {}", name));
        return Err(WebMockError::config(format!(
            "Snapshot '{}' already exists",
            name
        )));
    }
    Ok(())
}
