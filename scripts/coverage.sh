#!/bin/bash

# WebMock CLI Coverage Report Generator
set -e

echo "🔍 Generating test coverage report for WebMock CLI..."

# Create coverage directory
mkdir -p target/coverage

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "📦 Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Generate coverage report
echo "📊 Running coverage analysis..."
cargo llvm-cov --no-cfg-coverage --lib --no-report
cargo llvm-cov --no-cfg-coverage --tests --lib --no-report -- --ignored
cargo llvm-cov report --html --output-dir target/coverage

# Display summary
echo ""
echo "✅ Coverage report generated!"
echo "📁 Reports saved to: target/coverage/"
echo "🌐 HTML Report: target/coverage/html/index.html"

# Open HTML report if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🚀 Opening HTML report..."
    open target/coverage/html/index.html
fi
