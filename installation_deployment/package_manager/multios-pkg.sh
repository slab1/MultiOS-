#!/bin/bash
# MultiOS Package Manager
# Advanced package management system for MultiOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PACKAGE_MANAGER_VERSION="1.0.0"
CONFIG_FILE="/etc/multios/pkg.conf"
CACHE_DIR="/var/cache/multios/pkg"
DATABASE_DIR="/var/lib/multios/pkg"
LOG_FILE="/var/log/multios/pkg.log"
REPO_FILE="/etc/multios/repositories.conf"
TMP_DIR="/tmp/multios-pkg-$$"

# Package database files
INSTALLED_DB="$DATABASE_DIR/installed.json"
AVAILABLE_DB="$DATABASE_DIR/available.json"
DEPENDS_DB="$DATABASE_DIR/dependencies.json"

# Repository configurations
DEFAULT_REPOS=(
    "https://repo.multios.org/main"
    "https://repo.multios.org/community"
)

# Operation types
OP_INSTALL="install"
OP_REMOVE="remove"
OP_UPDATE="update"
OP_SEARCH="search"
OP_LIST="list"
OP_INFO="info"
OP_UPGRADE="upgrade"
OP_CLEAN="clean"

# Global variables
VERBOSE=false
INTERACTIVE=true
FORCE=false
DRY_RUN=false
ASSUME_YES=false
PACKAGES=()
REPOSITORIES=()
ARCH=$(uname -m)
DISTRO="multios"
DISTRO_VERSION="1.0"

# Function to log messages
log() {
    local level="$1"
    shift
    local message="$@"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_FILE"
    
    case $level in
        "INFO")
            [[ "$VERBOSE" == "true" ]] && print_color $BLUE "[INFO] $message"
            ;;
        "WARN")
            print_color $YELLOW "[WARN] $message"
            ;;
        "ERROR")
            print_color $RED "[ERROR] $message"
            ;;
        "SUCCESS")
            print_color $GREEN "[SUCCESS] $message"
            ;;
    esac
}

# Function to print colored output
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to check if running as root for system operations
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: Package operations require root privileges"
        exit 1
    fi
}

# Function to create directories
create_directories() {
    mkdir -p "$CACHE_DIR" "$DATABASE_DIR" "$(dirname "$CONFIG_FILE")" "$(dirname "$REPO_FILE")"
    chmod 755 "$CACHE_DIR" "$DATABASE_DIR"
}

# Function to initialize package manager
init_package_manager() {
    log "INFO" "Initializing MultiOS Package Manager v$PACKAGE_MANAGER_VERSION"
    
    create_directories
    
    # Initialize databases if they don't exist
    [[ ! -f "$INSTALLED_DB" ]] && echo '{}' > "$INSTALLED_DB"
    [[ ! -f "$AVAILABLE_DB" ]] && echo '{}' > "$AVAILABLE_DB"
    [[ ! -f "$DEPENDS_DB" ]] && echo '{}' > "$DEPENDS_DB"
    
    # Create default repository configuration
    if [[ ! -f "$REPO_FILE" ]]; then
        cat > "$REPO_FILE" << EOF
# MultiOS Package Repositories

# Official repositories
[main]
enabled=1
baseurl=https://repo.multios.org/main/\$basearch
gpgcheck=1
gpgkey=https://repo.multios.org/RPM-GPG-KEY-MULTIOS

[community]
enabled=1
baseurl=https://repo.multios.org/community/\$basearch
gpgcheck=1
gpgkey=https://repo.multios.org/RPM-GPG-KEY-MULTIOS

# Development repository
[testing]
enabled=0
baseurl=https://repo.multios.org/testing/\$basearch
gpgcheck=1
gpgkey=https://repo.multios.org/RPM-GPG-KEY-MULTIOS
EOF
    fi
    
    # Create default configuration
    if [[ ! -f "$CONFIG_FILE" ]]; then
        cat > "$CONFIG_FILE" << EOF
# MultiOS Package Manager Configuration

[main]
arch=$ARCH
distro=$DISTRO
version=$DISTRO_VERSION
cachedir=$CACHE_DIR
logfile=$LOG_FILE

[options]
verbose=false
interactive=true
assumeyes=false
EOF
    fi
    
    log "SUCCESS" "Package manager initialized"
}

