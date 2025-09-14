# Configuration Guide

## Environment Variables

### Browser
- `CHROME_PATH`: Chrome executable path (auto-detected)

### Logging
- `RUST_LOG`: Log level (default: `info`)
- `RUST_LOG_STYLE`: Output style (default: `auto`)

## Quick Setup

```bash
# Custom Chrome
export CHROME_PATH="/usr/bin/chromium"

# Debug mode
export RUST_LOG=debug

# Run capture
webmock capture http://httpbin.org/get --name demo

# Start server
webmock serve demo --port 8080

# Test
curl -x http://localhost:8080 http://httpbin.org/get
```
