#!/bin/bash
# QEMU Testing Environment Setup Script
# This script sets up the QEMU testing environment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=================================${NC}"
echo -e "${BLUE}QEMU Testing Environment Setup${NC}"
echo -e "${BLUE}=================================${NC}"

# Script directory
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="$BASE_DIR/scripts"

echo ""
echo -e "${YELLOW}Base directory: $BASE_DIR${NC}"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    echo -e "${RED}Warning: Running as root is not recommended${NC}"
fi

echo ""
echo -e "${BLUE}Step 1: Checking QEMU installation...${NC}"

# Check for QEMU installations
ARCHS=("qemu-system-x86_64" "qemu-system-aarch64" "qemu-system-riscv64")
INSTALLED_ARCHS=()

for arch in "${ARCHS[@]}"; do
    if command -v "$arch" &> /dev/null; then
        VERSION=$("$arch" --version | head -n1)
        echo -e "  ${GREEN}✓${NC} $arch: $VERSION"
        INSTALLED_ARCHS+=("$arch")
    else
        echo -e "  ${RED}✗${NC} $arch: Not installed"
    fi
done

echo ""
echo -e "${BLUE}Step 2: Installing QEMU (if needed)...${NC}"

if [[ ${#INSTALLED_ARCHS[@]} -eq 0 ]]; then
    echo -e "${YELLOW}No QEMU installations found.${NC}"
    echo ""
    echo -e "${BLUE}To install QEMU, run:${NC}"
    echo "  Ubuntu/Debian: sudo apt update && sudo apt install qemu-system qemu-system-arm qemu-system-riscv qemu-utils"
    echo "  CentOS/RHEL:   sudo yum install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv"
    echo "  Fedora:        sudo dnf install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv"
    echo ""
    read -p "Press Enter to continue without installation..."
else
    echo -e "${GREEN}QEMU is already installed${NC}"
fi

echo ""
echo -e "${BLUE}Step 3: Setting up directory structure...${NC}"

# Create necessary directories
mkdir -p "$BASE_DIR/images"
mkdir -p "$BASE_DIR/disks"
mkdir -p "$BASE_DIR/logs"
mkdir -p "$BASE_DIR/templates"

echo -e "  ${GREEN}✓${NC} Created directory structure"

# Make scripts executable
chmod +x "$SCRIPTS_DIR"/*.sh
echo -e "  ${GREEN}✓${NC} Made scripts executable"

echo ""
echo -e "${BLUE}Step 4: Creating disk images...${NC}"

# Create sample disk images
if command -v qemu-img &> /dev/null; then
    echo "Creating sample disk images..."
    qemu-img create -f qcow2 "$BASE_DIR/disks/x86_64_disk.qcow2" 10G
    qemu-img create -f qcow2 "$BASE_DIR/disks/arm64_disk.qcow2" 10G
    qemu-img create -f qcow2 "$BASE_DIR/disks/riscv_disk.qcow2" 10G
    echo -e "  ${GREEN}✓${NC} Created sample disk images"
else
    echo -e "  ${YELLOW}⚠${NC} qemu-img not found, skipping disk creation"
fi

echo ""
echo -e "${BLUE}Step 5: Checking for sample images...${NC}"

SAMPLE_IMAGES=(
    "$BASE_DIR/images/ubuntu.iso"
    "$BASE_DIR/images/ubuntu-arm64.iso"
    "$BASE_DIR/images/fedora-riscv.iso"
)

for img in "${SAMPLE_IMAGES[@]}"; do
    if [[ -f "$img" ]]; then
        echo -e "  ${GREEN}✓${NC} $img"
    else
        echo -e "  ${YELLOW}○${NC} $img (not found)"
    fi
done

echo ""
echo -e "${BLUE}Step 6: Creating symlinks for easy access...${NC}"

# Create symlinks in /usr/local/bin for easy access
if [[ -w "/usr/local/bin" ]]; then
    ln -sf "$SCRIPTS_DIR/run_x86_64.sh" /usr/local/bin/qemu-x86_64
    ln -sf "$SCRIPTS_DIR/run_arm64.sh" /usr/local/bin/qemu-arm64
    ln -sf "$SCRIPTS_DIR/run_riscv.sh" /usr/local/bin/qemu-riscv
    ln -sf "$SCRIPTS_DIR/run_all.sh" /usr/local/bin/qemu-all
    echo -e "  ${GREEN}✓${NC} Created symlinks in /usr/local/bin"
else
    echo -e "  ${YELLOW}⚠${NC} Cannot create symlinks (no write access to /usr/local/bin)"
    echo "  Run 'sudo ln -sf $SCRIPTS_DIR/*.sh /usr/local/bin/' to add them manually"
fi

echo ""
echo -e "${BLUE}Step 7: Testing configuration...${NC}"

# Test if scripts are executable
for script in run_x86_64.sh run_arm64.sh run_riscv.sh run_all.sh; do
    if [[ -x "$SCRIPTS_DIR/$script" ]]; then
        echo -e "  ${GREEN}✓${NC} $script is executable"
    else
        echo -e "  ${RED}✗${NC} $script is not executable"
    fi
done

echo ""
echo -e "${GREEN}=================================${NC}"
echo -e "${GREEN}Setup completed successfully!${NC}"
echo -e "${GREEN}=================================${NC}"
echo ""
echo -e "${BLUE}Quick start:${NC}"
echo "  cd $BASE_DIR"
echo "  ./scripts/run_all.sh"
echo ""
echo -e "${BLUE}Individual architecture tests:${NC}"
echo "  ./scripts/run_x86_64.sh --help"
echo "  ./scripts/run_arm64.sh --help"
echo "  ./scripts/run_riscv.sh --help"
echo ""
echo -e "${BLUE}Configuration files:${NC}"
echo "  $BASE_DIR/configs/*.conf"
echo ""
echo -e "${YELLOW}Note: Make sure to download ISO images for full testing${NC}"
echo ""