# Troubleshooting

## Quick Check
```bash
webmock --version          # Check installation
which google-chrome        # Check Chrome
webmock --help            # Test basic
```

## Common Issues

### Chrome Not Found
```bash
# Install Chrome
brew install --cask google-chrome      # macOS
sudo apt install google-chrome-stable  # Ubuntu

# Or set path
export CHROME_PATH="/path/to/chrome"
```

### Permission Error
```bash
# Fix permissions
mkdir -p ~/.webmock/snapshots
chmod 755 ~/.webmock

# Or use custom path
export WEBMOCK_STORAGE_PATH="/tmp/webmock"
```

### Timeout Issues
```bash
# Increase timeout
webmock capture https://site.com --timeout 120

# Test simple site
webmock capture https://httpbin.org/get --name test
```

### Debug Mode
```bash
export RUST_LOG=debug
webmock capture https://site.com --name debug
```

## Debug Commands
```bash
# Test Chrome manually
google-chrome --headless --dump-dom https://example.com

# Check logs
webmock list
webmock serve test --port 8080 &
curl -I http://localhost:8080
```