#!/bin/bash
# MultiOS QEMU Monitor Setup Script
# Launches QEMU with debugging features for MultiOS kernel development

set -e

# Configuration
ARCH=${1:-"x86_64"}
KERNEL=${2:-"target/${ARCH}-unknown-none-elf/release/multios"}
SERIAL_PORT=${3:-"/tmp/multios_serial"}
GDB_PORT=${4:-"1234"}
MEMORY=${5:-"512M"}

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Print usage
print_usage() {
    echo "Usage: $0 [ARCH] [KERNEL] [SERIAL_PORT] [GDB_PORT] [MEMORY]"
    echo ""
    echo "Arguments:"
    echo "  ARCH        Target architecture (x86_64, aarch64, riscv64) [default: x86_64]"
    echo "  KERNEL      Path to kernel binary [default: target/\$ARCH-unknown-none-elf/release/multios]"
    echo "  SERIAL_PORT Unix socket for serial console [default: /tmp/multios_serial]"
    echo "  GDB_PORT    TCP port for GDB server [default: 1234]"
    echo "  MEMORY      Memory size [default: 512M]"
    echo ""
    echo "Examples:"
    echo "  $0 x86_64"
    echo "  $0 aarch64 target/aarch64-unknown-none-elf/release/multios /tmp/aarch64_serial 1235"
    echo "  $0 riscv64 /custom/kernel 1G"
}

# Check if kernel file exists
if [ ! -f "$KERNEL" ]; then
    print_message $RED "Error: Kernel file not found: $KERNEL"
    print_message $YELLOW "Build the kernel first with: cargo build --target ${ARCH}-unknown-none-elf"
    exit 1
fi

# Create serial socket directory
mkdir -p "$(dirname "$SERIAL_PORT")"

# Architecture-specific setup
case $ARCH in
    "x86_64"|"amd64")
        QEMU_CMD="qemu-system-x86_64"
        QEMU_ARGS="-kernel $KERNEL -m $MEMORY -boot m"
        QEMU_ARGS="$QEMU_ARGS -serial unix:$SERIAL_PORT,server,nowait"
        QEMU_ARGS="$QEMU_ARGS -gdb tcp::$GDB_PORT"
        QEMU_ARGS="$QEMU_ARGS -nographic -enable-kvm"
        ARCH_DESC="Intel/AMD 64-bit"
        ;;
    "aarch64"|"arm64")
        QEMU_CMD="qemu-system-aarch64"
        QEMU_ARGS="-machine virt -cpu cortex-a57 -kernel $KERNEL -m $MEMORY"
        QEMU_ARGS="$QEMU_ARGS -serial unix:$SERIAL_PORT,server,nowait"
        QEMU_ARGS="$QEMU_ARGS -gdb tcp::$GDB_PORT"
        QEMU_ARGS="$QEMU_ARGS -nographic"
        ARCH_DESC="ARM 64-bit"
        ;;
    "riscv64"|"riscv")
        QEMU_CMD="qemu-system-riscv64"
        QEMU_ARGS="-machine virt -kernel $KERNEL -m $MEMORY"
        QEMU_ARGS="$QEMU_ARGS -serial unix:$SERIAL_PORT,server,nowait"
        QEMU_ARGS="$QEMU_ARGS -gdb tcp::$GDB_PORT"
        QEMU_ARGS="$QEMU_ARGS -nographic"
        ARCH_DESC="RISC-V 64-bit"
        ;;
    *)
        print_message $RED "Unknown architecture: $ARCH"
        print_message $YELLOW "Supported architectures: x86_64, aarch64, riscv64"
        exit 1
        ;;
esac

# Check if QEMU binary exists
if ! command -v $QEMU_CMD &> /dev/null; then
    print_message $RED "Error: $QEMU_CMD not found"
    print_message $YELLOW "Install QEMU with: sudo apt install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64"
    exit 1
fi

# Print startup information
print_message $GREEN "MultiOS QEMU Monitor"
print_message $BLUE "===================="
print_message $BLUE "Architecture: $ARCH_DESC ($ARCH)"
print_message $BLUE "Kernel: $KERNEL"
print_message $BLUE "Memory: $MEMORY"
print_message $BLUE "Serial: $SERIAL_PORT"
print_message $BLUE "GDB Server: tcp::$GDB_PORT"
echo ""

print_message $YELLOW "QEMU Monitor Commands:"
print_message $YELLOW "  Press Ctrl-A then C to enter monitor mode"
print_message $YELLOW "  In monitor mode:"
print_message $YELLOW "    info registers    - Show CPU registers"
print_message $YELLOW "    info memory       - Show memory mapping"
print_message $YELLOW "    info network      - Show network config"
print_message $YELLOW "    system_powerdown  - Power down system"
print_message $YELLOW "    system_reset      - Reset system"
print_message $YELLOW "    quit              - Quit QEMU"
echo ""

print_message $GREEN "Debugging Setup:"
print_message $GREEN "  GDB: gdb-multiarch $KERNEL"
print_message $GREEN "  Connect: target remote localhost:$GDB_PORT"
echo ""

print_message $GREEN "Serial Console:"
print_message $GREEN "  Run in another terminal: ./scripts/serial_console.sh $SERIAL_PORT"
echo ""

print_message $YELLOW "Starting QEMU... (Press Ctrl-C to exit)"
echo ""

# Change to project directory to ensure relative paths work
cd "$(dirname "$0")/../.."

# Run QEMU
exec $QEMU_CMD $QEMU_ARGS
