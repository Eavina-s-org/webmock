use crate::capture::browser::BrowserController;
use crate::capture::proxy::HttpProxy;
use crate::capture::validation::validate_url;
use crate::capture::CaptureSession;
use crate::error::{Result, WebMockError};
use crate::storage::Snapshot;
use chrono::Utc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

impl CaptureSession {
    /// Start the capture process for the given URL and snapshot name
    /// This orchestrates the entire capture session:
    /// 1. Start HTTP proxy server
    /// 2. Launch browser with proxy configuration
    /// 3. Navigate to target URL
    /// 4. Wait for page load and network requests
    /// 5. Record all HTTP traffic
    pub async fn capture(&mut self, url: &str, name: &str, timeout_seconds: u64) -> Result<()> {
        info!(
            "Starting capture session for URL: {} with name: {}",
            url, name
        );

        // Validate URL format early
        validate_url(url)?;

        // Step 1: Start HTTP proxy server
        info!("Step 1/4: Starting HTTP proxy server");
        let proxy_port = self.find_available_port().await?;
        let proxy = HttpProxy::start(proxy_port).await.map_err(|e| {
            error!("Failed to start proxy server on port {}: {}", proxy_port, e);
            e
        })?;

        self.proxy_port = proxy_port;
        self.proxy = Some(proxy);
        info!("HTTP proxy started successfully on port {}", proxy_port);

        // Give the proxy server time to fully initialize
        tokio::time::sleep(Duration::from_millis(1000)).await;
        info!("Proxy server initialization complete");

        // Step 2: Launch browser with proxy configuration
        info!("Step 2/4: Launching browser with proxy configuration");
        let browser = BrowserController::new(proxy_port).await.map_err(|e| {
            error!("Failed to launch browser: {}", e);
            e
        })?;

        self.browser = Some(browser);
        info!("Browser launched successfully");

        // Step 3: Navigate to target URL with timeout
        info!("Step 3/4: Navigating to target URL: {}", url);
        let navigation_result = timeout(
            Duration::from_secs(timeout_seconds),
            self.navigate_and_wait(url),
        )
        .await;

        match navigation_result {
            Ok(Ok(_)) => {
                info!("Successfully navigated to {} and page loaded", url);
            }
            Ok(Err(e)) => {
                error!("Navigation failed: {}", e);
                return Err(e);
            }
            Err(_) => {
                error!("Navigation timed out after {} seconds", timeout_seconds);
                return Err(WebMockError::Timeout(timeout_seconds));
            }
        }

        // Step 4: Wait for additional network requests to complete
        info!("Step 4/4: Waiting for network requests to settle");
        self.wait_for_network_idle().await?;

        info!("Capture session completed successfully for {}", url);
        Ok(())
    }

    /// Stop the capture session and return the recorded snapshot
    pub async fn stop(&mut self, name: &str, url: &str) -> Result<Snapshot> {
        info!("Stopping capture session and creating snapshot: {}", name);

        // Get recorded requests from proxy
        let requests = if let Some(proxy) = &self.proxy {
            let records = proxy.get_records().await;
            info!("Captured {} HTTP requests", records.len());

            // Debug: show what types of requests were captured
            let request_types: std::collections::HashMap<String, usize> =
                records
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut acc, req| {
                        *acc.entry(req.method.clone()).or_insert(0) += 1;
                        acc
                    });
            info!("Request types captured: {:?}", request_types);

            // Show first few requests for debugging
            for (i, record) in records.iter().take(5).enumerate() {
                info!("Request {}: {} {}", i, record.method, record.url);
            }

            // Warn if no requests were captured for HTTPS (this shouldn't happen with CONNECT support)
            if url.starts_with("https://") && records.is_empty() {
                warn!(
                    "No network requests recorded for HTTPS URL - this may indicate a proxy issue"
                );
            }

            records
        } else {
            warn!("No proxy found, returning empty request list");
            Vec::new()
        };

        // Create snapshot
        let snapshot = Snapshot {
            name: name.to_string(),
            url: url.to_string(),
            created_at: Utc::now(),
            requests,
        };

        // Save snapshot to storage
        info!("Saving snapshot to storage");
        self.storage
            .save_snapshot(snapshot.clone())
            .await
            .map_err(|e| {
                error!("Failed to save snapshot: {}", e);
                e
            })?;

        // Cleanup resources
        self.cleanup().await?;

        info!(
            "Capture session stopped and snapshot '{}' saved successfully",
            name
        );
        Ok(snapshot)
    }

    /// Navigate to URL and wait for page load
    pub(crate) async fn navigate_and_wait(&mut self, url: &str) -> Result<()> {
        if let Some(browser) = &mut self.browser {
            // Give the proxy a moment to start up properly
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Test proxy connectivity first
            info!("Testing proxy connectivity before navigation");
            let proxy_port = self.proxy_port;
            if let Err(e) = Self::test_proxy_connectivity_static(proxy_port).await {
                warn!("Proxy connectivity test failed: {}", e);
            } else {
                info!("Proxy connectivity test passed");
            }

            // Navigate to the URL with retry logic
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 3;

            while attempts < MAX_ATTEMPTS {
                attempts += 1;
                info!(
                    "Navigation attempt {} of {} to: {}",
                    attempts, MAX_ATTEMPTS, url
                );

                match browser.navigate(url).await {
                    Ok(_) => {
                        info!("Navigation successful, waiting for page load");
                        // Wait for page to fully load
                        match browser.wait_for_load().await {
                            Ok(_) => {
                                info!("Page load completed successfully");
                                return Ok(());
                            }
                            Err(e) => {
                                warn!("Page load failed on attempt {}: {}", attempts, e);
                                if attempts >= MAX_ATTEMPTS {
                                    return Err(e);
                                }
                                // Wait before retry
                                tokio::time::sleep(Duration::from_secs(2)).await;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Navigation failed on attempt {}: {}", attempts, e);
                        if attempts >= MAX_ATTEMPTS {
                            return Err(e);
                        }
                        // Wait before retry
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }

            Err(WebMockError::config("Navigation failed after all attempts"))
        } else {
            error!("Browser not initialized");
            Err(WebMockError::config("Browser not initialized"))
        }
    }

    /// Test proxy connectivity by making a simple request
    async fn test_proxy_connectivity_static(proxy_port: u16) -> Result<()> {
        use std::net::SocketAddr;
        use tokio::net::TcpStream;

        let proxy_addr: SocketAddr = format!("127.0.0.1:{}", proxy_port)
            .parse()
            .map_err(|e| WebMockError::config(format!("Invalid proxy address: {}", e)))?;

        match TcpStream::connect(proxy_addr).await {
            Ok(_) => {
                info!(
                    "Proxy server is accepting connections on port {}",
                    proxy_port
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to proxy on port {}: {}", proxy_port, e);
                Err(WebMockError::config(format!(
                    "Proxy connection test failed: {}",
                    e
                )))
            }
        }
    }
}
