#!/bin/bash
# MultiOS Bootable Media Creation Tool
# Creates bootable USB/CD/DVD media for MultiOS installation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SUPPORTED_ARCHS=("x86_64" "arm64" "riscv64")
SUPPORTED_FORMATS=("iso" "img" "usb")
DEFAULT_ISO_NAME="multios-installer"
DEFAULT_ISO_VERSION="1.0"

# Variables
SOURCE_DIR=""
OUTPUT_PATH=""
MEDIA_TYPE="iso"
ARCH="x86_64"
ISO_NAME="$DEFAULT_ISO_NAME"
ISO_VERSION="$DEFAULT_ISO_VERSION"
WORK_DIR="/tmp/multios-iso-$$"
MOUNT_DIR=""
TARGET_DEVICE=""
INTERACTIVE_MODE=true
CUSTOM_PACKAGES=()
KERNEL_VERSION=""
INITRD_VERSION=""
EFI_SUPPORT=true
GRUB_SUPPORT=true
MEMTEST_INCLUDED=false
RESCUE_TOOLS_INCLUDED=false
NETINSTALL=false
PACKAGES_LIST="/dev/null"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to print colored output
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to check if running as root for device operations
check_device_access() {
    if [[ "$MEDIA_TYPE" == "usb" ]] && [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: USB media creation requires root privileges"
        exit 1
    fi
}

# Function to detect system architecture
detect_architecture() {
    local arch=$(uname -m)
    case $arch in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        riscv64)
            echo "riscv64"
            ;;
        *)
            echo "$arch"
            ;;
    esac
}

# Function to check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    local missing_deps=()
    local required_commands=("mkisofs" "genisoimage" "xorriso" "parted" "mkfs.vfat" "mkfs.ext4")
    
    # Check for CD/DVD creation tools
    for cmd in "mkisofs" "genisoimage" "xorriso"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    # Check for USB creation tools if needed
    if [[ "$MEDIA_TYPE" == "usb" ]]; then
        local usb_commands=("dd" "parted" "mkfs.vfat")
        for cmd in "${usb_commands[@]}"; do
            if ! command -v "$cmd" &> /dev/null; then
                missing_deps+=("$cmd")
            fi
        done
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_color $RED "Error: Missing required dependencies: ${missing_deps[*]}"
        print_color $YELLOW "Install with: sudo apt-get install genisoimage parted dosfstools"
        exit 1
    fi
    
    log "All dependencies available"
}

# Function to create working directory
create_working_directory() {
    log "Creating working directory: $WORK_DIR"
    mkdir -p "$WORK_DIR"
    chmod 755 "$WORK_DIR"
}

# Function to create directory structure
create_directory_structure() {
    log "Creating ISO directory structure..."
    
    local iso_dir="$WORK_DIR/iso"
    
    # Create basic directory structure
    mkdir -p "$iso_dir"/{boot,isolinux,efi,EFI,multios,packages,live,rescue,docs}
    
    # Create architecture-specific directories
    case $ARCH in
        "x86_64")
            mkdir -p "$iso_dir/boot/grub"
            mkdir -p "$iso_dir/EFI/boot"
            ;;
        "arm64")
            mkdir -p "$iso_dir/boot/grub"
            mkdir -p "$iso_dir/EFI/boot"
            ;;
        "riscv64")
            mkdir -p "$iso_dir/boot"
            ;;
    esac
    
    # Create live system directory
    mkdir -p "$iso_dir/live"/{filesystem.squashfs,kernel,vmlinuz,initrd}
    
    echo "$iso_dir"
}

