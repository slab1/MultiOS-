# MultiOS User Manual

Welcome to the MultiOS User Manual! This comprehensive guide covers all aspects of using MultiOS, from basic desktop operations to advanced system administration.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Desktop Environment](#desktop-environment)
4. [Command Line Interface](#command-line-interface)
5. [File Management](#file-management)
6. [Applications](#applications)
7. [Networking](#networking)
8. [Multimedia](#multimedia)
9. [System Settings](#system-settings)
10. [User Accounts](#user-accounts)
11. [System Administration](#system-administration)
12. [Troubleshooting](#troubleshooting)
13. [Tips and Tricks](#tips-and-tricks)

## Introduction

### What is MultiOS?

MultiOS is a modern, educational operating system designed to run across multiple architectures and platforms. It features:

- **Cross-Platform Support**: Runs on x86_64, ARM64, and RISC-V processors
- **Educational Focus**: Built for learning operating system concepts
- **Memory Safety**: Implemented in Rust for enhanced security
- **Modular Design**: Hybrid microkernel architecture for flexibility
- **User-Friendly Interface**: Intuitive desktop and command-line environments

### System Overview

MultiOS provides:
- **Graphical Desktop Environment**: Modern GUI with window management
- **Command Line Interface**: Powerful terminal for advanced users
- **File System**: Robust file management with virtual file system
- **Network Support**: TCP/IP networking with various protocols
- **Device Support**: Support for various hardware devices
- **Application Framework**: Environment for running applications

## Getting Started

### First Boot

When you start MultiOS for the first time:

1. **Boot Screen**: MultiOS logo and loading animation
2. **Welcome Setup**: Initial configuration wizard
3. **User Creation**: Set up your user account
4. **Desktop**: Launch into the graphical desktop environment

### Desktop Layout

The MultiOS desktop consists of:

```
┌─────────────────────────────────────────────────────────────┐
│ Menu Bar (File, Edit, View, Applications, Help)           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                     Desktop Area                            │
│                  (Application Windows)                      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Task Bar (Start Menu | Running Apps | System Tray)        │
└─────────────────────────────────────────────────────────────┘
```

### Basic Navigation

- **Mouse**: Left-click to select, right-click for context menu
- **Keyboard**: Use Tab to navigate, Enter to activate
- **Touch**: Tap to select, long-press for context menu (on touch devices)

## Desktop Environment

### Start Menu

The Start Menu provides access to all applications and system functions:

**Accessing the Start Menu:**
- Click the Start button in the taskbar
- Press the Windows key (or equivalent)
- Search for applications and files

**Start Menu Sections:**
- **Recently Used**: Applications and files you've recently accessed
- **All Applications**: Complete list of installed applications
- **System Tools**: System administration and maintenance tools
- **Settings**: System configuration options

### Window Management

#### Basic Window Operations

**Moving Windows:**
- Click and drag the title bar to move windows
- Use keyboard: Alt + F7, then arrow keys to move

**Resizing Windows:**
- Drag window borders or corners
- Double-click title bar to maximize/restore
- Use keyboard: Alt + F8, then arrow keys to resize

**Closing Windows:**
- Click the X button in the title bar
- Press Alt + F4
- Use the window menu: Alt + Space, then C

#### Window Controls

| Control | Function | Shortcut |
|---------|----------|----------|
| Minimize | Hide window to taskbar | Alt + F9 |
| Maximize | Fill available screen space | Alt + F10 |
| Restore | Return to previous size | Alt + F5 |
| Close | Close application | Alt + F4 |

#### Workspace Management

**Virtual Desktops:**
- Create multiple workspaces for organization
- Switch between workspaces using taskbar buttons
- Move windows between workspaces via context menu

**Workspace Shortcuts:**
- Ctrl + Alt + Left/Right: Switch workspaces
- Ctrl + Alt + 1-9: Jump to specific workspace

### File Manager

MultiOS includes a powerful file manager with the following features:

#### Basic Operations

**Navigation:**
- **Browse folders**: Double-click folders to open
- **Breadcrumb navigation**: Click path elements to navigate up
- **Address bar**: Type paths directly

**File Operations:**
- **Copy**: Ctrl + C, then Ctrl + V
- **Move**: Ctrl + X, then Ctrl + V
- **Delete**: Delete key or right-click → Delete
- **Rename**: F2 or right-click → Rename

**Selection:**
- **Single file**: Click once
- **Multiple files**: Ctrl + Click
- **Range select**: Shift + Click
- **All files**: Ctrl + A

#### File Manager Views

**Icon View**: Shows files as large icons with names below
**List View**: Shows files in a detailed list format
**Tree View**: Shows folder hierarchy in left panel

**View Controls:**
- Ctrl + 1: Icon view
- Ctrl + 2: List view  
- Ctrl + 3: Tree view
- Ctrl + +: Zoom in
- Ctrl + -: Zoom out

#### Advanced Features

**File Properties:**
- Right-click file → Properties
- View file size, permissions, creation date
- Change file associations and permissions

**Search:**
- Press Ctrl + F to open search
- Search by filename, content, type, or date
- Use wildcards: * (any characters), ? (single character)

## Command Line Interface

### Accessing the Terminal

**From Desktop:**
- Start Menu → System Tools → Terminal
- Right-click desktop → Open Terminal Here
- Taskbar terminal button

**Keyboard Shortcuts:**
- Ctrl + Alt + T: Open new terminal
- Ctrl + Shift + T: Open new terminal tab

### Basic Commands

#### File System Navigation

```bash
# Print working directory
pwd

# List directory contents
ls              # Basic listing
ls -l           # Detailed listing
ls -a           # Include hidden files
ls -la          # Detailed with hidden files

# Change directory
cd /path/to/directory
cd ~            # Go to home directory
cd -            # Go to previous directory
cd ..           # Go up one level

# Make directory
mkdir newfolder

# Remove directory
rmdir emptyfolder
rm -rf folder   # Remove directory and contents
```

#### File Operations

```bash
# Copy files
cp source.txt destination.txt
cp source.txt /path/to/destination/

# Move/rename files
mv oldname.txt newname.txt
mv file.txt /path/to/destination/

# Delete files
rm filename.txt
rm -f filename.txt     # Force deletion
rm *.txt              # Delete all .txt files

# View file contents
cat filename.txt      # Display entire file
less filename.txt     # View file with paging
head filename.txt     # Show first 10 lines
tail filename.txt     # Show last 10 lines
```

#### System Information

```bash
# System information
uname -a              # System information
uptime                # System uptime
whoami               # Current username
date                 # Current date/time

# Memory and disk usage
free                 # Memory usage
df                   # Disk space
du                   # Directory space usage

# Process information
ps                   # Running processes
top                  # Interactive process viewer
jobs                 # Background jobs
```

### Text Editor

MultiOS includes a simple text editor accessible via command line:

```bash
# Start text editor
nano filename.txt
# or
vim filename.txt

# Save and exit (nano)
Ctrl + O, Enter, Ctrl + X

# Save and exit (vim)
Escape, :wq, Enter
```

### Command Line Tips

**Command History:**
- Up/Down arrows: Navigate command history
- Ctrl + R: Search command history
- history: Show command history

**Command Line Shortcuts:**
- Ctrl + A: Move to beginning of line
- Ctrl + E: Move to end of line
- Ctrl + U: Clear line before cursor
- Ctrl + K: Clear line after cursor
- Tab: Auto-complete
- Tab Tab: Show completion options

## File Management

### File System Structure

MultiOS uses a hierarchical file system:

```
/
├── bin/           # System binaries
├── boot/          # Boot files
├── dev/           # Device files
├── etc/           # Configuration files
├── home/          # User home directories
├── lib/           # System libraries
├── media/         # Removable media
├── mnt/           # Mount points
├── proc/          # Process information
├── root/          # Root user home
├── run/           # Runtime data
├── sbin/          # System binaries
├── sys/           # System information
├── tmp/           # Temporary files
├── usr/           # User programs and data
└── var/           # Variable data
```

### Home Directory

Your personal files are stored in your home directory:

```
~/                    # Your home directory
├── Documents/        # Personal documents
├── Downloads/        # Downloaded files
├── Pictures/         # Image files
├── Music/           # Audio files
├── Videos/          # Video files
├── Desktop/         # Desktop files
├── .config/         # Application settings
└── .local/          # Local application data
```

### File Permissions

MultiOS supports Unix-style permissions:

**Permission Types:**
- r (read): 4
- w (write): 2  
- x (execute): 1

**Permission Groups:**
- u (user/owner): Owner's permissions
- g (group): Group permissions
- o (others): Other users' permissions

**Setting Permissions:**
```bash
# Numeric method
chmod 755 filename.txt     # rwxr-xr-x
chmod 644 filename.txt     # rw-r--r--

# Symbolic method
chmod u+rwx filename.txt   # Owner: read, write, execute
chmod g+rw filename.txt    # Group: read, write
chmod o=r filename.txt     # Others: read only
chmod a+r filename.txt     # All: read
```

### File Searching

```bash
# Find files by name
find /path -name "*.txt"
find ~/Documents -name "report*"

# Find files by type
find /path -type f          # Files only
find /path -type d          # Directories only

# Find files by size
find /path -size +100M      # Larger than 100MB
find /path -size -10K       # Smaller than 10KB

# Search file contents
grep "search term" filename.txt
grep -r "search term" /path  # Recursive search
```

## Applications

### Installing Applications

#### Package Manager (when available)
```bash
# Search for applications
multios-search application-name

# Install application
multios-install application-name

# Remove application
multios-remove application-name

# Update package database
multios-update
```

#### Manual Installation
1. Download application package
2. Extract to appropriate directory
3. Create desktop shortcut if needed

### Built-in Applications

#### Web Browser
**Access**: Start Menu → Internet → Web Browser

**Features:**
- Multiple tabs support
- Bookmarks
- Download manager
- Privacy settings

**Navigation:**
- Ctrl + T: New tab
- Ctrl + W: Close tab
- Ctrl + R: Refresh
- Ctrl + L: Address bar

#### Text Editor
**Access**: Start Menu → Accessories → Text Editor

**Features:**
- Syntax highlighting
- Find and replace
- Multiple documents
- Printing support

**Shortcuts:**
- Ctrl + S: Save
- Ctrl + O: Open
- Ctrl + Z: Undo
- Ctrl + Y: Redo
- Ctrl + F: Find

#### Calculator
**Access**: Start Menu → Accessories → Calculator

**Features:**
- Basic and scientific modes
- Memory functions
- History
- Keyboard shortcuts

#### Image Viewer
**Access**: Start Menu → Graphics → Image Viewer

**Features:**
- Support for common image formats
- Zoom and pan
- Slideshow mode
- Basic editing tools

### Application Management

#### Running Applications
**From Desktop:**
- Double-click application icon
- Use Start Menu to search and launch

**From Command Line:**
```bash
# Run application in background
application-name &

# Run with specific options
application-name --option value

# Check if application is running
pgrep application-name

# Stop application
pkill application-name
```

#### Application Settings
**Location**: Settings → Applications

**Options:**
- List installed applications
- Configure default applications
- Manage startup applications
- Application permissions

## Networking

### Network Configuration

#### WiFi Setup
**Access**: System Tray → Network Icon → WiFi Settings

**Steps:**
1. Select available network
2. Enter password if required
3. Connect

#### Ethernet Setup
**Access**: System Tray → Network Icon → Wired Settings

**Options:**
- Automatic (DHCP)
- Manual IP configuration
- DNS settings

#### Network Information
```bash
# Check network status
ifconfig
ip addr show

# Test connectivity
ping google.com
ping 8.8.8.8

# Check routing
route -n
ip route show
```

### Web Browsing

#### Browser Features
- **Tabs**: Multiple websites open simultaneously
- **Bookmarks**: Save favorite websites
- **History**: Track browsing history
- **Downloads**: Download manager
- **Privacy**: Clear browsing data

#### Browser Shortcuts
- Ctrl + T: New tab
- Ctrl + W: Close tab
- Ctrl + R: Refresh
- Ctrl + L: Address bar
- Ctrl + U: View source
- F11: Full screen

### File Sharing

#### Network Drives
```bash
# Mount network share
mount -t cifs //server/share /mnt/network -o username=user

# Unmount network share
umount /mnt/network

# List mounted shares
mount | grep cifs
```

#### Transfer Files
```bash
# SCP (secure copy)
scp file.txt user@remote:/path/to/destination/
scp user@remote:/path/to/file.txt ./

# FTP
ftp remote.server.com
```

## Multimedia

### Audio System

#### Audio Controls
**System Tray**: Click speaker icon for volume control
**Keyboard**: Volume up/down/mute keys
**Command Line**: Use audio control commands

#### Audio Applications
**Audio Player**: Start Menu → Multimedia → Audio Player
**Volume Control**: System Settings → Audio
**Audio Recording**: Sound Recorder application

### Video System

#### Video Playback
**Video Player**: Start Menu → Multimedia → Video Player
**Supported Formats**: Common video formats supported
**Controls**: Play, pause, stop, seek, fullscreen

#### Screen Capture
**Screenshot**: Print Screen key or Start Menu → Multimedia → Screenshot
**Screen Recording**: Screen Recorder application
**Image Formats**: PNG, JPEG, BMP

### Graphics

#### Image Viewing
**Image Viewer**: Start Menu → Graphics → Image Viewer
**Supported Formats**: PNG, JPEG, GIF, BMP
**Features**: Zoom, rotate, slideshow

#### Image Editing
**Basic Editor**: Start Menu → Graphics → Image Editor
**Features**: Crop, resize, basic filters
**Advanced Editing**: Install additional applications

## System Settings

### Display Settings

#### Resolution and Refresh Rate
**Access**: Settings → Display

**Options:**
- Screen resolution
- Refresh rate
- Orientation
- Multiple displays

**Hotkeys:**
- Ctrl + Alt + +: Increase resolution
- Ctrl + Alt + -: Decrease resolution

#### Appearance
**Themes**: Choose system theme (light/dark)
**Wallpaper**: Set desktop background
**Icons**: Configure icon style
**Fonts**: Adjust font settings

### Input Devices

#### Keyboard
**Settings**: Settings → Keyboard

**Options:**
- Keyboard layout
- Repeat rate
- Delay before repeat
- Function key behavior

**Accessibility:**
- Sticky keys
- Filter keys
- Toggle keys

#### Mouse
**Settings**: Settings → Mouse

**Options:**
- Button configuration (left/right handed)
- Pointer speed
- Double-click speed
- Scroll wheel behavior

**Accessibility:**
- Mouse keys
- Click locking
- Hover clicking

### Power Management

#### Power Settings
**Access**: Settings → Power

**Options:**
- Power plan
- Display timeout
- System sleep timeout
- Battery settings (laptops)

#### Advanced Power
```bash
# Command line power management
# Suspend system
systemctl suspend

# Hibernate system
systemctl hibernate

# Power off
systemctl poweroff

# Restart
systemctl reboot
```

## User Accounts

### User Management

#### Creating User Accounts
**Access**: Settings → Users

**Steps:**
1. Click "Add User"
2. Enter user information
3. Set password
4. Assign user type (Standard/Administrator)

#### User Types
**Standard User:**
- Can run applications
- Can access personal files
- Cannot change system settings

**Administrator:**
- Can install software
- Can change system settings
- Can manage other users

#### Changing Passwords
```bash
# Change your password
passwd

# Change another user's password (admin only)
sudo passwd username
```

### File Ownership and Permissions

#### File Ownership
```bash
# Change file owner
chown user:group filename.txt

# Change group ownership
chgrp groupname filename.txt

# View ownership
ls -l filename.txt
```

#### Access Control Lists (ACLs)
```bash
# Set ACL permissions
setfacl -m u:username:rwx filename.txt

# View ACL permissions
getfacl filename.txt

# Remove ACL
setfacl -x u:username filename.txt
```

## System Administration

### System Monitoring

#### System Information
```bash
# Detailed system information
inxi -Fxz

# Hardware information
lspci
lsusb
lshw

# System resources
top
htop
glances
```

#### Performance Monitoring
```bash
# CPU usage
top
mpstat 1

# Memory usage
free -h
cat /proc/meminfo

# Disk usage
df -h
du -sh /*
```

#### Log Files
```bash
# View system logs
journalctl

# View specific service logs
journalctl -u service-name

# View real-time logs
journalctl -f
```

### System Maintenance

#### Updates
```bash
# Update system (when package manager available)
multios-update

# Update package database
multios-update-db

# Install updates
multios-upgrade
```

#### File System Maintenance
```bash
# Check file system
fsck /dev/sda1

# Check disk space
df -h

# Clean temporary files
rm -rf /tmp/*
```

#### Backup and Restore
```bash
# Create backup
tar -czf backup.tar.gz /home/user

# Restore from backup
tar -xzf backup.tar.gz -C /
```

### Service Management

#### System Services
```bash
# List all services
systemctl list-units --type=service

# Start service
systemctl start service-name

# Stop service
systemctl stop service-name

# Enable service (start on boot)
systemctl enable service-name

# Disable service
systemctl disable service-name

# Check service status
systemctl status service-name
```

## Troubleshooting

### Common Issues

#### System Won't Boot
**Symptoms**: System hangs during boot or shows error messages

**Solutions:**
1. Check boot media integrity
2. Verify hardware connections
3. Try safe mode
4. Check file system for errors

#### Applications Won't Start
**Symptoms**: Applications fail to launch or crash immediately

**Solutions:**
1. Check application permissions
2. Verify dependencies
3. Check system logs
4. Reinstall application

#### Network Connection Issues
**Symptoms**: Cannot connect to network or internet

**Solutions:**
1. Check network cable/WiFi
2. Restart network manager
3. Check IP configuration
4. Verify DNS settings

#### Performance Issues
**Symptoms**: System runs slowly or hangs

**Solutions:**
1. Close unused applications
2. Check memory usage
3. Look for resource-intensive processes
4. Restart system if necessary

### Getting Help

#### System Information for Support
```bash
# Generate system report
inxi -Fxz > system-report.txt

# Check system logs
journalctl --since "1 hour ago" > recent-logs.txt
```

#### Community Support
- **Forums**: MultiOS community forums
- **IRC**: #multios on libera.chat
- **GitHub**: Issue tracker for bugs
- **Email**: Support email list

## Tips and Tricks

### Productivity Tips

#### Keyboard Shortcuts
| Shortcut | Function |
|----------|----------|
| Alt + Tab | Switch between applications |
| Ctrl + Alt + L | Lock screen |
| Ctrl + Alt + Delete | System menu |
| Win + D | Show desktop |
| Win + E | File manager |
| Win + R | Run dialog |
| F11 | Toggle fullscreen |
| Alt + F4 | Close window |

#### Power User Commands
```bash
# Quick directory navigation
cd -                    # Go to previous directory
cd ~                   # Go to home
cd                     # Go to home

# File operations
cp file1 file2.bak     # Quick backup
mv *.txt text/         # Move all .txt files
rm *~                  # Remove backup files

# Process management
Ctrl + Z               # Suspend process
bg                     # Resume in background
fg                     # Resume in foreground

# System monitoring
watch 'command'        # Run command repeatedly
nohup command &        # Run command after logout
```

#### Desktop Customization
- **Themes**: Change appearance via Settings
- **Shortcuts**: Create custom keyboard shortcuts
- **Workspace**: Use multiple virtual desktops
- **Widgets**: Add system widgets to desktop

### Advanced Features

#### Scripting
Create simple automation scripts:

```bash
#!/bin/bash
# Daily backup script
DATE=$(date +%Y%m%d)
tar -czf /backup/home_$DATE.tar.gz /home
```

#### Command Line Automation
```bash
# Run command on schedule
crontab -e

# Examples:
# 0 2 * * * /path/to/backup.sh    # Backup daily at 2 AM
# */15 * * * * /path/to/check.sh  # Check every 15 minutes
```

#### Development Environment
```bash
# Set up development environment
export EDITOR=nano
export PATH=$PATH:/opt/multios/bin

# Use project templates
multios-create-project myproject
```

---

**Up**: [Documentation Index](../README.md)  
**Related**: [Quick Start Guide](../getting_started/README.md) | [CLI Guide](cli.md)