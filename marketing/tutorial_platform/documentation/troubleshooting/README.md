# MultiOS Troubleshooting Guide

## Quick Reference

### Common Issues by Category

- [Installation Problems](#installation-problems)
- [Boot Issues](#boot-issues)
- [Performance Issues](#performance-issues)
- [Network Problems](#network-problems)
- [Hardware Compatibility](#hardware-compatibility)
- [Software Issues](#software-issues)
- [Development Problems](#development-problems)

## Installation Problems

### Issue: Installation Freezes During Boot

**Symptoms:**
- System freezes at boot logo
- Installation stops responding
- No progress after boot menu

**Solutions:**

1. **Check Hardware Compatibility**
   ```bash
   # Verify system requirements
   lscpu | head -20
   free -h
   lspci | grep -E "VGA|Audio|Network"
   ```

2. **Try Safe Graphics Mode**
   - At boot menu, press `e`
   - Add `nomodeset` to kernel parameters
   - Press `F10` to boot

3. **Disable Hardware Acceleration**
   - Try installation in virtual machine
   - Use software rendering mode
   - Disable GPU acceleration in BIOS

4. **Memory Issues**
   ```bash
   # Check memory with memtest
   # Boot with memtest option
   # Replace faulty RAM if needed
   ```

### Issue: USB Installation Fails

**Symptoms:**
- "Missing operating system" error
- Boot from USB doesn't work
- Installation media not recognized

**Solutions:**

1. **Verify USB Creation**
   ```bash
   # On Linux/Mac
   sudo dd if=multios.iso of=/dev/sdX bs=4M status=progress
   
   # Verify checksum
   sha256sum multios.iso
   ```

2. **Check BIOS Settings**
   - Enable USB boot support
   - Set USB as first boot device
   - Disable Secure Boot
   - Enable Legacy Boot mode

3. **Try Different USB Port**
   - Use USB 2.0 port if issues with USB 3.0
   - Try different USB drive
   - Check USB drive integrity

4. **Format USB Drive**
   ```bash
   # Format as FAT32
   sudo mkfs.vfat -F 32 /dev/sdX1
   
   # Create bootable USB using Rufus (Windows)
   # or dd command (Linux/Mac)
   ```

### Issue: Partitioning Errors

**Symptoms:**
- "No space left on device" error
- Partition table errors
- Installation fails at disk formatting

**Solutions:**

1. **Clear Existing Partitions**
   ```bash
   # Use GParted or fdisk
   sudo fdisk /dev/sdX
   # Delete existing partitions
   # Create new partition table
   ```

2. **Check Disk Space**
   ```bash
   # Verify available space
   df -h
   sudo fdisk -l /dev/sdX
   ```

3. **Disable Automatic Mounting**
   ```bash
   # Unmount all partitions
   sudo umount /dev/sdX*
   ```

4. **Check Disk Health**
   ```bash
   # Check disk for errors
   sudo smartctl -a /dev/sdX
   ```

## Boot Issues

### Issue: System Won't Boot After Installation

**Symptoms:**
- Black screen on boot
- GRUB error messages
- System returns to BIOS

**Solutions:**

1. **Repair Boot Loader**
   ```bash
   # Boot from installation media
   # Select "Try MultiOS"
   # Open terminal
   
   # Mount the installed system
   sudo mount /dev/sdX1 /mnt
   sudo mount --bind /dev /mnt/dev
   sudo mount --bind /proc /mnt/proc
   sudo mount --bind /sys /mnt/sys
   
   # Chroot to system
   sudo chroot /mnt
   
   # Reinstall GRUB
   grub-install /dev/sdX
   update-grub
   
   # Exit and reboot
   exit
   sudo reboot
   ```

2. **Check Kernel Parameters**
   ```bash
   # Edit GRUB configuration
   sudo nano /etc/default/grub
   
   # Add or modify parameters
   GRUB_CMDLINE_LINUX="rootfstype=ext4"
   
   # Update GRUB
   sudo update-grub
   ```

3. **Verify Hardware Compatibility**
   ```bash
   # Check hardware
   dmesg | grep -i error
   lspci
   lsusb
   ```

### Issue: GRUB Error Messages

**Common Error Messages:**

#### Error: "No such partition"
```bash
# Fix UUID mismatch
sudo blkid
sudo nano /etc/fstab

# Update UUIDs in fstab
```

#### Error: "File not found"
```bash
# Regenerate GRUB configuration
sudo grub-mkconfig -o /boot/grub/grub.cfg

# Check file system
sudo fsck /dev/sdX1
```

#### Error: "Out of disk"
```bash
# Check disk geometry
sudo fdisk -l /dev/sdX

# Reinstall GRUB to MBR
sudo grub-install --recheck /dev/sdX
```

### Issue: Slow Boot Process

**Symptoms:**
- Boot takes several minutes
- Long delay during initialization

**Solutions:**

1. **Identify Boot Bottlenecks**
   ```bash
   # Analyze boot time
   systemd-analyze
   
   # Check individual services
   systemd-analyze blame
   ```

2. **Disable Unnecessary Services**
   ```bash
   # List enabled services
   systemctl list-unit-files --state=enabled
   
   # Disable unwanted services
   sudo systemctl disable service-name
   ```

3. **Optimize File System**
   ```bash
   # Check disk for errors
   sudo fsck /dev/sdX1
   
   # Optimize ext4 file system
   sudo tune2fs -o journal_data_writeback /dev/sdX1
   ```

4. **Update System**
   ```bash
   # Update all packages
   sudo pkg update
   sudo pkg upgrade
   ```

## Performance Issues

### Issue: System Runs Slowly

**Symptoms:**
- Applications take long time to start
- High CPU usage
- Slow response times

**Solutions:**

1. **Check System Resources**
   ```bash
   # Monitor CPU and memory usage
   top
   htop
   
   # Check disk usage
   df -h
   du -sh /* | sort -hr
   
   # Check I/O performance
   iotop
   ```

2. **Identify Resource Hogs**
   ```bash
   # Find processes using most CPU
   ps aux --sort=-%cpu | head
   
   # Find processes using most memory
   ps aux --sort=-%mem | head
   ```

3. **Optimize Startup**
   ```bash
   # Check startup applications
   gnome-session-properties
   
   # Disable unnecessary startup apps
   ```

4. **Update System**
   ```bash
   # Update all software
   sudo pkg update
   sudo pkg upgrade
   
   # Update firmware
   sudo fwupdmgr update
   ```

### Issue: High Memory Usage

**Symptoms:**
- System runs out of memory
- Frequent swapping
- Applications crash

**Solutions:**

1. **Identify Memory Usage**
   ```bash
   # Check memory usage
   free -h
   cat /proc/meminfo
   
   # Find memory leaks
   valgrind --tool=memcheck ./program
   ```

2. **Optimize Applications**
   ```bash
   # Check for memory-intensive applications
   ps aux --sort=-%mem | head
   ```

3. **Increase Swap Space**
   ```bash
   # Create swap file
   sudo fallocate -l 2G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   
   # Make permanent
   echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
   ```

### Issue: Disk Space Running Low

**Symptoms:**
- "No space left on device" errors
- System becomes unresponsive

**Solutions:**

1. **Check Disk Usage**
   ```bash
   # Find largest directories
   sudo du -sh /* | sort -hr
   
   # Find largest files
   find / -type f -exec du -h {} + | sort -hr | head
   ```

2. **Clean System**
   ```bash
   # Remove old packages
   sudo pkg autoremove
   
   # Clean package cache
   sudo pkg clean
   
   # Remove temporary files
   sudo rm -rf /tmp/*
   sudo rm -rf ~/.cache/*
   ```

3. **Log Rotation**
   ```bash
   # Clean old log files
   sudo journalctl --vacuum-time=7d
   
   # Limit log size
   sudo journalctl --vacuum-size=100M
   ```

## Network Problems

### Issue: Cannot Connect to WiFi

**Symptoms:**
- No WiFi networks detected
- Connection fails with authentication errors

**Solutions:**

1. **Check Hardware**
   ```bash
   # Verify wireless interface
   ip link show
   
   # Check for errors
   dmesg | grep -i wifi
   ```

2. **Restart Network Service**
   ```bash
   # Restart network manager
   sudo systemctl restart networkmanager
   
   # Or using nmcli
   nmcli radio wifi off
   nmcli radio wifi on
   ```

3. **Update Drivers**
   ```bash
   # Check for driver updates
   sudo pkg update
   
   # Install WiFi drivers if needed
   sudo pkg install wireless-tools
   ```

4. **Manual Configuration**
   ```bash
   # Connect using command line
   nmcli device wifi connect "SSID" password "password"
   ```

### Issue: Slow Network Performance

**Symptoms:**
- Slow download/upload speeds
- High latency
- Frequent disconnections

**Solutions:**

1. **Check Network Configuration**
   ```bash
   # Check connection speed
   ethtool eth0
   
   # Test network speed
   iperf3 -c server.example.com
   ```

2. **Update Network Drivers**
   ```bash
   # Update network drivers
   sudo pkg upgrade
   
   # Install specific drivers if needed
   sudo pkg install network-driver-name
   ```

3. **Optimize Network Settings**
   ```bash
   # Check network configuration
   cat /etc/network/interfaces
   
   # Optimize TCP settings
   echo 'net.core.rmem_default = 262144' | sudo tee -a /etc/sysctl.conf
   ```

### Issue: DNS Resolution Problems

**Symptoms:**
- Cannot resolve domain names
- Internet works but specific sites fail

**Solutions:**

1. **Check DNS Configuration**
   ```bash
   # Check DNS servers
   cat /etc/resolv.conf
   
   # Test DNS resolution
   nslookup google.com
   ```

2. **Update DNS Servers**
   ```bash
   # Use Google DNS
   echo 'nameserver 8.8.8.8' | sudo tee /etc/resolv.conf
   
   # Or Cloudflare DNS
   echo 'nameserver 1.1.1.1' | sudo tee /etc/resolv.conf
   ```

3. **Flush DNS Cache**
   ```bash
   # Flush DNS cache
   sudo systemctl restart systemd-resolved
   
   # Or clear nscd cache
   sudo nscd -i hosts
   ```

## Hardware Compatibility

### Issue: Graphics Display Problems

**Symptoms:**
- Low resolution display
- Screen flickering
- Graphics driver errors

**Solutions:**

1. **Identify Graphics Hardware**
   ```bash
   # Check graphics card
   lspci | grep VGA
   
   # Check current driver
   glxinfo | grep "OpenGL renderer"
   ```

2. **Install Graphics Drivers**
   ```bash
   # Install recommended drivers
   sudo pkg install multios-graphics-driver
   
   # For NVIDIA
   sudo pkg install nvidia-driver
   
   # For AMD
   sudo pkg install mesa-amdgpu
   ```

3. **Configure X11**
   ```bash
   # Reconfigure X server
   sudo dpkg-reconfigure xserver-xorg
   
   # Create xorg.conf if needed
   sudo X -configure
   ```

### Issue: Audio Problems

**Symptoms:**
- No sound output
- Audio device not detected
- Poor audio quality

**Solutions:**

1. **Check Audio Hardware**
   ```bash
   # List audio devices
   aplay -l
   arecord -l
   
   # Check pulse audio
   pactl list cards
   ```

2. **Restart Audio Service**
   ```bash
   # Restart pulse audio
   pulseaudio -k
   pulseaudio --start
   
   # Or restart alsa
   sudo alsa force-reload
   ```

3. **Install Audio Drivers**
   ```bash
   # Install audio packages
   sudo pkg install alsa-utils pulseaudio
   ```

### Issue: USB Device Problems

**Symptoms:**
- USB devices not recognized
- Device disconnects randomly
- Slow USB performance

**Solutions:**

1. **Check USB Subsystem**
   ```bash
   # List USB devices
   lsusb
   
   # Check USB errors
   dmesg | grep -i usb
   ```

2. **Test Different Ports**
   ```bash
   # Try different USB ports
   # USB 2.0 ports often more stable
   # Avoid USB hubs initially
   ```

3. **Update USB Drivers**
   ```bash
   # Update system
   sudo pkg upgrade
   
   # Install USB utilities
   sudo pkg install usbutils
   ```

## Software Issues

### Issue: Application Crashes

**Symptoms:**
- Application exits unexpectedly
- Segmentation faults
- Application freezes

**Solutions:**

1. **Check Application Logs**
   ```bash
   # Check system logs
   journalctl -xe
   
   # Check application logs
   tail -f ~/.config/app/logs
   ```

2. **Run in Safe Mode**
   ```bash
   # Run with safe settings
   app --safe-mode
   
   # Reset application settings
   rm -rf ~/.config/app
   ```

3. **Update Application**
   ```bash
   # Update the application
   sudo pkg update app-name
   sudo pkg upgrade app-name
   ```

### Issue: Package Manager Problems

**Symptoms:**
- Package installation fails
- Repository errors
- Dependency conflicts

**Solutions:**

1. **Fix Repository Issues**
   ```bash
   # Update package lists
   sudo pkg update
   
   # Fix repository signatures
   sudo pkg install archlinux-keyring
   ```

2. **Handle Dependency Conflicts**
   ```bash
   # Remove conflicting packages
   sudo pkg remove conflicting-package
   
   # Force reinstall
   sudo pkg install --force package-name
   ```

3. **Clear Package Cache**
   ```bash
   # Clean package cache
   sudo pkg clean
   
   # Or use pacman cache cleaner
   sudo pkg-cache-clean
   ```

### Issue: Update Failures

**Symptoms:**
- System updates fail
- Broken packages
- System in inconsistent state

**Solutions:**

1. **Fix Broken Packages**
   ```bash
   # Check for broken packages
   sudo pkg-check
   
   # Fix broken dependencies
   sudo pkg -D --asdeps package
   ```

2. **Reinstall Core Packages**
   ```bash
   # Reinstall base system
   sudo pkg -S base base-devel
   
   # Reinstall bootloader
   sudo pkg -S grub
   ```

## Development Problems

### Issue: Build Failures

**Symptoms:**
- Compilation errors
- Missing dependencies
- Linker failures

**Solutions:**

1. **Check Dependencies**
   ```bash
   # Install build dependencies
   sudo pkg -S base-devel cmake
   
   # Install MultiOS SDK
   sudo pkg install multios-sdk
   ```

2. **Environment Issues**
   ```bash
   # Set environment variables
   export MULTIOS_ROOT=/opt/multios
   export PATH=$PATH:$MULTIOS_ROOT/bin
   
   # Clear build cache
   rm -rf build/ CMakeCache.txt
   ```

3. **Compiler Problems**
   ```bash
   # Check compiler version
   gcc --version
   
   # Install recommended compiler
   sudo pkg install gcc-multios
   ```

### Issue: Debugging Difficulties

**Symptoms:**
- Cannot attach debugger
- Debug symbols missing
- GDB not working

**Solutions:**

1. **Install Debug Tools**
   ```bash
   # Install debugging packages
   sudo pkg install gdb multios-debug-symbols
   
   # Install development headers
   sudo pkg install multios-dev
   ```

2. **Enable Debug Symbols**
   ```bash
   # Compile with debug symbols
   gcc -g -O0 program.c -o program
   
   # Install debug packages
   sudo pkg install multios-dbg
   ```

### Issue: Performance Profiling Issues

**Symptoms:**
- Profiler not working
- No performance data
- System instability during profiling

**Solutions:**

1. **Check Profiling Tools**
   ```bash
   # Install profiling tools
   sudo pkg install perf valgrind
   
   # Check system support
   perf list
   ```

2. **System Configuration**
   ```bash
   # Enable perf events
   echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
   
   # Install debugging symbols
   sudo pkg install multios-dbgsym
   ```

## Getting Help

### Community Resources

#### Documentation
- **User Guide**: `/usr/share/doc/multios/`
- **FAQ**: `https://faq.multios.org/`
- **Wiki**: `https://wiki.multios.org/`

#### Forums and Support
- **Official Forums**: `https://forums.multios.org/`
- **Reddit Community**: `r/MultiOS`
- **Discord Server**: `https://discord.gg/multios`

#### Professional Support
- **Commercial Support**: `https://support.multios.com/`
- **Enterprise Services**: `https://enterprise.multios.com/`

### Providing Useful Information

When asking for help, include:

1. **System Information**
   ```bash
   # Generate system report
   inxi -Fxxx
   ```

2. **Error Messages**
   - Complete error output
   - Steps to reproduce
   - Expected vs actual behavior

3. **Hardware Details**
   ```bash
   # Hardware information
   lscpu
   lspci -nn
   lsusb
   ```

4. **Configuration Files**
   - Relevant config files
   - Log excerpts
   - System settings

### Reporting Bugs

#### Bug Reporting Process
1. **Search existing reports**
2. **Gather system information**
3. **Create detailed report**
4. **Follow up on progress**

#### Information to Include
- System specifications
- Steps to reproduce
- Expected behavior
- Actual behavior
- Log files
- Screenshots (if applicable)

---

**Remember**: Most problems have been encountered by others before. Search the community resources first, and don't hesitate to ask for help when needed!