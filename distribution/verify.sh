#!/bin/bash
# MultiOS Distribution Verification Script
# Version: 1.0.0
# Description: Verify the integrity of MultiOS distribution packages

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}" >&2
}

warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

# Verification counters
TOTAL_FILES=0
VERIFIED_FILES=0
FAILED_FILES=0

# Check if checksums file exists
check_checksums_file() {
    if [ ! -f "checksums.sha256" ]; then
        error "checksums.sha256 file not found"
        error "Cannot verify distribution integrity"
        exit 1
    fi
    
    info "Found checksums file: checksums.sha256"
}

# Verify checksums
verify_checksums() {
    log "Verifying file checksums..."
    
    local failed=0
    local total=0
    
    while IFS= read -r line; do
        # Parse checksum line
        if [[ $line =~ ^([a-f0-9]+)\ \.\/(.+)$ ]]; then
            local checksum="${BASH_REMATCH[1]}"
            local file="${BASH_REMATCH[2]}"
            
            # Skip if file doesn't exist
            if [ ! -f "$file" ]; then
                error "Missing file: $file"
                failed=$((failed + 1))
                continue
            fi
            
            # Verify checksum
            local actual_checksum=$(sha256sum "$file" | awk '{print $1}')
            
            if [ "$checksum" == "$actual_checksum" ]; then
                info "✓ $file"
                VERIFIED_FILES=$((VERIFIED_FILES + 1))
            else
                error "✗ $file (checksum mismatch)"
                error "  Expected: $checksum"
                error "  Actual:   $actual_checksum"
                failed=$((failed + 1))
            fi
            
            total=$((total + 1))
            TOTAL_FILES=$((TOTAL_FILES + 1))
        fi
    done < checksums.sha256
    
    if [ $failed -eq 0 ]; then
        log "All $total files verified successfully"
    else
        error "$failed files failed verification out of $total"
        return 1
    fi
}

# Check file structure
check_file_structure() {
    log "Checking distribution file structure..."
    
    local required_dirs=(
        "kernel"
        "bootloader"
        "libraries"
        "scripts"
        "documentation"
        "tools"
        "installation"
        "examples"
        "testing"
        "config"
        "resources"
    )
    
    local required_files=(
        "install.sh"
        "checksums.sha256"
        "README.md"
        "LICENSE"
    )
    
    local missing_dirs=0
    local missing_files=0
    
    # Check directories
    for dir in "${required_dirs[@]}"; do
        if [ -d "$dir" ]; then
            info "✓ Directory: $dir"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        else
            error "✗ Missing directory: $dir"
            missing_dirs=$((missing_dirs + 1))
            FAILED_FILES=$((FAILED_FILES + 1))
        fi
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
    
    # Check files
    for file in "${required_files[@]}"; do
        if [ -f "$file" ]; then
            info "✓ File: $file"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        else
            error "✗ Missing file: $file"
            missing_files=$((missing_files + 1))
            FAILED_FILES=$((FAILED_FILES + 1))
        fi
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
    
    # Check installation scripts
    local install_scripts=(
        "installation/desktop/install_multios_desktop.sh"
        "installation/server/install_multios_server.sh"
        "installation/embedded/install_multios_embedded.sh"
        "installation/development/install_multios_dev.sh"
    )
    
    for script in "${install_scripts[@]}"; do
        if [ -f "$script" ] && [ -x "$script" ]; then
            info "✓ Installation script: $script"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        elif [ -f "$script" ]; then
            warning "⚠ Installation script not executable: $script"
            warning "  Run: chmod +x $script"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        else
            error "✗ Missing installation script: $script"
            FAILED_FILES=$((FAILED_FILES + 1))
        fi
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
    
    if [ $missing_dirs -eq 0 ] && [ $missing_files -eq 0 ]; then
        log "File structure verification completed successfully"
    else
        error "File structure verification failed"
        return 1
    fi
}

# Check core components
check_core_components() {
    log "Checking core components..."
    
    # Check kernel directory
    if [ -d "kernel/src" ]; then
        info "✓ Kernel source directory found"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
        
        # Check for essential kernel files
        local kernel_files=(
            "kernel/src/lib.rs"
            "kernel/Cargo.toml"
            "kernel/src/arch"
            "kernel/src/hal"
            "kernel/src/drivers"
        )
        
        for file in "${kernel_files[@]}"; do
            if [ -e "$file" ]; then
                info "  ✓ $file"
                VERIFIED_FILES=$((VERIFIED_FILES + 1))
            else
                warning "  ⚠ Missing: $file"
            fi
            TOTAL_FILES=$((TOTAL_FILES + 1))
        done
    else
        error "✗ Kernel source directory not found"
        FAILED_FILES=$((FAILED_FILES + 1))
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    # Check bootloader directory
    if [ -d "bootloader/src" ]; then
        info "✓ Bootloader directory found"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
    else
        warning "⚠ Bootloader directory not found"
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    # Check libraries directory
    if [ -d "libraries" ]; then
        info "✓ Libraries directory found"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
        
        # Count library crates
        local crate_count=$(find libraries -name "Cargo.toml" | wc -l)
        info "  Found $crate_count library crates"
    else
        warning "⚠ Libraries directory not found"
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    # Check documentation
    if [ -d "documentation" ] && [ "$(ls -A documentation 2>/dev/null)" ]; then
        info "✓ Documentation directory found and not empty"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
    else
        warning "⚠ Documentation directory missing or empty"
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
}

