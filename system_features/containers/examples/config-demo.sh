#!/bin/bash
# MultiOS Container System - Configuration Example
# This script demonstrates how to use JSON configuration files

set -e

CONFIG_FILE="sample-container-config.json"
CONTAINER_NAME="configured-python-app"

echo "=========================================="
echo "MultiOS Container Configuration Demo"
echo "=========================================="

# Check if config file exists
if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "❌ Configuration file not found: $CONFIG_FILE"
    echo "Please run this script from the examples directory"
    exit 1
fi

echo "📋 Using configuration file: $CONFIG_FILE"

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo "❌ This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to verify daemon is running
check_daemon() {
    echo "🔍 Checking container daemon..."
    if ! pgrep -x multios-containerd > /dev/null; then
        echo "📦 Starting container daemon..."
        multios-containerd --daemon
        sleep 2
    else
        echo "✅ Container daemon is running"
    fi
}

# Create container from configuration file
create_from_config() {
    echo ""
    echo "📦 Creating container from configuration file..."
    
    # Extract basic information from config
    local image=$(jq -r '.image' "$CONFIG_FILE")
    local name=$(jq -r '.name' "$CONFIG_FILE")
    
    if [[ "$name" == "null" || -z "$name" ]]; then
        name="$CONTAINER_NAME"
    fi
    
    echo "📋 Configuration Summary:"
    echo "  Name: $name"
    echo "  Image: $image"
    
    # Extract resources
    local cpu_limit=$(jq -r '.resources.cpu_limit // "1.0"' "$CONFIG_FILE")
    local memory_limit=$(jq -r '.resources.memory_limit // "1GB"' "$CONFIG_FILE")
    local disk_limit=$(jq -r '.resources.disk_limit // "5GB"' "$CONFIG_FILE")
    
    echo "  CPU Limit: $cpu_limit"
    echo "  Memory Limit: $memory_limit"
    echo "  Disk Limit: $disk_limit"
    
    # Extract ports
    local ports=$(jq -r '.networking.ports[] | "\(.container_port):\(.host_port)"' "$CONFIG_FILE" 2>/dev/null | tr '\n' ',' | sed 's/,$//')
    echo "  Ports: ${ports:-None}"
    
    # Extract volumes
    local volumes=$(jq -r '.volumes[] | "\(.host_path):\(.container_path):\(.mode)"' "$CONFIG_FILE" 2>/dev/null | tr '\n' ',' | sed 's/,$//')
    echo "  Volumes: ${volumes:-None}"
    
    # Create container with extracted configuration
    echo ""
    echo "🚀 Creating container..."
    
    multios-container create \
        --name "$name" \
        --cpu-limit "$cpu_limit" \
        --memory-limit "$memory_limit" \
        --disk-limit "$disk_limit" \
        --image "$image" \
        "$name" || {
        echo "⚠️  Container might already exist, trying to use existing one"
    }
}

# Configure container networking
configure_networking() {
    echo ""
    echo "🌐 Configuring networking..."
    
    # Get container name from config or use default
    local name=$(jq -r '.name // "'"$CONTAINER_NAME"'"' "$CONFIG_FILE")
    
    # Configure port forwarding
    local port_configs=$(jq -r '.networking.ports[]' "$CONFIG_FILE" 2>/dev/null)
    if [[ -n "$port_configs" ]]; then
        echo "📡 Configuring port forwarding:"
        
        while IFS= read -r port_config; do
            if [[ -n "$port_config" && "$port_config" != "null" ]]; then
                local container_port=$(echo "$port_config" | jq -r '.container_port')
                local host_port=$(echo "$port_config" | jq -r '.host_port')
                local description=$(echo "$port_config" | jq -r '.description // "Port"')
                
                echo "  Mapping $container_port -> $host_port ($description)"
                # Note: Port mapping is typically done at container creation time
                # This is for demonstration purposes
            fi
        done <<< "$port_configs"
    fi
}

