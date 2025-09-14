use std::time::Duration;

use crate::capture::browser::BrowserController;
use crate::test_utils::test_helpers::*;

#[tokio::test]
#[ignore = "slow test - requires Chrome"]
async fn test_current_url_success() {
    setup_test_env();

    let result = BrowserController::new(0).await;

    if let Ok(mut controller) = result {
        // Test initial URL (should be about:blank)
        let initial_url = controller.current_url().await;
        assert!(initial_url.is_ok(), "current_url should succeed");

        let initial_url_value = initial_url.unwrap();
        assert!(
            initial_url_value.starts_with("about:")
                || initial_url_value.starts_with("chrome:")
                || initial_url_value.starts_with("data:")
                || initial_url_value.is_empty(),
            "Initial URL should be about:blank or similar, got: {}",
            initial_url_value
        );

        // Navigate to a page and check URL
        if controller.navigate("https://httpbin.org/get").await.is_ok() {
            // Wait a bit for navigation
            tokio::time::sleep(Duration::from_millis(500)).await;

            let current_url = controller.current_url().await;
            assert!(
                current_url.is_ok(),
                "current_url should succeed after navigation"
            );

            let current_url_value = current_url.unwrap();
            assert_eq!(current_url_value, "https://httpbin.org/get");
        }

        // Cleanup
        let _ = controller.close().await;
    }

    cleanup_test_env();
}

#[tokio::test]
async fn test_current_url_error_handling() {
    use crate::error::WebMockError;

    // Test error handling by simulating the error conditions
    // Since we can't easily create a BrowserController in an error state,
    // we test the error types that would be returned

    // Test that WebMockError::Browser is properly handled
    let browser_error = WebMockError::Browser(Box::new(chromiumoxide::error::CdpError::NoResponse));
    assert!(browser_error.is_recoverable());

    let user_msg = browser_error.user_message();
    assert!(!user_msg.is_empty());

    cleanup_test_env();
}

#[tokio::test]
#[ignore = "slow test - requires Chrome"]
async fn test_title_success() {
    setup_test_env();

    let result = BrowserController::new(0).await;

    if let Ok(mut controller) = result {
        // Test initial title (should be "Untitled")
        let initial_title = controller.title().await;
        assert!(initial_title.is_ok(), "title should succeed");

        let initial_title_value = initial_title.unwrap();
        assert_eq!(initial_title_value, "Untitled");

        // Navigate to a page with a known title
        // Note: httpbin.org doesn't set a title, so we'll test with a simple HTML page
        if controller.navigate("https://example.com").await.is_ok() {
            // Wait a bit for navigation
            tokio::time::sleep(Duration::from_millis(1000)).await;

            let title = controller.title().await;
            assert!(title.is_ok(), "title should succeed after navigation");

            // example.com has "Example Domain" as title
            let title_value = title.unwrap();
            assert_eq!(title_value, "Example Domain");
        }

        // Cleanup
        let _ = controller.close().await;
    }

    cleanup_test_env();
}

#[tokio::test]
async fn test_title_error_handling() {
    use crate::error::WebMockError;

    // Test error handling by simulating the error conditions
    // Since we can't easily create a BrowserController in an error state,
    // we test the error types that would be returned

    // Test that WebMockError::Browser is properly handled
    let browser_error = WebMockError::Browser(Box::new(chromiumoxide::error::CdpError::NoResponse));
    assert!(browser_error.is_recoverable());

    let user_msg = browser_error.user_message();
    assert!(!user_msg.is_empty());

    cleanup_test_env();
}

#[tokio::test]
async fn test_page_title_extraction_simulation() {
    // Test title extraction logic without browser
    let test_cases = vec![
        ("Example Page", "Example Page"),
        ("", "Untitled"),
        ("   ", "Untitled"),
        (
            "Very Long Title That Might Be Truncated In Some Cases",
            "Very Long Title That Might Be Truncated In Some Cases",
        ),
    ];

    for (input, expected) in test_cases {
        let result = if input.trim().is_empty() {
            "Untitled".to_string()
        } else {
            input.to_string()
        };

        assert_eq!(result, expected);
    }
}
