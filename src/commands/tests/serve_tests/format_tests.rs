use crate::commands::inspect::{format_size, truncate_url};

#[test]
fn test_format_size_edge_cases() {
    assert_eq!(format_size(0), "0 B");
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

#[test]
fn test_truncate_url_edge_cases() {
    assert_eq!(truncate_url("", 10), "");
    assert_eq!(truncate_url("short", 10), "short");
    assert_eq!(truncate_url("exactly20charslong", 20), "exactly20charslong");
    assert_eq!(truncate_url("https://example.com", 3), "...");
    assert_eq!(truncate_url("https://example.com", 4), "h...");
    assert_eq!(truncate_url("https://example.com", 5), "ht...");
}
