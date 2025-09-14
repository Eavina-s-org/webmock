pub mod capture;
pub mod delete;
pub mod inspect;
pub mod list;
pub mod serve;

#[cfg(test)]
mod tests;

pub use capture::capture_command;
pub use delete::delete_command;
pub use inspect::inspect_command;
pub use list::list_command;
pub use serve::serve_command;

use crate::error::{Result, WebMockError};
use std::path::PathBuf;

/// Get storage path from CLI argument or use default
pub fn get_storage_path(storage_arg: Option<String>) -> Result<PathBuf> {
    if let Some(custom_path) = storage_arg {
        let path = PathBuf::from(custom_path);
        if !path.exists() {
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&path).map_err(|e| {
                WebMockError::config(format!("Failed to create storage directory: {}", e))
            })?;
        }
        Ok(path)
    } else {
        // Use default ~/.webmock
        let home_dir = dirs::home_dir()
            .ok_or_else(|| WebMockError::config("Could not determine home directory"))?;
        Ok(home_dir.join(".webmock"))
    }
}
