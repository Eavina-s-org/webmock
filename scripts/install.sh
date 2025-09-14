#!/bin/bash

# WebMock CLI Installation Script
# This script installs WebMock CLI and its dependencies
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Install Chrome/Chromium based on OS
install_chrome() {
    local os=$(detect_os)
    
    print_status "Installing Chrome/Chromium for $os..."
    
    case $os in
        "linux")
            if command_exists apt-get; then
                # Ubuntu/Debian
                print_status "Installing Chrome on Ubuntu/Debian..."
                wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
                echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
                sudo apt-get update
                sudo apt-get install -y google-chrome-stable
            elif command_exists yum; then
                # CentOS/RHEL/Fedora
                print_status "Installing Chrome on CentOS/RHEL/Fedora..."
                sudo yum install -y wget
                wget https://dl.google.com/linux/direct/google-chrome-stable_current_x86_64.rpm
                sudo yum localinstall -y google-chrome-stable_current_x86_64.rpm
                rm google-chrome-stable_current_x86_64.rpm
            elif command_exists pacman; then
                # Arch Linux
                print_status "Installing Chromium on Arch Linux..."
                sudo pacman -S --noconfirm chromium
            else
                print_warning "Unsupported Linux distribution. Please install Chrome or Chromium manually."
                return 1
            fi
            ;;
        "macos")
            if command_exists brew; then
                print_status "Installing Chrome via Homebrew..."
                brew install --cask google-chrome
                sudo ln -sf "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" /usr/local/bin/google-chrome
            else
                print_warning "Homebrew not found. Please install Chrome manually from https://www.google.com/chrome/"
                return 1
            fi
            ;;
        "windows")
            print_warning "Windows detected. Please install Chrome manually from https://www.google.com/chrome/"
            return 1
            ;;
        *)
            print_warning "Unknown OS. Please install Chrome or Chromium manually."
            return 1
            ;;
    esac
}

# Check Chrome installation
check_chrome() {
    print_status "Checking for Chrome/Chromium installation..."
    
    if [ -n "$CHROME_PATH" ] && [ -x "$CHROME_PATH" ]; then
        print_success "Chrome found via CHROME_PATH: $("$CHROME_PATH" --version)"
        return 0
    elif [ -n "$CHROMIUM_PATH" ] && [ -x "$CHROMIUM_PATH" ]; then
        print_success "Chromium found via CHROMIUM_PATH: $("$CHROMIUM_PATH" --version)"
        return 0
    elif command_exists google-chrome; then
        print_success "Google Chrome found: $(google-chrome --version)"
        return 0
    elif command_exists chromium; then
        print_success "Chromium found: $(chromium --version)"
        return 0
    elif command_exists chromium-browser; then
        print_success "Chromium browser found: $(chromium-browser --version)"
        return 0
    else
        print_warning "Chrome/Chromium not found in PATH"
        return 1
    fi
}

# Install Rust if not present
install_rust() {
    if command_exists cargo; then
        print_success "Rust is already installed: $(rustc --version)"
        return 0
    fi
    
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    
    if command_exists cargo; then
        print_success "Rust installed successfully: $(rustc --version)"
    else
        print_error "Failed to install Rust"
        return 1
    fi
}

