use tracing::debug;

/// Memory usage tracker
#[derive(Debug)]
pub struct MemoryTracker {
    initial_memory: u64,
    current_memory: u64,
    peak_memory: u64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        let initial_memory = Self::get_current_memory_usage();
        Self {
            initial_memory,
            current_memory: initial_memory,
            peak_memory: initial_memory,
        }
    }

    /// Get current memory usage in bytes
    pub fn get_current_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(size) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = size.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Rough estimation based on process info
            let pid = std::process::id();
            if let Ok(output) = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "rss="])
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(kb) = rss_str.trim().parse::<u64>() {
                        return kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }

        // Fallback: return 0 if we can't determine memory usage
        0
    }

    /// Update current memory usage
    pub fn update(&mut self) {
        self.current_memory = Self::get_current_memory_usage();
        if self.current_memory > self.peak_memory {
            self.peak_memory = self.current_memory;
        }
    }

    /// Get memory usage delta from initial
    pub fn memory_delta(&self) -> i64 {
        self.current_memory as i64 - self.initial_memory as i64
    }

    /// Get peak memory usage
    pub fn peak_memory(&self) -> u64 {
        self.peak_memory
    }

    /// Print memory usage summary
    pub fn print_summary(&self) {
        debug!("Memory Usage Summary:");
        debug!("  Initial: {} bytes", self.initial_memory);
        debug!("  Current: {} bytes", self.current_memory);
        debug!("  Peak: {} bytes", self.peak_memory);
        debug!("  Delta: {} bytes", self.memory_delta());
    }

    /// Convert bytes to human-readable format
    pub fn bytes_to_human(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
