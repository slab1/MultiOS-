#!/bin/bash

# Cross-Compilation Validation Script
# Validates that binaries can be built for all target architectures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Supported architectures
ARCHITECTURES=("x86_64" "arm64" "riscv64")
TARGETS=(
    "x86_64-unknown-none"
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-none"
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
    "riscv64gc-unknown-none-elf"
    "riscv64gc-unknown-linux-gnu"
    "riscv64gc-unknown-linux-musl"
)

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Validate toolchain installation
validate_toolchain() {
    local target=$1
    
    log "Validating toolchain for target: $target"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "^${target}$"; then
        log "Installing Rust target: $target"
        rustup target add "$target"
    else
        success "Target $target is already installed"
    fi
}

# Validate cross compilation setup
validate_cross_setup() {
    local target=$1
    
    case "$target" in
        "x86_64"*)
            # Native compilation - no cross setup needed
            log "Native compilation for x86_64"
            ;;
        "aarch64"*)
            # ARM64 cross compilation
            log "Setting up ARM64 cross compilation"
            
            # Check if cross is installed
            if ! command -v cross >/dev/null 2>&1; then
                log "Installing cross..."
                cargo install cross --git https://github.com/cross-rs/cross
            fi
            
            # Set up QEMU for testing ARM64 binaries
            if ! command -v qemu-aarch64-static >/dev/null 2>&1; then
                warning "QEMU for ARM64 not found. Install qemu-system-aarch64"
            fi
            ;;
        "riscv64"*)
            # RISC-V cross compilation
            log "Setting up RISC-V cross compilation"
            
            # Check if cross is installed
            if ! command -v cross >/dev/null 2>&1; then
                log "Installing cross..."
                cargo install cross --git https://github.com/cross-rs/cross
            fi
            
            # Set up QEMU for testing RISC-V binaries
            if ! command -v qemu-riscv64-static >/dev/null 2>&1; then
                warning "QEMU for RISC-V not found. Install qemu-system-riscv64"
            fi
            ;;
    esac
}

# Test basic compilation
test_basic_compilation() {
    local target=$1
    local log_file="/workspace/logs/compile_${target//-/_}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Testing basic compilation for target: $target"
    
    # Create a minimal test program
    cat > /tmp/test_main.rs << 'EOF'
#[no_mangle]
pub extern "C" fn test_function() -> i32 {
    42
}
EOF

    # Compile the test program
    case "$target" in
        "x86_64"*)
            rustc --target "$target" -o /tmp/test_binary /tmp/test_main.rs 2>&1 | tee "$log_file"
            ;;
        "aarch64"*|"riscv64"*)
            cross rustc --target "$target" -o /tmp/test_binary /tmp/test_main.rs 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Basic compilation successful for $target"
        
        # Check if binary was created
        if [ -f "/tmp/test_binary" ]; then
            success "Binary created successfully"
            
            # Validate binary format
            case "$target" in
                "x86_64"*)
                    file /tmp/test_binary | grep -q "ELF 64-bit LSB executable" && success "Binary format valid" || warning "Binary format unexpected"
                    ;;
                "aarch64"*)
                    file /tmp/test_binary | grep -q "ELF 64-bit LSB executable" && success "ARM64 binary format valid" || warning "ARM64 binary format unexpected"
                    ;;
                "riscv64"*)
                    file /tmp/test_binary | grep -q "ELF 64-bit LSB executable" && success "RISC-V binary format valid" || warning "RISC-V binary format unexpected"
                    ;;
            esac
            
            # Test binary execution (if QEMU is available)
            if command -v qemu-$([ "$target" = "x86_64" ] && echo "x86_64" || echo "${target%*}")-static >/dev/null 2>&1; then
                log "Testing binary execution with QEMU"
                if qemu-$([ "$target" = "x86_64" ] && echo "x86_64" || echo "${target%*}")-static /tmp/test_binary; then
                    success "Binary execution successful"
                else
                    warning "Binary execution failed or timed out"
                fi
            else
                warning "QEMU not available for binary execution test"
            fi
        else
            error "Binary was not created"
            return 1
        fi
    else
        error "Basic compilation failed for $target"
        return 1
    fi
    
    # Cleanup
    rm -f /tmp/test_main.rs /tmp/test_binary
}

# Test library compilation
test_library_compilation() {
    local target=$1
    local log_file="/workspace/logs/lib_compile_${target//-/_}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Testing library compilation for target: $target"
    
    # Create a minimal library test
    cat > /tmp/test_lib.rs << 'EOF'
pub struct TestStruct {
    pub value: i32,
}

impl TestStruct {
    pub fn new(value: i32) -> Self {
        TestStruct { value }
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creation() {
        let test = TestStruct::new(42);
        assert_eq!(test.get_value(), 42);
    }
}
EOF

    # Compile the library
    case "$target" in
        "x86_64"*)
            rustc --target "$target" --crate-type rlib -o /tmp/test_lib.rlib /tmp/test_lib.rs 2>&1 | tee "$log_file"
            ;;
        "aarch64"*|"riscv64"*)
            cross rustc --target "$target" --crate-type rlib -o /tmp/test_lib.rlib /tmp/test_lib.rs 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Library compilation successful for $target"
        
        if [ -f "/tmp/test_lib.rlib" ]; then
            success "Library archive created successfully"
        else
            error "Library archive was not created"
            return 1
        fi
    else
        error "Library compilation failed for $target"
        return 1
    fi
    
    # Cleanup
    rm -f /tmp/test_lib.rs /tmp/test_lib.rlib
}

