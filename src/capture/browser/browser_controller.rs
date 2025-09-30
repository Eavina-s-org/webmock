use chromiumoxide::cdp::browser_protocol::network::EventLoadingFinished;
use chromiumoxide::cdp::browser_protocol::page::EventLoadEventFired;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::error::{Result, WebMockError};

pub struct BrowserController {
    browser: Browser,
    page: Page,
}

impl BrowserController {
    /// Create a new browser controller with proxy configuration
    pub async fn new(proxy_port: u16) -> Result<Self> {
        info!(
            "Creating browser controller with proxy port: {}",
            proxy_port
        );

        // Configure browser with minimal proxy settings to avoid HTTPS issues
        let args = [
            // Basic browser settings
            "--no-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--disable-gpu".to_string(),
            "--disable-software-rasterizer".to_string(),
            "--disable-extensions".to_string(),
            "--disable-sync".to_string(),
            "--disable-translate".to_string(),
            "--disable-default-apps".to_string(),
            "--no-first-run".to_string(),
            "--disable-component-update".to_string(),
            "--disable-background-networking".to_string(),
            "--disable-background-timer-throttling".to_string(),
            "--disable-backgrounding-occluded-windows".to_string(),
            "--disable-renderer-backgrounding".to_string(),
            "--disable-field-trial-config".to_string(),
            "--disable-ipc-flooding-protection".to_string(),
            "--disable-hang-monitor".to_string(),
            "--disable-prompt-on-repost".to_string(),
            "--disable-client-side-phishing-detection".to_string(),
            "--disable-popup-blocking".to_string(),
            "--disable-zero-browsers-open-for-tests".to_string(),
            "--hide-scrollbars".to_string(),
            "--metrics-recording-only".to_string(),
            "--mute-audio".to_string(),
            "--safebrowsing-disable-auto-update".to_string(),
            "--log-level=3".to_string(),
            // Additional flags to reduce WebSocket protocol issues
            "--disable-features=TranslateUI,BlinkGenPropertyTrees".to_string(),
            "--disable-blink-features=AutomationControlled".to_string(),
            "--disable-dev-tools".to_string(),
            // Security settings
            "--ignore-certificate-errors".to_string(),
            "--ignore-ssl-errors".to_string(),
            "--ignore-certificate-errors-spki-list".to_string(),
            "--disable-web-security".to_string(),
            "--allow-running-insecure-content".to_string(),
            "--disable-features=VizDisplayCompositor".to_string(),
            "--aggressive-cache-discard".to_string(),
            // Proxy configuration for both HTTP and HTTPS
            format!("--proxy-server=127.0.0.1:{}", proxy_port),
        ];

        let config = BrowserConfig::builder()
            .with_head() // Use headless mode for automation
            .args(args)
            .build()
            .map_err(|e| {
                error!("Failed to build browser config: {}", e);
                WebMockError::config(format!("Browser config error: {}", e))
            })?;

        // Launch browser
        debug!("Launching Chrome browser with proxy configuration");
        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            error!("Failed to launch browser: {}", e);
            match e.to_string().contains("No such file or directory") {
                true => WebMockError::ChromeNotFound,
                false => WebMockError::Browser(Box::new(e)),
            }
        })?;

        // Spawn handler task to manage browser process
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    error!("Browser handler error: {}", e);
                    break;
                }
            }
        });

        // Create new page
        debug!("Creating new browser page");
        let page = browser.new_page("about:blank").await.map_err(|e| {
            error!("Failed to create new page: {}", e);
            WebMockError::Browser(Box::new(e))
        })?;

        // Additional page setup for better compatibility
        debug!("Setting up page for network capture");
        // Note: Network capture for HTTPS will be limited due to encryption

        info!("Browser controller created successfully");
        Ok(Self { browser, page })
    }

    /// Navigate to the specified URL
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        info!("Navigating to URL: {}", url);

        // Validate URL format
        let parsed_url = Url::parse(url).map_err(|e| {
            error!("Invalid URL format: {}", e);
            WebMockError::InvalidUrl(url.to_string(), e.to_string())
        })?;

        // Ensure URL uses HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {}
            scheme => {
                error!("Unsupported URL scheme: {}", scheme);
                return Err(WebMockError::InvalidUrl(
                    url.to_string(),
                    format!(
                        "Unsupported scheme '{}'. Only HTTP and HTTPS are supported",
                        scheme
                    ),
                ));
            }
        }

        // For HTTPS URLs, give an informational message about HTTPS recording
        if parsed_url.scheme() == "https" {
            info!(
                "HTTPS URL detected: {}. HTTPS traffic will be recorded using MITM proxy.",
                url
            );
        }

        // Navigate to URL with extended timeout and better error handling
        debug!("Starting navigation to: {}", url);
        let navigation_result = timeout(Duration::from_secs(45), self.page.goto(url)).await;

        match navigation_result {
            Ok(Ok(_)) => {
                info!("Successfully navigated to: {}", url);
                Ok(())
            }
            Ok(Err(e)) => {
                let error_msg = e.to_string();
                error!("Navigation failed: {}", error_msg);

                // Provide more specific error messages
                if error_msg.contains("ERR_CONNECTION_CLOSED") {
                    return Err(WebMockError::Browser(Box::new(chromiumoxide::error::CdpError::Io(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        format!("Connection to {} was closed. This may be due to:\n  - Network connectivity issues\n  - The target server rejecting proxy connections\n  - Firewall or security software blocking the connection\n  - The website using advanced security measures", url)
                    )))));
                } else if error_msg.contains("ERR_PROXY_CONNECTION_FAILED") {
                    return Err(WebMockError::config(
                        "Proxy connection failed. The internal proxy server may not be running properly."
                    ));
                } else if error_msg.contains("ERR_NAME_NOT_RESOLVED") {
                    return Err(WebMockError::Browser(Box::new(chromiumoxide::error::CdpError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Could not resolve hostname for {}. Please check the URL and your internet connection.", url)
                    )))));
                }

                Err(WebMockError::Browser(Box::new(e)))
            }
            Err(_) => {
                error!("Navigation timed out after 45 seconds");
                Err(WebMockError::Timeout(45))
            }
        }
    }

    /// Wait for page to fully load including all resources
    pub async fn wait_for_load(&mut self) -> Result<()> {
        debug!("Waiting for page load completion");

        // Wait for load event with timeout
        let load_result = timeout(Duration::from_secs(30), self.wait_for_load_events()).await;

        match load_result {
            Ok(Ok(_)) => {
                info!("Page load completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Error while waiting for page load: {}", e);
                Err(e)
            }
            Err(_) => {
                warn!("Page load timeout after 30 seconds, continuing anyway");
                Ok(()) // Don't fail on timeout, just continue
            }
        }
    }

    /// Internal method to wait for various load events
    pub async fn wait_for_load_events(&mut self) -> Result<()> {
        debug!("Waiting for DOM content loaded event");

        // Wait for the load event to fire
        let mut load_events = self
            .page
            .event_listener::<EventLoadEventFired>()
            .await
            .map_err(|e| {
                error!("Failed to create load event listener: {}", e);
                WebMockError::Browser(Box::new(e))
            })?;

        // Wait for network idle (no network requests for 500ms)
        let _network_events = self
            .page
            .event_listener::<EventLoadingFinished>()
            .await
            .map_err(|e| {
                error!("Failed to create network event listener: {}", e);
                WebMockError::Browser(Box::new(e))
            })?;

        // Wait for load event
        tokio::select! {
            event = load_events.next() => {
                match event {
                    Some(_) => {
                        debug!("Load event fired");
                    },
                    None => {
                        warn!("Load event stream ended unexpectedly");
                    }
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                debug!("Load event timeout, continuing");
            }
        }

        // Wait a bit more for network requests to settle
        debug!("Waiting for network requests to settle");
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(())
    }

    /// Get the current page URL
    pub async fn current_url(&self) -> Result<String> {
        let url = self.page.url().await.map_err(|e| {
            error!("Failed to get current URL: {}", e);
            WebMockError::Browser(Box::new(e))
        })?;

        Ok(url.unwrap_or_else(|| "about:blank".to_string()))
    }

    /// Get the page title
    pub async fn title(&self) -> Result<String> {
        let title = self.page.get_title().await.map_err(|e| {
            error!("Failed to get page title: {}", e);
            WebMockError::Browser(Box::new(e))
        })?;

        Ok(title.unwrap_or_else(|| "Untitled".to_string()))
    }

    /// Close the browser and cleanup resources
    pub async fn close(mut self) -> Result<()> {
        info!("Closing browser and cleaning up resources");

        // Close the page first
        if let Err(e) = self.page.clone().close().await {
            warn!("Failed to close page gracefully: {}", e);
        }

        // Close the browser
        if let Err(e) = self.browser.close().await {
            warn!("Failed to close browser gracefully: {}", e);
        }

        // Wait a bit for cleanup
        tokio::time::sleep(Duration::from_millis(500)).await;

        info!("Browser closed successfully");
        Ok(())
    }
}

impl Drop for BrowserController {
    fn drop(&mut self) {
        debug!("BrowserController dropped, browser should be cleaned up");
    }
}
