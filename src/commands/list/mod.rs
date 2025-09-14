use crate::error::{Result, WebMockError};
use crate::feedback::{ProgressReporter, UserFeedback, ValidationHelper};
use crate::storage::Storage;
use colored::*;

/// Handle the list command with enhanced feedback
pub async fn list_command(storage_arg: Option<String>) -> Result<()> {
    // Step 0: Check system requirements (skip in test environment)
    if std::env::var("WEBMOCK_SKIP_PERMISSION_CHECK").is_err() {
        UserFeedback::info("Checking system access...");
        ValidationHelper::check_permissions()?;
    }

    // Initialize storage
    UserFeedback::info("Initializing storage...");
    let storage_path = crate::commands::get_storage_path(storage_arg)?;
    let storage = Storage::new(storage_path);

    // Get list of snapshots with progress indicator
    let progress = ProgressReporter::new();
    let loading_spinner = progress.create_spinner("Loading snapshots...");

    let snapshots = match storage.list_snapshots().await {
        Ok(snapshots) => {
            loading_spinner.finish_with_message("✅ Snapshots loaded");
            snapshots
        }
        Err(e) => {
            loading_spinner.finish_with_message("❌ Failed to load snapshots");
            UserFeedback::error(&format!("Failed to list snapshots: {}", e.user_message()));

            // Provide helpful context for common errors
            if matches!(e, WebMockError::Storage(_)) {
                UserFeedback::tip("Check that ~/.webmock directory exists and is readable");
                UserFeedback::tip("Try running a capture command first to initialize storage");
            }

            return Err(e);
        }
    };

    // Handle empty snapshot list with friendly message and guidance
    if snapshots.is_empty() {
        UserFeedback::section("📭 No Snapshots Found");
        println!("You haven't created any snapshots yet.");
        println!();
        UserFeedback::info("To get started:");
        UserFeedback::tip(
            "Create your first snapshot: webmock capture https://example.com --name my-site",
        );
        UserFeedback::tip("Then serve it locally: webmock serve my-site");
        return Ok(());
    }

    // Format and display snapshot information with enhanced styling
    UserFeedback::section(&format!(
        "📋 Found {} Snapshot{}",
        snapshots.len(),
        if snapshots.len() == 1 { "" } else { "s" }
    ));

    for (index, snapshot) in snapshots.iter().enumerate() {
        // Format creation date in a user-friendly way
        let formatted_date = snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC");

        println!("{}. 📸 {}", index + 1, snapshot.name.bright_cyan());
        println!("   🌐 URL: {}", snapshot.url);
        println!("   📅 Created: {}", formatted_date.to_string().dimmed());

        // Add separator between snapshots (except for the last one)
        if index < snapshots.len() - 1 {
            println!("{}", "─".repeat(60).dimmed());
        }
    }

    // Show helpful next steps
    println!();
    UserFeedback::section("💡 Next Steps");
    UserFeedback::tip("Start a mock server: webmock serve <snapshot-name>");
    UserFeedback::tip("Delete a snapshot: webmock delete <snapshot-name>");
    UserFeedback::tip("Get detailed help: webmock <command> --help");

    Ok(())
}
