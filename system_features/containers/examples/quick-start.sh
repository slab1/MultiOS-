#!/bin/bash
# MultiOS Container System - Quick Start Script
# This script demonstrates basic container operations

set -e  # Exit on any error

echo "=========================================="
echo "MultiOS Container System - Quick Start"
echo "=========================================="

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo "❌ This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to check if multios-containerd is running
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

# Function to demonstrate basic container operations
demo_basic_operations() {
    echo ""
    echo "🔍 Listing available templates..."
    multios-container template list
    
    echo ""
    echo "📦 Creating Python learning container..."
    multios-container create --template python-learning demo-python || {
        echo "❌ Failed to create container"
        return 1
    }
    
    echo ""
    echo "▶️  Starting container..."
    multios-container start demo-python
    
    echo ""
    echo "🔍 Container status:"
    multios-container ps
    
    echo ""
    echo "🐍 Testing Python environment..."
    multios-container exec demo-python python3 --version
    multios-container exec demo-python python3 -c "print('Hello from MultiOS Container!')"
    
    echo ""
    echo "📊 Container information:"
    multios-container inspect demo-python | head -20
    
    echo ""
    echo "📋 Container logs:"
    multios-container logs demo-python
    
    echo ""
    echo "⏹️  Stopping container..."
    multios-container stop demo-python
    
    echo ""
    echo "🗑️  Removing container..."
    multios-container delete demo-python
    
    echo ""
    echo "✅ Basic operations demo completed!"
}

# Function to demonstrate resource management
demo_resource_management() {
    echo ""
    echo "=========================================="
    echo "Resource Management Demo"
    echo "=========================================="
    
    echo ""
    echo "📦 Creating container with resource limits..."
    multios-container create \
        --template python-learning \
        --cpu-limit 2 \
        --memory-limit 1G \
        --disk-limit 5G \
        resource-demo
    
    echo ""
    echo "▶️  Starting container..."
    multios-container start resource-demo
    
    echo ""
    echo "📊 Resource usage:"
    multios-container stats resource-demo
    
    echo ""
    echo "🔧 Updating resource limits..."
    multios-container update resource-demo --memory-limit 2G --cpu-limit 1
    
    echo ""
    echo "📊 Updated resource usage:"
    multios-container stats resource-demo
    
    echo ""
    echo "⏹️  Cleaning up..."
    multios-container stop resource-demo
    multios-container delete resource-demo
    
    echo ""
    echo "✅ Resource management demo completed!"
}

# Function to demonstrate networking
demo_networking() {
    echo ""
    echo "=========================================="
    echo "Networking Demo"
    echo "=========================================="
    
    echo ""
    echo "🌐 Listing networks..."
    multios-container network list
    
    echo ""
    echo "🌉 Creating custom network..."
    multios-container network create demo-network --subnet 172.25.0.0/16 || {
        echo "⚠️  Network might already exist"
    }
    
    echo ""
    echo "📦 Creating container in custom network..."
    multios-container create \
        --template python-learning \
        --network demo-network \
        network-demo
    
    echo ""
    echo "▶️  Starting container..."
    multios-container start network-demo
    
    echo ""
    echo "🌐 Container network info:"
    multios-container inspect network-demo | grep -A 10 NetworkSettings
    
    echo ""
    echo "🔗 Testing network connectivity..."
    multios-container exec network-demo ping -c 1 8.8.8.8
    
    echo ""
    echo "⏹️  Cleaning up..."
    multios-container stop network-demo
    multios-container delete network-demo
    multios-container network rm demo-network || {
        echo "⚠️  Could not remove network (might be in use)"
    }
    
    echo ""
    echo "✅ Networking demo completed!"
}

# Function to demonstrate educational templates
demo_educational_templates() {
    echo ""
    echo "=========================================="
    echo "Educational Templates Demo"
    echo "=========================================="
    
    # Create a few different containers to showcase templates
    local templates=("python-learning" "cpp-learning")
    
    for template in "${templates[@]}"; do
        echo ""
        echo "📦 Creating container from $template template..."
        multios-container create --template "$template" "demo-${template}"
        
        echo ""
        echo "▶️  Starting container..."
        multios-container start "demo-${template}"
        
        echo ""
        echo "🔍 Testing container environment..."
        case $template in
            "python-learning")
                multios-container exec "demo-${template}" python3 --version
                multios-container exec "demo-${template}" python3 -c "print('Python is working!')"
                ;;
            "cpp-learning")
                multios-container exec "demo-${template}" gcc --version
                multios-container exec "demo-${template}" bash -c 'echo "#include <iostream>" > test.cpp && echo "int main() { std::cout << \"C++ is working!\" << std::endl; return 0; }" >> test.cpp && g++ test.cpp && ./a.out'
                ;;
        esac
        
        echo ""
        echo "⏹️  Stopping and removing container..."
        multios-container stop "demo-${template}"
        multios-container delete "demo-${template}"
    done
    
    echo ""
    echo "✅ Educational templates demo completed!"
}

# Function to show system information
show_system_info() {
    echo ""
    echo "=========================================="
    echo "System Information"
    echo "=========================================="
    
    echo ""
    echo "🖥️  Host system:"
    uname -a
    
    echo ""
    echo "🐳 Container daemon:"
    multios-container --version
    
    echo ""
    echo "📊 Current containers:"
    multios-container ps
    
    echo ""
    echo "🌐 Available images:"
    multios-container images 2>/dev/null || echo "No images found"
    
    echo ""
    echo "🔌 System resources:"
    echo "Memory: $(free -h | grep '^Mem:' | awk '{print $2}') total"
    echo "CPU: $(nproc) cores"
    echo "Disk: $(df -h / | tail -1 | awk '{print $4}') available"
}

# Main execution
main() {
    echo "🚀 Starting MultiOS Container System Demo..."
    
    # Pre-flight checks
    check_root
    check_daemon
    show_system_info
    
    # Run demos
    demo_basic_operations
    demo_resource_management
    demo_networking
    demo_educational_templates
    
    echo ""
    echo "=========================================="
    echo "🎉 Demo Complete!"
    echo "=========================================="
    echo ""
    echo "You can now explore more features:"
    echo "• Try other educational templates: nodejs-learning, java-learning, web-learning"
    echo "• Use multios-container exec for interactive sessions"
    echo "• Check multios-container logs for troubleshooting"
    echo "• Experiment with custom configurations and resource limits"
    echo ""
    echo "For more examples, see: /workspace/system_features/containers/examples/python-learning.md"
    echo "For full documentation, see: /workspace/system_features/containers/README.md"
}

# Error handling
trap 'echo ""; echo "❌ Script interrupted or failed at line $LINENO"; exit 1' ERR

# Run main function
main "$@"