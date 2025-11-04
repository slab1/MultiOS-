#!/bin/bash

# Build script for Boot Process Optimization System

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Boot Process Optimization System..."
echo

# Make scripts executable
echo "Making scripts executable..."
chmod +x scripts/*.sh
echo "✓ Scripts made executable"

# Build Rust components
if command -v cargo &> /dev/null; then
    echo "Building Rust components..."
    cargo build --release
    echo "✓ Rust components built successfully"
else
    echo "⚠️  Cargo not found - Rust components not built"
    echo "Install Rust: https://rustup.rs/"
fi

# Create required directories
echo "Creating required directories..."
mkdir -p /var/log/boot_optimization
mkdir -p /var/lib/boot_monitor
mkdir -p /tmp/boot_reports
mkdir -p /etc/boot_optimizer_backup
echo "✓ Directories created"

# Check system dependencies
echo "Checking system dependencies..."
MISSING_DEPS=()

if ! command -v systemd-analyze &> /dev/null; then
    MISSING_DEPS+=("systemd")
fi

if ! command -v bc &> /dev/null; then
    MISSING_DEPS+=("bc")
fi

if ! command -v jq &> /dev/null; then
    MISSING_DEPS+=("jq")
fi

if [[ ${#MISSING_DEPS[@]} -gt 0 ]]; then
    echo "⚠️  Missing dependencies: ${MISSING_DEPS[*]}"
    echo "Install with:"
    echo "  Ubuntu/Debian: sudo apt-get install ${MISSING_DEPS[*]}"
    echo "  Fedora/RHEL:   sudo dnf install ${MISSING_DEPS[*]}"
    echo "  Arch Linux:    sudo pacman -S ${MISSING_DEPS[*]}"
else
    echo "✓ All dependencies available"
fi

# Validate installation
echo
echo "=== Installation Validation ==="

# Test boot analyzer
echo "Testing boot analyzer..."
if ./scripts/boot_analyzer.sh --help &> /dev/null; then
    echo "✓ Boot analyzer working"
else
    echo "✗ Boot analyzer test failed"
fi

# Test boot optimizer
echo "Testing boot optimizer..."
if ./scripts/boot_optimizer.sh --help &> /dev/null; then
    echo "✓ Boot optimizer working"
else
    echo "✗ Boot optimizer test failed"
fi

# Test boot monitor
echo "Testing boot monitor..."
if ./scripts/boot_monitor.sh --help &> /dev/null; then
    echo "✓ Boot monitor working"
else
    echo "✗ Boot monitor test failed"
fi

# Check file permissions
echo "Checking file permissions..."
if [[ -r "README.md" ]]; then
    echo "✓ README.md readable"
else
    echo "✗ README.md not readable"
fi

if [[ -x "scripts/boot_analyzer.sh" ]]; then
    echo "✓ Scripts executable"
else
    echo "✗ Scripts not executable"
fi

echo
echo "=== Build Complete ==="
echo
echo "Next steps:"
echo "1. Review the README.md file"
echo "2. Run: sudo ./scripts/boot_analyzer.sh --quick"
echo "3. Apply optimizations: sudo ./scripts/boot_optimizer.sh --dry-run"
echo "4. Start monitoring: sudo ./scripts/boot_monitor.sh &"
echo
echo "For help: ./scripts/boot_analyzer.sh --help"
echo
