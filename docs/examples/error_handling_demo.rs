use webmock_cli::feedback::{ErrorDisplay, UserFeedback, ValidationHelper, ProgressReporter, chrome_detection::ChromeDetection};
use webmock_cli::error::WebMockError;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("🎯 WebMock CLI - Enhanced Error Handling Demo");
    println!();

    // Demo 1: Input validation with helpful feedback
    UserFeedback::section("📋 Input Validation Demo");
    
    let test_cases = [
        ("", "Empty URL"),
        ("ftp://example.com", "Unsupported protocol"),
        ("not-a-url", "Invalid URL format"),
        ("https://example.com", "Valid URL"),
    ];

    for (url, description) in test_cases {
        print!("Testing {}: ", description);
        match ValidationHelper::validate_url(url) {
            Ok(_) => UserFeedback::success("Valid"),
            Err(e) => {
                UserFeedback::error("Invalid");
                println!("  Reason: {}", e.user_message());
            }
        }
    }

    // Demo 2: Snapshot name validation
    UserFeedback::section("📸 Snapshot Name Validation Demo");
    
    let name_cases = [
        ("my-snapshot", "Valid name"),
        ("test/invalid", "Contains path separator"),
        ("test snapshot", "Contains space"),
        ("", "Empty name"),
        ("valid_name_123", "Valid with underscore and numbers"),
    ];

    for (name, description) in name_cases {
        print!("Testing {}: ", description);
        match ValidationHelper::validate_snapshot_name(name) {
            Ok(_) => UserFeedback::success("Valid"),
            Err(e) => {
                UserFeedback::error("Invalid");
                println!("  Reason: {}", e.user_message());
            }
        }
    }

    // Demo 3: Enhanced error display
    UserFeedback::section("🚨 Enhanced Error Display Demo");
    
    let errors = [
        WebMockError::ChromeNotFound,
        WebMockError::PortInUse(8080),
        WebMockError::SnapshotNotFound("missing-snapshot".to_string()),
        WebMockError::Timeout(30),
        WebMockError::InvalidUrl("bad-url".to_string(), "invalid format".to_string()),
    ];

    for error in errors {
        println!("\n--- Error Demo ---");
        ErrorDisplay::show_error(&error);
        println!();
    }

    // Demo 4: User feedback styles
    UserFeedback::section("🎨 User Feedback Styles Demo");
    
    UserFeedback::success("This is a success message");
    UserFeedback::info("This is an info message");
    UserFeedback::warning("This is a warning message");
    UserFeedback::error("This is an error message");
    UserFeedback::tip("This is a helpful tip");
    
    // Demo 5: Progress reporting
    demo_progress_reporting().await;
    
    // Demo 6: Chrome detection
    demo_chrome_detection().await;
    
    // Demo 7: System requirements check
    demo_system_requirements().await;
    
    // Demo 8: Help system
    demo_help_system();
    
    UserFeedback::separator();
    UserFeedback::success("All demos completed! 🎉");
    UserFeedback::tip("This demonstrates the comprehensive error handling and user feedback system");
}

async fn demo_progress_reporting() {
    UserFeedback::section("📊 Progress Reporting Demo");
    
    let mut progress = ProgressReporter::new();
    
    // Demo capture progress
    let capture_progress = progress.start_capture_progress("https://example.com");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    progress.update_capture_step("Starting browser...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    progress.update_capture_step("Loading page...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    progress.update_capture_step("Recording requests...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    progress.finish_capture_success("demo-snapshot");
    
    // Demo file progress
    let file_progress = progress.create_file_progress("Saving snapshot", 1024 * 1024);
    for i in 0..=100 {
        file_progress.set_position((1024 * 1024 * i) / 100);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    file_progress.finish_with_message("✅ Snapshot saved");
    
    // Demo network progress
    let network_progress = progress.create_network_progress("Checking connectivity");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    network_progress.finish_with_message("✅ Network check complete");
}

async fn demo_chrome_detection() {
    UserFeedback::section("🌐 Chrome Detection Demo");
    
    match ChromeDetection::check_chrome_availability() {
        Ok(info) => {
            UserFeedback::success(&format!("Chrome detected: {}", info));
        }
        Err(_) => {
            UserFeedback::warning("Chrome not found - showing installation help");
            ChromeDetection::show_installation_help();
        }
    }
}

async fn demo_system_requirements() {
    UserFeedback::section("💻 System Requirements Demo");
    
    match ValidationHelper::check_system_requirements() {
        Ok(_) => UserFeedback::success("System requirements check passed"),
        Err(e) => {
            UserFeedback::warning("System requirements issues detected");
            ErrorDisplay::show_error(&e);
        }
    }
}

fn demo_help_system() {
    UserFeedback::section("📚 Help System Demo");
    
    UserFeedback::info("Showing command help examples:");
    UserFeedback::show_command_help("capture");
    
    println!();
    UserFeedback::info("Troubleshooting guide:");
    UserFeedback::show_troubleshooting_guide();
    
    println!();
    UserFeedback::info("Performance tips:");
    UserFeedback::show_performance_tips();
}