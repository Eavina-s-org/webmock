# Changelog

All notable changes to WebMock CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of WebMock CLI
- Complete web page capture functionality
- HTTP proxy server for request interception
- Browser automation using Chrome/Chromium
- Mock server for replaying captured sessions
- MessagePack-based snapshot storage
- Comprehensive CLI with capture, list, serve, and delete commands
- Progress indicators and user-friendly error messages
- Support for all HTTP methods and content types
- Automatic port conflict resolution
- Graceful shutdown handling

### Technical Features
- Async Rust implementation using Tokio
- Streaming support for large response bodies
- Connection pooling for HTTP operations
- Efficient binary serialization with MessagePack
- Comprehensive error handling with user-friendly messages
- Modular architecture with clear separation of concerns
- Extensive test coverage including integration tests
- Performance optimizations for memory and disk usage

## [0.1.0] - 2025-09-10

### Added
- Initial project setup and core architecture
- Basic CLI interface with clap
- Storage module with MessagePack serialization
- HTTP proxy implementation with hyper
- Browser controller using chromiumoxide
- Mock server for serving captured content
- Error handling system with thiserror
- Comprehensive test suite
- Documentation and examples

### Features
- **Capture Command**: Record complete web sessions
  - Support for HTTP and HTTPS URLs
  - Configurable timeout settings
  - Progress indicators during capture
  - Automatic browser management

- **List Command**: View saved snapshots
  - Formatted output with metadata
  - Creation timestamps and request counts
  - Error detection for corrupted snapshots

- **Serve Command**: Start mock server
  - Exact response replay with original headers
  - Static asset serving
  - 404 handling for missing resources
  - Port conflict resolution

- **Delete Command**: Remove snapshots
  - Confirmation prompts for safety
  - Clear error messages
  - Graceful handling of missing snapshots

### Technical Implementation
- Modular Rust architecture
- Async/await throughout for performance
- Comprehensive error handling
- MessagePack for efficient storage
- Chrome DevTools Protocol integration
- HTTP proxy with request recording
- Hyper-based HTTP server
- Cross-platform compatibility

### Dependencies
- `clap` 4.0 - Command-line argument parsing
- `tokio` 1.0 - Async runtime
- `chromiumoxide` 0.5 - Browser automation
- `hyper` 0.14 - HTTP client/server
- `serde` 1.0 - Serialization framework
- `rmp-serde` 1.1 - MessagePack format
- `thiserror` 1.0 - Error handling
- `anyhow` 1.0 - Error context
- `chrono` 0.4 - Date/time handling
- `tracing` 0.1 - Structured logging

### Documentation
- Comprehensive README with examples
- API documentation with rustdoc
- Contributing guidelines
- Troubleshooting guide
- Installation instructions

### Testing
- Unit tests for all modules
- Integration tests for workflows
- Performance tests for large snapshots
- Error handling test coverage
- Cross-platform testing

---

## Version History Notes

### Versioning Strategy
- **Major versions** (1.0, 2.0): Breaking API changes, major feature overhauls
- **Minor versions** (0.1, 0.2): New features, backward-compatible changes
- **Patch versions** (0.1.1, 0.1.2): Bug fixes, security updates

### Planned Features (Future Versions)

#### v0.2.0 - Enhanced Capture
- WebSocket recording support
- Custom request filtering
- Capture session resumption
- Multiple page capture workflows

#### v0.3.0 - Advanced Serving
- Request modification capabilities
- Response templating
- Load balancing between snapshots
- API endpoint mocking enhancements

#### v0.4.0 - Developer Experience
- Configuration file support
- Plugin system for extensions
- Interactive capture mode
- Snapshot comparison tools

#### v1.0.0 - Production Ready
- Stable API guarantees
- Performance optimizations
- Enterprise features
- Comprehensive documentation

### Breaking Changes Policy

We follow semantic versioning strictly:
- **Patch releases** (0.1.x): No breaking changes, safe to update
- **Minor releases** (0.x.0): New features, backward compatible
- **Major releases** (x.0.0): May include breaking changes

Breaking changes will be:
1. Clearly documented in changelog
2. Announced in advance when possible
3. Include migration guides
4. Provide deprecation warnings in prior versions

### Support Policy

- **Current version**: Full support with bug fixes and security updates
- **Previous minor version**: Security updates only
- **Older versions**: Community support only

### Contributing to Changelog

When contributing:
1. Add entries to `[Unreleased]` section
2. Use appropriate categories (Added, Changed, Deprecated, Removed, Fixed, Security)
3. Include issue/PR references where applicable
4. Follow the established format and style

### Release Process

1. Update version in `Cargo.toml`
2. Move unreleased changes to new version section
3. Add release date
4. Create git tag
5. Publish to crates.io
6. Create GitHub release with changelog excerpt