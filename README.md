# WebMock CLI

A powerful command-line tool for recording and mocking web pages, built with Rust.

## Features

- 🎥 **Complete Session Recording**: Captures all HTTP/HTTPS requests and responses
- 🌐 **Full Web Page Support**: Records HTML, CSS, JS, images, fonts, and API calls
- 🚀 **Local Mock Server**: Replay captured sessions with a built-in HTTP server
- 📦 **Efficient Storage**: Compressed snapshots using MessagePack format
- 🔧 **Developer Friendly**: Simple CLI with helpful error messages
- ⚡ **Fast Performance**: Async Rust implementation

## Quick Start

### Installation

```bash
# from source
git clone https://github.com/Eavina/webmock-cli.git
cd webmock-cli
cargo install --path .
```

### Usage

```bash
# (Optional): Delete snapshot
webmock delete my-site

# Record a web session
webmock capture https://httpbin.org/json --name my-site

# List saved snapshots
webmock list

# Start mock server
webmock serve my-site --port 8080
```

```bash
# Test with curl
curl -x http://localhost:8080 https://httpbin.org/json --insecure
# Test with chrome
google-chrome https://httpbin.org/json --ignore-certificate-errors --ignore-ssl-errors --proxy-server=127.0.0.1:8080
```

### Requirements

- **Rust** 1.70+
- **Google Chrome** or Chromium browser

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `capture` | Record a web session | `webmock capture <url> --name <name>` |
| `list` | Show all snapshots | `webmock list` |
| `serve` | Start mock server | `webmock serve <name> --port 8080` |
| `delete` | Remove snapshot | `webmock delete <name>` |

## Documentation

- [Installation Guide](docs/INSTALLATION.md)
- [Configuration](docs/CONFIGURATION.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT License - see [LICENSE](LICENSE) file for details.