use colored::*;
use std::io::{self, Write};

/// Core user feedback system
pub struct UserFeedback;

impl UserFeedback {
    /// Display a success message
    pub fn success(message: &str) {
        println!("{} {}", "✅".bright_green(), message.bright_green());
    }

    /// Display an error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "❌".bright_red(), message.bright_red());
    }

    /// Display a warning message
    pub fn warning(message: &str) {
        println!("{} {}", "⚠️".bright_yellow(), message.bright_yellow());
    }

    /// Display an info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ️".bright_blue(), message.bright_blue());
    }

    /// Display a progress message
    pub fn progress(message: &str) {
        print!("{} {}... ", "⏳".bright_cyan(), message.bright_cyan());
        io::stdout().flush().unwrap();
    }

    /// Display a completed message
    pub fn completed(message: &str) {
        println!("{} {}", "✨".bright_magenta(), message.bright_magenta());
    }

    /// Ask for user confirmation
    pub fn confirm(prompt: &str) -> io::Result<bool> {
        print!("{} {} [y/N]: ", "❓".bright_cyan(), prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        Ok(input == "y" || input == "yes")
    }

    /// Display a loading spinner (simplified)
    pub fn loading(message: &str) {
        println!("{} {}", "🔄".bright_blue(), message);
    }

    /// Display a tip message
    pub fn tip(message: &str) {
        println!("{} {}", "💡".bright_yellow(), message.bright_yellow());
    }

    /// Display a section header
    pub fn section(message: &str) {
        println!(
            "\n{} {}",
            "📍".bright_cyan().bold(),
            message.bright_cyan().bold()
        );
        println!("{}", "─".repeat(message.len() + 4).bright_black());
    }

    /// Display a separator line
    pub fn separator() {
        println!("{}", "═".repeat(60).bright_black());
    }

    /// Show command-specific help
    pub fn show_command_help(command: &str) {
        match command {
            "capture" => {
                Self::section("Capture Command Help");
                Self::info("Usage: webmock capture [OPTIONS]");
                Self::tip("Use --url to specify the target URL");
                Self::tip("Use --output to specify output directory");
                Self::tip("Use --browser to specify browser type (chrome, firefox, safari)");
            }
            "serve" => {
                Self::section("Serve Command Help");
                Self::info("Usage: webmock serve [OPTIONS]");
                Self::tip("Use --port to specify the port number");
                Self::tip("Use --host to specify the host address");
                Self::tip("Use --tls to enable HTTPS");
            }
            "list" => {
                Self::section("List Command Help");
                Self::info("Usage: webmock list [OPTIONS]");
                Self::tip("Shows all captured snapshots");
                Self::tip("Use --format to specify output format (table, json)");
            }
            "delete" => {
                Self::section("Delete Command Help");
                Self::info("Usage: webmock delete [SNAPSHOT_ID]");
                Self::tip("Deletes a specific snapshot");
                Self::warning("This action cannot be undone!");
            }
            _ => {
                Self::warning("Unknown command");
                Self::tip("Available commands: capture, serve, list, delete");
            }
        }
    }

    /// Show troubleshooting guide
    pub fn show_troubleshooting_guide() {
        Self::section("Troubleshooting Guide");
        Self::info("Common issues and solutions:");
        Self::tip("Browser not found: Install Chrome, Firefox, or Safari");
        Self::tip("Permission denied: Run with appropriate permissions or check file access");
        Self::tip("Network timeout: Check internet connection and firewall settings");
        Self::tip("Port already in use: Try a different port number");
    }

    /// Show system requirements
    pub fn show_system_requirements() {
        Self::section("System Requirements");
        Self::info("Minimum requirements:");
        Self::tip("Operating System: macOS 10.15+, Windows 10+, or Linux");
        Self::tip("RAM: 4GB minimum, 8GB recommended");
        Self::tip("Disk Space: 1GB free space");
        Self::tip("Browser: Chrome 90+, Firefox 88+, or Safari 14+");
    }

    /// Show performance tips
    pub fn show_performance_tips() {
        Self::section("Performance Tips");
        Self::info("Optimization suggestions:");
        Self::tip("Use headless mode for faster captures");
        Self::tip("Limit concurrent browser instances");
        Self::tip("Use SSD storage for better I/O performance");
        Self::tip("Close unnecessary applications to free memory");
        Self::tip("Use wired connection instead of WiFi when possible");
    }
}
