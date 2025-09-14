# Contributing to WebMock CLI

Thank you for your interest in contributing to WebMock CLI! This document provides guidelines and information for contributors.

## Development Environment Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Google Chrome or Chromium browser
- Git

### Setup Steps

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/webmock-cli.git
   cd webmock-cli
   ```

2. **Install Dependencies**
   ```bash
   cargo build
   ```

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Verify Installation**
   ```bash
   cargo run -- --help
   ```

## Project Structure

```
webmock-cli/
├── src/
│   ├── bin/webmock.rs          # Main binary entry point
│   ├── lib.rs                  # Library root
│   ├── cli                     # Command-line interface
│   ├── error                   # Error types and handling
│   ├── capture/                # Web page capture functionality
│   │   ├── mod.rs
│   │   ├── browser             # Browser automation
│   │   ├── network             # Network request handling
│   │   ├── session             # Capture session management
│   │   └── proxy               # HTTP proxy implementation
│   ├── storage/                # Snapshot storage and management
│   │   ├── mod.rs
│   │   ├── storage.rs          # Storage operations
│   │   └── serialization.rs    # Data serialization
│   ├── serve/                  # Mock server functionality
│   │   ├── mod.rs
│   │   └── handlers.rs           # HTTP server implementation
│   └── commands/               # CLI command implementations
│       ├── mod.rs
│       ├── capture
│       ├── list
│       ├── serve
│       └── delete
├── tests/                      # Integration tests
└── docs/                       # Documentation
    ├── README.md               # Documentation index
    ├── INSTALLATION.md         # Installation guide
    ├── CONFIGURATION.md        # Configuration guide
    ├── TROUBLESHOOTING.md      # Troubleshooting guide
    └── examples/               # Usage examples and workflows
```

## Development Guidelines

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Run clippy for linting: `cargo clippy`
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and reasonably sized

### Error Handling

- Use the `WebMockError` enum for all error types
- Provide helpful error messages for users
- Use `anyhow::Result` for internal error propagation
- Convert to `WebMockError` at module boundaries

### Testing

- Write unit tests for all new functionality
- Add integration tests for end-to-end workflows
- Use `cargo test` to run all tests
- Aim for good test coverage of critical paths

### Async Code

- Use `tokio` for async runtime
- Prefer `async/await` over manual futures
- Handle cancellation gracefully
- Use appropriate timeout values

## Making Changes

### Before You Start

1. Check existing issues and PRs to avoid duplication
2. Create an issue to discuss major changes
3. Fork the repository and create a feature branch

### Development Process

1. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   - Write code following the guidelines above
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Your Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

   Use conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions
   - `refactor:` for code refactoring

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

### Pull Request Guidelines

- Provide a clear description of changes
- Reference related issues with `Fixes #123`
- Include tests for new functionality
- Update documentation if needed
- Ensure CI passes

## Testing

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific test
cargo test test_capture_workflow

# With output
cargo test -- --nocapture
```

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test complete workflows
3. **Performance Tests**: Verify performance characteristics

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());
        
        // Test implementation
        assert!(result.is_ok());
    }
}
```

## Documentation

### Code Documentation

- Add `///` doc comments for public functions
- Include examples in doc comments when helpful
- Document error conditions and panics
- Use `cargo doc --open` to view generated docs

### User Documentation

- Update README.md for user-facing changes
- Add examples in `docs/examples/` for new features
- Update `docs/TROUBLESHOOTING.md` for new error conditions
- Update `docs/CONFIGURATION.md` for new configuration options

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release PR
5. Tag release after merge
6. Publish to crates.io (maintainers only)

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Code Review**: PR comments and suggestions

### Common Development Tasks

#### Adding a New Command

1. Add command variant to `Commands` enum in `src/cli.rs`
2. Create handler in `src/commands/new_command.rs`
3. Add command module to `src/commands/mod.rs`
4. Update main command router
5. Add tests and documentation

#### Adding New Error Types

1. Add variant to `WebMockError` enum in `src/error.rs`
2. Implement error message and display
3. Add conversion traits if needed
4. Update error handling in relevant modules

#### Modifying Storage Format

1. Update data structures in `src/storage/types.rs`
2. Handle backward compatibility
3. Add migration logic if needed
4. Update serialization tests

## Code of Conduct

Please be respectful and constructive in all interactions. We want to maintain a welcoming environment for all contributors.

### Guidelines

- Be respectful of different viewpoints and experiences
- Focus on what is best for the community
- Show empathy towards other community members
- Use welcoming and inclusive language

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- GitHub contributor statistics

Thank you for contributing to WebMock CLI!