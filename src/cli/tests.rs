use crate::cli::*;
use clap::Parser;

#[test]
fn test_cli_parsing_capture_command() {
    // Test basic capture command
    let args = [
        "webmock",
        "capture",
        "https://example.com",
        "--name",
        "test-site",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture {
            url, name, timeout, ..
        }) => {
            assert_eq!(url, "https://example.com".to_string());
            assert_eq!(name, "test-site".to_string());
            assert_eq!(timeout, 30); // default value
        }
        _ => panic!("Expected Capture command"),
    }
}

#[test]
fn test_cli_parsing_capture_with_timeout() {
    // Test capture command with custom timeout
    let args = [
        "webmock",
        "capture",
        "https://example.com",
        "--name",
        "test-site",
        "--timeout",
        "60",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture {
            url, name, timeout, ..
        }) => {
            assert_eq!(url, "https://example.com");
            assert_eq!(name, "test-site");
            assert_eq!(timeout, 60);
        }
        _ => panic!("Expected Capture command"),
    }
}

#[test]
fn test_cli_parsing_serve_command() {
    // Test basic serve command
    let args = ["webmock", "serve", "test-snapshot"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Serve {
            snapshot_name,
            port,
            ..
        }) => {
            assert_eq!(snapshot_name, "test-snapshot");
            assert_eq!(port, 8080); // default value
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parsing_serve_with_port() {
    // Test serve command with custom port
    let args = ["webmock", "serve", "test-snapshot", "--port", "3000"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Serve {
            snapshot_name,
            port,
            ..
        }) => {
            assert_eq!(snapshot_name, "test-snapshot");
            assert_eq!(port, 3000);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parsing_list_command() {
    // Test list command
    let args = ["webmock", "list"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::List { storage: None }) => {
            // List command has no parameters
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_cli_parsing_delete_command() {
    // Test delete command
    let args = ["webmock", "delete", "old-snapshot"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Delete { snapshot_name, .. }) => {
            assert_eq!(snapshot_name, "old-snapshot");
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn test_cli_parsing_missing_required_args() {
    // Test capture command without required name argument
    let args = ["webmock", "capture", "https://example.com"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());

    // Test capture command without URL
    let args = ["webmock", "capture"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());

    // Test serve command without snapshot name
    let args = ["webmock", "serve"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());

    // Test delete command without snapshot name
    let args = ["webmock", "delete"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_parsing_invalid_timeout() {
    // Test with non-numeric timeout
    let args = [
        "webmock",
        "capture",
        "https://example.com",
        "--name",
        "test",
        "--timeout",
        "invalid",
    ];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_parsing_invalid_port() {
    // Test with non-numeric port
    let args = ["webmock", "serve", "test", "--port", "invalid"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());

    // Test with port out of range (greater than u16::MAX)
    let args = ["webmock", "serve", "test", "--port", "70000"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_parsing_help_flags() {
    // Test --help flag
    let args = ["webmock", "--help"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // Help causes early exit

    // Test -h flag
    let args = ["webmock", "-h"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // Help causes early exit
}

#[test]
fn test_cli_parsing_version_flags() {
    // Test --version flag
    let args = ["webmock", "--version"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // Version causes early exit

    // Test -V flag
    let args = ["webmock", "-V"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // Version causes early exit
}

#[test]
fn test_cli_parsing_unknown_command() {
    // Test unknown command
    let args = ["webmock", "unknown"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_parsing_edge_cases() {
    // Test with minimum valid timeout
    let args = [
        "webmock",
        "capture",
        "https://example.com",
        "--name",
        "test",
        "--timeout",
        "1",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture { timeout, .. }) => {
            assert_eq!(timeout, 1);
        }
        _ => panic!("Expected Capture command"),
    }

    // Test with minimum valid port
    let args = ["webmock", "serve", "test", "--port", "1"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Serve { port, .. }) => {
            assert_eq!(port, 1);
        }
        _ => panic!("Expected Serve command"),
    }

    // Test with maximum valid port
    let args = ["webmock", "serve", "test", "--port", "65535"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Serve { port, .. }) => {
            assert_eq!(port, 65535);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parsing_special_characters_in_names() {
    // Test snapshot names with hyphens and underscores
    let args = [
        "webmock",
        "capture",
        "https://example.com",
        "--name",
        "test-snapshot_123",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture { name, .. }) => {
            assert_eq!(name, "test-snapshot_123");
        }
        _ => panic!("Expected Capture command"),
    }

    // Test serving snapshot with special characters
    let args = ["webmock", "serve", "my-test_snapshot-v2"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Serve { snapshot_name, .. }) => {
            assert_eq!(snapshot_name, "my-test_snapshot-v2");
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parsing_urls_with_paths_and_params() {
    // Test URL with path and query parameters
    let url = "https://api.example.com/v1/users?page=1&limit=10";
    let args = ["webmock", "capture", url, "--name", "api-test"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture {
            url: parsed_url, ..
        }) => {
            assert_eq!(parsed_url, url);
        }
        _ => panic!("Expected Capture command"),
    }

    // Test localhost URL
    let localhost_url = "http://localhost:3000/dashboard";
    let args = ["webmock", "capture", localhost_url, "--name", "local-app"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::Capture {
            url: parsed_url, ..
        }) => {
            assert_eq!(parsed_url, localhost_url);
        }
        _ => panic!("Expected Capture command"),
    }
}

#[test]
fn test_cli_parsing_generate_completion() {
    // Test generate completion for bash
    let args = ["webmock", "--generate-completion", "bash"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.generate_completion.is_some());

    // Test generate completion for zsh
    let args = ["webmock", "--generate-completion", "zsh"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.generate_completion.is_some());

    // Test generate completion for fish
    let args = ["webmock", "--generate-completion", "fish"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.generate_completion.is_some());
}
