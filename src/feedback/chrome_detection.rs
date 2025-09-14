use crate::error::{Result, WebMockError};
use crate::feedback::UserFeedback;
use std::process::Command;
use std::time::{Duration, Instant};

/// Chrome detection and installation guidance
pub struct ChromeDetection;

impl ChromeDetection {
    /// Check if Chrome or Chromium is available on the system with detailed diagnostics
    pub fn check_chrome_availability() -> Result<String> {
        // Platform-specific Chrome locations
        let chrome_commands = Self::get_platform_chrome_paths();

        let mut attempted_paths = Vec::new();

        if let Ok(env_path) = std::env::var("CHROME_PATH") {
            attempted_paths.push(env_path.clone());
            if let Ok(output) = Self::run_command_with_timeout(&env_path, Duration::from_secs(2)) {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    return Ok(format!(
                        "Found Chrome via CHROME_PATH: {} ({})",
                        env_path,
                        version.trim()
                    ));
                }
            }
        }

        for cmd in &chrome_commands {
            attempted_paths.push(cmd.to_string());

            if let Ok(output) = Self::run_command_with_timeout(cmd, Duration::from_secs(1)) {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    return Ok(format!("Found Chrome: {} ({})", cmd, version.trim()));
                }
            }
        }

        // If we get here, Chrome wasn't found - provide detailed diagnostics
        UserFeedback::warning("Chrome browser not found in standard locations");
        UserFeedback::info("Searched the following locations:");
        for path in &attempted_paths {
            UserFeedback::tip(&format!("• {}", path));
        }

        Err(WebMockError::ChromeNotFound)
    }

    /// Run a command with a timeout to prevent hanging
    fn run_command_with_timeout(
        cmd: &str,
        timeout: Duration,
    ) -> std::io::Result<std::process::Output> {
        let start = Instant::now();

        match Command::new(cmd).arg("--version").spawn() {
            Ok(mut child) => {
                // Poll with timeout
                while start.elapsed() < timeout {
                    if let Ok(Some(status)) = child.try_wait() {
                        // Process completed, return basic output
                        return Ok(std::process::Output {
                            status,
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }

                // Timeout reached
                let _ = child.kill();
                let _ = child.wait();
                Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Command timed out",
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Get platform-specific Chrome installation paths
    fn get_platform_chrome_paths() -> Vec<&'static str> {
        #[cfg(target_os = "macos")]
        {
            vec![
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
                "google-chrome",
                "chromium",
            ]
        }

        #[cfg(target_os = "linux")]
        {
            vec![
                "google-chrome",
                "google-chrome-stable",
                "chromium",
                "chromium-browser",
                "/usr/bin/google-chrome",
                "/usr/bin/google-chrome-stable",
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
                "/snap/bin/chromium",
                "/opt/google/chrome/chrome",
                "/usr/local/bin/chrome",
            ]
        }

        #[cfg(target_os = "windows")]
        {
            vec![
                "chrome.exe",
                "google-chrome.exe",
                "chromium.exe",
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
                r"C:\Users\%USERNAME%\AppData\Local\Google\Chrome\Application\chrome.exe",
            ]
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            vec!["google-chrome", "chromium", "chrome"]
        }
    }

    /// Provide platform-specific installation instructions
    pub fn show_installation_help() {
        UserFeedback::section("🌐 Chrome Installation Required");

        println!("WebMock CLI requires Google Chrome or Chromium to capture web pages.");
        println!();

        // Detect platform and show appropriate instructions
        if cfg!(target_os = "macos") {
            Self::show_macos_instructions();
        } else if cfg!(target_os = "linux") {
            Self::show_linux_instructions();
        } else if cfg!(target_os = "windows") {
            Self::show_windows_instructions();
        } else {
            Self::show_generic_instructions();
        }

        println!();
        UserFeedback::tip("After installation, restart your terminal and try the command again");
    }

    fn show_macos_instructions() {
        UserFeedback::info("macOS Installation Options:");
        println!();
        println!("Option 1 - Homebrew (Recommended):");
        println!("  brew install --cask google-chrome");
        println!();
        println!("Option 2 - Direct Download:");
        println!("  Visit: https://www.google.com/chrome/");
        println!("  Download and install the .dmg file");
        println!();
        println!("Option 3 - Chromium (Open Source):");
        println!("  brew install --cask chromium");
    }

    fn show_linux_instructions() {
        UserFeedback::info("Linux Installation Options:");
        println!();
        println!("Ubuntu/Debian:");
        println!("  sudo apt update");
        println!("  sudo apt install google-chrome-stable");
        println!("  # Or for Chromium:");
        println!("  sudo apt install chromium-browser");
        println!();
        println!("Fedora/RHEL:");
        println!("  sudo dnf install google-chrome-stable");
        println!("  # Or for Chromium:");
        println!("  sudo dnf install chromium");
        println!();
        println!("Arch Linux:");
        println!("  sudo pacman -S google-chrome");
        println!("  # Or for Chromium:");
        println!("  sudo pacman -S chromium");
        println!();
        println!("Snap (Universal):");
        println!("  sudo snap install chromium");
    }

    fn show_windows_instructions() {
        UserFeedback::info("Windows Installation Options:");
        println!();
        println!("Option 1 - Direct Download (Recommended):");
        println!("  Visit: https://www.google.com/chrome/");
        println!("  Download and run the installer");
        println!();
        println!("Option 2 - Chocolatey:");
        println!("  choco install googlechrome");
        println!();
        println!("Option 3 - Winget:");
        println!("  winget install Google.Chrome");
    }

    fn show_generic_instructions() {
        UserFeedback::info("Installation Instructions:");
        println!();
        println!("Please install Google Chrome or Chromium:");
        println!("  • Google Chrome: https://www.google.com/chrome/");
        println!("  • Chromium: https://www.chromium.org/");
        println!();
        println!("Make sure the browser is installed in a standard location");
        println!("or available in your system's PATH.");
    }

    /// Validate Chrome installation and provide helpful feedback
    pub fn validate_and_guide() -> Result<()> {
        match Self::check_chrome_availability() {
            Ok(chrome_info) => {
                UserFeedback::success(&chrome_info);
                Ok(())
            }
            Err(e) => {
                UserFeedback::error("Chrome browser not found");
                Self::show_installation_help();
                Err(e)
            }
        }
    }
}
