# WebMock CLI Examples

This directory contains practical examples and sample workflows for using WebMock CLI effectively.

## Basic Examples

### Simple Website Capture

```bash
# Capture a news website
webmock capture https://news.ycombinator.com --name hackernews

# Start mock server
webmock serve hackernews --port 8080

# Visit http://localhost:8080 to see the captured site
```

### Creating Test Snapshots Programmatically

For development and testing purposes, you can create snapshots programmatically:

```bash
# Run the example to create a comprehensive test snapshot
cargo run --example create_test_snapshot

# Serve the created test snapshot
webmock serve test-snapshot --port 8080

# Visit http://localhost:8080 to see the interactive test page
```

This example ([`create_test_snapshot.rs`](create_test_snapshot.rs)) creates a complete test snapshot with:
- Interactive HTML page with CSS and JavaScript
- API endpoints with JSON responses  
- Different HTTP methods (GET, POST)
- Various content types (HTML, CSS, JS, JSON)

Perfect for:
- Testing WebMock CLI functionality
- Demonstrating capabilities to others
- Creating reproducible development environments

### API-Heavy Application

```bash
# Capture a dashboard with many API calls (longer timeout)
webmock capture https://dashboard.example.com --name dashboard --timeout 120

# Serve on custom port
webmock serve dashboard --port 3000
```

### Local Development Site

```bash
# Capture your local development server
webmock capture http://localhost:3000 --name local-app

# Later, serve it when the dev server is down
webmock serve local-app --port 8080
```

## Advanced Workflows

### Development Against Production APIs

**Scenario**: You want to develop frontend features against production API responses without hitting the live API repeatedly.

```bash
# 1. Capture your app in the desired state
webmock capture https://app.example.com/dashboard?user=testuser --name prod-api-state

# 2. During development, serve the captured state
webmock serve prod-api-state --port 8080

# 3. Configure your dev environment to use localhost:8080
# Your frontend can now make API calls to the mock server
```

### Testing Different User States

**Scenario**: Capture different application states for comprehensive testing.

```bash
# Capture different user scenarios
webmock capture https://app.com/dashboard --name empty-dashboard
webmock capture https://app.com/dashboard?user=admin --name admin-dashboard  
webmock capture https://app.com/dashboard?user=premium --name premium-dashboard

# Use in automated tests
webmock serve empty-dashboard --port 8081 &
webmock serve admin-dashboard --port 8082 &
webmock serve premium-dashboard --port 8083 &

# Run tests against different scenarios
npm test -- --config test-empty.json
npm test -- --config test-admin.json
npm test -- --config test-premium.json
```

### Demo Preparation

**Scenario**: Prepare a perfect demo that works without internet dependency.

```bash
# Capture your app in perfect demo state
webmock capture https://demo.myapp.com --name perfect-demo --timeout 60

# During presentation
webmock serve perfect-demo --port 8080
# Show http://localhost:8080 - guaranteed to work!
```

### API Documentation and Examples

**Scenario**: Create interactive API documentation with real responses.

```bash
# Capture API endpoints
webmock capture https://api.example.com/users --name api-users
webmock capture https://api.example.com/products --name api-products

# Serve for documentation
webmock serve api-users --port 8080
# Now your docs can show live examples at localhost:8080
```

## Environment-Specific Examples

### Using Environment Variables

```bash
# Custom storage location
export WEBMOCK_STORAGE_PATH="/project/snapshots"
webmock capture https://example.com --name project-snapshot

# Custom Chrome path
export CHROME_PATH="/Applications/Chromium.app/Contents/MacOS/Chromium"
webmock capture https://example.com --name chromium-test

# Debug logging
export RUST_LOG=debug
webmock capture https://example.com --name debug-capture
```

### CI/CD Integration

```bash
#!/bin/bash
# ci-capture.sh - Capture snapshots in CI

set -e

# Capture production state for testing
webmock capture https://prod.example.com --name prod-snapshot --timeout 180

# Start mock server for tests
webmock serve prod-snapshot --port 8080 &
MOCK_PID=$!

# Wait for server to start
sleep 5

# Run tests against mock
npm test

# Cleanup
kill $MOCK_PID
```

### Docker Integration

```dockerfile
# Dockerfile for containerized testing
FROM rust:1.70

# Install Chrome
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    && wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable

# Install WebMock CLI
COPY . /app
WORKDIR /app
RUN cargo install --path .

# Capture and serve
CMD ["webmock", "serve", "snapshot", "--port", "8080"]
```

## Troubleshooting Examples

### Handling Timeouts

```bash
# For slow-loading sites, increase timeout
webmock capture https://slow-site.com --name slow-site --timeout 300

# For sites with lots of async content
webmock capture https://spa-app.com --name spa --timeout 120
```

### Debugging Captures

