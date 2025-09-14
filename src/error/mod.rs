#[cfg(test)]
mod tests;

use thiserror::Error;

/// Main error type for WebMock CLI operations
#[derive(Debug, Error)]
pub enum WebMockError {
    /// Browser automation errors
    #[error("Browser error: {0}")]
    Browser(#[from] Box<chromiumoxide::error::CdpError>),

    /// HTTP proxy server errors
    #[error("Proxy server error: {0}")]
    Proxy(String),

    /// File system and storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    /// Snapshot not found error
    #[error("Snapshot '{0}' not found")]
    SnapshotNotFound(String),

    /// Port already in use error
    #[error("Port {0} is already in use. Please try a different port or stop the service using this port")]
    PortInUse(u16),

    /// Invalid URL format error
    #[error("Invalid URL '{0}': {1}")]
    InvalidUrl(String, String),

    /// Network timeout error
    #[error("Network timeout after {0} seconds")]
    Timeout(u64),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    /// Deserialization errors
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    /// HTTP server errors
    #[error("HTTP server error: {0}")]
    HttpServer(#[from] hyper::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Permission denied errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Chrome browser not found
    #[error("Chrome browser not found. Please install Google Chrome or Chromium")]
    ChromeNotFound,

    /// Invalid snapshot data
    #[error("Invalid snapshot data: {0}")]
    InvalidSnapshot(String),

    /// Command execution errors
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

impl WebMockError {
    /// Create a new proxy error
    pub fn proxy<S: Into<String>>(msg: S) -> Self {
        WebMockError::Proxy(msg.into())
    }

    /// Create a new config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        WebMockError::Config(msg.into())
    }

    /// Create a new permission denied error
    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        WebMockError::PermissionDenied(msg.into())
    }

    /// Create a new invalid snapshot error
    pub fn invalid_snapshot<S: Into<String>>(msg: S) -> Self {
        WebMockError::InvalidSnapshot(msg.into())
    }

    /// Create a new command failed error
    pub fn command_failed<S: Into<String>>(msg: S) -> Self {
        WebMockError::CommandFailed(msg.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            WebMockError::PortInUse(_) => true,
            WebMockError::Timeout(_) => true,
            WebMockError::SnapshotNotFound(_) => false,
            WebMockError::ChromeNotFound => false,
            WebMockError::PermissionDenied(_) => false,
            WebMockError::InvalidUrl(_, _) => false,
            _ => true,
        }
    }

    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            WebMockError::Browser(e) => {
                format!("Browser automation failed: {}. Try restarting the browser or check if Chrome is properly installed.", e)
            }
            WebMockError::Proxy(msg) => {
                format!("Network proxy error: {}. Check your network connection and firewall settings.", msg)
            }
            WebMockError::Storage(e) => {
                format!("File system error: {}. Check file permissions and available disk space.", e)
            }
            WebMockError::SnapshotNotFound(name) => {
                format!("Snapshot '{}' not found. Use 'webmock list' to see available snapshots.", name)
            }
            WebMockError::PortInUse(port) => {
                format!("Port {} is already in use. Try using a different port with --port option or stop the service using this port.", port)
            }
            WebMockError::InvalidUrl(url, reason) => {
                format!("Invalid URL '{}': {}. Please provide a valid HTTP or HTTPS URL.", url, reason)
            }
            WebMockError::Timeout(seconds) => {
                format!("Operation timed out after {} seconds. Try increasing the timeout with --timeout option.", seconds)
            }
            WebMockError::ChromeNotFound => {
                "Chrome browser not found. Please install Google Chrome or Chromium browser and ensure it's in your PATH.".to_string()
            }
            WebMockError::PermissionDenied(msg) => {
                format!("Permission denied: {}. Check file permissions or run with appropriate privileges.", msg)
            }
            WebMockError::InvalidSnapshot(msg) => {
                format!("Invalid snapshot data: {}. The snapshot file may be corrupted.", msg)
            }
            _ => self.to_string(),
        }
    }
}

/// Result type alias for WebMock operations
pub type Result<T> = std::result::Result<T, WebMockError>;

/// Convert from URL parsing errors
impl From<url::ParseError> for WebMockError {
    fn from(err: url::ParseError) -> Self {
        WebMockError::InvalidUrl("".to_string(), err.to_string())
    }
}

/// Convert from Tokio join errors
impl From<tokio::task::JoinError> for WebMockError {
    fn from(err: tokio::task::JoinError) -> Self {
        WebMockError::CommandFailed(format!("Task execution failed: {}", err))
    }
}

/// Convert from address parsing errors (for network operations)
impl From<std::net::AddrParseError> for WebMockError {
    fn from(err: std::net::AddrParseError) -> Self {
        WebMockError::Config(format!("Invalid network address: {}", err))
    }
}
