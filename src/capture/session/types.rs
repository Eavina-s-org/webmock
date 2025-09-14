use crate::capture::browser::BrowserController;
use crate::capture::proxy::HttpProxy;
use crate::capture::ResourceManager;
use crate::error::Result;
use crate::storage::Storage;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Main capture session that coordinates browser and proxy components
pub struct CaptureSession {
    pub(crate) browser: Option<BrowserController>,
    pub(crate) proxy: Option<HttpProxy>,
    pub(crate) storage: Arc<Storage>,
    pub(crate) proxy_port: u16,
    pub(crate) resource_manager: Arc<ResourceManager>,
}

impl CaptureSession {
    /// Create a new capture session with the given storage backend
    pub async fn new(storage: Arc<Storage>) -> Result<Self> {
        info!("Creating new capture session");
        let resource_manager = Arc::new(ResourceManager::new());

        Ok(Self {
            browser: None,
            proxy: None,
            storage,
            proxy_port: 0, // Will be set when proxy starts
            resource_manager,
        })
    }

    /// Get the current proxy port (for testing/debugging)
    pub fn get_proxy_port(&self) -> u16 {
        self.proxy_port
    }

    /// Get the number of recorded requests (for progress reporting)
    pub async fn get_request_count(&self) -> usize {
        if let Some(proxy) = &self.proxy {
            proxy.get_records().await.len()
        } else {
            0
        }
    }

    /// Check if capture session is active
    pub fn is_active(&self) -> bool {
        self.browser.is_some() && self.proxy.is_some()
    }

    /// Cleanup browser and proxy resources
    pub(crate) async fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up capture session resources");

        // Shutdown resource manager first (this will abort any running tasks)
        self.resource_manager.shutdown().await;

        // Close browser first
        if let Some(browser) = self.browser.take() {
            debug!("Closing browser");
            if let Err(e) = browser.close().await {
                warn!("Failed to close browser gracefully: {}", e);
            }
        }

        // Stop proxy server
        if let Some(proxy) = self.proxy.take() {
            debug!("Stopping proxy server");
            if let Err(e) = proxy.stop().await {
                warn!("Failed to stop proxy server gracefully: {}", e);
            }
        }

        // Wait a bit for cleanup to complete
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        info!("Cleanup completed");
        Ok(())
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        debug!("CaptureSession dropped, resources should be cleaned up");
    }
}
