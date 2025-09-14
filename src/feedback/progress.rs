use crate::feedback::UserFeedback;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress reporter for long-running operations
pub struct ProgressReporter {
    multi_progress: MultiProgress,
    main_bar: Option<ProgressBar>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new() -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            main_bar: None,
        }
    }

    /// Start a new progress bar for capture operations
    pub fn start_capture_progress(&mut self, url: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .expect("Invalid progress template"),
        );
        pb.set_message(format!("🚀 Capturing {}", url));
        pb.enable_steady_tick(Duration::from_millis(120));

        self.main_bar = Some(pb.clone());
        pb
    }

    /// Update capture progress with current step
    pub fn update_capture_step(&self, step: &str) {
        if let Some(pb) = &self.main_bar {
            pb.set_message(format!("📡 {}", step));
        }
    }

    /// Start a sub-progress bar for specific operations
    pub fn start_sub_progress(&self, message: &str, total: u64) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}",
                )
                .expect("Invalid progress template")
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Finish capture progress with success
    pub fn finish_capture_success(&self, snapshot_name: &str) {
        if let Some(pb) = &self.main_bar {
            pb.finish_with_message(format!("✅ Capture completed: {}", snapshot_name));
        }
    }

    /// Finish capture progress with error
    pub fn finish_capture_error(&self, error: &str) {
        if let Some(pb) = &self.main_bar {
            pb.finish_with_message(format!("❌ Capture failed: {}", error));
        }
    }

    /// Create a simple spinner for loading operations
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.blue} {msg}")
                .expect("Invalid progress template"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Create a progress bar for file operations
    pub fn create_file_progress(&self, message: &str, total_bytes: u64) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .expect("Invalid progress template")
                .progress_chars("#>-")
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a progress bar for network operations
    pub fn create_network_progress(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
                .template("{spinner:.cyan} {msg} [{elapsed_precise}]")
                .expect("Invalid progress template"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Update progress with detailed status
    pub fn update_detailed_progress(&self, step: &str, details: &str) {
        if let Some(pb) = &self.main_bar {
            pb.set_message(format!("📡 {} - {}", step, details));
        }
    }

    /// Show operation summary
    pub fn show_operation_summary(
        &self,
        operation: &str,
        duration: Duration,
        details: &[(&str, String)],
    ) {
        println!();
        UserFeedback::section(&format!("📊 {} Summary", operation));
        println!("   ⏱️  Duration: {:.2}s", duration.as_secs_f64());

        for (label, value) in details {
            println!("   📋 {}: {}", label, value);
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_creation() {
        let mut reporter = ProgressReporter::new();

        // Test capture progress creation
        let _progress = reporter.start_capture_progress("https://example.com");

        // Test sub-progress creation
        let sub_progress = reporter.start_sub_progress("Loading data", 100);
        assert_eq!(sub_progress.length(), Some(100));

        // Test spinner creation
        let _spinner = reporter.create_spinner("Processing...");
    }
}
