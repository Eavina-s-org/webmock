use colored::*;

/// Display performance optimization tips
pub fn show_performance_tips() {
    println!();
    println!("{} {}", "⚡".bright_yellow(), "Performance Optimization Tips".bold().bright_yellow());
    println!();
    
    println!("{}:", "Memory Usage".bold());
    println!("   • {}: Use --performance flag to monitor memory", "Monitor".cyan());
    println!("   • {}: Enable streaming for large responses", "Stream".cyan());
    println!("   • {}: Reduce concurrent connection limits", "Limit".cyan());
    println!("   • {}: Use compression for storage efficiency", "Compress".cyan());
    println!();
    
    println!("{}:", "Response Time".bold());
    println!("   • {}: Use connection pooling", "Pool".cyan());
    println!("   • {}: Implement caching for repeated requests", "Cache".cyan());
    println!("   • {}: Optimize regex patterns for matching", "Optimize".cyan());
    println!("   • {}: Reduce logging verbosity in production", "Log".cyan());
    println!();
    
    println!("{}:", "Storage".bold());
    println!("   • {}: Use efficient serialization formats", "Format".cyan());
    println!("   • {}: Implement data cleanup policies", "Clean".cyan());
    println!("   • {}: Use compression for snapshot files", "Compress".cyan());
    println!("   • {}: Archive old recordings", "Archive".cyan());
    println!();
    
    println!("{}:", "Network".bold());
    println!("   • {}: Use efficient HTTP/2 or HTTP/3", "Protocol".cyan());
    println!("   • {}: Implement connection reuse", "Reuse".cyan());
    println!("   • {}: Use appropriate timeout values", "Timeout".cyan());
    println!("   • {}: Monitor bandwidth usage", "Monitor".cyan());
}