# Get the directory where the script is located
get_script_dir() {
    local script_path="$0"
    # If script_path is a symlink, resolve it
    while [ -h "$script_path" ]; do
        local dir="$(cd -P "$(dirname "$script_path")" && pwd)"
        script_path="$(readlink "$script_path")"
        # If $script_path was a relative symlink, resolve it relative to the path where the symlink file was located
        [[ $script_path != /* ]] && script_path="$dir/$script_path"
    done
    cd -P "$(dirname "$script_path")" && pwd
}

# Install WebMock CLI
install_webmock() {
    print_status "Installing WebMock CLI..."
    
    # Get the project root directory (parent of scripts directory)
    local script_dir=$(get_script_dir)
    local project_root=$(dirname "$script_dir")
    
    if [ -f "$project_root/Cargo.toml" ]; then
        # Install from source
        print_status "Installing from source..."
        cargo install --path "$project_root"
    else
        # Install from crates.io (when available)
        print_status "Installing from crates.io..."
        cargo install webmock-cli
    fi
    
    if command_exists webmock; then
        print_success "WebMock CLI installed successfully: $(webmock --version)"
    else
        print_error "Failed to install WebMock CLI"
        return 1
    fi
}

# Setup storage directory
setup_storage() {
    print_status "Setting up storage directory..."
    
    local storage_dir="$HOME/.webmock"
    mkdir -p "$storage_dir/snapshots"
    
    if [ -d "$storage_dir" ]; then
        print_success "Storage directory created: $storage_dir"
    else
        print_error "Failed to create storage directory"
        return 1
    fi
}

# Run basic test
run_test() {
    print_status "Running basic functionality test..."
    
    # Test help command
    if webmock --help >/dev/null 2>&1; then
        print_success "Help command works"
    else
        print_error "Help command failed"
        return 1
    fi
    
    # Test simple capture (if network available)
    print_status "Testing capture functionality..."
    if webmock capture https://httpbin.org/get --name install-test --timeout 30 >/dev/null 2>&1; then
        print_success "Capture test successful"
        
        # Test list command
        if webmock list | grep -q "install-test"; then
            print_success "List command works"
        fi
        
        # Test serve command (background)
        print_status "Testing serve functionality..."
        webmock serve install-test --port 8080 &
        local server_pid=$!
        sleep 3
        
        if curl -s http://localhost:8080 >/dev/null 2>&1; then
            print_success "Serve test successful"
        else
            print_warning "Serve test failed (this might be normal)"
        fi
        
        # Cleanup
        kill $server_pid 2>/dev/null || true
        webmock delete install-test >/dev/null 2>&1 || true
    else
        print_warning "Capture test failed (network might be unavailable)"
    fi
}

# Print usage information
print_usage() {
    # Get the project root directory (parent of scripts directory)
    local script_dir=$(get_script_dir)
    local project_root=$(dirname "$script_dir")
    
    echo -e "
${GREEN}WebMock CLI Installation Complete!${NC}

${BLUE}Quick Start:${NC}
  1. Capture a web page:
     ${YELLOW}webmock capture https://example.com --name my-site${NC}

  2. Start mock server:
     ${YELLOW}webmock serve my-site --port 8080${NC}

  3. Visit http://localhost:8080

${BLUE}Useful Commands:${NC}
  webmock --help              Show help
  webmock list                List all snapshots
  webmock delete <name>       Delete a snapshot

${BLUE}Configuration:${NC}
  Storage location: ~/.webmock/snapshots/
  
  Environment variables:
    WEBMOCK_STORAGE_PATH      Custom storage location
    CHROME_PATH               Custom Chrome path
    RUST_LOG=debug            Enable debug logging

${BLUE}Documentation:${NC}
  $project_root/README.md                   Full documentation
  $project_root/docs/TROUBLESHOOTING.md          Common issues and solutions
  $project_root/docs/examples/README.md          Usage examples

${BLUE}Need Help?${NC}
  GitHub: https://github.com/your-org/webmock-cli
  Issues: https://github.com/your-org/webmock-cli/issues
"
}

# Main installation function
main() {
    echo -e "${BLUE}"
    cat << "EOF"
 _    _      _     __  __            _      _____ _      _____ 
| |  | |    | |   |  \/  |          | |    / ____| |    |_   _|
| |  | | ___| |__ | \  / | ___   ___| | __| |    | |      | |  
| |/\| |/ _ \ '_ \| |\/| |/ _ \ / __| |/ /| |    | |      | |  
\  /\  /  __/ |_) | |  | | (_) | (__|   < | |____| |____ _| |_ 
 \/  \/ \___|_.__/|_|  |_|\___/ \___|_|\_\ \_____|______|_____|

EOF
    echo -e "${NC}"
    
    print_status "Starting WebMock CLI installation..."
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    # Install Rust if needed
    if ! install_rust; then
        print_error "Failed to install Rust"
        exit 1
    fi
    
    # Check/install Chrome
    if ! check_chrome; then
        print_warning "Chrome/Chromium not found. Attempting to install..."
        if ! install_chrome; then
            print_error "Failed to install Chrome. Please install manually and re-run this script."
            exit 1
        fi
        
        # Check again after installation
        if ! check_chrome; then
            print_error "Chrome installation verification failed"
            exit 1
        fi
    fi
    
    # Install WebMock CLI
    if ! install_webmock; then
        print_error "Failed to install WebMock CLI"
        exit 1
    fi
    
    # Setup storage
    if ! setup_storage; then
        print_error "Failed to setup storage directory"
        exit 1
    fi
    
    # Run tests
    print_status "Running installation tests..."
    run_test
    
    # Print usage information
    print_usage
    
    print_success "Installation completed successfully!"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        cat << EOF
WebMock CLI Installation Script

Usage: $0 [OPTIONS]

Options:
  --help, -h          Show this help message
  --no-test          Skip functionality tests
  --chrome-only      Only install Chrome/Chromium
  --webmock-only     Only install WebMock CLI (skip Chrome)

Examples:
  $0                 Full installation
  $0 --no-test       Install without running tests
  $0 --chrome-only   Only install Chrome
EOF
        exit 0
        ;;
    --no-test)
        SKIP_TESTS=1
        ;;
    --chrome-only)
        install_chrome
        exit $?
        ;;
    --webmock-only)
        install_rust && install_webmock && setup_storage
        exit $?
        ;;
esac

# Run main installation
main