use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::session::CaptureSession;
use crate::error::{Result, WebMockError};

impl CaptureSession {
    /// Find an available port for the proxy server
    pub async fn find_available_port(&self) -> Result<u16> {
        use std::net::{SocketAddr, TcpListener};

        // Try a wider range of ports to avoid conflicts
        let port_ranges = [
            8000..9000,   // Primary range
            9000..10000,  // Secondary range
            10000..11000, // Tertiary range
        ];

        for range in port_ranges {
            for port in range {
                let addr = SocketAddr::from(([127, 0, 0, 1], port));
                if TcpListener::bind(addr).is_ok() {
                    debug!("Found available port: {}", port);
                    return Ok(port);
                }
            }
        }

        error!("No available ports found in any range");
        Err(WebMockError::config(
            "No available ports found for proxy server",
        ))
    }

    /// Wait for network requests to settle (network idle detection)
    pub(crate) async fn wait_for_network_idle(&self) -> Result<()> {
        debug!("Waiting for network idle state");

        let mut last_request_count = 0;
        let mut stable_count = 0;
        const REQUIRED_STABLE_ITERATIONS: u32 = 3;
        const CHECK_INTERVAL_MS: u64 = 500;
        const MAX_WAIT_TIME_MS: u64 = 10000; // 10 seconds max

        let start_time = std::time::Instant::now();

        while start_time.elapsed().as_millis() < MAX_WAIT_TIME_MS as u128 {
            let current_request_count = if let Some(proxy) = &self.proxy {
                proxy.get_records().await.len()
            } else {
                0
            };

            if current_request_count == last_request_count {
                stable_count += 1;
                debug!(
                    "Network stable iteration {}/{}",
                    stable_count, REQUIRED_STABLE_ITERATIONS
                );

                if stable_count >= REQUIRED_STABLE_ITERATIONS {
                    info!(
                        "Network idle detected after {} requests",
                        current_request_count
                    );
                    break;
                }
            } else {
                debug!(
                    "New requests detected: {} -> {}",
                    last_request_count, current_request_count
                );
                stable_count = 0;
                last_request_count = current_request_count;
            }

            sleep(Duration::from_millis(CHECK_INTERVAL_MS)).await;
        }

        if start_time.elapsed().as_millis() >= MAX_WAIT_TIME_MS as u128 {
            warn!(
                "Network idle detection timed out after {}ms",
                MAX_WAIT_TIME_MS
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_find_available_port() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
        let session = CaptureSession::new(storage).await.unwrap();

        let port = session.find_available_port().await;
        assert!(port.is_ok());

        let port_num = port.unwrap();
        assert!((8000..11000).contains(&port_num));
    }

    #[tokio::test]
    async fn test_wait_for_network_idle_no_proxy() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path().to_path_buf()));
        let session = CaptureSession::new(storage).await.unwrap();

        // Should complete quickly when no proxy is active
        let result = session.wait_for_network_idle().await;
        assert!(result.is_ok());
    }
}
