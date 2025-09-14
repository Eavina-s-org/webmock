use tracing::error;
use url::Url;

use crate::error::{Result, WebMockError};

/// Validate URL format and scheme
pub fn validate_url(url: &str) -> Result<()> {
    // Validate URL format early
    let parsed_url = Url::parse(url).map_err(|e| {
        error!("Invalid URL format: {}", e);
        WebMockError::InvalidUrl(url.to_string(), e.to_string())
    })?;

    // Ensure URL uses HTTP or HTTPS
    match parsed_url.scheme() {
        "http" | "https" => Ok(()),
        scheme => {
            error!("Unsupported URL scheme: {}", scheme);
            Err(WebMockError::InvalidUrl(
                url.to_string(),
                format!(
                    "Unsupported scheme '{}'. Only HTTP and HTTPS are supported",
                    scheme
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://example.com/path?query=value").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("file:///path/to/file").is_err());
        assert!(validate_url("mailto:test@example.com").is_err());
    }
}
