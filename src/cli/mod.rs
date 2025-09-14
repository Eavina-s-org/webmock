#[cfg(test)]
mod tests;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Parser)]
#[command(name = "webmock")]
#[command(about = "A CLI tool for recording and mocking web pages")]
#[command(version = "0.1.0")]
#[command(
    long_about = "WebMock CLI allows you to capture complete web page sessions (including all API requests and responses) and replay them locally for development, testing, and demonstration purposes.

KEY FEATURES:
    🎥 Complete session recording (HTML, CSS, JS, images, API calls)
    🚀 Local mock server for offline development
    📦 Efficient compressed storage
    🔧 Developer-friendly CLI with progress indicators

QUICK START:
    # 1. Capture a web page
    webmock capture https://example.com --name my-site

    # 2. Start mock server  
    webmock serve my-site --port 8080

    # 3. Visit http://localhost:8080

COMMON WORKFLOWS:
    # Development against production APIs
    webmock capture https://app.com/dashboard --name prod-api --timeout 60
    webmock serve prod-api --port 8080

    # Testing with different data states  
    webmock capture https://app.com?user=admin --name admin-state
    webmock capture https://app.com?user=guest --name guest-state
    webmock serve admin-state --port 8081 & 
    webmock serve guest-state --port 8082 &

    # Demo preparation (works offline)
    webmock capture https://demo.myapp.com --name perfect-demo --timeout 90
    webmock serve perfect-demo --port 8080

    # CI/CD integration
    webmock capture https://api.com --name ci-baseline --timeout 120
    webmock serve ci-baseline --port 8080 &
    npm test  # Tests run against localhost:8080

MANAGEMENT:
    webmock list                    # View all snapshots
    webmock delete old-snapshot     # Clean up storage

REQUIREMENTS:
    • Google Chrome or Chromium browser
    • Internet connection (for capture)
    • ~/.webmock directory (auto-created)

For detailed help on any command:
    webmock <command> --help

For troubleshooting and examples:
    https://github.com/your-org/webmock-cli/blob/main/README.md"
)]
pub struct Cli {
    /// Generate shell completion script
    #[arg(long, value_enum, hide = true)]
    pub generate_completion: Option<Shell>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Capture a web page and all its network requests
    #[command(
        long_about = "Capture a complete web page session including all network requests (HTML, CSS, JS, images, API calls) and save it as a named snapshot.

The capture process:
1. Starts a local HTTP proxy to intercept requests
2. Launches a headless Chrome browser configured to use the proxy
3. Navigates to the specified URL and waits for the page to load
4. Records all network traffic during the session
5. Saves everything as a compressed snapshot file

EXAMPLES:
    # Basic capture
    webmock capture https://example.com --name my-site

    # Capture with custom timeout
    webmock capture https://api.example.com/dashboard --name dashboard --timeout 60

    # Capture with custom storage directory
    webmock capture https://example.com --name my-site --storage /path/to/custom/storage

    # Capture a local development site
    webmock capture http://localhost:3000 --name local-app

REQUIREMENTS:
    • Google Chrome or Chromium browser must be installed
    • Internet connection (for external URLs)
    • Write permissions to the storage directory"
    )]
    Capture {
        /// The URL to capture (must be HTTP or HTTPS)
        #[arg(help = "The URL to capture (e.g., https://example.com)")]
        url: String,

        /// Name for the snapshot (alphanumeric, hyphens, underscores only)
        #[arg(long, help = "Name for the snapshot (e.g., my-site, api-v1)")]
        name: String,

        /// Timeout in seconds for the capture process (1-600)
        #[arg(
            long,
            default_value = "30",
            help = "Timeout in seconds (default: 30, max: 600)"
        )]
        timeout: u64,

        /// Custom storage directory path (default: ~/.webmock)
        #[arg(long, help = "Custom storage directory path (default: ~/.webmock)")]
        storage: Option<String>,
    },

    /// List all saved snapshots with details
    #[command(
        long_about = "Display all saved snapshots with their metadata including:
• Snapshot name and original URL
• Creation date and time
• Number of recorded requests
• File size information

This command helps you manage your snapshots and see what's available to serve or delete.

EXAMPLES:
    # List all snapshots
    webmock list

    # List snapshots from custom storage directory
    webmock list --storage /path/to/custom/storage

