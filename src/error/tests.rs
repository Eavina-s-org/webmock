use super::*;

#[test]
fn test_error_display() {
    let error = WebMockError::SnapshotNotFound("test-snapshot".to_string());
    assert_eq!(error.to_string(), "Snapshot 'test-snapshot' not found");
}

#[test]
fn test_user_message() {
    let error = WebMockError::PortInUse(8080);
    let message = error.user_message();
    assert!(message.contains("Port 8080 is already in use"));
    assert!(message.contains("--port option"));
}

#[test]
fn test_recoverable_errors() {
    assert!(WebMockError::PortInUse(8080).is_recoverable());
    assert!(WebMockError::Timeout(30).is_recoverable());
    assert!(!WebMockError::ChromeNotFound.is_recoverable());
    assert!(!WebMockError::SnapshotNotFound("test".to_string()).is_recoverable());
}

#[test]
fn test_error_creation_helpers() {
    let proxy_error = WebMockError::proxy("Connection failed");
    assert!(matches!(proxy_error, WebMockError::Proxy(_)));

    let config_error = WebMockError::config("Invalid setting");
    assert!(matches!(config_error, WebMockError::Config(_)));
}