# Function to copy kernel and initrd
copy_kernel_initrd() {
    local iso_dir="$1"
    log "Copying kernel and initrd..."
    
    # Find kernel and initrd in current system or provided source
    local kernel_sources=("/boot/vmlinuz" "/boot/kernel")
    local initrd_sources=("/boot/initrd.img" "/boot/initrd" "/boot/initramfs")
    
    local kernel_found=false
    local initrd_found=false
    
    # Find kernel
    for source in "${kernel_sources[@]}"; do
        if [[ -f "$source" ]]; then
            local kernel_name=$(basename "$source")
            cp "$source" "$iso_dir/live/$kernel_name" 2>/dev/null || true
            KERNEL_VERSION="$kernel_name"
            kernel_found=true
            break
        fi
    done
    
    # Find initrd
    for source in "${initrd_sources[@]}"; do
        if [[ -f "$source" ]]; then
            local initrd_name=$(basename "$source")
            cp "$source" "$iso_dir/live/$initrd_name" 2>/dev/null || true
            INITRD_VERSION="$initrd_name"
            initrd_found=true
            break
        fi
    done
    
    if [[ "$kernel_found" == "false" ]]; then
        print_color $YELLOW "Warning: Kernel not found, creating minimal kernel stub"
        echo "kernel_image_missing" > "$iso_dir/live/kernel_missing"
    fi
    
    if [[ "$initrd_found" == "false" ]]; then
        print_color $YELLOW "Warning: Initrd not found, creating minimal initrd stub"
        echo "initrd_image_missing" > "$iso_dir/live/initrd_missing"
    fi
}

# Function to create bootloaders
create_bootloaders() {
    local iso_dir="$1"
    log "Creating bootloaders for $ARCH..."
    
    case $ARCH in
        "x86_64"|"arm64")
            create_grub_bootloader "$iso_dir"
            ;;
        "riscv64")
            create_riscv_bootloader "$iso_dir"
            ;;
    esac
    
    if [[ "$MEMTEST_INCLUDED" == "true" ]]; then
        include_memtest "$iso_dir"
    fi
}

# Function to create GRUB bootloader
create_grub_bootloader() {
    local iso_dir="$1"
    log "Creating GRUB bootloader..."
    
    # Create GRUB configuration
    local grub_config="$iso_dir/boot/grub/grub.cfg"
    
    cat > "$grub_config" << EOF
set timeout=10
set default=0

# MultiOS Installer Boot Menu

menuentry "MultiOS Installer" {
    linux /live/vmlinuz quiet splash boot=live
    initrd /live/initrd.img
}

menuentry "MultiOS (safe graphics)" {
    linux /live/vmlinuz nomodeset quiet splash boot=live
    initrd /live/initrd.img
}

menuentry "MultiOS (text mode)" {
    linux /live/vmlinuz text-only boot=live
    initrd /live/initrd.img
}

menuentry "MultiOS (rescue mode)" {
    linux /live/vmlinuz rescue boot=live
    initrd /live/initrd.img
}
EOF
    
    if [[ "$MEMTEST_INCLUDED" == "true" ]]; then
        cat >> "$grub_config" << EOF

menuentry "Memory Test" {
    linux /boot/memtest.bin
}
EOF
    fi
    
    cat >> "$grub_config" << EOF

menuentry "Boot from first hard disk" {
    exit 1
}
EOF
    
    # Create ISOLINUX configuration
    create_isolinux_config "$iso_dir"
    
    # Create EFI bootloader
    if [[ "$EFI_SUPPORT" == "true" ]]; then
        create_efi_bootloader "$iso_dir"
    fi
}

# Function to create ISOLINUX configuration
create_isolinux_config() {
    local iso_dir="$1"
    log "Creating ISOLINUX configuration..."
    
    # Create isolinux configuration
    cat > "$iso_dir/isolinux/isolinux.cfg" << EOF
UI menu.c32
PROMPT 0
TIMEOUT 10
DEFAULT MultiOS

MENU TITLE MultiOS Installer Boot Menu

LABEL MultiOS Installer
    MENU LABEL MultiOS Installer
    KERNEL /live/vmlinuz
    APPEND initrd=/live/initrd.img quiet splash boot=live

LABEL MultiOS (safe graphics)
    MENU LABEL MultiOS (safe graphics)
    KERNEL /live/vmlinuz
    APPEND initrd=/live/initrd.img nomodeset quiet splash boot=live

LABEL MultiOS (text mode)
    MENU LABEL MultiOS (text mode)
    KERNEL /live/vmlinuz
    APPEND initrd=/live/initrd.img text-only boot=live

LABEL MultiOS (rescue mode)
    MENU LABEL MultiOS (rescue mode)
    KERNEL /live/vmlinuz
    APPEND initrd=/live/initrd.img rescue boot=live
EOF
    
    if [[ "$MEMTEST_INCLUDED" == "true" ]]; then
        cat >> "$iso_dir/isolinux/isolinux.cfg" << EOF

LABEL Memtest86+
    MENU LABEL Memory Test (Memtest86+)
    KERNEL /boot/memtest.bin
EOF
    fi
    
    # Copy menu.c32 from syslinux if available
    for path in /usr/lib/ISOLINUX/menu.c32 /usr/lib/syslinux/menu.c32 /usr/share/syslinux/menu.c32; do
        if [[ -f "$path" ]]; then
            cp "$path" "$iso_dir/isolinux/"
            break
        fi
    done
}

