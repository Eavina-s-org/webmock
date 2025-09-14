//! Input validation for capture command

use crate::error::Result;
use crate::feedback::ValidationHelper;
use tracing::info;

/// Validate command inputs with enhanced feedback
pub fn validate_inputs(url: &str, name: &str, timeout: u64) -> Result<()> {
    // Validate URL format and scheme
    ValidationHelper::validate_url(url)?;

    // Validate snapshot name
    ValidationHelper::validate_snapshot_name(name)?;

    // Validate timeout
    ValidationHelper::validate_timeout(timeout)?;

    info!("Input validation passed");
    Ok(())
}
