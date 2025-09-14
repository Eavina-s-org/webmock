//! Tests for capture command

use crate::commands::capture::{check_snapshot_exists, initialize_storage, validate_inputs};
use crate::error::WebMockError;
use crate::storage::Storage;
use tempfile::TempDir;

#[test]
fn test_validate_inputs_valid() {
    let result = validate_inputs("https://example.com", "test-snapshot", 30);
    assert!(result.is_ok());
}

#[test]
fn test_validate_inputs_invalid_url() {
    let result = validate_inputs("not-a-url", "test", 30);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WebMockError::InvalidUrl(_, _)
    ));
}

#[test]
fn test_validate_inputs_unsupported_scheme() {
    let result = validate_inputs("ftp://example.com", "test", 30);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WebMockError::InvalidUrl(_, _)
    ));
}

#[test]
fn test_validate_inputs_empty_name() {
    let result = validate_inputs("https://example.com", "", 30);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}

#[test]
fn test_validate_inputs_name_with_path_separator() {
    let result = validate_inputs("https://example.com", "test/invalid", 30);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}

#[test]
fn test_validate_inputs_long_name() {
    let long_name = "a".repeat(101);
    let result = validate_inputs("https://example.com", &long_name, 30);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}

#[test]
fn test_validate_inputs_zero_timeout() {
    let result = validate_inputs("https://example.com", "test", 0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}

#[tokio::test]
async fn test_initialize_storage() {
    // Use direct storage path instead of environment variables
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("webmock-storage");

    let result = initialize_storage(Some(storage_path.to_string_lossy().to_string())).await;
    assert!(result.is_ok());

    let storage = result.unwrap();
    let snapshots_dir = storage.ensure_snapshots_dir();
    assert!(snapshots_dir.is_ok());
}

#[test]
fn test_check_snapshot_exists() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path().to_path_buf());

    // Should pass when snapshot doesn't exist
    let result = check_snapshot_exists(&storage, "non-existent");
    assert!(result.is_ok());

    // Create a dummy snapshot file
    let snapshots_dir = temp_dir.path().join("snapshots");
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    std::fs::write(snapshots_dir.join("existing.msgpack"), b"dummy").unwrap();

    // Should fail when snapshot exists
    let result = check_snapshot_exists(&storage, "existing");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebMockError::Config(_)));
}
