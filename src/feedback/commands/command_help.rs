use colored::*;

/// Display help for capture command
pub fn show_capture_help() {
    println!();
    println!("{} {}", "🕷️".bright_green(), "Capture Mode".bold().bright_green());
    println!("   {} {}", "webmock capture".bright_cyan(), "[OPTIONS] [URL]".dimmed());
    println!();
    println!("   {}:", "Options".bold());
    println!("      {}          Enable debug logging", "--debug".bright_yellow());
    println!("      {}        Set port (default: 8080)", "--port".bright_yellow());
    println!("      {}      Set output file", "--output".bright_yellow());
    println!("      {}     Enable HTTPS interception", "--https".bright_yellow());
    println!("      {}    Enable performance tracking", "--performance".bright_yellow());
    println!();
    println!("   {}:", "Examples".bold());
    println!("      webmock capture");
    println!("      webmock capture --port 9090");
    println!("      webmock capture --https --performance");
    println!("      webmock capture --output my-recording.json");
}

/// Display help for serve command
pub fn show_serve_help() {
    println!();
    println!("{} {}", "🚀".bright_blue(), "Serve Mode".bold().bright_blue());
    println!("   {} {}", "webmock serve".bright_cyan(), "[OPTIONS] [SNAPSHOT_FILE]".dimmed());
    println!();
    println!("   {}:", "Options".bold());
    println!("      {}          Enable debug logging", "--debug".bright_yellow());
    println!("      {}        Set port (default: 8080)", "--port".bright_yellow());
    println!("      {}        Enable TLS/HTTPS", "--tls".bright_yellow());
    println!("      {}      Set hostname (default: localhost)", "--host".bright_yellow());
    println!();
    println!("   {}:", "Examples".bold());
    println!("      webmock serve");
    println!("      webmock serve recording.json");
    println!("      webmock serve --port 9090 --tls");
    println!("      webmock serve --host 0.0.0.0 --port 8080");
}

/// Display help for list command
pub fn show_list_help() {
    println!();
    println!("{} {}", "📋".bright_magenta(), "List Mode".bold().bright_magenta());
    println!("   {} {}", "webmock list".bright_cyan(), "[OPTIONS] [SNAPSHOT_FILE]".dimmed());
    println!();
    println!("   {}:", "Options".bold());
    println!("      {}        Show detailed request/response info", "--verbose".bright_yellow());
    println!("      {}        Filter by method (GET, POST, etc.)", "--method".bright_yellow());
    println!("      {}        Filter by URL pattern", "--url".bright_yellow());
    println!("      {}       Filter by status code", "--status".bright_yellow());
    println!();
    println!("   {}:", "Examples".bold());
    println!("      webmock list");
    println!("      webmock list recording.json");
    println!("      webmock list --verbose");
    println!("      webmock list --method GET --status 200");
}

/// Display help for delete command
pub fn show_delete_help() {
    println!();
    println!("{} {}", "🗑️".bright_red(), "Delete Mode".bold().bright_red());
    println!("   {} {}", "webmock delete".bright_cyan(), "[OPTIONS] SNAPSHOT_FILE".dimmed());
    println!();
    println!("   {}:", "Options".bold());
    println!("      {}      Delete by request ID", "--id".bright_yellow());
    println!("      {}    Delete by method", "--method".bright_yellow());
    println!("      {}      Delete by URL pattern", "--url".bright_yellow());
    println!("      {}    Delete by status code", "--status".bright_yellow());
    println!("      {}        Confirm deletion", "--confirm".bright_yellow());
    println!();
    println!("   {}:", "Examples".bold());
    println!("      webmock delete recording.json --id 1");
    println!("      webmock delete recording.json --method POST --confirm");
    println!("      webmock delete recording.json --status 404");
}