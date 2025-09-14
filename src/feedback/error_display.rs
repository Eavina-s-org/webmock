use crate::error::WebMockError;
use crate::feedback::UserFeedback;
use console::Term;

/// Enhanced error display with context and suggestions
pub struct ErrorDisplay;

impl ErrorDisplay {
    /// Display an error with enhanced formatting and suggestions
    pub fn show_error(error: &WebMockError) {
        // Clear any progress indicators
        let term = Term::stderr();
        let _ = term.clear_line();

        // Show the main error
        UserFeedback::error(&error.user_message());

        // Show context and suggestions based on error type
        Self::show_error_context(error);

        // Show recovery suggestions if applicable
        if error.is_recoverable() {
            Self::show_recovery_suggestions(error);
        }
    }

    /// Show additional context for specific error types
    fn show_error_context(error: &WebMockError) {
        match error {
            WebMockError::ChromeNotFound => {
                println!();
                UserFeedback::info("Chrome browser is required for web page capture");
                println!("Please install one of the following:");
                println!("  • Google Chrome: https://www.google.com/chrome/");
                println!("  • Chromium: https://www.chromium.org/");
                println!();
                UserFeedback::tip("On macOS: brew install --cask google-chrome");
                UserFeedback::tip("On Ubuntu: sudo apt install chromium-browser");
            }
            WebMockError::PortInUse(port) => {
                println!();
                UserFeedback::info(&format!(
                    "Port {} is currently being used by another service",
                    port
                ));
                println!("You can:");
                println!("  • Use a different port: --port <PORT>");
                println!("  • Stop the service using port {}", port);
                println!("  • Let WebMock find an available port automatically");
            }
            WebMockError::PermissionDenied(_) => {
                println!();
                UserFeedback::info("This operation requires file system permissions");
                println!("Possible solutions:");
                println!("  • Check that ~/.webmock directory is writable");
                println!("  • Run with appropriate permissions");
                println!("  • Check disk space availability");
            }
            WebMockError::InvalidUrl(url, _) => {
                println!();
                UserFeedback::info("URL format requirements:");
                println!("  • Must start with http:// or https://");
                println!("  • Must be a valid, accessible URL");
                println!("  • Example: https://example.com");
                if !url.is_empty() {
                    UserFeedback::tip(&format!("You provided: {}", url));
                }
            }
            WebMockError::SnapshotNotFound(name) => {
                println!();
                UserFeedback::tip("Use 'webmock list' to see available snapshots");
                UserFeedback::tip(&format!("Check spelling: '{}'", name));
            }
            WebMockError::Timeout(seconds) => {
                println!();
                UserFeedback::info(&format!("Operation timed out after {} seconds", seconds));
                println!("Possible solutions:");
                println!("  • Increase timeout: --timeout <SECONDS>");
                println!("  • Check network connectivity");
                println!("  • Try a simpler page first");
            }
            _ => {}
        }
    }

    /// Show recovery suggestions for recoverable errors
    fn show_recovery_suggestions(error: &WebMockError) {
        println!();
        UserFeedback::section("🔄 Recovery Suggestions");

        match error {
            WebMockError::PortInUse(port) => {
                println!(
                    "• Try a different port: webmock serve <snapshot> --port {}",
                    port + 1
                );
                println!("• Let WebMock auto-select: webmock serve <snapshot>");
                println!("• Stop conflicting service and retry");
                Self::show_port_troubleshooting(*port);
            }
            WebMockError::Timeout(seconds) => {
                println!(
                    "• Increase timeout: webmock capture <url> --name <name> --timeout {}",
                    seconds * 2
                );
                println!("• Check internet connection speed");
                println!("• Try capturing a simpler page first");
                println!("• Capture during off-peak hours");
                Self::show_network_troubleshooting();
            }
            WebMockError::Browser(_) => {
                println!("• Restart Chrome/Chromium browser");
                println!("• Check browser installation");
                println!("• Wait a few seconds and retry");
                println!("• Close other browser instances");
                Self::show_browser_troubleshooting();
            }
            WebMockError::ChromeNotFound => {
                println!("• Install Google Chrome or Chromium");
                println!("• Ensure browser is in system PATH");
                println!("• Restart terminal after installation");
                Self::show_chrome_installation_quick_guide();
            }
            WebMockError::PermissionDenied(_) => {
                println!("• Check ~/.webmock directory permissions");
                println!("• Ensure sufficient disk space");
                println!("• Run with appropriate user privileges");
                println!("• Check parent directory permissions");
            }
            WebMockError::SnapshotNotFound(name) => {
                println!("• Check spelling: '{}'", name);
                println!("• List available snapshots: webmock list");
                println!(
                    "• Create the snapshot: webmock capture <url> --name {}",
                    name
                );
            }
            WebMockError::InvalidUrl(url, _) => {
                println!("• Ensure URL starts with http:// or https://");
                println!("• Check URL spelling and format");
                println!("• Test URL in browser first");
                if !url.is_empty() {
                    println!("• Your URL: {}", url);
                }
            }
            _ => {
                println!("• Check the error message above for specific guidance");
                println!("• Try the operation again after a short wait");
                println!("• Use --help for command usage information");
                println!("• Enable debug logging: RUST_LOG=debug webmock <command>");
            }
        }

        // Always show general help
        println!();
        UserFeedback::tip("For more help: webmock --help");
        UserFeedback::tip("Report issues: https://github.com/webmock-cli/webmock-cli/issues");
    }

    /// Show port-specific troubleshooting
    fn show_port_troubleshooting(port: u16) {
        println!();
        UserFeedback::info(&format!("Port {} Troubleshooting:", port));

        #[cfg(unix)]
        {
            println!("• Find what's using the port: lsof -i :{}", port);
            println!("• Kill process by PID: kill <PID>");
        }

        #[cfg(windows)]
        {
            println!(
                "• Find what's using the port: netstat -ano | findstr :{}",
                port
            );
            println!("• Kill process by PID: taskkill /PID <PID> /F");
        }

        println!("• Common port users:");
        match port {
            8080 => println!("  - Development servers, proxies, Jenkins"),
            3000 => println!("  - React dev server, Express.js"),
            8000 => println!("  - Django dev server, Python HTTP server"),
            9000 => println!("  - Various development tools"),
            _ => println!("  - Web servers, development tools, other applications"),
        }
    }

    /// Show network troubleshooting
    fn show_network_troubleshooting() {
        println!();
        UserFeedback::info("Network Troubleshooting:");
        println!("• Test connectivity: ping google.com");
        println!("• Check DNS: nslookup <target-domain>");
        println!("• Try with VPN disabled");
        println!("• Check firewall settings");
        println!("• Test target URL in browser");
    }

    /// Show browser troubleshooting
    fn show_browser_troubleshooting() {
        println!();
        UserFeedback::info("Browser Troubleshooting:");
        println!("• Close all Chrome/Chromium instances");
        println!("• Clear browser cache and data");
        println!("• Disable browser extensions");
        println!("• Try incognito/private mode");
        println!("• Update browser to latest version");
    }

    /// Show quick Chrome installation guide
    fn show_chrome_installation_quick_guide() {
        println!();
        UserFeedback::info("Quick Installation:");

        #[cfg(target_os = "macos")]
        println!("• macOS: brew install --cask google-chrome");

        #[cfg(target_os = "linux")]
        println!("• Ubuntu/Debian: sudo apt install google-chrome-stable");

        #[cfg(target_os = "windows")]
        println!("• Windows: Download from https://www.google.com/chrome/");

        println!("• Alternative: Install Chromium (open-source version)");
    }
}
