# WebMock CLI Usage Examples

This document provides comprehensive examples for using WebMock CLI in various scenarios.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Development Workflows](#development-workflows)
- [Testing Scenarios](#testing-scenarios)
- [CI/CD Integration](#cicd-integration)
- [Advanced Use Cases](#advanced-use-cases)
- [Troubleshooting Examples](#troubleshooting-examples)

## Basic Usage

### Simple Website Capture

```bash
# Capture a basic website
webmock capture https://example.com --name example-site

# List captured snapshots
webmock list

# Serve the captured site
webmock serve example-site --port 8080

# Visit http://localhost:8080 to see the captured site
```

### API Endpoint Capture

```bash
# Capture API responses
webmock capture https://jsonplaceholder.typicode.com/posts --name api-posts
webmock capture https://httpbin.org/json --name httpbin-json

# Serve API endpoints
webmock serve api-posts --port 8080
curl http://localhost:8080/posts  # Returns captured API response
```

### Complex Application Capture

```bash
# Capture a complex web application with longer timeout
webmock capture https://dashboard.example.com --name dashboard --timeout 120

# The capture will include:
# - Main HTML page
# - All CSS and JavaScript files
# - Images, fonts, and other assets
# - API calls made during page load
# - AJAX requests and responses
```

## Development Workflows

### Frontend Development Against Production APIs

```bash
# 1. Capture production API state
webmock capture https://api.production.com/dashboard --name prod-api-v1 --timeout 90

# 2. Start development server with captured APIs
webmock serve prod-api-v1 --port 8080 &

# 3. Configure your frontend to use the mock
export REACT_APP_API_URL=http://localhost:8080
npm start

# 4. Develop without hitting production APIs
# - No rate limits
# - Consistent data
# - Works offline
# - Fast responses
```

### Backend API Development

```bash
# Capture frontend expectations
webmock capture https://frontend.example.com --name frontend-expectations

# Develop backend to match captured API calls
webmock serve frontend-expectations --port 8080
# Analyze network requests to understand what APIs the frontend expects
```

### Full-Stack Development

```bash
# Capture complete application state
webmock capture https://app.example.com/complete-flow --name full-app --timeout 180

# During development, serve the complete flow
webmock serve full-app --port 8080

# Benefits:
# - See how changes affect the complete user experience
# - Test without complex setup
# - Consistent development environment
```

## Testing Scenarios

### Unit Testing with Mocked APIs

```bash
# Capture test data scenarios
webmock capture https://api.com/users?empty=true --name empty-users
webmock capture https://api.com/users?page=1 --name users-page1
webmock capture https://api.com/error-scenario --name api-errors

# Use in tests
webmock serve empty-users --port 8081 &
webmock serve users-page1 --port 8082 &
webmock serve api-errors --port 8083 &

# Configure tests to use different ports for different scenarios
```

### Integration Testing

```bash
# Capture baseline system state
webmock capture https://system.example.com/integration-test --name baseline --timeout 300

# Run integration tests against baseline
webmock serve baseline --port 8080 &
pytest tests/integration/ --base-url=http://localhost:8080
```

### End-to-End Testing

```bash
# Capture complete user journeys
webmock capture https://app.com/user-journey-1 --name journey1 --timeout 120
webmock capture https://app.com/user-journey-2 --name journey2 --timeout 120

# Run E2E tests against captured journeys
webmock serve journey1 --port 8080 &
webmock serve journey2 --port 8081 &

# Run Selenium/Playwright tests against localhost
```

### Performance Testing

```bash
# Capture heavy application state
webmock capture https://heavy-app.com/dashboard --name perf-baseline --timeout 300

# Run performance tests against consistent baseline
webmock serve perf-baseline --port 8080 &

# Performance testing tools can now test against localhost:8080
lighthouse http://localhost:8080 --output json --output-path ./perf-report.json
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/webmock-tests.yml
name: Tests with WebMock

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup WebMock
        run: |
          # Install dependencies
          sudo apt-get update
          sudo apt-get install -y google-chrome-stable
          
          # Install WebMock CLI
          cargo install --path .
          
          # Configure environment
          echo "CHROME_ARGS=--headless --no-sandbox --disable-dev-shm-usage" >> $GITHUB_ENV
          echo "WEBMOCK_STORAGE_PATH=${{ github.workspace }}/snapshots" >> $GITHUB_ENV
      
      - name: Capture baseline
        run: |
          webmock capture https://api.example.com/baseline --name ci-baseline --timeout 120
      
      - name: Run tests
        run: |
          webmock serve ci-baseline --port 8080 &
          sleep 5
          npm test
          pkill -f "webmock serve"
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any
    
    environment {
        CHROME_ARGS = '--headless --no-sandbox --disable-dev-shm-usage'
        WEBMOCK_STORAGE_PATH = "${WORKSPACE}/snapshots"
    }
    
    stages {
        stage('Capture') {
            steps {
                sh 'webmock capture https://prod.example.com --name jenkins-baseline --timeout 180'
            }
        }
        
        stage('Test') {
            parallel {
                stage('Unit Tests') {
                    steps {
                        sh '''
                            webmock serve jenkins-baseline --port 8080 &
                            MOCK_PID=$!
                            npm test
                            kill $MOCK_PID
                        '''
                    }
                }
                stage('Integration Tests') {
                    steps {
                        sh '''
                            webmock serve jenkins-baseline --port 8081 &
                            MOCK_PID=$!
                            pytest tests/integration/
                            kill $MOCK_PID
                        '''
                    }
                }
            }
        }
    }
}
```

### Docker-based CI

```dockerfile
# Dockerfile.ci
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

# Configure for CI environment
ENV CHROME_ARGS="--headless --no-sandbox --disable-dev-shm-usage --disable-gpu"
ENV WEBMOCK_STORAGE_PATH="/snapshots"

# CI script
COPY ci-test.sh /ci-test.sh
RUN chmod +x /ci-test.sh

CMD ["/ci-test.sh"]
```

```bash
#!/bin/bash
# ci-test.sh

set -e

echo "🚀 Starting CI test with WebMock..."

# Capture baseline
echo "📸 Capturing baseline..."
webmock capture https://api.example.com/ci --name ci-baseline --timeout 120

# Start mock server
echo "🌐 Starting mock server..."
webmock serve ci-baseline --port 8080 &
MOCK_PID=$!

# Wait for server
sleep 5

# Run tests
echo "🧪 Running tests..."
npm test
pytest tests/

# Cleanup
echo "🧹 Cleaning up..."
kill $MOCK_PID

echo "✅ CI tests completed successfully!"
```

## Advanced Use Cases

### Multi-Environment Testing

```bash
# Capture different environments
webmock capture https://dev.example.com --name dev-env --timeout 60
webmock capture https://staging.example.com --name staging-env --timeout 60
webmock capture https://prod.example.com --name prod-env --timeout 60

# Test against all environments
webmock serve dev-env --port 8080 &
webmock serve staging-env --port 8081 &
webmock serve prod-env --port 8082 &

# Run cross-environment compatibility tests
npm test -- --dev-url=http://localhost:8080
npm test -- --staging-url=http://localhost:8081
npm test -- --prod-url=http://localhost:8082
```

### API Version Compatibility Testing

```bash
# Capture different API versions
webmock capture https://api.example.com/v1 --name api-v1 --timeout 30
webmock capture https://api.example.com/v2 --name api-v2 --timeout 30
webmock capture https://api.example.com/v3-beta --name api-v3-beta --timeout 30

# Test backward compatibility
for version in v1 v2 v3-beta; do
    port=$((8080 + ${version: -1}))
    webmock serve api-$version --port $port &
    npm test -- --api-version=$version --api-url=http://localhost:$port
done
```

### Microservices Testing

```bash
# Capture each microservice
webmock capture https://user-service.com/api --name user-service --timeout 45
webmock capture https://order-service.com/api --name order-service --timeout 45
webmock capture https://payment-service.com/api --name payment-service --timeout 45
webmock capture https://notification-service.com/api --name notification-service --timeout 45

# Start all services for integration testing
webmock serve user-service --port 8081 &
webmock serve order-service --port 8082 &
webmock serve payment-service --port 8083 &
webmock serve notification-service --port 8084 &

# Configure application to use mocked services
export USER_SERVICE_URL=http://localhost:8081
export ORDER_SERVICE_URL=http://localhost:8082
export PAYMENT_SERVICE_URL=http://localhost:8083
export NOTIFICATION_SERVICE_URL=http://localhost:8084

# Run integration tests
npm test -- --integration
```

### Load Testing Preparation

```bash
# Capture application under normal load
webmock capture https://app.example.com/heavy-page --name load-test-baseline --timeout 180

# Use for load testing
webmock serve load-test-baseline --port 8080 &

# Run load tests against consistent baseline
k6 run load-test.js  # Configure k6 to use localhost:8080
ab -n 1000 -c 10 http://localhost:8080/  # Apache Bench
wrk -t12 -c400 -d30s http://localhost:8080/  # wrk
```

### Security Testing

```bash
# Capture application in various security states
webmock capture https://app.com/login --name login-page --timeout 30
webmock capture https://app.com/admin --name admin-panel --timeout 60
webmock capture https://app.com/user-profile --name user-profile --timeout 45

# Run security tests against captured states
webmock serve login-page --port 8080 &
webmock serve admin-panel --port 8081 &
webmock serve user-profile --port 8082 &

# Run security scanning tools
zap-baseline.py -t http://localhost:8080
nikto -h http://localhost:8081
sqlmap -u "http://localhost:8082/profile?id=1"
```

## Troubleshooting Examples

### Debugging Capture Issues

```bash
# Enable debug logging
export RUST_LOG=debug

# Capture with debug information
webmock capture https://problematic-site.com --name debug-capture --timeout 60

# Check what was captured
webmock list
echo "Captured requests for debug-capture:"
# Debug logs will show detailed information about:
# - Browser startup
# - Network interception
# - Request/response processing
# - Storage operations
```

### Handling Slow Sites

```bash
# For very slow-loading sites
webmock capture https://slow-site.com --name slow-site --timeout 300

# For sites with lots of async content
webmock capture https://spa-heavy.com --name spa-heavy --timeout 180

# For sites that load content progressively
webmock capture https://infinite-scroll.com --name infinite-scroll --timeout 240
```

### Dealing with Authentication

```bash
# Capture authenticated sessions
# Method 1: Capture after manual login
# 1. Open Chrome manually
# 2. Log into the application
# 3. Navigate to the page you want to capture
# 4. Note the URL
# 5. Capture that specific URL

webmock capture https://app.com/authenticated-dashboard --name auth-dashboard --timeout 90

# Method 2: Capture login flow
webmock capture https://app.com/login --name login-flow --timeout 60

# Method 3: Use environment variables for test accounts
export TEST_USER_TOKEN="your-test-token"
webmock capture "https://app.com/api/data?token=$TEST_USER_TOKEN" --name api-with-auth --timeout 45
```

### Network Issues

```bash
# Test with simple HTTP first
webmock capture http://httpbin.org/get --name simple-http --timeout 30

# Test with HTTPS
webmock capture https://httpbin.org/get --name simple-https --timeout 30

# For sites with certificate issues
export CHROME_ARGS="--ignore-certificate-errors --ignore-ssl-errors"
webmock capture https://self-signed-cert-site.com --name cert-issues --timeout 60
```

### Large Application Capture

```bash
# For applications with many resources
webmock capture https://large-app.com --name large-app --timeout 600

# Check snapshot size
ls -lh ~/.webmock/snapshots/large-app.msgpack

# If too large, capture specific sections
webmock capture https://large-app.com/section1 --name app-section1 --timeout 120
webmock capture https://large-app.com/section2 --name app-section2 --timeout 120
```

### Port Conflicts

```bash
# Check what's using a port
lsof -i :8080
netstat -tulpn | grep 8080

# Use different port
webmock serve snapshot --port 8081

# Let system choose port
webmock serve snapshot --port 0  # Will display chosen port

# Kill process using port
lsof -ti:8080 | xargs kill
```

### Storage Issues

```bash
# Check storage location
echo $WEBMOCK_STORAGE_PATH
ls -la ~/.webmock/snapshots/

# Use custom storage location
export WEBMOCK_STORAGE_PATH="/tmp/webmock-test"
mkdir -p /tmp/webmock-test/snapshots

# Check disk space
df -h ~/.webmock

# Clean up old snapshots
webmock list
webmock delete old-snapshot-name
```

## Best Practices

### Naming Conventions

```bash
# Use descriptive names with context
webmock capture https://app.com --name app-homepage-v2.1
webmock capture https://api.com/users --name api-users-endpoint
webmock capture https://admin.com --name admin-dashboard-full

# Include environment information
webmock capture https://dev.app.com --name dev-app-homepage
webmock capture https://staging.app.com --name staging-app-homepage
webmock capture https://prod.app.com --name prod-app-homepage

# Include date for time-sensitive captures
webmock capture https://news.com --name news-homepage-$(date +%Y%m%d)
webmock capture https://api.com --name api-baseline-$(date +%Y%m%d-%H%M)
```

### Snapshot Management

```bash
# Regular cleanup script
#!/bin/bash
echo "🧹 Cleaning up old WebMock snapshots..."

# List all snapshots with creation dates
webmock list

# Delete snapshots older than 30 days (manual process)
echo "Please review the list above and delete old snapshots manually:"
echo "webmock delete <snapshot-name>"

# Or create a cleanup script
for snapshot in $(webmock list | grep "📸" | awk '{print $2}'); do
    echo "Found snapshot: $snapshot"
    # Add your cleanup logic here
done
```

### Environment Configuration

```bash
# Create a project-specific configuration
# .webmock-config.sh
export WEBMOCK_STORAGE_PATH="$(pwd)/webmock-snapshots"
export CHROME_PATH="/usr/bin/chromium-browser"
export CHROME_ARGS="--headless --disable-gpu --no-sandbox"
export WEBMOCK_TIMEOUT=120
export RUST_LOG=info

# Source in your project
source .webmock-config.sh

# Add to .gitignore
echo "webmock-snapshots/" >> .gitignore
echo ".webmock-config.sh" >> .gitignore  # If it contains sensitive data
```

### Testing Workflow

```bash
# Complete testing session script
#!/bin/bash
# test-session.sh

set -e

echo "🚀 Starting WebMock testing workflow..."

# Configuration
SNAPSHOT_NAME="test-baseline-$(date +%Y%m%d)"
PROD_URL="https://api.example.com"
TEST_PORT=8080

# Capture baseline
echo "📸 Capturing baseline from $PROD_URL..."
webmock capture "$PROD_URL" --name "$SNAPSHOT_NAME" --timeout 120

# Start mock server
echo "🌐 Starting mock server on port $TEST_PORT..."
webmock serve "$SNAPSHOT_NAME" --port "$TEST_PORT" &
MOCK_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 5

# Verify server is running
if curl -f http://localhost:$TEST_PORT >/dev/null 2>&1; then
    echo "✅ Mock server is running"
else
    echo "❌ Mock server failed to start"
    kill $MOCK_PID 2>/dev/null || true
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
export TEST_API_URL="http://localhost:$TEST_PORT"

# Run different test suites
npm test || TEST_RESULT=$?
pytest tests/ || TEST_RESULT=$?

# Cleanup
echo "🧹 Cleaning up..."
kill $MOCK_PID 2>/dev/null || true
wait $MOCK_PID 2>/dev/null || true

# Report results
if [ ${TEST_RESULT:-0} -eq 0 ]; then
    echo "✅ All tests passed!"
    
    # Optionally clean up successful test snapshot
    # webmock delete "$SNAPSHOT_NAME"
else
    echo "❌ Tests failed!"
    echo "Snapshot '$SNAPSHOT_NAME' preserved for debugging"
    exit $TEST_RESULT
fi
```

This comprehensive guide should help you use WebMock CLI effectively in various scenarios!