```bash
# Enable debug logging to see what's happening
export RUST_LOG=debug
webmock capture https://problematic-site.com --name debug-test

# Check what was captured
webmock list
# Look at the request count - should match expectations
```

### Port Conflicts

```bash
# If default port is busy, use another
webmock serve snapshot --port 8081

# Or let the system find an available port
webmock serve snapshot --port 0  # Will use random available port
```

## Integration Examples

### With Jest/Node.js Testing

```javascript
// test-setup.js
const { spawn } = require('child_process');

let mockServer;

beforeAll(async () => {
  // Start mock server
  mockServer = spawn('webmock', ['serve', 'test-snapshot', '--port', '8080']);
  
  // Wait for server to start
  await new Promise(resolve => setTimeout(resolve, 2000));
});

afterAll(() => {
  if (mockServer) {
    mockServer.kill();
  }
});

// Your tests can now use http://localhost:8080
```

### With Python/pytest

```python
# conftest.py
import subprocess
import time
import pytest

@pytest.fixture(scope="session")
def mock_server():
    # Start mock server
    process = subprocess.Popen([
        'webmock', 'serve', 'test-snapshot', '--port', '8080'
    ])
    
    # Wait for startup
    time.sleep(2)
    
    yield 'http://localhost:8080'
    
    # Cleanup
    process.terminate()
    process.wait()

def test_api_endpoint(mock_server):
    import requests
    response = requests.get(f"{mock_server}/api/users")
    assert response.status_code == 200
```

### With Makefile

```makefile
# Makefile for project with WebMock integration

.PHONY: capture-prod serve-mock test-with-mock

capture-prod:
	webmock capture https://prod.example.com --name prod-snapshot --timeout 120

serve-mock:
	webmock serve prod-snapshot --port 8080

test-with-mock: serve-mock
	sleep 2  # Wait for server
	npm test &
	TEST_PID=$$!; \
	wait $$TEST_PID; \
	pkill -f "webmock serve"

clean-snapshots:
	rm -rf ~/.webmock/snapshots/*
```

## Performance Examples

### Large Site Capture

```bash
# For very large sites with many resources
webmock capture https://large-site.com --name large --timeout 600

# Check snapshot size
ls -lh ~/.webmock/snapshots/large.msgpack
```

### Memory-Efficient Serving

```bash
# For serving large snapshots efficiently
# WebMock automatically streams large responses
webmock serve large-snapshot --port 8080
```

## Security Examples

### Capturing Authenticated Sessions

```bash
# Note: WebMock captures the session as-is
# Make sure you're comfortable with storing auth tokens

# Capture after logging in manually
# 1. Open Chrome, log into your app
# 2. Note the URL of the authenticated page
# 3. Capture that URL
webmock capture https://app.com/dashboard --name authenticated-session
```

### Sanitizing Snapshots

```bash
# After capture, you might want to review/sanitize
# Snapshots are stored in ~/.webmock/snapshots/
# You can manually edit or delete sensitive data if needed
```

## Best Practices

### Naming Conventions

```bash
# Use descriptive names with context
webmock capture https://app.com --name app-homepage-v1
webmock capture https://app.com/admin --name app-admin-panel
webmock capture https://api.com/v2 --name api-v2-endpoints

# Include dates for time-sensitive captures
webmock capture https://news.com --name news-2024-01-15
```

### Snapshot Management

```bash
# Regular cleanup of old snapshots
webmock list  # Review what you have
webmock delete old-snapshot-name

# Organize by project
export WEBMOCK_STORAGE_PATH="/project/snapshots"
```

### Testing Workflow

```bash
# 1. Capture baseline
webmock capture https://app.com --name baseline

# 2. Make changes to your app
# ... development work ...

# 3. Test against baseline
webmock serve baseline --port 8080
# Run your tests

# 4. Capture new state if needed
webmock capture https://app.com --name updated-version
```

These examples should help you get started with WebMock CLI and integrate it effectively into your development workflow!
##
 Advanced Integration Examples

### With Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  webmock:
    build:
      context: .
      dockerfile: Dockerfile.webmock
    ports:
      - "8080:8080"
    volumes:
      - ./snapshots:/snapshots
    environment:
      - WEBMOCK_STORAGE_PATH=/snapshots
      - CHROME_ARGS=--headless --no-sandbox --disable-dev-shm-usage
    command: webmock serve test-snapshot --port 8080

  app:
    build: .
    depends_on:
      - webmock
    environment:
      - API_BASE_URL=http://webmock:8080
    ports:
      - "3000:3000"

  tests:
    build:
      context: .
      dockerfile: Dockerfile.test
    depends_on:
      - webmock
    environment:
      - TEST_API_URL=http://webmock:8080
    command: npm test
```

```dockerfile
# Dockerfile.webmock
FROM rust:1.70-slim