# Test full project compilation
test_project_compilation() {
    local target=$1
    local log_file="/workspace/logs/project_compile_${target//-/_}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Testing full project compilation for target: $target"
    
    # Build the actual project
    case "$target" in
        "x86_64"*)
            cargo build --target "$target" --all-features 2>&1 | tee "$log_file"
            ;;
        "aarch64"*|"riscv64"*)
            cross build --target "$target" --all-features 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Full project compilation successful for $target"
        
        # Check if binary was created
        local binary_path="target/$target/debug/multios"
        if [ "$target" = *"none"* ] || [ "$target" = *"musl"* ]; then
            binary_path="target/$target/release/multios"
        fi
        
        if [ -f "$binary_path" ]; then
            success "Project binary created: $binary_path"
            
            # Get binary size and info
            ls -lh "$binary_path"
            file "$binary_path"
            
            # Store binary info for later comparison
            echo "$target:$(stat -c%s "$binary_path"):$(date)" > "/workspace/artifacts/binary_info_${target//-/_}.txt"
        else
            warning "Project binary not found at expected location: $binary_path"
        fi
    else
        error "Full project compilation failed for $target"
        return 1
    fi
}

# Generate validation report
generate_validation_report() {
    local arch=$1
    local report_file="/workspace/reports/cross_compile_validation_${arch}_$(date +%Y%m%d_%H%M%S).json"
    
    # Collect validation results
    local compilation_status="PASS"
    local validation_results=()
    
    # Check logs for errors
    for log_file in /workspace/logs/compile_${arch}* /workspace/logs/lib_compile_${arch}* /workspace/logs/project_compile_${arch}*; do
        if [ -f "$log_file" ]; then
            if grep -q "error\|ERROR\|FAILED" "$log_file"; then
                compilation_status="FAIL"
                validation_results+=("$(basename "$log_file"):FAILED")
            else
                validation_results+=("$(basename "$log_file"):PASSED")
            fi
        fi
    done
    
    # Generate JSON report
    cat > "$report_file" << EOF
{
    "architecture": "$arch",
    "timestamp": "$(date -Iseconds)",
    "validation_status": "$compilation_status",
    "validation_results": [
$(printf '        {"test": "%s", "status": "%s"}' "${validation_results[@]}" | paste -sd, -)
    ],
    "target_platforms": [
        "${arch}-unknown-none",
        "${arch}-unknown-linux-gnu",
        "${arch}-unknown-linux-musl"
    ],
    "cross_compile_tools": {
        "rustup_targets": "installed",
        "cross_compiler": "$(command -v cross >/dev/null 2>&1 && echo "available" || echo "not_found")",
        "qemu_$(echo "$arch" | tr '_' '-'): "$(command -v qemu-$([ "$arch" = "x86_64" ] && echo "x86_64" || echo "${arch%_*}")-static >/dev/null 2>&1 && echo "available" || echo "not_found")"
    }
}
EOF

    success "Validation report generated: $report_file"
    return $([ "$compilation_status" = "PASS" ] && echo 0 || echo 1)
}

# Main validation function
main() {
    local arch=${1:-}
    
    log "Starting cross-compilation validation for MultiOS"
    
    if [ -z "$arch" ]; then
        log "Validating all architectures"
        for arch in "${ARCHITECTURES[@]}"; do
            log "=== Validating $arch ==="
            
            # Install Rust targets
            case "$arch" in
                "x86_64")
                    validate_toolchain "x86_64-unknown-none"
                    validate_toolchain "x86_64-unknown-linux-gnu"
                    validate_toolchain "x86_64-unknown-linux-musl"
                    ;;
                "arm64")
                    validate_toolchain "aarch64-unknown-none"
                    validate_toolchain "aarch64-unknown-linux-gnu"
                    validate_toolchain "aarch64-unknown-linux-musl"
                    ;;
                "riscv64")
                    validate_toolchain "riscv64gc-unknown-none-elf"
                    validate_toolchain "riscv64gc-unknown-linux-gnu"
                    validate_toolchain "riscv64gc-unknown-linux-musl"
                    ;;
            esac
            
            # Set up cross compilation environment
            validate_cross_setup "$arch"
            
            # Run validation tests
            local validation_passed=true
            
            # Test basic compilation
            if ! test_basic_compilation "${arch}-unknown-none"; then
                validation_passed=false
            fi
            
            # Test library compilation
            if ! test_library_compilation "${arch}-unknown-none"; then
                validation_passed=false
            fi
            
            # Test full project compilation
            if ! test_project_compilation "${arch}-unknown-none"; then
                validation_passed=false
            fi
            
            # Generate validation report
            if ! generate_validation_report "$arch"; then
                validation_passed=false
            fi
            
            if [ "$validation_passed" = true ]; then
                success "Cross-compilation validation PASSED for $arch"
            else
                error "Cross-compilation validation FAILED for $arch"
            fi
        done
    else
        # Validate specific architecture
        log "Validating specific architecture: $arch"
        
        if [[ " ${ARCHITECTURES[@]} " =~ " ${arch} " ]]; then
            # Install targets for this architecture
            case "$arch" in
                "x86_64")
                    validate_toolchain "x86_64-unknown-none"
                    ;;
                "arm64")
                    validate_toolchain "aarch64-unknown-none"
                    ;;
                "riscv64")
                    validate_toolchain "riscv64gc-unknown-none-elf"
                    ;;
            esac
            
            # Set up cross compilation
            validate_cross_setup "$arch"
            
            # Run tests
            test_basic_compilation "${arch}-unknown-none"
            test_library_compilation "${arch}-unknown-none"
            test_project_compilation "${arch}-unknown-none"
            generate_validation_report "$arch"
        else
            error "Unknown architecture: $arch"
            exit 1
        fi
    fi
    
    log "Cross-compilation validation complete"
}

main "$@"