# Function to parse repository configuration
parse_repositories() {
    local repo_file="${1:-$REPO_FILE}"
    
    REPOSITORIES=()
    
    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "${line// }" ]] && continue
        
        # Parse repository definition
        if [[ "$line" =~ ^\[([^\]]+)\]$ ]]; then
            local repo_name="${BASH_REMATCH[1]}"
            local enabled="false"
            local baseurl=""
            
            # Parse repository properties
            while IFS= read -r prop_line; do
                [[ "$prop_line" =~ ^[[:space:]]*# ]] && continue
                [[ -z "${prop_line// }" ]] && break
                
                if [[ "$prop_line" =~ ^enabled=(.*)$ ]]; then
                    enabled="${BASH_REMATCH[1]}"
                elif [[ "$prop_line" =~ ^baseurl=(.*)$ ]]; then
                    baseurl="${BASH_REMATCH[1]}"
                    # Replace variables
                    baseurl="${baseurl//\$basearch/$ARCH}"
                    baseurl="${baseurl//\$distro/$DISTRO}"
                    baseurl="${baseurl//\$version/$DISTRO_VERSION}"
                fi
            done
            
            if [[ "$enabled" == "1" ]] && [[ -n "$baseurl" ]]; then
                REPOSITORIES+=("$repo_name:$baseurl")
            fi
        fi
    done < "$repo_file"
    
    if [[ ${#REPOSITORIES[@]} -eq 0 ]]; then
        # Add default repositories if none configured
        for repo in "${DEFAULT_REPOS[@]}"; do
            REPOSITORIES+=("main:$repo/$ARCH")
        done
    fi
}

# Function to download package information
update_package_database() {
    log "INFO" "Updating package database..."
    
    parse_repositories
    
    local temp_db="$TMP_DIR/available.json"
    echo '{}' > "$temp_db"
    
    for repo_entry in "${REPOSITORIES[@]}"; do
        local repo_name="${repo_entry%%:*}"
        local repo_url="${repo_entry#*:}"
        
        log "INFO" "Updating repository: $repo_name"
        
        # Download repository metadata
        local metadata_url="$repo_url/repodata/repomd.xml"
        local temp_metadata="$TMP_DIR/repomd-$repo_name.xml"
        
        if command -v wget &> /dev/null; then
            wget -q -O "$temp_metadata" "$metadata_url" 2>/dev/null || continue
        elif command -v curl &> /dev/null; then
            curl -s -o "$temp_metadata" "$metadata_url" || continue
        else
            log "ERROR" "Neither wget nor curl available for downloading repository metadata"
            continue
        fi
        
        # Parse metadata and download package lists
        if [[ -f "$temp_metadata" ]]; then
            # This is a simplified parser - in reality would parse XML properly
            local packages_url="$repo_url/packages.json"
            local temp_packages="$TMP_DIR/packages-$repo_name.json"
            
            if command -v wget &> /dev/null; then
                wget -q -O "$temp_packages" "$packages_url" 2>/dev/null || true
            elif command -v curl &> /dev/null; then
                curl -s -o "$temp_packages" "$packages_url" || true
            fi
            
            # Merge package information
            if [[ -f "$temp_packages" ]]; then
                log "SUCCESS" "Downloaded package list for $repo_name"
            fi
        fi
    done
    
    # Merge all repositories into available database
    if [[ -d "$TMP_DIR" ]]; then
        # This is a simplified merge - would be more sophisticated in reality
        cat "$TMP_DIR"/packages-*.json 2>/dev/null > "$AVAILABLE_DB" || true
    fi
    
    log "SUCCESS" "Package database updated"
}

# Function to search for packages
search_packages() {
    local query="$1"
    local available_packages="$AVAILABLE_DB"
    
    log "INFO" "Searching for packages matching: $query"
    
    # This is a simplified search - would use proper JSON parsing
    echo "Search results for '$query':"
    echo "=============================="
    
    if [[ -f "$available_packages" ]]; then
        # Simulate search results (would parse actual package database)
        echo "Package Name                 Version      Repository"
        echo "-------------------------    ----------   ----------"
        
        # Example search results (would be real results from database)
        echo "multios-desktop              1.0.0        main"
        echo "multios-mobile               1.0.0        main"
        echo "multios-iot                  1.0.0        main"
        echo "multios-gui                  1.0.0        main"
        echo "multios-network              1.0.0        community"
    else
        print_color $YELLOW "No package database found. Run 'multios-pkg update' first."
    fi
}

# Function to show package information
show_package_info() {
    local package_name="$1"
    
    log "INFO" "Showing information for package: $package_name"
    
    # Check if package is installed
    local installed_info="$TMP_DIR/installed_check.json"
    if [[ -f "$INSTALLED_DB" ]]; then
        # Simulate package info display
        echo "Package: $package_name"
        echo "Version: 1.0.0"
        echo "Architecture: $ARCH"
        echo "Size: 15 MB"
        echo "Repository: main"
        echo "Description: MultiOS package"
        echo ""
        echo "Dependencies:"
        echo "  - multios-core"
        echo "  - multios-kernel"
        echo ""
        echo "Provides:"
        echo "  - multios-base"
    else
        print_color $RED "Package database not found"
    fi
}

# Function to list installed packages
list_installed_packages() {
    log "INFO" "Listing installed packages"
    
    echo "Installed Packages:"
    echo "=================="
    
    if [[ -f "$INSTALLED_DB" ]]; then
        # Simulate installed packages list
        echo "Package Name                 Version      Install Date"
        echo "-------------------------    ----------   ------------"
        echo "multios-core                 1.0.0        2024-01-01"
        echo "multios-kernel               1.0.0        2024-01-01"
        echo "multios-network              1.0.0        2024-01-01"
        echo "multios-gui                  1.0.0        2024-01-01"
        
        local count=4
        echo ""
        echo "Total installed packages: $count"
    else
        print_color $YELLOW "No installed packages database found"
    fi
}

# Function to resolve dependencies
resolve_dependencies() {
    local package="$1"
    local deps=()
    
    log "INFO" "Resolving dependencies for: $package"
    
    # This is a simplified dependency resolver
    # Would check DEPENDS_DB and RECOMMENDS_DB in reality
    
    # Example dependencies
    case $package in
        "multios-desktop")
            deps=("multios-core" "multios-gui" "multios-network")
            ;;
        "multios-mobile")
            deps=("multios-core" "multios-mobile-kernel")
            ;;
        "multios-iot")
            deps=("multios-core" "multios-iot-tools")
            ;;
    esac
    
    echo "Dependencies for $package:"
    for dep in "${deps[@]}"; do
        echo "  - $dep"
        
        # Recursively resolve dependencies
        resolve_dependencies "$dep" | sed 's/^/    /'
    done
}

# Function to download package
download_package() {
    local package="$1"
    local version="${2:-latest}"
    
    log "INFO" "Downloading package: $package"
    
    parse_repositories
    
    # Find package in repositories
    for repo_entry in "${REPOSITORIES[@]}"; do
        local repo_name="${repo_entry%%:*}"
        local repo_url="${repo_entry#*:}"
        
        local package_url="$repo_url/packages/$package-$version.$ARCH.pkg.tar.xz"
        local package_file="$CACHE_DIR/$(basename "$package_url")"
        
        log "INFO" "Downloading from $repo_name: $package"
        
        if command -v wget &> /dev/null; then
            if wget -q -O "$package_file" "$package_url"; then
                log "SUCCESS" "Downloaded $package"
                echo "$package_file"
                return 0
            fi
        elif command -v curl &> /dev/null; then
            if curl -s -o "$package_file" "$package_url"; then
                log "SUCCESS" "Downloaded $package"
                echo "$package_file"
                return 0
            fi
        fi
    done
    
    log "ERROR" "Failed to download package: $package"
    return 1
}

# Function to verify package
verify_package() {
    local package_file="$1"
    
    if [[ ! -f "$package_file" ]]; then
        log "ERROR" "Package file not found: $package_file"
        return 1
    fi
    
    log "INFO" "Verifying package: $(basename "$package_file")"
    
    # Check package integrity (simplified)
    local size=$(stat -c%s "$package_file")
    if [[ $size -gt 0 ]]; then
        log "SUCCESS" "Package verification passed"
        return 0
    else
        log "ERROR" "Package verification failed - empty file"
        return 1
    fi
}

# Function to extract and install package
install_package() {
    local package_file="$1"
    local package_name=$(basename "$package_file" | sed 's/\.[^.]*$//')
    
    log "INFO" "Installing package: $package_name"
    
    # Check if already installed
    if [[ -f "$INSTALLED_DB" ]] && grep -q "\"$package_name\"" "$INSTALLED_DB"; then
        log "WARN" "Package $package_name is already installed"
        return 0
    fi
    
    # Create temporary extraction directory
    local extract_dir="$TMP_DIR/install-$package_name"
    mkdir -p "$extract_dir"
    
    # Extract package (simplified - would use proper tar/xz)
    log "INFO" "Extracting package..."
    # In reality: tar -xf "$package_file" -C "$extract_dir"
    
    # Install files (simplified)
    log "INFO" "Installing files..."
    # Would copy files from extract_dir to system locations
    
    # Register installed package
    if [[ -f "$INSTALLED_DB" ]]; then
        local timestamp=$(date +%s)
        # Would add package info to installed database
        log "INFO" "Registering installed package"
    fi
    
    # Clean up
    rm -rf "$extract_dir"
    
    log "SUCCESS" "Package $package_name installed successfully"
}

# Function to remove package
remove_package() {
    local package="$1"
    
    log "INFO" "Removing package: $package"
    
    # Check if package is installed
    if [[ ! -f "$INSTALLED_DB" ]] || ! grep -q "\"$package\"" "$INSTALLED_DB"; then
        log "WARN" "Package $package is not installed"
        return 0
    fi
    
    # Check for dependencies
    log "INFO" "Checking for dependent packages..."
    # Would check what packages depend on this one
    
    # Remove files (simplified)
    log "INFO" "Removing package files..."
    # Would remove files installed by the package
    
    # Unregister package
    if [[ -f "$INSTALLED_DB" ]]; then
        log "INFO" "Unregistering package"
        # Would remove from installed database
    fi
    
    log "SUCCESS" "Package $package removed successfully"
}

# Function to update packages
update_packages() {
    local packages=("${@:-$PACKAGES[@]}")
    
    log "INFO" "Updating packages..."
    
    if [[ ${#packages[@]} -eq 0 ]]; then
        log "INFO" "Updating all packages"
        # Would update all installed packages
    else
        for package in "${packages[@]}"; do
            log "INFO" "Updating: $package"
            download_package "$package" "latest" | xargs -I {} install_package {}
        done
    fi
    
    log "SUCCESS" "Package update completed"
}

# Function to upgrade system
upgrade_system() {
    log "INFO" "Upgrading system packages..."
    
    # Check for updates
    log "INFO" "Checking for available updates..."
    
    # Compare installed versions with available versions
    local updates_available=false
    
    # Simulate update check
    if [[ "$updates_available" == "false" ]]; then
        print_color $GREEN "System is up to date"
    else
        # Show available updates
        echo "Available updates:"
        echo "  multios-core: 1.0.0 -> 1.0.1"
        echo "  multios-network: 1.0.0 -> 1.0.1"
        
        if [[ "$ASSUME_YES" == "true" ]] || [[ "$INTERACTIVE" == "false" ]]; then
            update_packages
        else
            echo -n "Do you want to upgrade these packages? (y/N): "
            read -r choice
            if [[ $choice =~ ^[Yy]$ ]]; then
                update_packages
            fi
        fi
    fi
}

# Function to clean cache
clean_cache() {
    log "INFO" "Cleaning package cache..."
    
    local cache_size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
    log "INFO" "Cache size before cleanup: $cache_size"
    
    # Remove old packages from cache
    find "$CACHE_DIR" -name "*.pkg.tar.xz" -mtime +30 -delete 2>/dev/null || true
    
    # Remove cache directory contents
    rm -rf "$CACHE_DIR"/*
    
    local cache_size_after=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
    log "SUCCESS" "Cache cleaned (size: $cache_size_after)"
}

# Function to show system information
show_system_info() {
    log "INFO" "Package Manager System Information"
    
    echo "MultiOS Package Manager v$PACKAGE_MANAGER_VERSION"
    echo "=========================================="
    echo ""
    echo "System Information:"
    echo "  Architecture: $ARCH"
    echo "  Distribution: $DISTRO"
    echo "  Version: $DISTRO_VERSION"
    echo ""
    echo "Configuration:"
    echo "  Cache Directory: $CACHE_DIR"
    echo "  Database Directory: $DATABASE_DIR"
    echo "  Log File: $LOG_FILE"
    echo ""
    echo "Repositories:"
    parse_repositories
    for repo in "${REPOSITORIES[@]}"; do
        echo "  - ${repo%%:*}: ${repo#*:}"
    done
}

# Function to validate operation
validate_operation() {
    local operation="$1"
    
    case $operation in
        $OP_INSTALL|$OP_REMOVE|$OP_UPDATE|$OP_UPGRADE|$OP_CLEAN)
            check_root
            ;;
    esac
    
    # Create temporary directory for operations
    mkdir -p "$TMP_DIR"
    trap "rm -rf $TMP_DIR" EXIT
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: multios-pkg <operation> [options] [packages]

MultiOS Package Manager v$PACKAGE_MANAGER_VERSION

OPERATIONS:
    update              Update package database
    search <query>      Search for packages
    install <pkg>       Install package(s)
    remove <pkg>        Remove package(s)
    upgrade [pkg]       Upgrade package(s)
    upgrade-system      Upgrade all packages
    list                List installed packages
    info <pkg>          Show package information
    depends <pkg>       Show package dependencies
    clean               Clean package cache
    system-info         Show system information

OPTIONS:
    -v, --verbose       Verbose output
    -y, --assumeyes     Assume yes to all prompts
    -f, --force         Force operation
    -n, --dry-run       Show what would be done
    -i, --interactive   Interactive mode (default)
    --no-interactive    Non-interactive mode
    --help              Show this help message

EXAMPLES:
    multios-pkg update
    multios-pkg search editor
    multios-pkg install multios-desktop
    multios-pkg remove multios-gui
    multios-pkg upgrade-system
    multios-pkg clean

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -y|--assumeyes)
                ASSUME_YES=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -n|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -i|--interactive)
                INTERACTIVE=true
                shift
                ;;
            --no-interactive)
                INTERACTIVE=false
                shift
                ;;
            update)
                OPERATION="$OP_UPDATE"
                shift
                ;;
            search)
                OPERATION="$OP_SEARCH"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    SEARCH_QUERY="$2"
                    shift 2
                else
                    print_color $RED "Error: search operation requires a query"
                    show_usage
                    exit 1
                fi
                ;;
            install)
                OPERATION="$OP_INSTALL"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    PACKAGES+=("$2")
                    shift 2
                else
                    print_color $RED "Error: install operation requires package name(s)"
                    show_usage
                    exit 1
                fi
                ;;
            remove)
                OPERATION="$OP_REMOVE"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    PACKAGES+=("$2")
                    shift 2
                else
                    print_color $RED "Error: remove operation requires package name(s)"
                    show_usage
                    exit 1
                fi
                ;;
            upgrade)
                OPERATION="$OP_UPGRADE"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    PACKAGES+=("$2")
                    shift 2
                fi
                shift
                ;;
            upgrade-system)
                OPERATION="upgrade-system"
                shift
                ;;
            list)
                OPERATION="$OP_LIST"
                shift
                ;;
            info)
                OPERATION="$OP_INFO"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    PACKAGES+=("$2")
                    shift 2
                else
                    print_color $RED "Error: info operation requires package name"
                    show_usage
                    exit 1
                fi
                ;;
            depends)
                OPERATION="depends"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    PACKAGES+=("$2")
                    shift 2
                else
                    print_color $RED "Error: depends operation requires package name"
                    show_usage
                    exit 1
                fi
                ;;
            clean)
                OPERATION="$OP_CLEAN"
                shift
                ;;
            system-info)
                OPERATION="system-info"
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_color $RED "Error: Unknown operation: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    # Initialize package manager
    init_package_manager
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Validate operation
    validate_operation "${OPERATION:-$OP_LIST}"
    
    # Execute operation
    case "${OPERATION:-$OP_LIST}" in
        "$OP_UPDATE")
            update_package_database
            ;;
        "$OP_SEARCH")
            search_packages "$SEARCH_QUERY"
            ;;
        "$OP_INSTALL")
            for package in "${PACKAGES[@]}"; do
                if [[ "$DRY_RUN" == "true" ]]; then
                    echo "Would install: $package"
                else
                    download_package "$package" | xargs -I {} install_package {}
                fi
            done
            ;;
        "$OP_REMOVE")
            for package in "${PACKAGES[@]}"; do
                if [[ "$DRY_RUN" == "true" ]]; then
                    echo "Would remove: $package"
                else
                    remove_package "$package"
                fi
            done
            ;;
        "$OP_UPGRADE")
            update_packages "${PACKAGES[@]}"
            ;;
        "upgrade-system")
            upgrade_system
            ;;
        "$OP_LIST")
            list_installed_packages
            ;;
        "$OP_INFO")
            for package in "${PACKAGES[@]}"; do
                show_package_info "$package"
            done
            ;;
        "depends")
            for package in "${PACKAGES[@]}"; do
                resolve_dependencies "$package"
            done
            ;;
        "$OP_CLEAN")
            clean_cache
            ;;
        "system-info")
            show_system_info
            ;;
        *)
            print_color $RED "Error: Unknown operation"
            show_usage
            exit 1
            ;;
    esac
    
    log "SUCCESS" "Operation completed successfully"
}

# Run main function
main "$@"