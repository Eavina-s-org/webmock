use crate::{error::WebMockError, feedback::chrome_detection::ChromeDetection};

#[test]
fn test_installation_help() {
    // This should not panic
    ChromeDetection::show_installation_help();
}

#[test]
fn test_chrome_detection() {
    // This test will vary based on the system it runs on
    // We mainly test that it doesn't panic and returns a proper Result
    let result = ChromeDetection::check_chrome_availability();

    match result {
        Ok(info) => {
            assert!(!info.is_empty());
            assert!(info.contains("Chrome") || info.contains("chromium"));
        }
        Err(WebMockError::ChromeNotFound) => {
            // This is expected on systems without Chrome
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

#[test]
fn test_installation_help_doesnt_panic() {
    // Test that showing installation help doesn't panic
    // We can't easily test the output, but we can ensure it runs
    ChromeDetection::show_installation_help();
}

#[test]
fn test_validate_and_guide() {
    // Test that validate_and_guide returns a proper Result
    let result = ChromeDetection::validate_and_guide();

    // Should either succeed or fail with ChromeNotFound
    match result {
        Ok(()) => {
            // Chrome is available
        }
        Err(WebMockError::ChromeNotFound) => {
            // Chrome is not available - this is fine for testing
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}