# Check scripts
check_scripts() {
    log "Checking distribution scripts..."
    
    local script_count=$(find scripts -name "*.sh" 2>/dev/null | wc -l)
    info "Found $script_count shell scripts"
    
    # Make scripts executable
    find scripts -name "*.sh" -type f -exec chmod +x {} \; 2>/dev/null || true
    
    # Check for essential scripts
    local required_scripts=(
        "scripts/build_*.sh"
        "scripts/test_*.sh"
        "scripts/setup_*.sh"
    )
    
    for pattern in "${required_scripts[@]}"; do
        local matches=$(find scripts -name "$pattern" -type f 2>/dev/null | wc -l)
        if [ $matches -gt 0 ]; then
            info "✓ Found scripts matching: $pattern"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        else
            warning "⚠ No scripts found matching: $pattern"
        fi
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
}

# Check configuration files
check_configuration() {
    log "Checking configuration files..."
    
    # Check for configuration templates
    local config_files=(
        "config/*.conf"
        "config/*.toml"
        "config/*.yaml"
        "config/*.json"
    )
    
    for pattern in "${config_files[@]}"; do
        local matches=$(find . -path "./$pattern" -type f 2>/dev/null | wc -l)
        if [ $matches -gt 0 ]; then
            info "✓ Found configuration files: $pattern"
            VERIFIED_FILES=$((VERIFIED_FILES + 1))
        fi
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
}

# Check examples and testing
check_examples_testing() {
    log "Checking examples and testing materials..."
    
    # Check examples
    if [ -d "examples" ] && [ "$(ls -A examples 2>/dev/null)" ]; then
        info "✓ Examples directory found and not empty"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
        
        local example_count=$(find examples -type f | wc -l)
        info "  Found $example_count example files"
    else
        warning "⚠ Examples directory missing or empty"
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    # Check testing
    if [ -d "testing" ] && [ "$(ls -A testing 2>/dev/null)" ]; then
        info "✓ Testing directory found and not empty"
        VERIFIED_FILES=$((VERIFIED_FILES + 1))
    else
        warning "⚠ Testing directory missing or empty"
    fi
    TOTAL_FILES=$((TOTAL_FILES + 1))
}

# Generate verification report
generate_report() {
    local report_file="/tmp/multios_verification_report_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "MultiOS Distribution Verification Report"
        echo "========================================="
        echo
        echo "Date: $(date)"
        echo "Directory: $(pwd)"
        echo "User: $(whoami)"
        echo
        echo "Verification Summary:"
        echo "  Total Files Checked: $TOTAL_FILES"
        echo "  Files Verified: $VERIFIED_FILES"
        echo "  Files Failed: $FAILED_FILES"
        echo
        if [ $FAILED_FILES -eq 0 ]; then
            echo "Status: PASSED"
            echo "The distribution package is complete and verified."
        else
            echo "Status: FAILED"
            echo "Some files failed verification. Please check the errors above."
        fi
        echo
        echo "System Information:"
        echo "  OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2 2>/dev/null || echo "Unknown")"
        echo "  Architecture: $(uname -m)"
        echo "  Kernel: $(uname -r)"
        echo "  Available Space: $(df -h . | tail -1 | awk '{print $4}')"
        echo
        echo "Files that failed verification:"
        if [ $FAILED_FILES -gt 0 ]; then
            # Re-run verification to capture failed files
            grep "✗" /tmp/multios_verify_errors.log 2>/dev/null || echo "  No detailed error log found"
        else
            echo "  None"
        fi
        
    } > "$report_file"
    
    log "Verification report saved to: $report_file"
    
    # Also display summary
    echo
    log "Verification Summary:"
    echo "  Total Files Checked: $TOTAL_FILES"
    echo "  Files Verified: $VERIFIED_FILES"
    echo "  Files Failed: $FAILED_FILES"
    echo
    
    if [ $FAILED_FILES -eq 0 ]; then
        echo -e "${GREEN}✓ Distribution verification PASSED${NC}"
        echo "The MultiOS distribution package is complete and all files are verified."
    else
        echo -e "${RED}✗ Distribution verification FAILED${NC}"
        echo "$FAILED_FILES files failed verification. Please check the errors above."
        return 1
    fi
}

# Main verification function
main() {
    echo "========================================"
    echo "  MultiOS Distribution Verification"
    echo "  Version: 1.0.0"
    echo "========================================"
    echo
    
    # Redirect errors to temp file for reporting
    exec 2> >(tee /tmp/multios_verify_errors.log >&2)
    
    check_checksums_file
    verify_checksums
    check_file_structure
    check_core_components
    check_scripts
    check_configuration
    check_examples_testing
    generate_report
}

# Check for help flag
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "MultiOS Distribution Verification Script"
    echo
    echo "Usage: $0"
    echo
    echo "This script verifies the integrity of a MultiOS distribution"
    echo "package by checking:"
    echo "  - File checksums"
    echo "  - Directory structure"
    echo "  - Core components"
    echo "  - Installation scripts"
    echo "  - Configuration files"
    echo
    echo "Run this script from the root of the distribution directory."
    exit 0
fi

# Run main function
main "$@"