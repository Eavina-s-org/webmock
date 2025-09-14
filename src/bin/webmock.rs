use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::io;
use std::process;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

use webmock_cli::{
    cli::{Cli, Commands, Shell as CompletionShell},
    commands::{capture_command, delete_command, inspect_command, list_command, serve_command},
    error::{Result, WebMockError},
    feedback::{ErrorDisplay, UserFeedback},
};

#[tokio::main]
async fn main() {
    // Initialize logging with better error handling
    if let Err(e) = initialize_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle shell completion generation
    if let Some(shell) = cli.generate_completion {
        generate_completion(shell);
        return;
    }

    // Show welcome message for better user experience
    show_welcome_message();

    // Run the main application with comprehensive error handling
    match run(cli).await {
        Ok(()) => {
            // Success - exit normally
            process::exit(0);
        }
        Err(e) => {
            // Enhanced error display with recovery suggestions
            ErrorDisplay::show_error(&e);

            // Set appropriate exit code based on error type
            let exit_code = match &e {
                WebMockError::ChromeNotFound => 2,
                WebMockError::PermissionDenied(_) => 3,
                WebMockError::PortInUse(_) => 4,
                WebMockError::SnapshotNotFound(_) => 5,
                WebMockError::InvalidUrl(_, _) => 6,
                WebMockError::Config(_) => 7,
                _ => 1,
            };

            process::exit(exit_code);
        }
    }
}

/// Generate shell completion script
fn generate_completion(shell: CompletionShell) {
    let mut cmd = Cli::command();
    let shell_variant = match shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Elvish => Shell::Elvish,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::PowerShell => Shell::PowerShell,
        CompletionShell::Zsh => Shell::Zsh,
    };
    generate(shell_variant, &mut cmd, "webmock", &mut io::stdout());
}

/// Initialize logging with proper error handling
fn initialize_logging() -> std::result::Result<(), String> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| e.to_string())
}

/// Show a friendly welcome message
fn show_welcome_message() {
    // Only show welcome for interactive terminals and when not in CI
    if atty::is(atty::Stream::Stdout) && std::env::var("CI").is_err() {
        UserFeedback::info("🌐 WebMock CLI - Record and replay web pages locally");
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Check if a command was provided
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // No command provided, show help
            Cli::command().print_help().unwrap();
            println!();
            return Ok(());
        }
    };

    match command {
        Commands::Capture {
            url,
            name,
            timeout,
            storage,
        } => {
            info!("Starting capture for URL: {}", url);
            capture_command(&url, &name, timeout, storage).await?;
        }
        Commands::List { storage } => {
            info!("Listing snapshots");
            list_command(storage).await?;
        }
        Commands::Serve {
            snapshot_name,
            port,
            storage,
        } => {
            info!(
                "Starting server for snapshot: {} on port: {}",
                snapshot_name, port
            );
            serve_command(&snapshot_name, port, storage).await?;
        }
        Commands::Delete {
            snapshot_name,
            storage,
        } => {
            info!("Deleting snapshot: {}", snapshot_name);
            delete_command(&snapshot_name, storage).await?;
        }
        Commands::Inspect {
            snapshot_name,
            storage,
        } => {
            info!("Inspecting snapshot: {}", snapshot_name);
            inspect_command(&snapshot_name, storage).await?;
        }
    }

    Ok(())
}
