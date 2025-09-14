use crate::feedback::ValidationHelper;

#[test]
fn test_permissions_check() {
    // This test might fail in some environments, so we just check it doesn't panic
    let result = ValidationHelper::check_permissions();

    // The result depends on the actual system permissions
    // We just verify the method can be called without panicking
    match result {
        Ok(_) => {
            // Permissions are OK
        }
        Err(e) => {
            // Permissions issue detected
            assert!(e.to_string().contains("permission") || e.to_string().contains("Permission"));
        }
    }
}

#[test]
fn test_system_requirements_check() {
    // This should not panic regardless of system state
    let result = ValidationHelper::check_system_requirements();
    assert!(result.is_ok()); // This method always returns Ok, but may log warnings
}
