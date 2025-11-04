#!/bin/bash
# QEMU x86_64 Test Runner
# Usage: ./run_x86_64.sh [options]

set -e

# Default configuration
MEMORY="512M"
CPUS="2"
DISK_SIZE="10G"
BOOT_MODE="cdrom"
NETWORK="user"
DISPLAY="gtk"
SMP="2"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo "QEMU x86_64 Test Runner"
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -m, --memory SIZE       Memory size (default: 512M)"
    echo "  -c, --cpus NUMBER       Number of CPUs (default: 2)"
    echo "  -d, --disk SIZE         Disk size (default: 10G)"
    echo "  -b, --boot MODE         Boot mode: cdrom|hd|network (default: cdrom)"
    echo "  -n, --network TYPE      Network type: user|none|bridge (default: user)"
    echo "  -s, --smp NUMBER        SMP setting (default: 2)"
    echo "  -h, --help              Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run with defaults"
    echo "  $0 -m 1G -c 4          # Run with 1GB RAM and 4 CPUs"
    echo "  $0 -b hd               # Boot from hard disk"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--memory)
            MEMORY="$2"
            shift 2
            ;;
        -c|--cpus)
            CPUS="$2"
            shift 2
            ;;
        -d|--disk)
            DISK_SIZE="$2"
            shift 2
            ;;
        -b|--boot)
            BOOT_MODE="$2"
            shift 2
            ;;
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -s|--smp)
            SMP="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Configuration paths
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="$BASE_DIR/configs"
IMAGES_DIR="$BASE_DIR/images"
DISKS_DIR="$BASE_DIR/disks"

echo -e "${GREEN}Starting QEMU x86_64 test environment${NC}"
echo "Memory: $MEMORY, CPUs: $CPUS, Disk: $DISK_SIZE"

# Build QEMU command
QEMU_CMD="qemu-system-x86_64"
QEMU_ARGS="-machine type=pc"
QEMU_ARGS="$QEMU_ARGS -m $MEMORY"
QEMU_ARGS="$QEMU_ARGS -smp $SMP"

# Network configuration
case $NETWORK in
    user)
        QEMU_ARGS="$QEMU_ARGS -netdev user,id=net0 -device e1000,netdev=net0"
        ;;
    none)
        QEMU_ARGS="$QEMU_ARGS -net none"
        ;;
    bridge)
        QEMU_ARGS="$QEMU_ARGS -netdev bridge,id=net0 -device virtio-net,netdev=net0"
        ;;
esac

# Display configuration
case $DISPLAY in
    gtk)
        QEMU_ARGS="$QEMU_ARGS -display gtk"
        ;;
    curses)
        QEMU_ARGS="$QEMU_ARGS -display curses"
        ;;
    none)
        QEMU_ARGS="$QEMU_ARGS -display none"
        ;;
esac

# Boot configuration
case $BOOT_MODE in
    cdrom)
        if [ -f "$IMAGES_DIR/ubuntu.iso" ]; then
            QEMU_ARGS="$QEMU_ARGS -cdrom $IMAGES_DIR/ubuntu.iso"
        fi
        ;;
    hd)
        DISK_PATH="$DISKS_DIR/x86_64_disk.qcow2"
        if [ ! -f "$DISK_PATH" ]; then
            echo -e "${YELLOW}Creating disk image: $DISK_PATH${NC}"
            qemu-img create -f qcow2 "$DISK_PATH" "$DISK_SIZE"
        fi
        QEMU_ARGS="$QEMU_ARGS -hda $DISK_PATH"
        ;;
esac

# Additional features
QEMU_ARGS="$QEMU_ARGS -enable-kvm"
QEMU_ARGS="$QEMU_ARGS -rtc base=utc"
QEMU_ARGS="$QEMU_ARGS -soundhw ac97"
QEMU_ARGS="$QEMU_ARGS -device VGA"

# Check if QEMU is available
if ! command -v $QEMU_CMD &> /dev/null; then
    echo -e "${RED}Error: $QEMU_CMD not found. Please install QEMU first.${NC}"
    echo "Install with: sudo apt install qemu-system qemu-utils"
    exit 1
fi

echo -e "${GREEN}Launching QEMU with command:${NC}"
echo "$QEMU_CMD $QEMU_ARGS"
echo ""

# Add monitoring and logging
LOG_FILE="$BASE_DIR/logs/x86_64_$(date +%Y%m%d_%H%M%S).log"
mkdir -p "$BASE_DIR/logs"

echo "Logging to: $LOG_FILE"
echo "Monitor: Press Ctrl+C to exit"

# Run QEMU
exec $QEMU_CMD $QEMU_ARGS 2>&1 | tee "$LOG_FILE"