#!/bin/bash

# RISC-V Build Script for IoT Projects
# This script builds IoT projects specifically for RISC-V architectures

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to print colored output
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Default values
PROJECT_NAME=""
TARGET="riscv64gc-unknown-none-elf"
DEBUG=false
OPTIMIZATION_LEVEL="s" # Size optimization by default
FEATURES=""
VERBOSE=false

# Help function
show_help() {
    cat << EOF
RISC-V IoT Project Build Script

Usage: $0 [OPTIONS] <project-name>

Options:
    -t, --target TARGET         RISC-V target triple (default: riscv64gc-unknown-none-elf)
    -O, --opt-level LEVEL       Optimization level: 0, 1, 2, 3, s, z (default: s)
    -d, --debug                 Build in debug mode
    -f, --features FEATURES     Enable Rust features (comma-separated)
    -v, --verbose               Verbose output
    -h, --help                  Show this help message

Examples:
    $0 smart_sensor_network
    $0 --debug industrial_iot_monitoring
    $0 --features "wifi,ble" --optimize z home_automation
    $0 --target riscv64imac-unknown-none-elf environmental_monitoring

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -O|--opt-level)
            OPTIMIZATION_LEVEL="$2"
            shift 2
            ;;
        -d|--debug)
            DEBUG=true
            shift
            ;;
        -f|--features)
            FEATURES="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            PROJECT_NAME="$1"
            shift
            ;;
    esac
done

# Validate project name
if [[ -z "$PROJECT_NAME" ]]; then
    print_error "Project name is required"
    show_help
    exit 1
fi

# Check if project directory exists
PROJECT_DIR="../../../$PROJECT_NAME"
if [[ ! -d "$PROJECT_DIR" ]]; then
    # Try relative path from current script location
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_DIR="$SCRIPT_DIR/../$PROJECT_NAME"
    if [[ ! -d "$PROJECT_DIR" ]]; then
        print_error "Project '$PROJECT_NAME' not found"
        print_info "Available projects:"
        find . -maxdepth 1 -type d -name "*_*" | grep -E "^\./[0-9]_" | while read dir; do
            basename "$dir"
        done
        exit 1
    fi
fi

print_info "Building RISC-V IoT Project: $PROJECT_NAME"
print_info "Target: $TARGET"
print_info "Optimization Level: $OPTIMIZATION_LEVEL"
if [[ "$DEBUG" == "true" ]]; then
    print_info "Build Mode: Debug"
else
    print_info "Build Mode: Release"
fi

# Create build output directory
BUILD_DIR="$PROJECT_DIR/target/riscv_build"
mkdir -p "$BUILD_DIR"

# Create Rust configuration for RISC-V
RUST_CONFIG="$BUILD_DIR/rust_config.toml"
cat > "$RUST_CONFIG" << EOF
[target.riscv64gc-unknown-none-elf]
runner = "qemu-system-riscv64 -machine virt -nographic"
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "link-arg=--gc-sections",
]

[build]
target = "$TARGET"

[profile.release]
opt-level = "$OPTIMIZATION_LEVEL"
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
EOF

# Add features configuration if provided
if [[ -n "$FEATURES" ]]; then
    echo "" >> "$RUST_CONFIG"
    echo "# Features" >> "$RUST_CONFIG"
    IFS=',' read -ra FEATURE_ARRAY <<< "$FEATURES"
    for feature in "${FEATURE_ARRAY[@]}"; do
        echo "default-features = false" >> "$RUST_CONFIG"
        break
    done
fi

# Create linker script for RISC-V
LINKER_SCRIPT="$BUILD_DIR/link.x"
cat > "$LINKER_SCRIPT" << 'EOF'
/* RISC-V Memory Layout for IoT Applications */
OUTPUT_ARCH("riscv")
ENTRY(_start)

MEMORY
{
  /* On-chip instruction memory (128KB) */
  itcm (wx) : ORIGIN = 0x80000000, LENGTH = 0x00020000
  
  /* On-chip data memory (64KB) */
  dtcm (rw) : ORIGIN = 0x80020000, LENGTH = 0x00010000
  
  /* External RAM (16MB) */
  ram (rw)  : ORIGIN = 0x80200000, LENGTH = 0x01000000
}

SECTIONS
{
  .text :
  {
    *(.text .text.*)
    *(.rodata .rodata.*)
    _etext = .;
  } > itcm

  .data :
  {
    *(.data .data.*)
    *(.sdata .sdata.*)
    _edata = .;
  } > dtcm

  .bss :
  {
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    _ebss = .;
  } > dtcm

  .heap :
  {
    _sheap = .;
    . = . + 0x4000;  /* 16KB heap */
    _eheap = .;
  } > ram

  .stack :
  {
    _estack = ORIGIN(dtcm) + LENGTH(dtcm);
    . = . + 0x1000;  /* 4KB stack */
  } > dtcm
}
EOF

