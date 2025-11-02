#!/bin/bash

# MultiOS Kernel Internals Visualization System Demo Script
# This script demonstrates the capabilities of the visualization system

echo "=========================================="
echo "MultiOS Kernel Internals Visualization System"
echo "Demo Script"
echo "=========================================="
echo ""

# Check if project exists
if [ ! -d "/workspace/education/visualization/kernel-visualization" ]; then
    echo "❌ Project directory not found!"
    echo "Please ensure the project has been properly initialized."
    exit 1
fi

cd /workspace/education/visualization/kernel-visualization

echo "✅ Project structure verified"
echo ""

# Show project structure
echo "📁 Project Structure:"
echo "===================="
find . -type f -name "*.tsx" -o -name "*.ts" -o -name "*.css" -o -name "*.json" | head -20
echo "..."
echo ""

# Check if node_modules exists
if [ -d "node_modules" ]; then
    echo "✅ Dependencies installed"
else
    echo "⚠️  Dependencies not installed - run 'pnpm install'"
fi

# Show installed packages
echo ""
echo "📦 Key Dependencies:"
echo "==================="
if [ -f "package.json" ]; then
    grep -E '"(react|d3|@types)"' package.json | head -10
fi
echo ""

# Display visualization components
echo "🎨 Visualization Components:"
echo "==========================="
echo "1. Memory Map Visualization - Real-time memory allocation tracking"
echo "2. Process Tree Visualization - Interactive parent-child process hierarchy"
echo "3. CPU Scheduler Visualization - Multi-core processing with load balancing"
echo "4. File System Visualization - File system hierarchy with inode tracking"
echo "5. Network Stack Visualization - Network connections and protocol analysis"
echo "6. Kernel Module Graph - Module dependency relationships"
echo "7. System Call Flow - System call tracking with execution flow"
echo "8. Performance Overlay - Comprehensive performance metrics"
echo ""

# Show feature highlights
echo "🌟 Key Features:"
echo "==============="
echo "• Real-time data updates with customizable refresh rates"
echo "• Interactive visualizations with D3.js and force-directed graphs"
echo "• Responsive design for desktop, tablet, and mobile"
echo "• Search and filter capabilities across all data types"
echo "• Performance alerts and threshold monitoring"
echo "• Export functionality for data and visualizations"
echo "• Dark theme optimized for monitoring interfaces"
echo "• Comprehensive system metrics overlay"
echo ""

# Show technology stack
echo "🛠 Technology Stack:"
echo "==================="
echo "Frontend: React 18 + TypeScript"
echo "Build Tool: Vite 6.0"
echo "Styling: Tailwind CSS"
echo "Visualizations: D3.js, React Force Graph 2D"
echo "UI Components: Radix UI"
echo "Package Manager: pnpm"
echo ""

# Show usage instructions
echo "🚀 Usage Instructions:"
echo "====================="
echo "1. Install dependencies: pnpm install"
echo "2. Start development server: pnpm dev"
echo "3. Open browser to: http://localhost:5173"
echo "4. Navigate through tabs to explore visualizations"
echo "5. Use controls to filter, search, and interact"
echo ""

# Show project size
echo "📊 Project Statistics:"
echo "====================="
file_count=$(find src -type f | wc -l)
total_size=$(du -sh . | cut -f1)
echo "Total files: $file_count"
echo "Project size: $total_size"
echo ""

# Show architecture summary
echo "🏗 Architecture Overview:"
echo "========================"
echo "• Modular component-based architecture"
echo "• Real-time data simulation with configurable intervals"
echo "• Efficient D3.js rendering with canvas fallback"
echo "• Responsive grid layouts with Tailwind CSS"
echo "• Type-safe development with TypeScript"
echo "• Performance optimized with virtual scrolling"
echo ""

echo "=========================================="
echo "System ready for demonstration!"
echo "Start with: cd /workspace/education/visualization/kernel-visualization && pnpm dev"
echo "=========================================="