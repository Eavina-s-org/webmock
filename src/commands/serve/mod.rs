use colored::*;
use std::net::{SocketAddr, TcpListener};
use tokio::signal;

use crate::error::{Result, WebMockError};
use crate::feedback::{ProgressReporter, UserFeedback, ValidationHelper};
use crate::serve::MockServer;
use crate::storage::Storage;

/// Check if a port is available with detailed diagnostics
pub fn is_port_available(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpListener::bind(addr).is_ok()
}

/// Find an available port starting from the given port with progress feedback
pub fn find_available_port(start_port: u16) -> Result<u16> {
    const MAX_ATTEMPTS: u16 = 100;

    for attempt in 0..MAX_ATTEMPTS {
        let port = start_port + attempt;
        if is_port_available(port) {
            return Ok(port);
        }

        // Show progress for longer searches
        if attempt > 0 && attempt % 10 == 0 {
            UserFeedback::info(&format!(
                "Still searching for available port... (checked {} ports)",
                attempt + 1
            ));
        }
    }

    Err(WebMockError::PortInUse(start_port))
}

/// Get detailed information about what might be using a port
pub fn get_port_usage_info(port: u16) -> String {
    // Try to get process information (this is platform-specific)
    #[cfg(unix)]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output()
        {
            if !output.stdout.is_empty() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return format!("Port {} appears to be used by:\n{}", port, output_str);
            }
        }
    }

    format!("Port {} is in use by another process", port)
}

/// Check port availability and resolve conflicts with enhanced feedback
pub fn check_and_resolve_port(requested_port: u16) -> Result<u16> {
    // Validate port number first
    ValidationHelper::validate_port(requested_port)?;

    if is_port_available(requested_port) {
        UserFeedback::success(&format!("Port {} is available", requested_port));
        Ok(requested_port)
    } else {
        UserFeedback::warning(&format!("Port {} is already in use", requested_port));

        // Show detailed port usage information
        let usage_info = get_port_usage_info(requested_port);
        UserFeedback::info(&usage_info);

        UserFeedback::info("Searching for alternative port...");

        match find_available_port(requested_port + 1) {
            Ok(alternative_port) => {
                UserFeedback::success(&format!("Found alternative port: {}", alternative_port));
                UserFeedback::tip("You can specify a different port with --port <PORT>");
                UserFeedback::tip("To use the original port, stop the conflicting service first");
                Ok(alternative_port)
            }
            Err(e) => {
                UserFeedback::error(&format!(
                    "Could not find available port in range {}-{}",
                    requested_port + 1,
                    requested_port + 100
                ));

                // Enhanced troubleshooting suggestions
                UserFeedback::section("🔧 Troubleshooting Port Conflicts");
                UserFeedback::tip("Try a different port range: --port 9000");
                UserFeedback::tip("Check active connections: netstat -tulpn | grep LISTEN");

                #[cfg(unix)]
                UserFeedback::tip(&format!(
                    "Find what's using port {}: lsof -i :{}",
                    requested_port, requested_port
                ));

                #[cfg(windows)]
                UserFeedback::tip(&format!(
                    "Find what's using port {}: netstat -ano | findstr :{}",
                    requested_port, requested_port
                ));

                UserFeedback::tip("Common conflicting services: web servers, development tools, other WebMock instances");

                Err(e)
            }
        }
    }
}

/// Handle the serve command with enhanced feedback
pub async fn serve_command(
    snapshot_name: &str,
    requested_port: u16,
    storage_arg: Option<String>,
) -> Result<()> {
    // Step 0: Validate inputs
    UserFeedback::info("Validating inputs...");
    ValidationHelper::validate_snapshot_name(snapshot_name)?;
    ValidationHelper::validate_port(requested_port)?;
    UserFeedback::success("Input validation passed");

    // Initialize storage
    UserFeedback::info("Initializing storage...");
    let storage_path = crate::commands::get_storage_path(storage_arg)?;
    let storage = Storage::new(storage_path);

    // Load the snapshot with detailed status reporting and progress
    let progress = ProgressReporter::new();
    let loading_spinner = progress.create_spinner(&format!("Loading snapshot '{}'", snapshot_name));

    let snapshot = match storage.load_snapshot(snapshot_name).await {
        Ok(snapshot) => {
            loading_spinner.finish_with_message(format!("✅ Loaded snapshot '{}'", snapshot.name));

            // Display snapshot details
            UserFeedback::section("📋 Snapshot Details");
            println!("   📍 Original URL: {}", snapshot.url);
            println!("   📊 Recorded requests: {}", snapshot.requests.len());
            println!(
                "   📅 Created: {}",
                snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );

            snapshot
        }
        Err(e) => {
            loading_spinner.finish_with_message("❌ Failed to load snapshot");
            UserFeedback::error(&format!(
                "Failed to load snapshot '{}': {}",
                snapshot_name,
                e.user_message()
            ));

            if matches!(e, WebMockError::SnapshotNotFound(_)) {
                UserFeedback::tip("Use 'webmock list' to see available snapshots");
            }

            return Err(e);
        }
    };

    // Port availability checking and conflict resolution
    UserFeedback::info("Checking port availability...");
    let port = check_and_resolve_port(requested_port)?;

    // Create and start the mock server with enhanced status reporting
    let mock_server = MockServer::new(snapshot);

    UserFeedback::section("🚀 Starting Mock Server");
    println!(
        "   🌐 Server URL: {}",
        format!("http://localhost:{}", port).bright_green()
    );
    println!("   🎯 Serving snapshot: {}", snapshot_name);
    println!(
        "   ⏹️  Press {} to stop the server",
        "Ctrl+C".bright_yellow()
    );

    UserFeedback::separator();
    UserFeedback::info("Server logs:");

    // Set up graceful shutdown handling
    let server_future = mock_server.start(port);
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    // Run server with graceful shutdown
    tokio::select! {
        result = server_future => {
            match result {
                Ok(_) => {
                    UserFeedback::success("Server stopped normally");
                }
                Err(e) => {
                    UserFeedback::error(&format!("Server error: {}", e.user_message()));
                    return Err(e);
                }
            }
        }
        _ = shutdown_signal => {
            println!();
            UserFeedback::separator();
            UserFeedback::info("Received shutdown signal (Ctrl+C)");

            let shutdown_spinner = progress.create_spinner("Gracefully stopping server...");
            // Give a moment for any ongoing requests to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            shutdown_spinner.finish_with_message("✅ Server stopped successfully");
        }
    }

    Ok(())
}
