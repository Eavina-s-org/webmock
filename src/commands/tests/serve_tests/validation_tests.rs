use crate::feedback::ValidationHelper;

#[test]
fn test_validate_port_zero() {
    let result = ValidationHelper::validate_port(0);
    assert!(result.is_ok()); // Port 0 is valid in ValidationHelper
}

#[test]
fn test_validate_port_max() {
    let result = ValidationHelper::validate_port(65535);
    assert!(result.is_ok());
}

#[test]
fn test_validate_port_common() {
    let result = ValidationHelper::validate_port(8080);
    assert!(result.is_ok());
}

#[test]
fn test_validate_snapshot_name_empty() {
    let result = ValidationHelper::validate_snapshot_name("");
    assert!(result.is_err());
}

#[test]
fn test_validate_snapshot_name_with_special_chars() {
    let result = ValidationHelper::validate_snapshot_name("test/snapshot");
    assert!(result.is_err());
}

#[test]
fn test_validate_snapshot_name_with_spaces() {
    let result = ValidationHelper::validate_snapshot_name("test snapshot");
    assert!(result.is_err());
}

#[test]
fn test_validate_snapshot_name_valid() {
    let result = ValidationHelper::validate_snapshot_name("valid-snapshot_123");
    assert!(result.is_ok());
}
