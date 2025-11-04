#!/bin/bash
# QEMU ARM64 (AArch64) Test Runner
# Usage: ./run_arm64.sh [options]

set -e

# Default configuration
MEMORY="1G"
CPUS="2"
DISK_SIZE="10G"
MACHINE="virt"
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
    echo "QEMU ARM64 Test Runner"
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -m, --memory SIZE       Memory size (default: 1G)"
    echo "  -c, --cpus NUMBER       Number of CPUs (default: 2)"
    echo "  -d, --disk SIZE         Disk size (default: 10G)"
    echo "  -M, --machine TYPE      Machine type: virt|vexpress (default: virt)"
    echo "  -n, --network TYPE      Network type: user|none|bridge (default: user)"
    echo "  -s, --smp NUMBER        SMP setting (default: 2)"
    echo "  -h, --help              Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run with defaults (virt machine)"
    echo "  $0 -m 2G -c 4          # Run with 2GB RAM and 4 CPUs"
    echo "  $0 -M vexpress         # Run with Versatile Express machine"
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
        -M|--machine)
            MACHINE="$2"
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

echo -e "${GREEN}Starting QEMU ARM64 test environment${NC}"
echo "Memory: $MEMORY, CPUs: $CPUS, Machine: $MACHINE"

# Build QEMU command
QEMU_CMD="qemu-system-aarch64"
QEMU_ARGS="-machine type=$MACHINE"
QEMU_ARGS="$QEMU_ARGS -m $MEMORY"
QEMU_ARGS="$QEMU_ARGS -smp $SMP"

# CPU configuration
QEMU_ARGS="$QEMU_ARGS -cpu cortex-a57"

# Network configuration
case $NETWORK in
    user)
        QEMU_ARGS="$QEMU_ARGS -netdev user,id=net0 -device virtio-net,netdev=net0"
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

# Add firmware for ARM64
if [ "$MACHINE" = "virt" ]; then
    # Look for UEFI firmware
    if [ -f "/usr/share/AAVMF/AAVMF_CODE.fd" ]; then
        QEMU_ARGS="$QEMU_ARGS -drive if=pflash,format=raw,readonly=on,file=/usr/share/AAVMF/AAVMF_CODE.fd"
    fi
    if [ -f "/usr/share/AAVMF/AAVMF_VARS.fd" ]; then
        QEMU_ARGS="$QEMU_ARGS -drive if=pflash,format=raw,file=$IMAGES_DIR/efivars.fd"
    fi
fi

# Add disk if available
DISK_PATH="$DISKS_DIR/arm64_disk.qcow2"
if [ -f "$DISK_PATH" ]; then
    QEMU_ARGS="$QEMU_ARGS -hda $DISK_PATH"
else
    # Create a small boot disk
    echo -e "${YELLOW}Creating boot disk: $DISK_PATH${NC}"
    qemu-img create -f qcow2 "$DISK_PATH" "$DISK_SIZE"
    QEMU_ARGS="$QEMU_ARGS -hda $DISK_PATH"
fi

# Add CD-ROM if available
if [ -f "$IMAGES_DIR/ubuntu-arm64.iso" ]; then
    QEMU_ARGS="$QEMU_ARGS -cdrom $IMAGES_DIR/ubuntu-arm64.iso"
fi

# Additional features
QEMU_ARGS="$QEMU_ARGS -rtc base=utc"
QEMU_ARGS="$QEMU_ARGS -device qemu-xhci"
QEMU_ARGS="$QEMU_ARGS -device usb-tablet"
QEMU_ARGS="$QEMU_ARGS -device usb-kbd"

# Check if QEMU is available
if ! command -v $QEMU_CMD &> /dev/null; then
    echo -e "${RED}Error: $QEMU_CMD not found. Please install QEMU first.${NC}"
    echo "Install with: sudo apt install qemu-system-arm qemu-utils"
    exit 1
fi

echo -e "${GREEN}Launching QEMU with command:${NC}"
echo "$QEMU_CMD $QEMU_ARGS"
echo ""

# Add monitoring and logging
LOG_FILE="$BASE_DIR/logs/arm64_$(date +%Y%m%d_%H%M%S).log"
mkdir -p "$BASE_DIR/logs"

echo "Logging to: $LOG_FILE"
echo "Monitor: Press Ctrl+C to exit"

# Run QEMU
exec $QEMU_CMD $QEMU_ARGS 2>&1 | tee "$LOG_FILE"