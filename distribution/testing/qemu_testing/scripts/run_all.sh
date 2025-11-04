#!/bin/bash
# Unified QEMU Test Runner
# Supports x86_64, ARM64, and RISC-V architectures
# Usage: ./run_all.sh [arch] [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo "Unified QEMU Test Runner"
    echo "Usage: $0 [arch] [options]"
    echo ""
    echo "Supported architectures: x86_64, arm64, riscv"
    echo ""
    echo "Examples:"
    echo "  $0 x86_64              # Run x86_64 test"
    echo "  $0 arm64 -m 2G         # Run ARM64 test with 2GB RAM"
    echo "  $0 riscv -c 4          # Run RISC-V test with 4 CPUs"
    echo "  $0 all                 # Run all architectures in sequence"
    echo ""
    echo "For architecture-specific options, see:"
    echo "  ./run_x86_64.sh --help"
    echo "  ./run_arm64.sh --help"
    echo "  ./run_riscv.sh --help"
}

# Check if script is run with --help
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    show_help
    exit 0
fi

# Check if no arguments provided
if [[ $# -eq 0 ]]; then
    echo -e "${RED}Error: No architecture specified${NC}"
    show_help
    exit 1
fi

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Architecture mapping
declare -A ARCH_MAP=(
    ["x86_64"]="run_x86_64.sh"
    ["arm64"]="run_arm64.sh"
    ["riscv"]="run_riscv.sh"
    ["all"]="all"
)

# Check if architecture is supported
ARCH="$1"
if [[ ! "${ARCH_MAP[$ARCH]+isset}" ]]; then
    echo -e "${RED}Error: Unsupported architecture '$ARCH'${NC}"
    echo "Supported architectures: ${!ARCH_MAP[@]}"
    exit 1
fi

# Shift to get remaining arguments
shift

# Function to run architecture-specific script
run_arch() {
    local arch="$1"
    shift
    local script="${ARCH_MAP[$arch]}"
    
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}Running $arch test${NC}"
    echo -e "${BLUE}================================${NC}"
    
    "$SCRIPT_DIR/$script" "$@"
}

# Function to run all architectures
run_all() {
    echo -e "${GREEN}Running tests for all architectures...${NC}"
    echo ""
    
    for arch in x86_64 arm64 riscv; do
        echo -e "${YELLOW}Press Enter to continue to $arch test (or Ctrl+C to exit)...${NC}"
        read -r
        run_arch "$arch"
        echo ""
        echo -e "${GREEN}$arch test completed${NC}"
        echo ""
    done
    
    echo -e "${GREEN}All tests completed!${NC}"
}

# Execute based on architecture
if [[ "$ARCH" == "all" ]]; then
    run_all
else
    run_arch "$ARCH" "$@"
fi