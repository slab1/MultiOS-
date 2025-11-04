#!/bin/bash

# MultiOS Build Script
# Handles building MultiOS for different target architectures

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
DIST_DIR="$PROJECT_ROOT/dist"
LOG_FILE="$PROJECT_ROOT/build.log"

# Target configurations
declare -A TARGETS=(
    ["x86_64"]="x86_64-unknown-none"
    ["arm64"]="aarch64-unknown-none"
    ["riscv64"]="riscv64gc-unknown-none"
)

# Default values
TARGET=""
RELEASE=false
CLEAN=false
PARALLEL=true
VERBOSE=false
HELP=false

# Logging function
log() {
    echo -e "${GREEN}[BUILD]${NC} $1" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
    exit 1
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
MultiOS Build Script

Usage: $0 [OPTIONS]

Options:
    -t, --target TARGET     Target architecture (x86_64, arm64, riscv64)
    -r, --release          Build in release mode
    -c, --clean            Clean build artifacts before building
    -j, --parallel N       Enable parallel builds with N jobs
    -v, --verbose          Enable verbose output
    -h, --help             Show this help message

Examples:
    $0 --target x86_64 --release
    $0 --target arm64 --clean
    $0 --target riscv64 --verbose --parallel 4

Targets:
    x86_64   - Intel/AMD 64-bit processors
    arm64    - ARM 64-bit processors (AArch64)
    riscv64  - RISC-V 64-bit processors

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--target)
                TARGET="$2"
                shift 2
                ;;
            -r|--release)
                RELEASE=true
                shift
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -j|--parallel)
                PARALLEL="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                HELP=true
                shift
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done
}

# Validate target
validate_target() {
    if [[ -z "$TARGET" ]]; then
        error "Target architecture must be specified. Use --help for usage information."
    fi
    
    if [[ ! "${TARGETS[$TARGET]+_}" ]]; then
        error "Invalid target: $TARGET. Valid targets: ${!TARGETS[*]}"
    fi
}

# Setup environment
setup_environment() {
    log "Setting up build environment..."
    
    # Create directories
    mkdir -p "$BUILD_DIR" "$DIST_DIR"
    
    # Initialize log file
    echo "MultiOS Build Log - $(date)" > "$LOG_FILE"
    
    # Export target triple
    export CARGO_BUILD_TARGET="${TARGETS[$TARGET]}"
    
    # Setup cross-compilation tools if needed
    if [[ "$TARGET" != "x86_64" ]]; then
        log "Setting up cross-compilation for $TARGET..."
        setup_cross_compilation "$TARGET"
    fi
}

# Setup cross-compilation tools
setup_cross_compilation() {
    local target=$1
    
    case $target in
        "arm64")
            if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
                warn "ARM64 GCC not found. Install with: sudo apt install gcc-aarch64-linux-gnu"
            fi
            ;;
        "riscv64")
            if ! command -v riscv64-linux-gnu-gcc &> /dev/null; then
                warn "RISC-V GCC not found. Install with: sudo apt install gcc-riscv64-linux-gnu"
            fi
            ;;
    esac
}

# Clean build artifacts
clean_build() {
    if [[ "$CLEAN" == "true" ]]; then
        log "Cleaning build artifacts..."
        cargo clean
        rm -rf "$BUILD_DIR"/* "$DIST_DIR"/*
    fi
}

# Build the project
build_project() {
    log "Building MultiOS for $TARGET..."
    
    local cargo_args=()
    
    if [[ "$RELEASE" == "true" ]]; then
        cargo_args+=("--release")
    fi
    
    if [[ "$PARALLEL" != "false" ]]; then
        if [[ "$PARALLEL" == "true" ]]; then
            cargo_args+=("--jobs=$(nproc)")
        else
            cargo_args+=("--jobs=$PARALLEL")
        fi
    fi
    
    if [[ "$VERBOSE" == "true" ]]; then
        cargo_args+=("--verbose")
    fi
    
    # Build workspace
    log "Building workspace crates..."
    cargo build "${cargo_args[@]}" || error "Build failed"
    
    # Build kernel specifically
    log "Building kernel..."
    cd "$PROJECT_ROOT/kernel"
    cargo build "${cargo_args[@]}" || error "Kernel build failed"
    
    # Build bootloader
    log "Building bootloader..."
    cd "$PROJECT_ROOT/bootloader"
    cargo build "${cargo_args[@]}" || error "Bootloader build failed"
    
    # Build userland
    log "Building userland..."
    cd "$PROJECT_ROOT/userland"
    cargo build "${cargo_args[@]}" || error "Userland build failed"
}

# Package build artifacts
package_artifacts() {
    log "Packaging build artifacts for $TARGET..."
    
    local target_triple="${TARGETS[$TARGET]}"
    local build_type="debug"
    
    if [[ "$RELEASE" == "true" ]]; then
        build_type="release"
    fi
    
    # Create target-specific directory
    local target_dir="$DIST_DIR/$TARGET"
    mkdir -p "$target_dir"
    
    # Copy kernel
    if [[ -f "$BUILD_DIR/$target_triple/$build_type/deps/libmultios_kernel.rlib" ]]; then
        cp "$BUILD_DIR/$target_triple/$build_type/deps/libmultios_kernel.rlib" "$target_dir/"
        log "Copied kernel library"
    fi
    
    # Copy bootloader
    if [[ -f "$BUILD_DIR/$target_triple/$build_type/multios_bootloader" ]]; then
        cp "$BUILD_DIR/$target_triple/$build_type/multios_bootloader" "$target_dir/"
        log "Copied bootloader binary"
    fi
    
    # Copy userland binaries
    if [[ -d "$BUILD_DIR/$target_triple/$build_type/deps" ]]; then
        cp -r "$BUILD_DIR/$target_triple/$build_type/deps/"* "$target_dir/" 2>/dev/null || true
        log "Copied userland binaries"
    fi
    
    # Create metadata file
    cat > "$target_dir/build_info.txt" << EOF
MultiOS Build Information
========================
Target: $TARGET
Target Triple: $target_triple
Build Type: $build_type
Build Date: $(date)
Rust Version: $(rustc --version)
Cargo Version: $(cargo --version)
EOF
    
    log "Artifacts packaged in $target_dir"
}

# Generate build report
generate_report() {
    local report_file="$DIST_DIR/build_report.txt"
    
    cat > "$report_file" << EOF
MultiOS Build Report
===================
Build Date: $(date)
Target: $TARGET
Release Build: $RELEASE
Clean Build: $CLEAN
Parallel Jobs: $PARALLEL
Verbose: $VERBOSE

Target Triple: ${TARGETS[$TARGET]}
Output Directory: $DIST_DIR/$TARGET

Build Log: $LOG_FILE

EOF
    
    log "Build report generated: $report_file"
}

# Main build function
main() {
    # Setup log
    exec > >(tee -a "$LOG_FILE")
    exec 2>&1
    
    info "MultiOS Build Script Starting..."
    info "Target: ${TARGET:-not specified}"
    info "Release: $RELEASE"
    
    # Setup environment
    setup_environment
    
    # Clean if requested
    clean_build
    
    # Build project
    build_project
    
    # Package artifacts
    package_artifacts
    
    # Generate report
    generate_report
    
    log "Build completed successfully!"
    info "Artifacts available in: $DIST_DIR/$TARGET"
    info "Build log: $LOG_FILE"
}

# Run main function
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    parse_args "$@"
    
    if [[ "$HELP" == "true" ]]; then
        show_help
        exit 0
    fi
    
    validate_target
    main
fi