# Set up volumes
configure_volumes() {
    echo ""
    echo "💾 Configuring volumes..."
    
    local volume_configs=$(jq -r '.volumes[]' "$CONFIG_FILE" 2>/dev/null")
    if [[ -n "$volume_configs" ]]; then
        echo "📁 Setting up volume mounts:"
        
        while IFS= read -r volume_config; do
            if [[ -n "$volume_config" && "$volume_config" != "null" ]]; then
                local host_path=$(echo "$volume_config" | jq -r '.host_path')
                local container_path=$(echo "$volume_config" | jq -r '.container_path')
                local mode=$(echo "$volume_config" | jq -r '.mode // "rw"')
                local description=$(echo "$volume_config" | jq -r '.description // "Volume"')
                
                # Create host directory if it doesn't exist
                if [[ ! -d "$host_path" ]]; then
                    echo "  Creating host directory: $host_path"
                    mkdir -p "$host_path"
                fi
                
                echo "  Mounting $host_path -> $container_path ($mode) - $description"
                # Note: Volume mounting is typically done at container creation time
            fi
        done <<< "$volume_configs"
    fi
}

# Set environment variables
configure_environment() {
    echo ""
    echo "🔧 Configuring environment variables..."
    
    local env_vars=$(jq -r '.environment | to_entries[] | "\(.key)=\(.value)"' "$CONFIG_FILE" 2>/dev/null")
    if [[ -n "$env_vars" ]]; then
        echo "🌍 Environment variables:"
        while IFS= read -r env_var; do
            if [[ -n "$env_var" ]]; then
                echo "  $env_var"
            fi
        done <<< "$env_vars"
    fi
}

# Start the container
start_container() {
    echo ""
    echo "▶️  Starting container..."
    
    local name=$(jq -r '.name // "'"$CONTAINER_NAME"'"' "$CONFIG_FILE")
    
    multios-container start "$name"
    
    echo ""
    echo "📊 Container status:"
    multios-container ps | grep "$name"
}

# Demonstrate container operations
test_container() {
    echo ""
    echo "🧪 Testing container functionality..."
    
    local name=$(jq -r '.name // "'"$CONTAINER_NAME"'"' "$CONFIG_FILE")
    
    # Test if container is running
    if multios-container exec "$name" echo "Container is responding" > /dev/null 2>&1; then
        echo "✅ Container is responding to commands"
        
        # Show hostname
        echo "🏷️  Hostname in container:"
        multios-container exec "$name" hostname
        
        # Test Python if available
        if multios-container exec "$name" which python3 > /dev/null 2>&1; then
            echo "🐍 Python version:"
            multios-container exec "$name" python3 --version
        fi
        
        # Show running processes
        echo "🔄 Running processes:"
        multios-container exec "$name" ps aux | head -5
        
        # Show system information
        echo "💻 System information:"
        multios-container exec "$name" uname -a | head -1
        
        # Test network connectivity
        echo "🌐 Network connectivity test:"
        multios-container exec "$name" ping -c 1 8.8.8.8 > /dev/null 2>&1 && echo "  ✅ Network is working" || echo "  ⚠️  Network test failed"
        
    else
        echo "❌ Container is not responding"
        return 1
    fi
}

# Show container statistics
show_statistics() {
    echo ""
    echo "📈 Container statistics:"
    
    local name=$(jq -r '.name // "'"$CONTAINER_NAME"'"' "$CONFIG_FILE")
    
    multios-container stats "$name" 2>/dev/null || {
        echo "  Container stats not available"
        echo "  Container details:"
        multios-container inspect "$name" | head -20
    }
}

# Cleanup
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    
    local name=$(jq -r '.name // "'"$CONTAINER_NAME"'"' "$CONFIG_FILE")
    
    echo "⏹️  Stopping container..."
    multios-container stop "$name" 2>/dev/null || echo "  Container was not running"
    
    echo "🗑️  Removing container..."
    multios-container delete "$name" 2>/dev/null || echo "  Container could not be removed"
    
    echo "✅ Cleanup completed"
}

# Main execution
main() {
    echo "🚀 Starting MultiOS Container Configuration Demo..."
    
    # Pre-flight checks
    check_root
    check_daemon
    
    # Configure container
    create_from_config
    configure_networking
    configure_volumes
    configure_environment
    
    # Start and test
    start_container
    test_container
    show_statistics
    
    # Cleanup
    echo ""
    read -p "🗑️  Remove container? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup
    else
        echo "📋 Container '$CONTAINER_NAME' is still running"
        echo "   To stop it manually, run: multios-container stop $CONTAINER_NAME"
        echo "   To remove it manually, run: multios-container delete $CONTAINER_NAME"
    fi
    
    echo ""
    echo "=========================================="
    echo "🎉 Configuration Demo Complete!"
    echo "=========================================="
    echo ""
    echo "This demo showed:"
    echo "• How to use JSON configuration files"
    echo "• Resource allocation and limits"
    echo "• Port forwarding and networking"
    echo "• Volume mounting and persistence"
    echo "• Environment variable configuration"
    echo ""
    echo "📖 For more information, see:"
    echo "• Configuration reference: README.md"
    echo "• Python learning example: python-learning.md"
    echo "• Full documentation: /workspace/system_features/containers/README.md"
}

# Error handling
trap 'echo ""; echo "❌ Script failed at line $LINENO"; exit 1' ERR

# Run main function
main "$@"