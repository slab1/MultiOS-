# MultiOS Installation Guide

This comprehensive guide covers all aspects of installing MultiOS on various hardware platforms and configurations.

## Table of Contents

1. [Pre-Installation](#pre-installation)
2. [System Requirements](#system-requirements)
3. [Installation Methods](#installation-methods)
4. [Step-by-Step Installation](#step-by-step-installation)
5. [Advanced Installation Options](#advanced-installation-options)
6. [Post-Installation Configuration](#post-installation-configuration)
7. [Troubleshooting](#troubleshooting)
8. [Uninstallation](#uninstallation)

## Pre-Installation

### Planning Your Installation

**Backup Existing Data**: Before installing MultiOS, backup all important data from your system.

**Choose Installation Type**:
- **Single-boot**: MultiOS as the only operating system
- **Multi-boot**: MultiOS alongside existing OS (Windows, Linux, macOS)
- **Virtual Machine**: Install in a VM for testing
- **Dual-boot with existing OS**: Share disk space

**Select Target Architecture**:
- **x86_64**: Intel/AMD processors (most common)
- **ARM64 (AArch64)**: ARM 64-bit processors (modern mobile/desktop)
- **RISC-V64**: RISC-V 64-bit processors (emerging architecture)

### Download MultiOS

#### Option 1: Official Downloads
Visit the [MultiOS Downloads Page](https://releases.multios.org/) for the latest release.

```bash
# Direct download links (replace with latest version)
# x86_64 Desktop Edition
wget https://releases.multios.org/v1.0/x86_64/multios-desktop-x86_64-v1.0.iso

# ARM64 Desktop Edition
wget https://releases.multios.org/v1.0/arm64/multios-desktop-arm64-v1.0.iso

# RISC-V64 Edition
wget https://releases.multios.org/v1.0/riscv64/multios-desktop-riscv64-v1.0.iso
```

#### Option 2: Direct Downloads by Edition

**Desktop Edition** (3.2 GB)
```bash
# x86_64
wget https://releases.multios.org/v1.0/x86_64/multios-desktop-x86_64-v1.0.iso

# ARM64
wget https://releases.multios.org/v1.0/arm64/multios-desktop-arm64-v1.0.iso
```

**Server Edition** (2.1 GB)
```bash
# Minimal installation for servers
wget https://releases.multios.org/v1.0/x86_64/multios-server-x86_64-v1.0.iso
```

**Development Edition** (4.5 GB)
```bash
# Includes source code and build tools
wget https://releases.multios.org/v1.0/x86_64/multios-dev-x86_64-v1.0.iso
```

**Educational Edition** (3.8 GB)
```bash
# Includes tutorials and learning materials
wget https://releases.multios.org/v1.0/x86_64/multios-edu-x86_64-v1.0.iso
```

### Verify Download Integrity

```bash
# Download SHA256 checksums
wget https://releases.multios.org/v1.0/SHA256SUMS

# Verify integrity
sha256sum -c SHA256SUMS
```

Example output:
```
multios-desktop-x86_64-v1.0.iso: OK
multios-desktop-arm64-v1.0.iso: OK
```

## System Requirements

### Minimum Requirements

| Component | Desktop Edition | Server Edition | Development Edition |
|-----------|----------------|----------------|-------------------|
| **CPU** | 64-bit x86_64/ARM64/RISC-V | 64-bit x86_64/ARM64/RISC-V | 64-bit x86_64/ARM64/RISC-V |
| **Memory** | 512 MB RAM | 256 MB RAM | 1 GB RAM |
| **Storage** | 2 GB available | 1 GB available | 5 GB available |
| **Graphics** | VGA-compatible | Optional | VGA-compatible |
| **Network** | Optional | Ethernet | Ethernet/WiFi |

### Recommended Requirements

| Component | Desktop Edition | Server Edition | Development Edition |
|-----------|----------------|----------------|-------------------|
| **CPU** | Dual-core 64-bit | Quad-core 64-bit | Quad-core 64-bit |
| **Memory** | 2 GB RAM | 1 GB RAM | 4 GB RAM |
| **Storage** | 20 GB available | 10 GB available | 50 GB available |
| **Graphics** | Modern GPU | Optional | Modern GPU |
| **Network** | Ethernet/WiFi | Ethernet | Ethernet/WiFi |

### Hardware Compatibility

**Fully Supported Hardware**:
- ✅ Intel/AMD x86_64 processors (all generations)
- ✅ ARM Cortex-A53/A57/A72/A73 processors
- ✅ RISC-V 64-bit processors (SiFive, StarFive)
- ✅ SATA/NVMe storage controllers
- ✅ USB 3.0/2.0 devices
- ✅ Intel/Realtek/AMD network cards
- ✅ Intel HDA/AC'97/USB audio
- ✅ VGA/HDMI/DisplayPort displays

**Partially Supported**:
- 🔶 Some ARM GPUs (Mali, Adreno)
- 🔶 Wireless networking (limited drivers)
- 🔶 Bluetooth devices (basic support)
- 🔶 Touchscreen input (tablet mode)

**Not Yet Supported**:
- ❌ Proprietary GPUs (NVIDIA proprietary drivers)
- ❌ Some network adapters (Limited Linux support)
- ❌ Thunderbolt devices
- ❌ Apple-specific hardware

## Installation Methods

### Method 1: USB Installation (Recommended)

#### Creating Bootable USB Drive

**Linux/macOS**:
```bash
# Identify your USB device (WARNING: be very careful!)
lsblk
# Output will show devices like /dev/sdb, /dev/sdc, etc.

# Create bootable USB (replace /dev/sdX with your device)
sudo dd if=multios-desktop-x86_64-v1.0.iso of=/dev/sdX bs=4M status=progress
sync
```

**Windows**:
```bash
# Using Rufus (GUI tool)
# 1. Download from https://rufus.ie/
# 2. Select ISO and USB device
# 3. Choose "Write in ISO Image mode"
# 4. Click START

# Using command line
# Install PowerISO or use built-in tools
```

**macOS**:
```bash
# Using diskutil to identify USB device
diskutil list

# Create bootable USB
sudo dd if=multios-desktop-x86_64-v1.0.iso of=/dev/rdiskN bs=4m
```

#### USB Requirements
- **Minimum size**: 4 GB
- **Recommended size**: 8 GB or larger
- **Speed**: USB 3.0 recommended
- **File system**: Will be reformatted during creation

### Method 2: DVD Installation

For systems without USB support:

```bash
# Burn ISO to DVD
# Linux
sudo Brasero multios-desktop-x86_64-v1.0.iso

# macOS
# Use Disk Utility or any DVD burning software

# Windows
# Use Windows Media Creation Tool or Nero
```

### Method 3: Network Installation (PXE)

For enterprise deployments:

```bash
# Setup PXE server (requires TFTP server)
# Configure DHCP to point to TFTP server
# Place MultiOS images in TFTP directory
# Boot client and select network installation
```

### Method 4: Virtual Machine Installation

#### Using QEMU

```bash
# Install QEMU
# Ubuntu/Debian
sudo apt-get install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64

# Create virtual machine
qemu-system-x86_64 \
  -m 1024 \
  -cdrom multios-desktop-x86_64-v1.0.iso \
  -boot d \
  -hda multios-disk.qcow2 \
  -netdev user,id=net0 \
  -device e1000,netdev=net0 \
  -display gtk
```

#### Using VirtualBox

1. Download and install VirtualBox
2. Create new VM:
   - **Name**: MultiOS
   - **Type**: Linux
   - **Version**: Other Linux (64-bit)
   - **Memory**: 1024 MB
   - **Hard Disk**: 20 GB (dynamically allocated)
3. Attach ISO to virtual optical drive
4. Start VM and install

#### Using VMware

```bash
# VMware Workstation/Player configuration
vmware multios-installer.vmx
# Configure memory, CPU cores, and disk size
```

## Step-by-Step Installation

### Boot from Installation Media

1. **Insert USB/DVD** into system
2. **Restart computer**
3. **Enter BIOS/UEFI settings**:
   - Common keys: F2, F12, DEL, ESC, F10
   - Look for "Boot Menu" option
4. **Select boot device**: USB or DVD drive
5. **MultiOS Boot Menu appears**

### Installation Menu Options

**MultiOS (Standard Install)**
- Complete installation with GUI
- Recommended for desktop use

**MultiOS (Safe Mode)**
- Minimal installation with basic drivers
- Use if standard installation fails

**MultiOS (Debug Mode)**
- Installation with verbose logging
- For troubleshooting installation issues

**Memory Test**
- Run memory diagnostic (MemTest86)
- Check for hardware issues

**Boot from Hard Drive**
- Boot existing installation (if any)

### Installation Wizard

#### Step 1: Language and Region

```bash
Select Language: English (US)
Time Zone: [Your Timezone]
Keyboard Layout: [Your Keyboard]
```

#### Step 2: Installation Type

**Erase disk and install MultiOS**
- Complete disk formatting
- All existing data lost
- Clean installation

**Install alongside other operating systems**
- Automatic disk partitioning
- Creates dual-boot setup
- Preserves existing OS

**Something else (Manual partitioning)**
- Custom partition configuration
- Advanced users only
- Full control over disk layout

**Erase and use entire disk**
- Simple full-disk installation
- Recommended for new systems

#### Step 3: User Account Setup

```bash
Full Name: [Your Name]
Username: [yourname]
Password: [secure password]
Confirm Password: [same password]
Require password on boot: Yes/No
```

#### Step 4: Network Configuration

**DHCP (Automatic)**:
- Automatically configure network
- Recommended for most users

**Manual Configuration**:
- Specify IP address manually
- Enter network settings:
  ```bash
  IP Address: 192.168.1.100
  Netmask: 255.255.255.0
  Gateway: 192.168.1.1
  DNS: 8.8.8.8
  ```

#### Step 5: Software Selection

**Desktop Environment**:
- **MultiOS GUI**: Native Rust-based interface
- **Command Line Only**: CLI only (minimal resources)

**Pre-installed Software**:
- Development tools
- Network utilities
- Educational packages
- Media codecs

#### Step 6: Installation

**Copying files...**
- Installation progress bar
- ETA display
- Copy speed indicator

**Installing bootloader**:
- Configured automatically
- Multi-boot support added

**Configuration**:
- System services setup
- User account creation
- Network configuration
- Display settings

#### Step 7: Completion

**Installation Complete!**
- Reboot prompt
- Remove installation media
- First boot setup

```bash
Installation Summary:
- Installation Size: 2.1 GB
- Installation Time: 15 minutes
- Bootloader: MultiOS GRUB
- Partition: /dev/sda1 (ext4)
```

## Advanced Installation Options

### Multi-Boot Setup

#### Windows + MultiOS

1. **Install Windows first** (if not already installed)
2. **Resize Windows partition**:
   ```bash
   # Using Windows Disk Management
   # Right-click C: drive → Shrink Volume
   # Reserve at least 20 GB for MultiOS
   ```
3. **Install MultiOS** in remaining space
4. **Bootloader automatically detects Windows**

#### Linux + MultiOS

```bash
# Backup existing GRUB configuration
sudo cp /boot/grub/grub.cfg /boot/grub/grub.cfg.backup

# Install MultiOS (will update GRUB automatically)
# MultiOS installer detects existing Linux installation
# Adds MultiOS to boot menu

# Manual GRUB update (if needed)
sudo update-grub
```

### Encrypted Installation

**Full Disk Encryption**:
```bash
# Enable encryption during installation
Encryption Method: LUKS
Encryption Password: [strong password]
Confirm Password: [same password]

# Encrypted partitions:
# - /boot (encrypted)
# - / (root, encrypted)
# - /home (encrypted, optional)
```

**Manual LUKS Setup**:
```bash
# Create encrypted container
sudo cryptsetup luksFormat /dev/sda2
sudo cryptsetup luksOpen /dev/sda2 multios-root

# Create filesystem
sudo mkfs.ext4 /dev/mapper/multios-root
sudo mount /dev/mapper/multios-root /mnt

# Install MultiOS to /mnt
```

### LVM Configuration

**Logical Volume Manager**:
```bash
# Manual LVM setup
sudo pvcreate /dev/sda2
sudo vgcreate multios-vg /dev/sda2
sudo lvcreate -L 10G -n root multios-vg
sudo lvcreate -L 2G -n swap multios-vg
sudo lvcreate -l 100%FREE -n home multios-vg

# Create filesystems
sudo mkfs.ext4 /dev/mapper/multios-vg-root
sudo mkswap /dev/mapper/multios-vg-swap
sudo mkfs.ext4 /dev/mapper/multios-vg-home
```

### Server Installation

**Headless Installation**:
```bash
# Command-line installer
multios-installer-cli \
  --target /dev/sda \
  --edition server \
  --network dhcp \
  --timezone UTC \
  --user admin:password \
  --no-gui \
  --interactive false
```

**Automated Installation**:
```bash
# Use preseed file
multios-installer \
  --preseed multios-server.preseed \
  --target /dev/sda
```

**Preseed File Example**:
```bash
# multios-server.preseed
d-i partman-auto/method string lvm
d-i partman-auto-lvm/guided_size string max
d-i partman-lvm/confirm boolean true
d-i partman-lvm/confirm_nooverwrite boolean true
d-i partman-partitioning/confirm_write_new_label boolean true
d-i passwd/root-login boolean true
d-i passwd/root-password password rootpassword
d-i passwd/root-password-again password rootpassword
d-i passwd/user-fullname string MultiOS User
d-i passwd/username string user
d-i passwd/user-password password userpassword
d-i passwd/user-password-again password userpassword
```

### ARM Installation

**Raspberry Pi Installation**:

1. **Prepare SD card**:
   ```bash
   # Flash image to SD card
   sudo dd if=multios-arm64-raspberry-pi.img of=/dev/mmcblk0 bs=4M status=progress
   
   # Mount and configure
   sudo mount /dev/mmcblk0p1 /mnt
   echo "console=serial0,115200 root=/dev/mmcblk0p2 rootwait" >> /mnt/cmdline.txt
   sudo umount /mnt
   ```

2. **Boot Raspberry Pi** with SD card

**ARM64 Server Installation**:
```bash
# Install on ARM64 server
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a57 \
  -m 1024 \
  -cdrom multios-server-arm64-v1.0.iso \
  -boot d \
  -nographic
```

### Network Installation

**PXE Boot Setup**:

1. **Configure DHCP server**:
   ```bash
   # /etc/dhcp/dhcpd.conf
   subnet 192.168.1.0 netmask 255.255.255.0 {
       range 192.168.1.100 192.168.1.200;
       next-server 192.168.1.50;
       filename "pxelinux.0";
   }
   ```

2. **Configure TFTP server**:
   ```bash
   # Place MultiOS images in TFTP directory
   /var/lib/tftpboot/multios/
   ├── initrd.img
   ├── vmlinuz
   └── installer.iso
   ```

3. **Client boots and downloads installer**

## Post-Installation Configuration

### First Boot Setup

**Initial Configuration Wizard**:
```bash
Welcome to MultiOS Setup!

1. Language: [English]
2. Time Zone: [Your Timezone]
3. Keyboard Layout: [US]
4. Network: [DHCP]
5. User Account: [Create]
6. Services: [Enable recommended]
7. Updates: [Check for updates]
```

### Update System

```bash
# Update package database
sudo multios-update

# Upgrade system
sudo multios-upgrade

# Check for new releases
sudo multios-release-check
```

### Install Additional Software

```bash
# Browse available packages
multios-pkg search [package-name]

# Install package
sudo multios-pkg install [package-name]

# Remove package
sudo multios-pkg remove [package-name]

# Update package cache
sudo multios-pkg update
```

### Configure Services

```bash
# Enable service
sudo systemctl enable service-name

# Start service
sudo systemctl start service-name

# Check status
sudo systemctl status service-name

# Disable service
sudo systemctl disable service-name
```

### Network Configuration

**Static IP Configuration**:
```bash
# Edit network configuration
sudo nano /etc/network/interfaces

# Example configuration:
# auto eth0
# iface eth0 inet static
# address 192.168.1.100
# netmask 255.255.255.0
# gateway 192.168.1.1
# dns-nameservers 8.8.8.8 8.8.4.4

# Apply changes
sudo systemctl restart networking
```

**WiFi Configuration**:
```bash
# Scan for networks
sudo iwlist scan

# Connect to network
sudo wpa_passphrase "SSID" "password" | sudo tee /etc/wpa_supplicant.conf
sudo wpa_supplicant -B -i wlan0 -c /etc/wpa_supplicant.conf
sudo dhclient wlan0
```

### Graphics Configuration

**Display Settings**:
```bash
# Open display settings
multios-display-settings

# Available resolutions
xrandr

# Set resolution
xrandr --output HDMI-1 --mode 1920x1080
```

**Multiple Monitors**:
```bash
# Configure dual monitor setup
multios-dual-monitor-setup

# Manual configuration
xrandr --output HDMI-1 --primary --mode 1920x1080
xrandr --output HDMI-2 --right-of HDMI-1 --mode 1920x1080
```

### Sound Configuration

```bash
# Audio settings
multios-audio-settings

# Test speakers
speaker-test -t sine -f 1000 -l 1

# Test microphone
arecord -d 5 test.wav && aplay test.wav
```

## Troubleshooting

### Boot Issues

**"No bootable device" error**:
```bash
# Check boot order in BIOS
# Ensure USB is first in boot priority
# Try different USB port
# Disable Secure Boot if needed
```

**"Kernel panic - not syncing"**:
```bash
# Try Safe Mode boot
# Check hardware compatibility
# Verify ISO integrity
# Try different RAM configuration
```

**Black screen after boot**:
```bash
# Add boot parameters
# Press 'e' at boot menu
# Add: nomodeset
# Press F10 to boot
```

### Installation Failures

**"Installation failed"**:
```bash
# Check disk space
df -h

# Verify ISO integrity
sha256sum multios-desktop-x86_64-v1.0.iso

# Check disk for errors
sudo fsck /dev/sda

# Try different USB drive
# Use USB 2.0 if USB 3.0 fails
```

**Partitioning issues**:
```bash
# Clear partition table
sudo dd if=/dev/zero of=/dev/sda bs=1M count=1

# Use GParted to create partitions manually
sudo apt-get install gparted
sudo gparted /dev/sda
```

### Hardware Detection

**Hardware not detected**:
```bash
# Check PCI devices
lspci

# Check USB devices
lsusb

# Check hardware info
sudo lshw

# Load kernel modules manually
sudo modprobe module-name
```

**Network not working**:
```bash
# Check network interfaces
ip link show

# Restart network service
sudo systemctl restart networking

# Check driver status
sudo ethtool eth0
```

### Performance Issues

**Slow performance**:
```bash
# Check system resources
top
htop

# Monitor disk I/O
iotop

# Check for memory issues
free -h

# Disable unnecessary services
sudo systemctl disable service-name
```

**High CPU usage**:
```bash
# Find CPU-intensive processes
top
ps aux --sort=-%cpu

# Check for runaway processes
sudo kill -9 PID
```

### Recovery

**Single User Mode**:
```bash
# Boot into single user mode
# Add 'single' to kernel parameters
# Access root shell without password
```

**Rescue Mode**:
```bash
# Boot from installation media
# Select "Rescue Mode"
# Mount existing installation
# Perform repairs
```

**Recovery Commands**:
```bash
# Reset root password
sudo passwd root

# Fix bootloader
sudo multios-bootloader-repair

# Restore GRUB
sudo grub-install /dev/sda
sudo update-grub
```

## Uninstallation

### Removing MultiOS

**Complete Removal**:
```bash
# Boot from other OS
# Delete MultiOS partitions using GParted
# Update bootloader
sudo update-grub
```

**Selective Removal** (keep data):
```bash
# Mount MultiOS partition
sudo mkdir /mnt/multios
sudo mount /dev/sda2 /mnt/multios

# Backup user data
sudo cp -r /mnt/multios/home/* /backup/

# Remove MultiOS installation
sudo rm -rf /mnt/multios
```

### Bootloader Cleanup

**Windows Bootloader**:
```cmd
# In Windows, fix MBR
bootsect /nt60 SYS /mbr

# Or use bcdedit
bcdedit /delete {multios-guid}
```

**Linux GRUB Restoration**:
```bash
# Boot from Linux live USB
sudo mount /dev/sda1 /mnt
sudo grub-install --boot-directory=/mnt/boot /dev/sda
sudo update-grub
```

## Advanced Topics

### Custom Installations

**Embedded Systems**:
```bash
# Minimal installation for embedded
multios-installer --target /dev/mmcblk0 \
  --edition minimal \
  --no-gui \
  --console-only
```

**Cloud/Server Deployment**:
```bash
# Cloud image creation
multios-image-builder \
  --type cloud \
  --size 10G \
  --output multios-cloud-x86_64.qcow2
```

### Customization

**Theme Customization**:
```bash
# Install themes
multios-pkg install multios-themes

# Apply theme
multios-theme-apply dark-modern

# Custom theme creation
multios-theme-create my-theme
```

**Language Support**:
```bash
# Install language packs
multios-pkg install multios-locale-[lang-code]

# Set system language
sudo dpkg-reconfigure locales
```

### Maintenance

**Regular Maintenance Tasks**:
```bash
# Weekly system check
sudo multios-system-check

# Clean package cache
sudo multios-pkg clean

# Check disk health
sudo smartctl -a /dev/sda

# Update bootloader
sudo update-grub
```

**Backup Creation**:
```bash
# Create system backup
sudo multios-backup-create --target /dev/sda1

# Backup configuration
tar -czf multios-config-backup.tar.gz /etc/multios/
```

---

This installation guide covers all aspects of MultiOS installation. For specific hardware issues or advanced configurations, consult the [Troubleshooting Guide](troubleshooting.md) or visit our [Community Forum](https://community.multios.org).