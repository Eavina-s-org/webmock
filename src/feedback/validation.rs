use crate::error::{Result, WebMockError};
use crate::feedback::UserFeedback;

/// Validation helpers with user-friendly error messages
pub struct ValidationHelper;

impl ValidationHelper {
    /// Validate URL with detailed error messages
    pub fn validate_url(url: &str) -> Result<()> {
        if url.trim().is_empty() {
            return Err(WebMockError::InvalidUrl(
                url.to_string(),
                "URL cannot be empty".to_string(),
            ));
        }

        let parsed_url = url::Url::parse(url)
            .map_err(|e| WebMockError::InvalidUrl(url.to_string(), e.to_string()))?;

        match parsed_url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(WebMockError::InvalidUrl(
                url.to_string(),
                format!(
                    "Unsupported scheme '{}'. Only HTTP and HTTPS are supported",
                    scheme
                ),
            )),
        }
    }

    /// Validate snapshot name with detailed requirements
    pub fn validate_snapshot_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(WebMockError::config("Snapshot name cannot be empty"));
        }

        if name.len() > 100 {
            return Err(WebMockError::config(
                "Snapshot name cannot exceed 100 characters",
            ));
        }

        if name.contains('/') || name.contains('\\') {
            return Err(WebMockError::config(
                "Snapshot name cannot contain path separators (/ or \\)",
            ));
        }

        if name.contains(' ') {
            return Err(WebMockError::config(
                "Snapshot name cannot contain spaces. Use hyphens or underscores instead",
            ));
        }

        // Check for invalid characters
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(WebMockError::config(
                "Snapshot name can only contain letters, numbers, hyphens, and underscores",
            ));
        }

        Ok(())
    }

    /// Validate timeout with reasonable limits
    pub fn validate_timeout(timeout: u64) -> Result<()> {
        if timeout == 0 {
            return Err(WebMockError::config(
                "Timeout must be greater than 0 seconds",
            ));
        }

        if timeout > 600 {
            return Err(WebMockError::config(
                "Timeout cannot exceed 600 seconds (10 minutes)",
            ));
        }

        if timeout > 300 {
            UserFeedback::warning(&format!(
                "Timeout of {} seconds is quite long. Consider using a shorter timeout for better user experience.",
                timeout
            ));
        }

        Ok(())
    }

    /// Validate port number with comprehensive checks
    pub fn validate_port(port: u16) -> Result<()> {
        // Check for well-known reserved ports
        if port < 1024 {
            UserFeedback::warning(&format!(
                "Port {} is in the reserved range (< 1024). You may need administrator privileges.",
                port
            ));

            // Provide specific guidance for common reserved ports
            match port {
                80 => UserFeedback::tip("Port 80 is typically used by web servers (HTTP)"),
                443 => UserFeedback::tip("Port 443 is typically used by web servers (HTTPS)"),
                22 => UserFeedback::tip("Port 22 is typically used by SSH"),
                21 => UserFeedback::tip("Port 21 is typically used by FTP"),
                25 => UserFeedback::tip("Port 25 is typically used by SMTP"),
                _ => {}
            }
        }

        // Check for commonly problematic ports
        match port {
            3000 => UserFeedback::info(
                "Port 3000 is commonly used by development servers (React, Express, etc.)",
            ),
            8000 => UserFeedback::info(
                "Port 8000 is commonly used by development servers (Django, etc.)",
            ),
            8080 => UserFeedback::info("Port 8080 is commonly used by web servers and proxies"),
            9000 => UserFeedback::info("Port 9000 is commonly used by development tools"),
            _ => {}
        }

        Ok(())
    }

    /// Validate command line arguments comprehensively
    pub fn validate_command_args(
        url: Option<&str>,
        name: Option<&str>,
        timeout: Option<u64>,
        port: Option<u16>,
    ) -> Result<()> {
        let mut validation_errors = Vec::new();

        // Validate URL if provided
        if let Some(url) = url {
            if let Err(e) = Self::validate_url(url) {
                validation_errors.push(format!("URL validation failed: {}", e));
            }
        }

        // Validate snapshot name if provided
        if let Some(name) = name {
            if let Err(e) = Self::validate_snapshot_name(name) {
                validation_errors.push(format!("Snapshot name validation failed: {}", e));
            }
        }

        // Validate timeout if provided
        if let Some(timeout) = timeout {
            if let Err(e) = Self::validate_timeout(timeout) {
                validation_errors.push(format!("Timeout validation failed: {}", e));
            }
        }

        // Validate port if provided
        if let Some(port) = port {
            if let Err(e) = Self::validate_port(port) {
                validation_errors.push(format!("Port validation failed: {}", e));
            }
        }

        if !validation_errors.is_empty() {
            UserFeedback::error("Multiple validation errors found:");
            for error in &validation_errors {
                UserFeedback::error(&format!("• {}", error));
            }
            return Err(WebMockError::config("Multiple validation errors"));
        }

        Ok(())
    }

    /// Check system requirements and provide guidance
    pub fn check_system_requirements() -> Result<()> {
        let mut issues = Vec::new();

        // Check available disk space
        if let Err(e) = Self::check_disk_space() {
            issues.push(format!("Disk space: {}", e));
        }

        // Check network connectivity
        if let Err(e) = Self::check_network_connectivity() {
            issues.push(format!("Network: {}", e));
        }

        // Check permissions
        if let Err(e) = Self::check_permissions() {
            issues.push(format!("Permissions: {}", e));
        }

        if !issues.is_empty() {
            UserFeedback::warning("System requirement issues detected:");
            for issue in &issues {
                UserFeedback::warning(&format!("• {}", issue));
            }
            UserFeedback::tip("These issues may cause problems during operation");
        }

        Ok(())
    }

    /// Check available disk space
    fn check_disk_space() -> Result<()> {
        // This is a simplified check - in a real implementation you'd use platform-specific APIs
        let home_dir = dirs::home_dir()
            .ok_or_else(|| WebMockError::config("Cannot determine home directory"))?;

        let webmock_dir = home_dir.join(".webmock");

        // Try to create a small test file to check write permissions and space
        if let Ok(mut test_file) = std::fs::File::create(webmock_dir.join(".test_write")) {
            use std::io::Write;
            if test_file.write_all(b"test").is_ok() {
                let _ = std::fs::remove_file(webmock_dir.join(".test_write"));
                return Ok(());
            }
        }

        Err(WebMockError::permission_denied(
            "Cannot write to storage directory",
        ))
    }

    /// Check basic network connectivity
    fn check_network_connectivity() -> Result<()> {
        // This is a basic check - we just verify we can resolve DNS
        use std::net::ToSocketAddrs;

        if "google.com:80".to_socket_addrs().is_ok() {
            Ok(())
        } else {
            Err(WebMockError::config("Network connectivity issues detected"))
        }
    }

    /// Check file system permissions
    pub fn check_permissions() -> Result<()> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| WebMockError::config("Cannot determine home directory"))?;

        let webmock_dir = home_dir.join(".webmock");

        // Check if we can create the directory
        if !webmock_dir.exists() {
            std::fs::create_dir_all(&webmock_dir)
                .map_err(|_| WebMockError::permission_denied("Cannot create storage directory"))?;
        }

        // Check if directory is writable
        let test_file = webmock_dir.join(".permission_test");
        std::fs::write(&test_file, b"test")
            .map_err(|_| WebMockError::permission_denied("Storage directory is not writable"))?;

        std::fs::remove_file(&test_file)
            .map_err(|_| WebMockError::permission_denied("Cannot remove test file"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(ValidationHelper::validate_url("https://example.com").is_ok());
        assert!(ValidationHelper::validate_url("http://localhost:8080").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(ValidationHelper::validate_url("").is_err());
        assert!(ValidationHelper::validate_url("ftp://example.com").is_err());
        assert!(ValidationHelper::validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_validate_snapshot_name_valid() {
        assert!(ValidationHelper::validate_snapshot_name("test-snapshot").is_ok());
        assert!(ValidationHelper::validate_snapshot_name("test_snapshot_123").is_ok());
    }

    #[test]
    fn test_validate_snapshot_name_invalid() {
        assert!(ValidationHelper::validate_snapshot_name("").is_err());
        assert!(ValidationHelper::validate_snapshot_name("test/invalid").is_err());
        assert!(ValidationHelper::validate_snapshot_name("test snapshot").is_err());
        assert!(ValidationHelper::validate_snapshot_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_timeout() {
        assert!(ValidationHelper::validate_timeout(30).is_ok());
        assert!(ValidationHelper::validate_timeout(0).is_err());
        assert!(ValidationHelper::validate_timeout(700).is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(ValidationHelper::validate_port(8080).is_ok());
        assert!(ValidationHelper::validate_port(80).is_ok());
    }
}
