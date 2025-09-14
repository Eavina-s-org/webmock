# Installation Guide

## Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/your-org/webmock-cli/main/install.sh | bash
```

## Manual Install

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Install Chrome
**macOS:**
```bash
brew install --cask google-chrome
```

**Ubuntu/Debian:**
```bash
wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
sudo apt update && sudo apt install google-chrome-stable
```

### 3. Install WebMock CLI
```bash
git clone https://github.com/your-org/webmock-cli.git
cd webmock-cli
cargo install --path .
```

## Verify Installation
```bash
webmock --version
webmock --help
```

## Platform Support
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)