# Function to create EFI bootloader
create_efi_bootloader() {
    local iso_dir="$1"
    log "Creating EFI bootloader..."
    
    # Create EFI GRUB configuration
    cat > "$iso_dir/EFI/boot/grub.cfg" << EOF
set timeout=10
set default=0

# MultiOS EFI Boot Menu

menuentry "MultiOS Installer" {
    linuxefi /live/vmlinuz quiet splash boot=live
    initrdefi /live/initrd.img
}

menuentry "MultiOS (safe graphics)" {
    linuxefi /live/vmlinuz nomodeset quiet splash boot=live
    initrdefi /live/initrd.img
}

menuentry "MultiOS (text mode)" {
    linuxefi /live/vmlinuz text-only boot=live
    initrdefi /live/initrd.img
}
EOF
    
    # Create EFI binary (requires grub-efi-bin package)
    if command -v grub-mkstandalone &> /dev/null; then
        log "Creating EFI boot binary..."
        grub-mkstandalone -d /usr/lib/grub/x86_64-efi -O x86_64-efi -o "$iso_dir/EFI/boot/bootx64.efi" \
            boot/grub/grub.cfg 2>/dev/null || true
    fi
}

# Function to create RISC-V bootloader
create_riscv_bootloader() {
    local iso_dir="$1"
    log "Creating RISC-V bootloader..."
    
    # Create OpenSBI/PK boot configuration
    cat > "$iso_dir/boot/bootconfig.txt" << EOF
# MultiOS RISC-V Boot Configuration
kernel=/live/vmlinuz
initrd=/live/initrd.img
cmdline=quiet splash boot=live console=ttyS0,115200
EOF
    
    # Create U-Boot script
    cat > "$iso_dir/boot/boot.scr" << EOF
# MultiOS U-Boot Script
echo "MultiOS RISC-V Boot..."
if fatload mmc 0:1 0x84000000 /live/vmlinuz; then
    if fatload mmc 0:1 0x88000000 /live/initrd.img; then
        booti 0x84000000 0x88000000 0x80000000
    else
        booti 0x84000000 - 0x80000000
    fi
fi
EOF
    
    # Try to create U-Boot image
    if command -v mkimage &> /dev/null; then
        mkimage -A riscv -O linux -T script -C none -n "MultiOS Boot" \
            -d "$iso_dir/boot/boot.scr" "$iso_dir/boot/boot.scr.uimg" 2>/dev/null || true
    fi
}

# Function to include MemTest
include_memtest() {
    local iso_dir="$1"
    log "Including MemTest..."
    
    # Download or copy MemTest
    local memtest_paths=("/usr/lib/memtest86+/memtest.bin" "/boot/memtest.bin" "/memtest.bin")
    
    for path in "${memtest_paths[@]}"; do
        if [[ -f "$path" ]]; then
            cp "$path" "$iso_dir/boot/memtest.bin"
            log "MemTest included"
            return 0
        fi
    done
    
    # Download MemTest if not found
    if command -v wget &> /dev/null; then
        log "Downloading MemTest..."
        wget -q https://memtest86.com/downloads/memtest86-usb.zip -O "$WORK_DIR/memtest86.zip" 2>/dev/null || true
        if [[ -f "$WORK_DIR/memtest86.zip" ]]; then
            unzip -q "$WORK_DIR/memtest86.zip" -d "$WORK_DIR/" 2>/dev/null || true
            if [[ -f "$WORK_DIR/memtest86.bin" ]]; then
                cp "$WORK_DIR/memtest86.bin" "$iso_dir/boot/memtest.bin"
                log "MemTest downloaded and included"
            fi
        fi
    fi
}

