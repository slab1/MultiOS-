#!/bin/bash

# MultiOS Installation Wizard Build Script
# This script builds and tests the installation wizard

set -e

echo "Building MultiOS Installation Wizard..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Using Rust version: $RUST_VERSION"

# Create build directory
mkdir -p target/debug
mkdir -p target/release

echo "Building debug version..."
cargo build

echo "Building release version..."
cargo build --release

echo "Running tests..."
cargo test

echo "Running basic installation example..."
cargo run --example basic_installation --quiet

echo "Running hardware detection example..."
cargo run --example hardware_detection --quiet

echo "Installing CLI tools (if available)..."
if cargo install --list | grep -q "multios-installer"; then
    echo "CLI tools already installed"
else
    echo "Installing CLI tools..."
    # Note: This would require adding a binary target to Cargo.toml
    # cargo install --path .
fi

echo "Build completed successfully!"

echo ""
echo "Available commands:"
echo "  cargo build              - Build debug version"
echo "  cargo build --release    - Build release version"
echo "  cargo test               - Run all tests"
echo "  cargo run                - Run installer"
echo "  cargo run -- --no-gui    - Run text mode installer"
echo "  cargo run --example basic_installation     - Run basic example"
echo "  cargo run --example hardware_detection     - Run hardware detection"
echo ""
echo "To enable GUI features, build with:"
echo "  cargo build --features gui"