# Prepare cargo project if it doesn't exist
if [[ ! -f "$PROJECT_DIR/Cargo.toml" ]]; then
    print_info "Creating Rust project structure..."
    
    # Create Cargo.toml template
    CARGO_TOML="$PROJECT_DIR/Cargo.toml"
    cat > "$CARGO_TOML" << EOF
[package]
name = "$PROJECT_NAME"
version = "0.1.0"
edition = "2021"
authors = ["IoT Development Team"]
description = "IoT demonstration project for RISC-V"
license = "MIT"

[dependencies]
# Core dependencies
embedded-hal = "1.0"
nb = "1.0"
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

# Communication
embassy-nrf = { version = "0.1", features = ["defmt"] }
embassy-ble = "0.1"
embassy-net = { version = "0.4", features = ["tcp", "dns"] }

# Sensors
dht-sensor = "0.2"
mma8451 = "0.1"
lsm303 = "0.1"
bme280 = "0.2"

# Logging
defmt = "0.3"
defmt-rtt = "0.4"

# Utilities
heapless = "0.7"
heapless-utils = "0.7"

[features]
default = []
debug = ["defmt"]
wifi = ["embassy-net"]
ble = ["embassy-ble"]
sensors = ["dht-sensor", "mma8451", "lsm303", "bme280"]

[profile.release]
opt-level = "$OPTIMIZATION_LEVEL"
lto = true
codegen-units = 1
panic = "abort"
EOF

    # Create basic source structure
    mkdir -p "$PROJECT_DIR/src/bin"
    mkdir -p "$PROJECT_DIR/src/drivers"
    mkdir -p "$PROJECT_DIR/src/sensors"
    mkdir -p "$PROJECT_DIR/src/communication"
    mkdir -p "$PROJECT_DIR/src/utils"
fi

# Build the project
print_info "Compiling for RISC-V target..."

cd "$PROJECT_DIR"

# Set build environment
export CARGO_TARGET_RISCV64GC_UNKNOWN_NONE_ELF_RUSTFLAGS="-C link-arg=-T$BUILD_DIR/link.x -C link-arg=--gc-sections"
export RUSTFLAGS="-C link-arg=-T$BUILD_DIR/link.x -C link-arg=--gc-sections"

# Build command
if [[ "$DEBUG" == "true" ]]; then
    BUILD_CMD="cargo build"
else
    BUILD_CMD="cargo build --release"
fi

# Add features if specified
if [[ -n "$FEATURES" ]]; then
    BUILD_CMD="$BUILD_CMD --features $FEATURES"
fi

# Add verbose flag if requested
if [[ "$VERBOSE" == "true" ]]; then
    BUILD_CMD="$BUILD_CMD --verbose"
fi

print_info "Running: $BUILD_CMD"
eval "$BUILD_CMD"

if [[ $? -eq 0 ]]; then
    print_success "✅ Build completed successfully!"
    
    # Show build output information
    if [[ "$DEBUG" == "true" ]]; then
        BINARY_PATH="$PROJECT_DIR/target/debug/$PROJECT_NAME"
    else
        BINARY_PATH="$PROJECT_DIR/target/release/$PROJECT_NAME"
    fi
    
    if [[ -f "$BINARY_PATH" ]]; then
        print_info "Binary location: $BINARY_PATH"
        print_info "Binary size: $(du -h "$BINARY_PATH" | cut -f1)"
    fi
    
    # Copy binary to build directory
    if [[ -f "$BINARY_PATH" ]]; then
        cp "$BINARY_PATH" "$BUILD_DIR/"
        print_success "Binary copied to: $BUILD_DIR/"
    fi
    
    # Generate build report
    BUILD_REPORT="$BUILD_DIR/build_report.md"
    cat > "$BUILD_REPORT" << EOF
# Build Report: $PROJECT_NAME

## Build Configuration
- Target: $TARGET
- Optimization Level: $OPTIMIZATION_LEVEL
- Debug Mode: $DEBUG
- Features: ${FEATURES:-none}

## Build Output
- Binary: $BINARY_PATH
- Size: $(du -h "$BINARY_PATH" | cut -f1 2>/dev/null || echo "N/A")
- Timestamp: $(date)

## Next Steps
1. Test in QEMU: \`qemu-system-riscv64 -machine virt -nographic -kernel $BINARY_PATH\`
2. Flash to hardware (if supported)
3. Deploy to target device

EOF
    print_success "Build report generated: $BUILD_REPORT"
    
else
    print_error "❌ Build failed!"
    exit 1
fi

print_success "🎉 RISC-V build process completed!"