# Function to create live system filesystem
create_live_filesystem() {
    local iso_dir="$1"
    log "Creating live system filesystem..."
    
    # Create squashfs for live system
    local live_dir="$WORK_DIR/live_root"
    mkdir -p "$live_dir"/{bin,sbin,etc,var,usr/{bin,sbin,lib},lib,home,root,tmp,run,mnt,media,opt,srv,proc,sys,dev}
    
    # Copy essential binaries and libraries
    local essential_files=("/bin/bash" "/bin/sh" "/sbin/init" "/bin/ls" "/bin/cat" "/bin/mount")
    for file in "${essential_files[@]}"; do
        if [[ -f "$file" ]]; then
            cp -a "$file" "$live_dir$file" 2>/dev/null || true
        fi
    done
    
    # Create installer script
    cat > "$live_dir/usr/bin/multios-installer" << 'EOF'
#!/bin/bash
# MultiOS Installer (Live System)
echo "MultiOS Installer Starting..."
echo "Please select installation type:"
echo "1. Desktop PC"
echo "2. Mobile Device"
echo "3. IoT Device"
read -p "Choose option (1-3): " choice

case $choice in
    1) /opt/multios/desktop_installer.sh "$@" ;;
    2) /opt/multios/mobile_installer.sh "$@" ;;
    3) /opt/multios/iot_installer.sh "$@" ;;
    *) echo "Invalid choice"; exit 1 ;;
esac
EOF
    
    chmod +x "$live_dir/usr/bin/multios-installer"
    
    # Create squashfs if mksquashfs is available
    if command -v mksquashfs &> /dev/null; then
        log "Creating squashfs filesystem..."
        mksquashfs "$live_dir" "$iso_dir/live/filesystem.squashfs" -comp xz 2>/dev/null || \
        mksquashfs "$live_dir" "$iso_dir/live/filesystem.squashfs"
    else
        print_color $YELLOW "Warning: mksquashfs not found, skipping filesystem compression"
    fi
    
    # Create filesystem manifest
    find "$live_dir" -type f -exec echo {} \; > "$iso_dir/live/filesystem.manifest" 2>/dev/null || true
}

# Function to include packages
include_packages() {
    local iso_dir="$1"
    log "Including installation packages..."
    
    local packages_dir="$iso_dir/packages"
    
    # Copy custom packages if specified
    if [[ -f "$PACKAGES_LIST" ]]; then
        while IFS= read -r package; do
            if [[ -f "$package" ]]; then
                cp "$package" "$packages_dir/"
                log "Included package: $(basename "$package")"
            fi
        done < "$PACKAGES_LIST"
    fi
    
    # Include default packages
    local default_packages=()
    
    # Architecture-specific packages
    case $ARCH in
        "x86_64")
            default_packages+=("grub-efi-amd64-bin" "grub-pc-bin")
            ;;
        "arm64")
            default_packages+=("grub-efi-arm64-bin")
            ;;
        "riscv64")
            default_packages+=("opensbi" "u-boot-tools")
            ;;
    esac
    
    # Include rescue tools if requested
    if [[ "$RESCUE_TOOLS_INCLUDED" == "true" ]]; then
        default_packages+=("gparted" "testdisk" "photorec" "ddrescue")
    fi
    
    for package in "${default_packages[@]}"; do
        echo "Would include package: $package" >> "$packages_dir/package_list.txt"
    done
}

# Function to create ISO image
create_iso() {
    local iso_dir="$1"
    log "Creating ISO image..."
    
    # Use mkisofs or genisoimage
    local mkiso_cmd="mkisofs"
    if ! command -v "$mkiso_cmd" &> /dev/null; then
        mkiso_cmd="genisoimage"
    fi
    
    local iso_file="$OUTPUT_PATH"
    
    # Check if we have xorriso (preferred)
    if command -v xorriso &> /dev/null; then
        log "Using xorriso for ISO creation..."
        xorriso -as mkisofs -o "$iso_file" -J -R -V "$ISO_NAME" -boot-load-size 4 \
            -boot-info-table -input-hfs-charset utf-8 -output-hfs-charset utf-8 \
            -hfsplus -partition_offset 16 -efi-boot-part --efi-boot-image \
            -iso-level 3 -allow-lowercase -allow-multidot -no-emul-boot \
            -cache-inodes "$iso_dir"
    else
        log "Using $mkiso_cmd for ISO creation..."
        "$mkiso_cmd" -o "$iso_file" -J -R -V "$ISO_NAME" -boot-load-size 4 \
            -boot-info-table -input-hfs-charset utf-8 -output-hfs-charset utf-8 \
            -hfsplus -partition_offset 16 -iso-level 3 -no-emul-boot \
            -b isolinux/isolinux.bin -c isolinux/boot.cat "$iso_dir"
    fi
    
    # Verify ISO
    if [[ -f "$iso_file" ]]; then
        local iso_size=$(du -h "$iso_file" | cut -f1)
        print_color $GREEN "ISO created successfully: $iso_file ($iso_size)"
    else
        print_color $RED "Error: Failed to create ISO"
        exit 1
    fi
}

