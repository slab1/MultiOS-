#!/bin/bash

# Performance Visualization Tools Setup Script

echo "🎯 Performance Visualization Tools Setup"
echo "========================================"

# Check Python version
echo "Checking Python version..."
python_version=$(python3 --version 2>&1 | grep -o '[0-9]\+\.[0-9]\+' | head -1)
python_major=$(echo $python_version | cut -d. -f1)
python_minor=$(echo $python_version | cut -d. -f2)

if [ "$python_major" -eq 3 ] && [ "$python_minor" -ge 8 ]; then
    echo "✅ Python version: $python_version (Compatible)"
else
    echo "❌ Python version $python_version found. Requires Python 3.8+"
    echo "Please install Python 3.8 or higher"
    exit 1
fi

# Install Python dependencies
echo ""
echo "Installing Python dependencies..."
pip3 install -r requirements.txt

if [ $? -eq 0 ]; then
    echo "✅ Dependencies installed successfully"
else
    echo "❌ Failed to install dependencies"
    exit 1
fi

# Make CLI tools executable
echo ""
echo "Setting up executable permissions..."
chmod +x cli/performance_cli.py
chmod +x cli/performance_monitor.py
chmod +x cli/performance_analyzer.py
chmod +x cli/performance_visualizer.py
chmod +x demo.py

echo "✅ Executable permissions set"

# Create necessary directories
echo ""
echo "Creating directories..."
mkdir -p data output
echo "✅ Directories created"

# Verify installation
echo ""
echo "Verifying installation..."
python3 -c "import psutil, pandas, matplotlib, seaborn; print('✅ All dependencies verified')"

if [ $? -eq 0 ]; then
    echo "✅ Installation completed successfully!"
else
    echo "❌ Verification failed"
    exit 1
fi

# Show next steps
echo ""
echo "🎉 Setup Complete!"
echo "=================="
echo ""
echo "📚 Quick Start:"
echo "1. Run demo: python3 demo.py interactive"
echo "2. Monitor system: python3 cli/performance_cli.py monitor --duration 60"
echo "3. Start dashboard: python3 cli/performance_cli.py dashboard"
echo "4. Analyze data: python3 cli/performance_cli.py analyze your_data.json --stats"
echo ""
echo "📖 Documentation: See docs/README.md for detailed usage"
echo ""
echo "🚀 Ready to monitor your system performance!"

exit 0