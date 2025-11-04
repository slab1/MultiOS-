#!/bin/bash

# Interactive Code Browser - Simple Startup Script
# This script starts a lightweight version using Python's built-in server

echo "🏗️  Starting Interactive Code Browser..."

# Check if Python is available
if command -v python3 &> /dev/null; then
    echo "✅ Python 3 found - using built-in HTTP server"
    PORT=3000
    echo "🌐 Starting server on http://localhost:$PORT"
    echo "📱 Open your browser and navigate to: http://localhost:$PORT"
    echo "🛑 Press Ctrl+C to stop the server"
    echo ""
    cd /workspace/education/code_browser
    python3 -m http.server $PORT
elif command -v python &> /dev/null; then
    echo "✅ Python found - using built-in HTTP server"
    PORT=3000
    echo "🌐 Starting server on http://localhost:$PORT"
    echo "📱 Open your browser and navigate to: http://localhost:$PORT"
    echo "🛑 Press Ctrl+C to stop the server"
    echo ""
    cd /workspace/education/code_browser
    python -m SimpleHTTPServer $PORT
else
    echo "❌ Python not found. Please install Python 3 or open index.html directly in your browser."
    echo "📁 You can open /workspace/education/code_browser/index.html directly in your browser."
fi