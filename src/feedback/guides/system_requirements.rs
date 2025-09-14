use colored::*;

/// Display system requirements
pub fn show_system_requirements() {
    println!();
    println!("{} {}", "💻".bright_blue(), "System Requirements".bold().bright_blue());
    println!();
    
    println!("{}:", "Operating System".bold());
    println!("   • {}: macOS 10.15 or later", "macOS".green());
    println!("   • {}: Ubuntu 18.04+ or equivalent", "Linux".green());
    println!("   • {}: Windows 10 or later (WSL2 recommended)", "Windows".yellow());
    println!();
    
    println!("{}:", "Dependencies".bold());
    println!("   • {}: 1.70 or later", "Rust".green());
    println!("   • {}: OpenSSL development libraries", "OpenSSL".green());
    println!("   • {}: pkg-config (Linux/macOS)", "pkg-config".green());
    println!();
    
    println!("{}:", "Network".bold());
    println!("   • {}: Internet connection for HTTPS interception", "HTTPS".green());
    println!("   • {}: Local port access (configurable)", "Port".green());
    println!("   • {}: Browser proxy configuration", "Browser".green());
    println!();
    
    println!("{}:", "Memory".bold());
    println!("   • {}: 512MB RAM minimum", "Minimum".yellow());
    println!("   • {}: 2GB RAM recommended", "Recommended".green());
    println!("   • {}: 8GB+ RAM for large recordings", "Heavy usage".yellow());
    println!();
    
    println!("{}:", "Disk".bold());
    println!("   • {}: 100MB for installation", "Installation".green());
    println!("   • {}: 1GB+ for recordings (variable)", "Recordings".green());
    println!("   • {}: SSD recommended for better performance", "Storage".green());
}