OUTPUT FORMAT:
    Each snapshot shows:
    📸 snapshot-name
       🌐 URL: https://original-url.com
       📅 Created: 2024-01-15 14:30:00 UTC
       📊 Requests: 25"
    )]
    List {
        /// Custom storage directory path (default: ~/.webmock)
        #[arg(long, help = "Custom storage directory path (default: ~/.webmock)")]
        storage: Option<String>,
    },

    /// Start a mock server serving a saved snapshot
    #[command(
        long_about = "Start a local HTTP server that replays a captured snapshot. The server will:
• Serve the main page at the root URL
• Return recorded responses for all captured requests
• Serve static resources (CSS, JS, images) as they were captured
• Return 404 for any requests not in the snapshot

The server runs until stopped with Ctrl+C and automatically handles:
• Port conflicts (finds alternative ports)
• Request routing and matching
• Content type detection
• CORS headers when needed

EXAMPLES:
    # Start server on default port (8080)
    webmock serve my-site

    # Start server on specific port
    webmock serve dashboard --port 3000

    # Start server with custom storage directory
    webmock serve my-site --storage /path/to/custom/storage

    # Server will show:
    🚀 Starting mock server...
       🌐 Server URL: http://localhost:8080
       🎯 Serving snapshot: my-site
       ⏹️  Press Ctrl+C to stop"
    )]
    Serve {
        /// Name of the snapshot to serve
        #[arg(
            help = "Name of the snapshot to serve (use 'webmock list' to see available snapshots)"
        )]
        snapshot_name: String,

        /// Port to run the server on (1024-65535)
        #[arg(
            long,
            default_value = "8080",
            help = "Port to run the server on (default: 8080)"
        )]
        port: u16,

        /// Custom storage directory path (default: ~/.webmock)
        #[arg(long, help = "Custom storage directory path (default: ~/.webmock)")]
        storage: Option<String>,
    },

    /// Delete a saved snapshot permanently
    #[command(
        long_about = "Permanently delete a saved snapshot from disk. This action cannot be undone.

The delete process:
1. Checks if the snapshot exists
2. Asks for confirmation (unless --force is used)
3. Removes the snapshot file from the storage directory/snapshots/

EXAMPLES:
    # Delete a snapshot
    webmock delete old-snapshot

    # Delete from custom storage directory
    webmock delete old-snapshot --storage /path/to/custom/storage
4. Reports success or failure

EXAMPLES:
    # Delete with confirmation prompt
    webmock delete old-snapshot

    # The command will ask:
    Are you sure you want to delete snapshot 'old-snapshot'? 
    This action cannot be undone. (y/N):

SAFETY:
    • Always asks for confirmation before deletion
    • Validates snapshot exists before prompting
    • Provides clear error messages if snapshot not found"
    )]
    Delete {
        /// Name of the snapshot to delete
        #[arg(
            help = "Name of the snapshot to delete (use 'webmock list' to see available snapshots)"
        )]
        snapshot_name: String,

        /// Custom storage directory path (default: ~/.webmock)
        #[arg(long, help = "Custom storage directory path (default: ~/.webmock)")]
        storage: Option<String>,
    },

    /// Inspect a saved snapshot's details and contents
    #[command(
        long_about = "Display detailed information about a saved snapshot including:
• Original URL and capture metadata
• List of all recorded requests with methods and paths
• Response status codes and content types
• File sizes and timing information

This command helps you understand what was captured in a snapshot before serving or deleting it.

EXAMPLES:
    # Inspect snapshot details
    webmock inspect my-site

    # Inspect from custom storage directory
    webmock inspect my-site --storage /path/to/custom/storage

OUTPUT FORMAT:
    📸 Snapshot: my-site
       🌐 Original URL: https://example.com
       📅 Created: 2024-01-15 14:30:00 UTC
       ⏱️  Capture duration: 15.3s
       
       📊 Recorded requests:
       GET  /              200  text/html    12.3 KB
       GET  /styles.css    200  text/css     45.6 KB
       GET  /api/data      200  application/json  2.1 KB"
    )]
    Inspect {
        /// Name of the snapshot to inspect
        #[arg(
            help = "Name of the snapshot to inspect (use 'webmock list' to see available snapshots)"
        )]
        snapshot_name: String,

        /// Custom storage directory path (default: ~/.webmock)
        #[arg(long, help = "Custom storage directory path (default: ~/.webmock)")]
        storage: Option<String>,
    },
}
