#!/bin/bash
# QEMU VM Monitor
# Shows system resources and QEMU VM status

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Show header
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}QEMU Virtual Machine Monitor${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check QEMU installation
echo -e "${CYAN}1. QEMU Installation Status${NC}"
echo "==============================="

ARCHS=("qemu-system-x86_64" "qemu-system-aarch64" "qemu-system-riscv64")
INSTALLED=0

for arch in "${ARCHS[@]}"; do
    if command_exists "$arch"; then
        VERSION=$("$arch" --version | head -n1)
        echo -e "  ${GREEN}✓${NC} $arch"
        echo "    $VERSION"
        INSTALLED=$((INSTALLED + 1))
    else
        echo -e "  ${RED}✗${NC} $arch: Not installed"
    fi
done

if command_exists qemu-img; then
    echo -e "  ${GREEN}✓${NC} qemu-img"
    echo "    Version: $(qemu-img --version | head -n1 | cut -d' ' -f3)"
else
    echo -e "  ${RED}✗${NC} qemu-img: Not installed"
fi

echo ""

# System resources
echo -e "${CYAN}2. System Resources${NC}"
echo "======================"

if command_exists free; then
    echo "Memory:"
    free -h
    echo ""
fi

if command_exists lscpu; then
    echo "CPU:"
    lscpu | grep -E "Model name|CPU\(|Thread|Socket"
    echo ""
fi

if command_exists df; then
    echo "Disk Space:"
    df -h / | tail -1
    echo ""
fi

# KVM status
echo -e "${CYAN}3. KVM Status${NC}"
echo "==============="

if [ -c /dev/kvm ]; then
    echo -e "  ${GREEN}✓${NC} KVM device exists"
    KVM_PERMS=$(ls -l /dev/kvm | cut -d' ' -f1)
    echo "    Permissions: $KVM_PERMS"
    
    if groups | grep -q kvm; then
        echo -e "  ${GREEN}✓${NC} User is in kvm group"
    else
        echo -e "  ${YELLOW}⚠${NC} User is NOT in kvm group"
    fi
else
    echo -e "  ${RED}✗${NC} KVM device not found"
    echo "    Install kvm package or enable virtualization in BIOS"
fi

echo ""

# Running VMs
echo -e "${CYAN}4. Running VMs${NC}"
echo "================"

if command_exists ps; then
    QEMU_PIDS=$(ps aux | grep -E '[q]emu-system' | wc -l)
    if [ "$QEMU_PIDS" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $QEMU_PIDS VM(s) running"
        echo ""
        ps aux | grep -E '[q]emu-system' | while read -r line; do
            PID=$(echo "$line" | awk '{print $2}')
            ARCH=$(echo "$line" | grep -o 'qemu-system-[a-z0-9]*')
            CMD=$(echo "$line" | sed 's/.*qemu-system-[a-z0-9]*//' | sed 's/^ *//' | cut -c1-60)
            echo -e "  ${BLUE}•${NC} PID: $PID | Arch: $ARCH"
            echo "    $CMD"
        done
    else
        echo -e "  ${YELLOW}○${NC} No VMs currently running"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Cannot check running processes (ps not found)"
fi

echo ""

# Disk images
echo -e "${CYAN}5. Disk Images${NC}"
echo "================="

if [ -d "disks" ]; then
    DISK_COUNT=$(ls disks/*.qcow2 2>/dev/null | wc -l)
    if [ "$DISK_COUNT" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $DISK_COUNT disk image(s) found"
        ls -lh disks/*.qcow2 2>/dev/null | while read -r line; do
            SIZE=$(echo "$line" | awk '{print $5}')
            FILE=$(echo "$line" | awk '{print $9}')
            echo -e "    ${BLUE}•${NC} $FILE ($SIZE)"
        done
    else
        echo -e "  ${YELLOW}○${NC} No disk images found"
        echo "    Run 'make disks' to create them"
    fi
else
    echo -e "  ${YELLOW}○${NC} Disks directory not found"
fi

echo ""

# ISO images
echo -e "${CYAN}6. ISO Images${NC}"
echo "==============="

if [ -d "images" ]; then
    ISO_COUNT=$(ls images/*.iso 2>/dev/null | wc -l)
    if [ "$ISO_COUNT" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $ISO_COUNT ISO image(s) found"
        ls -lh images/*.iso 2>/dev/null | while read -r line; do
            SIZE=$(echo "$line" | awk '{print $5}')
            FILE=$(echo "$line" | awk '{print $9}')
            echo -e "    ${BLUE}•${NC} $FILE ($SIZE)"
        done
    else
        echo -e "  ${YELLOW}○${NC} No ISO images found"
        echo "    Download ISO images to images/ directory"
    fi
else
    echo -e "  ${YELLOW}○${NC} Images directory not found"
fi

echo ""

# Recent logs
echo -e "${CYAN}7. Recent Activity${NC}"
echo "=================="

if [ -d "logs" ]; then
    LOG_COUNT=$(ls logs/*.log 2>/dev/null | wc -l)
    if [ "$LOG_COUNT" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $LOG_COUNT log file(s) found"
        echo "    Most recent:"
        ls -lt logs/*.log 2>/dev/null | head -3 | while read -r line; do
            DATE=$(echo "$line" | awk '{print $6, $7, $8}')
            FILE=$(echo "$line" | awk '{print $9}')
            echo -e "    ${BLUE}•${NC} $FILE ($DATE)"
        done
    else
        echo -e "  ${YELLOW}○${NC} No logs found"
    fi
else
    echo -e "  ${YELLOW}○${NC} Logs directory not found"
fi

echo ""

# Network status
echo -e "${CYAN}8. Network Status${NC}"
echo "=================="

if command_exists ip; then
    BR_COUNT=$(ip link show type bridge 2>/dev/null | grep -c "^.*[0-9]:.*" || true)
    if [ "$BR_COUNT" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $BR_COUNT bridge interface(s) found"
        ip link show type bridge 2>/dev/null | grep -E "^[0-9]+:" | while read -r line; do
            IFACE=$(echo "$line" | cut -d':' -f2 | sed 's/^ *//')
            echo -e "    ${BLUE}•${NC} $IFACE"
        done
    else
        echo -e "  ${YELLOW}○${NC} No bridge interfaces found"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Cannot check network interfaces (ip command not found)"
fi

echo ""

# Quick actions
echo -e "${CYAN}9. Quick Actions${NC}"
echo "================="
echo -e "  ${GREEN}1.${NC} Run x86_64 test:    make test-x86"
echo -e "  ${GREEN}2.${NC} Run ARM64 test:     make test-arm"
echo -e "  ${GREEN}3.${NC} Run RISC-V test:    make test-riscv"
echo -e "  ${GREEN}4.${NC} Run all tests:      make test-all"
echo -e "  ${GREEN}5.${NC} Create disks:        make disks"
echo -e "  ${GREEN}6.${NC} Check logs:          make logs"
echo -e "  ${GREEN}7.${NC} Watch logs:          make watch"
echo -e "  ${GREEN}8.${NC} Clean up:            make clean"

echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}Monitor completed!${NC}"
echo -e "${BLUE}================================${NC}"