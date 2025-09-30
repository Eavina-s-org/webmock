use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the webmock binary for testing
fn get_webmock_binary_path() -> &'static str {
    if cfg!(debug_assertions) {
        "./target/debug/webmock"
    } else {
        "./target/release/webmock"
    }
}

/// Helper function to run the webmock binary directly
fn run_webmock_command(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let binary_path = get_webmock_binary_path();

    if !Path::new(binary_path).exists() {
        panic!(
            "WebMock binary not found at {}. Please run `cargo build --bin webmock` first.",
            binary_path
        );
    }

    let mut cmd = Command::new(binary_path);
    cmd.args(args);

    for (key, value) in envs {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to execute webmock command")
}

/// Helper function to setup test environment
fn setup_test_env() -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    std::fs::create_dir(&snapshot_dir).unwrap();
    let data_dir = temp_dir.path().to_string_lossy().into_owned();
    (temp_dir, data_dir)
}

#[test]
fn test_cli_help_command() {
    let output = run_webmock_command(&["--help"], &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WebMock CLI"));
    assert!(stdout.contains("capture"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("serve"));
    assert!(stdout.contains("delete"));
    assert!(stdout.contains("inspect"));
}

#[test]
fn test_cli_generate_completion() {
    let output = run_webmock_command(&["--generate-completion", "bash"], &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("webmock"));
    assert!(stdout.contains("complete"));
}

#[test]
fn test_cli_list_empty_snapshots() {
    let (_temp_dir, data_dir) = setup_test_env();

    let output = run_webmock_command(&["list", "--storage", &data_dir], &[]);

    // 打印调试信息
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 期望看到空快照的输出
    assert!(
        stdout.contains("No Snapshots Found")
            || stdout.contains("No snapshots")
            || stdout.contains("You haven't created any snapshots yet"),
        "Expected empty snapshots message, but got: {}",
        stdout
    );
}

#[test]
fn test_cli_invalid_command() {
    let output = run_webmock_command(&["invalid-command"], &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("Error"));
}

#[test]
fn test_cli_no_command() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_webmock_command(&[], &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    Ok(())
}

#[test]
fn test_cli_inspect_nonexistent() {
    let (_temp_dir, data_dir) = setup_test_env();

    let output = run_webmock_command(&["inspect", "nonexistent", "--storage", &data_dir], &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Snapshot"));
}

#[test]
fn test_cli_delete_nonexistent() {
    let (_temp_dir, data_dir) = setup_test_env();

    let output = run_webmock_command(&["delete", "nonexistent", "--storage", &data_dir], &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Snapshot"));
}

#[test]
fn test_cli_serve_nonexistent() {
    let (_temp_dir, data_dir) = setup_test_env();

    let output = run_webmock_command(&["serve", "nonexistent", "--storage", &data_dir], &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Snapshot"));
}

#[test]
fn test_cli_inspect_help() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_webmock_command(&["inspect", "--help"], &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("inspect"));
    Ok(())
}

#[test]
fn test_cli_serve_help() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_webmock_command(&["serve", "--help"], &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("serve"));
    Ok(())
}

#[test]
fn test_cli_serve_with_invalid_port() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, data_dir) = setup_test_env();

    let output = run_webmock_command(
        &[
            "serve",
            "--port",
            "0",
            "nonexistent",
            "--storage",
            &data_dir,
        ],
        &[],
    );

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr);
    assert!(combined.contains("not found") || combined.contains("error"));
    Ok(())
}