# Install Chrome and dependencies
RUN apt-get update && apt-get install -y \
    wget gnupg curl \
    && wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

# Install WebMock CLI
COPY . /app
WORKDIR /app
RUN cargo install --path .

# Configure for container environment
ENV CHROME_ARGS="--headless --no-sandbox --disable-dev-shm-usage --disable-gpu"
ENV WEBMOCK_STORAGE_PATH="/snapshots"

# Create storage directory
RUN mkdir -p /snapshots

EXPOSE 8080
CMD ["webmock", "--help"]
```

### With GitHub Actions

```yaml
# .github/workflows/test-with-webmock.yml
name: Test with WebMock

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Chrome
        run: |
          wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
          echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
          sudo apt update
          sudo apt install google-chrome-stable
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install WebMock CLI
        run: cargo install --path .
      
      - name: Configure WebMock
        run: |
          echo "CHROME_ARGS=--headless --no-sandbox --disable-dev-shm-usage" >> $GITHUB_ENV
          echo "WEBMOCK_STORAGE_PATH=${{ github.workspace }}/snapshots" >> $GITHUB_ENV
          mkdir -p ${{ github.workspace }}/snapshots
      
      - name: Capture test snapshot
        run: |
          webmock capture https://httpbin.org/json --name ci-test --timeout 60
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install dependencies
        run: npm ci
      
      - name: Run tests with WebMock
        run: |
          # Start mock server in background
          webmock serve ci-test --port 8080 &
          MOCK_PID=$!
          
          # Wait for server to start
          sleep 5
          
          # Run tests
          npm test
          
          # Cleanup
          kill $MOCK_PID || true
        env:
          TEST_API_URL: http://localhost:8080
```

### With Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    environment {
        CHROME_ARGS = '--headless --no-sandbox --disable-dev-shm-usage'
        WEBMOCK_STORAGE_PATH = "${WORKSPACE}/snapshots"
    }
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    # Install WebMock CLI
                    cargo install --path .
                    
                    # Create storage directory
                    mkdir -p ${WEBMOCK_STORAGE_PATH}
                '''
            }
        }
        
        stage('Capture Baseline') {
            steps {
                sh '''
                    # Capture production state
                    webmock capture https://prod.example.com --name pipeline-baseline --timeout 180
                '''
            }
        }
        
        stage('Test') {
            steps {
                sh '''
                    # Start mock server
                    webmock serve pipeline-baseline --port 8080 &
                    MOCK_PID=$!
                    
                    # Wait for server
                    sleep 5
                    
                    # Run tests
                    npm test
                    
                    # Cleanup
                    kill $MOCK_PID || true
                '''
            }
        }
        
        stage('Archive Snapshots') {
            steps {
                archiveArtifacts artifacts: 'snapshots/*.msgpack', fingerprint: true
            }
        }
    }
    
    post {
        always {
            sh 'pkill -f "webmock serve" || true'
        }
    }
}
```

## Real-World Use Cases

### E-commerce Testing

```bash
# Capture different shopping cart states
webmock capture https://shop.com/cart --name empty-cart
webmock capture https://shop.com/cart?items=3 --name cart-with-items
webmock capture https://shop.com/checkout --name checkout-flow

# Test payment flows without hitting real payment APIs
webmock serve checkout-flow --port 8080
# Run payment integration tests against localhost:8080
```

### API Version Testing

```bash
# Capture different API versions
webmock capture https://api.example.com/v1/users --name api-v1
webmock capture https://api.example.com/v2/users --name api-v2

# Test backward compatibility
webmock serve api-v1 --port 8081 &
webmock serve api-v2 --port 8082 &

# Run tests against both versions
npm test -- --api-v1-url=http://localhost:8081
npm test -- --api-v2-url=http://localhost:8082
```

### Mobile App Backend Testing

```bash
# Capture mobile API endpoints
webmock capture https://mobile-api.com/login --name mobile-login
webmock capture https://mobile-api.com/profile --name mobile-profile
webmock capture https://mobile-api.com/feed --name mobile-feed

# Test mobile app against captured APIs
webmock serve mobile-login --port 8080 &
webmock serve mobile-profile --port 8081 &
webmock serve mobile-feed --port 8082 &

# Configure mobile app to use localhost endpoints
# Run mobile app tests or manual testing
```

### Microservices Integration Testing

```bash
# Capture each microservice
webmock capture https://user-service.com/api --name user-service
webmock capture https://order-service.com/api --name order-service
webmock capture https://payment-service.com/api --name payment-service

# Start all services for integration testing
webmock serve user-service --port 8081 &
webmock serve order-service --port 8082 &
webmock serve payment-service --port 8083 &

# Run integration tests with all services mocked
```

These examples should help you integrate WebMock CLI effectively into your development and testing workflows!