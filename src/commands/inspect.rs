use crate::error::{Result, WebMockError};
use crate::feedback::UserFeedback;
use crate::storage::Storage;

use std::collections::HashMap;

/// Handle the inspect command to view all records in a snapshot
pub async fn inspect_command(snapshot_name: &str, storage_arg: Option<String>) -> Result<()> {
    UserFeedback::info(&format!("🔍 Inspecting snapshot: {}", snapshot_name));

    // Initialize storage
    let storage_path = crate::commands::get_storage_path(storage_arg)?;
    let storage = Storage::new(storage_path);

    // Load the snapshot
    let snapshot = storage
        .load_snapshot(snapshot_name)
        .await
        .map_err(|_| WebMockError::SnapshotNotFound(snapshot_name.to_string()))?;

    UserFeedback::success(&format!("✅ Loaded snapshot: {}", snapshot_name));

    // Display snapshot summary
    println!();
    println!("📋 Snapshot Overview");
    println!("   📍 Original URL: {}", snapshot.url);
    println!("   📊 Total records: {}", snapshot.requests.len());
    println!(
        "   📅 Created: {}",
        snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!();

    if snapshot.requests.is_empty() {
        UserFeedback::warning("No records found in this snapshot");
        return Ok(());
    }

    // Sort records by URL for better readability
    let mut requests = snapshot.requests;
    requests.sort_by(|a, b| a.url.cmp(&b.url));

    println!("📄 Detailed Records:");
    println!();

    // Print header
    println!(
        "{:<4} {:<8} {:<50} {:<6} {:<15} {:<10}",
        "#", "Method", "URL", "Status", "Type", "Size"
    );
    println!(
        "{:-<4} {:-<8} {:-<50} {:-<6} {:-<15} {:-<10}",
        "", "", "", "", "", ""
    );

    for (index, record) in requests.iter().enumerate() {
        let method = &record.method;
        let url = truncate_url(&record.url, 47);
        let status = record.response.status;
        let content_type =
            extract_content_type(&record.response.headers).unwrap_or_else(|| "unknown".to_string());
        let size = format_size(record.response.body.len());

        // Color coding for terminal output
        let method_color = match method.as_str() {
            "GET" => "\x1b[32m",     // Green
            "POST" => "\x1b[34m",    // Blue
            "PUT" => "\x1b[33m",     // Yellow
            "DELETE" => "\x1b[31m",  // Red
            "CONNECT" => "\x1b[35m", // Magenta
            "PATCH" => "\x1b[36m",   // Cyan
            _ => "\x1b[0m",          // Reset
        };

        let status_color = if (200..300).contains(&status) {
            "\x1b[32m" // Green
        } else if (300..400).contains(&status) {
            "\x1b[33m" // Yellow
        } else if (400..600).contains(&status) {
            "\x1b[31m" // Red
        } else {
            "\x1b[0m" // Reset
        };

        let reset = "\x1b[0m";

        print!("{:<4} ", index + 1);
        print!("{}{:<8}{}", method_color, method, reset);
        print!(" {:<47}", url);
        print!(" {}{}{}", status_color, status, reset);
        print!(" {:<15}", content_type);
        println!(" {:<10}", size);
    }

    println!();
    display_summary_stats(&requests);

    UserFeedback::success("Inspection completed successfully!");

    Ok(())
}

/// Extract content type from headers
pub(crate) fn extract_content_type(
    headers: &std::collections::HashMap<String, String>,
) -> Option<String> {
    headers
        .get("content-type")
        .or_else(|| headers.get("Content-Type"))
        .map(|value| value.split(';').next().unwrap_or(value).trim().to_string())
}

/// Truncate long URLs for display
pub(crate) fn truncate_url(url: &str, max_length: usize) -> String {
    if url.len() <= max_length {
        url.to_string()
    } else {
        format!("{}...", &url[..max_length - 3])
    }
}

/// Format file size for display
pub(crate) fn format_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} B", size as usize)
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Display summary statistics
fn display_summary_stats(requests: &[crate::capture::proxy::records::RequestRecord]) {
    use std::collections::HashMap;

    let mut method_counts = HashMap::new();
    let mut status_counts = HashMap::new();
    let mut domain_counts = HashMap::new();
    let mut total_size = 0;

    for record in requests {
        *method_counts.entry(&record.method).or_insert(0) += 1;
        *status_counts.entry(record.response.status).or_insert(0) += 1;

        if let Ok(url) = url::Url::parse(&record.url) {
            if let Some(domain) = url.host_str() {
                *domain_counts.entry(domain.to_string()).or_insert(0) += 1;
            }
        }

        total_size += record.response.body.len();
    }

    println!("📊 Summary Statistics:");
    println!();

    // Method distribution
    print_distribution("Methods", method_counts);

    // Status distribution
    let status_ranges: HashMap<String, usize> =
        status_counts
            .iter()
            .fold(HashMap::new(), |mut acc, (&status, &count)| {
                let range = match status {
                    200..=299 => "2xx Success",
                    300..=399 => "3xx Redirect",
                    400..=499 => "4xx Client Error",
                    500..=599 => "5xx Server Error",
                    _ => "Other",
                };
                *acc.entry(range.to_string()).or_insert(0) += count;
                acc
            });
    print_distribution("Status Codes", status_ranges);

    // Top domains
    let mut domains: Vec<_> = domain_counts.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(a.1));
    println!("   🌐 Top domains:");
    for (i, (domain, count)) in domains.iter().take(5).enumerate() {
        println!("      {}. {} ({} requests)", i + 1, domain, count);
    }

    println!("   💾 Total response size: {}", format_size(total_size));
}

/// Helper to print key-value distributions
fn print_distribution<K: std::fmt::Display>(label: &str, counts: HashMap<K, usize>) {
    print!("   {}: ", label);
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));

    let formatted: Vec<String> = items
        .iter()
        .map(|(key, count)| format!("{} ({})", key, count))
        .collect();

    println!("{}", formatted.join(", "));
}
