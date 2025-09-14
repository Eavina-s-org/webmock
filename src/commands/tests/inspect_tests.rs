use crate::commands::inspect::{extract_content_type, format_size, truncate_url};
use std::collections::HashMap;

#[test]
fn test_extract_content_type() {
    let mut headers = HashMap::new();
    headers.insert(
        "content-type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    let content_type = extract_content_type(&headers);
    assert_eq!(content_type.unwrap(), "application/json");
}

#[test]
fn test_extract_content_type_case_insensitive() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/html".to_string());

    let content_type = extract_content_type(&headers);
    assert_eq!(content_type.unwrap(), "text/html");
}

#[test]
fn test_extract_content_type_missing() {
    let headers = HashMap::new();

    let content_type = extract_content_type(&headers);
    assert!(content_type.is_none());
}

#[test]
fn test_extract_content_type_edge_cases() {
    let mut headers = HashMap::new();

    // Empty content type
    headers.insert("content-type".to_string(), "".to_string());
    assert_eq!(extract_content_type(&headers).unwrap(), "");

    // Content type with only spaces
    headers.insert("content-type".to_string(), "   ".to_string());
    assert_eq!(extract_content_type(&headers).unwrap(), "");

    // Content type without semicolon
    headers.insert("content-type".to_string(), "text/plain".to_string());
    assert_eq!(extract_content_type(&headers).unwrap(), "text/plain");

    // Content type with multiple semicolons
    headers.insert(
        "content-type".to_string(),
        "application/json; charset=utf-8; boundary=something".to_string(),
    );
    assert_eq!(extract_content_type(&headers).unwrap(), "application/json");

    // Content type with uppercase
    headers.insert(
        "content-type".to_string(),
        "APPLICATION/XML; charset=UTF-8".to_string(),
    );
    assert_eq!(extract_content_type(&headers).unwrap(), "APPLICATION/XML");

    // Content type with spaces around semicolon
    headers.insert(
        "content-type".to_string(),
        "image/png ; charset=utf-8".to_string(),
    );
    assert_eq!(extract_content_type(&headers).unwrap(), "image/png");
}

#[test]
fn test_truncate_url() {
    let short_url = "https://example.com";
    assert_eq!(truncate_url(short_url, 20), "https://example.com");

    let long_url = "https://example.com/very/long/path/to/some/resource.html";
    let truncated = truncate_url(long_url, 20);
    assert!(truncated.len() <= 20);
    assert!(truncated.ends_with("..."));
}

#[test]
fn test_truncate_url_edge_cases() {
    assert_eq!(truncate_url("", 10), "");
    assert_eq!(truncate_url("short", 10), "short");
    assert_eq!(truncate_url("exactly20charslong", 20), "exactly20charslong");
    assert_eq!(truncate_url("https://example.com", 3), "...");
    assert_eq!(truncate_url("https://example.com", 4), "h...");
    assert_eq!(truncate_url("https://example.com", 5), "ht...");
}

#[test]
fn test_format_size() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(500), "500 B");
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(1536), "1.5 KB");
    assert_eq!(format_size(1024 * 1024), "1.0 MB");
    assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
}

#[test]
fn test_format_size_edge_cases() {
    assert_eq!(format_size(1), "1 B");
    assert_eq!(format_size(1023), "1023 B");
    assert_eq!(format_size(1025), "1.0 KB");
    assert_eq!(format_size(1024 * 1024 - 1), "1024.0 KB");
    assert_eq!(format_size(1024 * 1024 + 1), "1.0 MB");
}

#[test]
fn test_format_size_large_values() {
    assert_eq!(format_size(1024 * 1024 * 1024 * 5), "5.0 GB");
    assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1024.0 GB");
    assert_eq!(
        format_size(1024 * 1024 * 1024 * 1024 * 1024),
        "1048576.0 GB"
    );
}

#[test]
fn test_format_size_very_small_values() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(10), "10 B");
    assert_eq!(format_size(100), "100 B");
    assert_eq!(format_size(999), "999 B");
}
