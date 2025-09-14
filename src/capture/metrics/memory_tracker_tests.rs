use super::MemoryTracker;

#[test]
fn test_memory_tracker_creation() {
    let tracker = MemoryTracker::new();
    let _ = tracker.peak_memory();
}

#[test]
fn test_memory_tracker_update() {
    let mut tracker = MemoryTracker::new();
    let initial_peak = tracker.peak_memory();

    tracker.update();
    let new_peak = tracker.peak_memory();

    assert!(new_peak >= initial_peak);
}

#[test]
fn test_memory_tracker_bytes_to_human() {
    assert_eq!(MemoryTracker::bytes_to_human(0), "0 B");
    assert_eq!(MemoryTracker::bytes_to_human(1023), "1023 B");
    assert_eq!(MemoryTracker::bytes_to_human(1024), "1.00 KB");
    assert_eq!(MemoryTracker::bytes_to_human(1536), "1.50 KB");
    assert_eq!(MemoryTracker::bytes_to_human(1024 * 1024), "1.00 MB");
    assert_eq!(MemoryTracker::bytes_to_human(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(
        MemoryTracker::bytes_to_human(2 * 1024 * 1024 * 1024),
        "2.00 GB"
    );
}

#[test]
fn test_memory_tracker_memory_delta() {
    let mut tracker = MemoryTracker::new();
    let _initial_delta = tracker.memory_delta();

    tracker.update();
    let new_delta = tracker.memory_delta();

    // Delta should be a reasonable value (not necessarily positive)
    assert!(new_delta >= -(1024 * 1024 * 100)); // Not more than 100MB decrease
    assert!(new_delta <= 1024 * 1024 * 100); // Not more than 100MB increase
}

#[test]
fn test_memory_tracker_default() {
    let tracker = MemoryTracker::default();
    let _ = tracker.peak_memory();
}

#[test]
fn test_memory_tracker_get_current_memory_usage() {
    let memory = MemoryTracker::get_current_memory_usage();
    // Memory usage should be non-negative and reasonable
    // Memory usage should be reasonable
    assert!(memory < 1024 * 1024 * 1024 * 10); // Less than 10GB
}
