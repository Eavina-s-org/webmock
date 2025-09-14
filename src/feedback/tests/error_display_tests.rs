use crate::error::WebMockError;
use crate::feedback::ErrorDisplay;

#[test]
fn test_error_display_show_error() {
    let error = WebMockError::SnapshotNotFound("test".to_string());

    // This should not panic
    ErrorDisplay::show_error(&error);
}

#[test]
fn test_error_display_different_error_types() {
    let errors = vec![
        WebMockError::ChromeNotFound,
        WebMockError::PortInUse(8080),
        WebMockError::InvalidUrl("invalid".to_string(), "bad format".to_string()),
        WebMockError::Timeout(30),
    ];

    // All of these should not panic
    for error in errors {
        ErrorDisplay::show_error(&error);
    }
}
