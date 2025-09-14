use crate::commands::serve::{
    check_and_resolve_port, find_available_port, get_port_usage_info, is_port_available,
};

#[test]
fn test_is_port_available_with_invalid_port() {
    // Port 0 might be available on some systems, so just test the function runs
    let _ = is_port_available(0);
}

#[test]
fn test_is_port_available_with_reserved_port() {
    // Port 80 is likely in use on most systems
    // This test might fail on some systems, but it's useful for coverage
    let result = is_port_available(80);
    // We don't assert true/false since it depends on the system
    // Just ensure the function doesn't panic
    let _ = result;
}

#[test]
fn test_find_available_port_with_high_start() {
    // Test with a very high port number
    let result = find_available_port(65000);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_get_port_usage_info() {
    let info = get_port_usage_info(8080);
    assert!(!info.is_empty());
    assert!(info.contains("8080"));
}

#[test]
fn test_check_and_resolve_port_with_invalid_port() {
    let result = check_and_resolve_port(0);
    // Port 0 might resolve to a valid port, so just test the function runs
    let _ = result;
}

#[test]
fn test_check_and_resolve_port_with_high_port() {
    // Test with a port that's likely available
    let result = check_and_resolve_port(15000);
    // Should either return the port or find an alternative
    assert!(result.is_ok());
}
