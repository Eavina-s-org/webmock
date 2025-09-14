use colored::*;

/// Display troubleshooting guide for common issues
pub fn show_troubleshooting_guide() {
    println!();
    println!("{} {}", "🔧".bright_yellow(), "Troubleshooting Guide".bold().bright_yellow());
    println!();
    
    println!("{}:", "Connection Issues".bold().red());
    println!("   • {}: Check if another service is using the port", "Port already in use".yellow());
    println!("   • {}: Ensure the target server is accessible", "Connection refused".yellow());
    println!("   • {}: Verify firewall settings and network connectivity", "Network unreachable".yellow());
    println!();
    
    println!("{}:", "HTTPS/TLS Issues".bold().red());
    println!("   • {}: Install and trust the generated certificate", "Certificate warnings".yellow());
    println!("   • {}: Ensure proper certificate generation", "TLS handshake failed".yellow());
    println!("   • {}: Check if target server supports TLS", "SSL connection error".yellow());
    println!();
    
    println!("{}:", "Recording Issues".bold().red());
    println!("   • {}: Ensure browser/system proxy is configured", "No requests captured".yellow());
    println!("   • {}: Check if HTTPS interception is enabled", "Missing HTTPS requests".yellow());
    println!("   • {}: Verify target URLs are accessible", "Empty snapshot".yellow());
    println!();
    
    println!("{}:", "Performance Issues".bold().red());
    println!("   • {}: Monitor memory usage and connection limits", "High memory usage".yellow());
    println!("   • {}: Check for network bottlenecks", "Slow response times".yellow());
    println!("   • {}: Reduce concurrent connection limits", "Connection timeouts".yellow());
    println!();
    
    println!("{}:", "File Issues".bold().red());
    println!("   • {}: Ensure file exists and is readable", "Cannot read snapshot".yellow());
    println!("   • {}: Check file permissions and disk space", "Cannot write snapshot".yellow());
    println!("   • {}: Verify JSON format and structure", "Invalid snapshot format".yellow());
}