# Function to create bootable USB
create_bootable_usb() {
    local iso_file="$1"
    log "Creating bootable USB..."
    
    print_color $YELLOW "WARNING: This will erase all data on $TARGET_DEVICE"
    echo -n "Are you sure you want to continue? (yes/no): "
    read -r confirm
    
    if [[ "$confirm" != "yes" ]]; then
        print_color $YELLOW "USB creation cancelled"
        exit 0
    fi
    
    # Unmount any mounted partitions
    umount "${TARGET_DEVICE}"* 2>/dev/null || true
    
    # Create partition table
    log "Creating partition table..."
    parted -s "$TARGET_DEVICE" mklabel msdos
    parted -s "$TARGET_DEVICE" mkpart primary 0% 100%
    
    # Format as FAT32 for bootability
    local usb_part="${TARGET_DEVICE}1"
    mkfs.vfat -F32 -n "MULTIOS" "$usb_part"
    
    # Mount USB
    mkdir -p "$MOUNT_DIR"
    mount "$usb_part" "$MOUNT_DIR"
    
    # Copy ISO content to USB
    log "Copying installation files to USB..."
    cp -r "$WORK_DIR/iso/"* "$MOUNT_DIR/"
    
    # Make it bootable
    if command -v syslinux &> /dev/null; then
        syslinux -i "$usb_part" 2>/dev/null || true
    fi
    
    # Sync and unmount
    sync
    umount "$MOUNT_DIR"
    
    print_color $GREEN "Bootable USB created successfully: $TARGET_DEVICE"
}

