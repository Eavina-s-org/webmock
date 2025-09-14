use crate::feedback::ProgressReporter;
use std::time::Duration;

#[test]
fn test_progress_reporter_creation() {
    let reporter = ProgressReporter::new();

    // Test that we can create various progress indicators
    let spinner = reporter.create_spinner("Testing...");
    assert!(!spinner.is_finished());

    let file_progress = reporter.create_file_progress("Downloading", 1024);
    assert_eq!(file_progress.length(), Some(1024));

    let network_progress = reporter.create_network_progress("Connecting");
    assert!(!network_progress.is_finished());
}

#[test]
fn test_capture_progress() {
    let mut reporter = ProgressReporter::new();

    let progress = reporter.start_capture_progress("https://example.com");
    assert!(!progress.is_finished());

    reporter.update_capture_step("Loading page");
    reporter.finish_capture_success("test-snapshot");

    // Progress should be finished after success
    assert!(progress.is_finished());
}

#[test]
fn test_sub_progress() {
    let reporter = ProgressReporter::new();

    let sub_progress = reporter.start_sub_progress("Processing files", 50);
    assert_eq!(sub_progress.length(), Some(50));
    assert_eq!(sub_progress.position(), 0);

    sub_progress.inc(10);
    assert_eq!(sub_progress.position(), 10);
}

#[test]
fn test_operation_summary() {
    let reporter = ProgressReporter::new();
    let duration = Duration::from_secs(5);
    let details = vec![
        ("Files processed", "42".to_string()),
        ("Total size", "1.2 MB".to_string()),
    ];

    // This should not panic
    reporter.show_operation_summary("Test Operation", duration, &details);
}