# Function to cleanup
cleanup() {
    log "Cleaning up..."
    
    # Unmount if mounted
    if [[ -n "$MOUNT_DIR" ]] && mountpoint -q "$MOUNT_DIR" 2>/dev/null; then
        umount "$MOUNT_DIR" 2>/dev/null || true
    fi
    
    # Remove working directory
    if [[ -d "$WORK_DIR" ]]; then
        rm -rf "$WORK_DIR"
    fi
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

MultiOS Bootable Media Creation Tool

OPTIONS:
    --type TYPE              Media type: iso, usb (default: iso)
    --arch ARCH              Target architecture: x86_64, arm64, riscv64 (default: x86_64)
    --output FILE            Output file path
    --device DEVICE          Target USB device (required for --type usb)
    --name NAME              ISO name (default: $DEFAULT_ISO_NAME)
    --version VERSION        ISO version (default: $DEFAULT_ISO_VERSION)
    --packages FILE          File containing list of packages to include
    --memtest                Include MemTest86+
    --rescue                 Include rescue tools
    --efi                    Include EFI support (default: enabled)
    --grub                   Include GRUB bootloader (default: enabled)
    --netinstall             Create network installation media
    --non-interactive        Run in non-interactive mode
    --help                   Show this help message

EXAMPLES:
    $0 --type iso --output multios.iso
    $0 --type usb --device /dev/sdb --output multios-usb
    $0 --arch arm64 --output multios-arm64.iso --memtest --rescue
    $0 --output multios.iso --packages package_list.txt

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --type)
                MEDIA_TYPE="$2"
                shift 2
                ;;
            --arch)
                ARCH="$2"
                shift 2
                ;;
            --output)
                OUTPUT_PATH="$2"
                shift 2
                ;;
            --device)
                TARGET_DEVICE="$2"
                shift 2
                ;;
            --name)
                ISO_NAME="$2"
                shift 2
                ;;
            --version)
                ISO_VERSION="$2"
                shift 2
                ;;
            --packages)
                PACKAGES_LIST="$2"
                shift 2
                ;;
            --memtest)
                MEMTEST_INCLUDED=true
                shift
                ;;
            --rescue)
                RESCUE_TOOLS_INCLUDED=true
                shift
                ;;
            --efi)
                EFI_SUPPORT=true
                shift
                ;;
            --no-efi)
                EFI_SUPPORT=false
                shift
                ;;
            --grub)
                GRUB_SUPPORT=true
                shift
                ;;
            --no-grub)
                GRUB_SUPPORT=false
                shift
                ;;
            --netinstall)
                NETINSTALL=true
                shift
                ;;
            --non-interactive)
                INTERACTIVE_MODE=false
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_color $RED "Error: Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    print_color $CYAN "MultiOS Bootable Media Creation Tool"
    echo "======================================="
    
    # Check dependencies
    check_dependencies
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Set defaults based on type
    if [[ -z "$OUTPUT_PATH" ]]; then
        case $MEDIA_TYPE in
            "iso")
                OUTPUT_PATH="${ISO_NAME}-${ARCH}-${ISO_VERSION}.iso"
                ;;
            "usb")
                if [[ -z "$TARGET_DEVICE" ]]; then
                    print_color $RED "Error: --device is required for USB media creation"
                    show_usage
                    exit 1
                fi
                OUTPUT_PATH="${ISO_NAME}-${ARCH}-${ISO_VERSION}.img"
                ;;
        esac
    fi
    
    # Validate architecture
    if [[ ! " ${SUPPORTED_ARCHS[@]} " =~ " ${ARCH} " ]]; then
        print_color $RED "Error: Unsupported architecture: $ARCH"
        print_color $YELLOW "Supported architectures: ${SUPPORTED_ARCHS[*]}"
        exit 1
    fi
    
    # Interactive mode
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        echo
        print_color $YELLOW "Interactive Mode"
        echo
        
        if [[ -z "$OUTPUT_PATH" ]]; then
            echo -n "Enter output file path: "
            read -r OUTPUT_PATH
        fi
        
        if [[ "$MEDIA_TYPE" == "iso" ]] && [[ -z "$ISO_NAME" ]]; then
            echo -n "Enter ISO name [$DEFAULT_ISO_NAME]: "
            read -r ISO_NAME
            ISO_NAME=${ISO_NAME:-$DEFAULT_ISO_NAME}
        fi
        
        echo -n "Include MemTest? (y/N): "
        read -r mem_choice
        MEMTEST_INCLUDED=false
        if [[ $mem_choice =~ ^[Yy]$ ]]; then
            MEMTEST_INCLUDED=true
        fi
        
        echo -n "Include rescue tools? (y/N): "
        read -r rescue_choice
        RESCUE_TOOLS_INCLUDED=false
        if [[ $rescue_choice =~ ^[Yy]$ ]]; then
            RESCUE_TOOLS_INCLUDED=true
        fi
        
        echo
        echo "Media Creation Summary:"
        echo "  Type: $MEDIA_TYPE"
        echo "  Architecture: $ARCH"
        echo "  Output: $OUTPUT_PATH"
        echo "  MemTest: $MEMTEST_INCLUDED"
        echo "  Rescue Tools: $RESCUE_TOOLS_INCLUDED"
        echo
        echo -n "Proceed with media creation? (y/N): "
        read -r confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            print_color $YELLOW "Media creation cancelled"
            exit 0
        fi
    fi
    
    # Check device access for USB
    check_device_access
    
    # Set cleanup trap
    trap cleanup EXIT
    
    # Create working directory
    create_working_directory
    
    # Create directory structure
    ISO_DIR=$(create_directory_structure)
    
    # Copy kernel and initrd
    copy_kernel_initrd "$ISO_DIR"
    
    # Create bootloaders
    if [[ "$GRUB_SUPPORT" == "true" ]]; then
        create_bootloaders "$ISO_DIR"
    fi
    
    # Create live filesystem
    create_live_filesystem "$ISO_DIR"
    
    # Include packages
    include_packages "$ISO_DIR"
    
    # Create media
    case $MEDIA_TYPE in
        "iso")
            create_iso "$ISO_DIR"
            ;;
        "usb")
            create_bootable_usb "$OUTPUT_PATH"
            ;;
        *)
            print_color $RED "Error: Unknown media type: $MEDIA_TYPE"
            exit 1
            ;;
    esac
    
    log "Media creation completed successfully"
}

# Run main function
main "$@"