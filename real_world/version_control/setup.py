#!/usr/bin/env python3
"""
Educational VCS Setup Script
Sets up and runs the complete educational version control system
"""

import os
import sys
import subprocess
import time
import signal
from pathlib import Path

def print_banner():
    """Print welcome banner"""
    print("""
╔══════════════════════════════════════════════════════════════╗
║                    Educational VCS Setup                     ║
║            Collaborative Version Control System              ║
║                                                              ║
║  🚀 Features:                                                ║
║     • Git-like version control with branches & merges       ║
║     • Real-time collaborative editing                       ║
║     • Code review and approval workflows                    ║
║     • Assignment submission and grading                     ║
║     • Automated code quality analysis                       ║
║     • Interactive educational tutorials                     ║
╚══════════════════════════════════════════════════════════════╝
    """)

def check_python_version():
    """Check if Python version is compatible"""
    version = sys.version_info
    if version.major < 3 or (version.major == 3 and version.minor < 8):
        print("❌ Error: Python 3.8 or higher is required")
        print(f"   Current version: {version.major}.{version.minor}.{version.micro}")
        sys.exit(1)
    print(f"✅ Python version {version.major}.{version.minor}.{version.micro} is compatible")

def install_python_dependencies():
    """Install required Python packages"""
    print("\n📦 Installing Python dependencies...")
    
    packages = [
        'flask',
        'flask-cors', 
        'flask-socketio',
        'python-socketio',
        'analyzedifflib'
    ]
    
    for package in packages:
        try:
            print(f"   Installing {package}...")
            subprocess.run([sys.executable, '-m', 'pip', 'install', package], 
                         check=True, capture_output=True)
            print(f"   ✅ {package} installed successfully")
        except subprocess.CalledProcessError as e:
            print(f"   ❌ Failed to install {package}: {e}")
            return False
    
    return True

def check_node_npm():
    """Check if Node.js and npm are available"""
    try:
        node_result = subprocess.run(['node', '--version'], 
                                   capture_output=True, text=True, check=True)
        npm_result = subprocess.run(['npm', '--version'], 
                                  capture_output=True, text=True, check=True)
        
        print(f"✅ Node.js version {node_result.stdout.strip()} detected")
        print(f"✅ npm version {npm_result.stdout.strip()} detected")
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ Node.js or npm not found")
        print("   Please install Node.js from https://nodejs.org/")
        return False

def install_client_dependencies():
    """Install client dependencies"""
    print("\n📦 Installing client dependencies...")
    
    client_dir = Path(__file__).parent / "src" / "client" / "educational-vcs-client"
    
    if not client_dir.exists():
        print("❌ Client directory not found")
        return False
    
    try:
        print("   Running npm install...")
        subprocess.run(['npm', 'install'], cwd=client_dir, check=True)
        print("   ✅ Client dependencies installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"   ❌ Failed to install client dependencies: {e}")
        return False

def create_demo_directory():
    """Create demo repository directory"""
    demo_dir = Path("/workspace/demo-repos")
    demo_dir.mkdir(exist_ok=True)
    
    # Create a sample repository
    sample_repo = demo_dir / "sample-project"
    sample_repo.mkdir(exist_ok=True)
    
    # Create some sample files
    (sample_repo / "README.md").write_text("""# Sample Educational Project

This is a sample project for demonstrating the Educational VCS system.

## Features
- Version control
- Collaborative editing
- Code reviews
- Quality analysis

## Getting Started
1. Clone the repository
2. Make changes
3. Commit your work
4. Submit for review
""")
    
    (sample_repo / "main.py").write_text("""#!/usr/bin/env python3
\"\"\"
Main application file
\"\"\"

def hello_world():
    \"\"\"Print hello world message\"\"\"
    print("Hello, Educational VCS!")
    return "Hello, World!"

if __name__ == "__main__":
    result = hello_world()
    print(f"Function returned: {result}")
""")
    
    (sample_repo / ".gitignore").write_text("""__pycache__/
*.pyc
*.pyo
*.pyd
.Python
env/
venv/
.venv/
pip-log.txt
.env
.DS_Store
""")
    
    print("   ✅ Demo directory created")
    return True

def run_backend_server(port=5000):
    """Start the backend Flask server"""
    print(f"\n🚀 Starting backend server on port {port}...")
    
    backend_file = Path(__file__).parent / "src" / "server" / "api.py"
    
    if not backend_file.exists():
        print("❌ Backend server file not found")
        return None
    
    try:
        # Set environment variables
        env = os.environ.copy()
        env['FLASK_ENV'] = 'development'
        env['FLASK_DEBUG'] = '1'
        
        process = subprocess.Popen([
            sys.executable, str(backend_file)
        ], env=env)
        
        # Wait a moment for server to start
        time.sleep(2)
        
        print("   ✅ Backend server started successfully")
        print(f"   📡 Server running at http://localhost:{port}")
        print(f"   📊 API endpoints available at http://localhost:{port}/api/")
        print(f"   🔌 WebSocket server running for real-time collaboration")
        
        return process
    except Exception as e:
        print(f"   ❌ Failed to start backend server: {e}")
        return None

def run_frontend_server(port=3000):
    """Start the React frontend server"""
    print(f"\n🎨 Starting frontend server on port {port}...")
    
    client_dir = Path(__file__).parent / "src" / "client" / "educational-vcs-client"
    
    if not client_dir.exists():
        print("❌ Client directory not found")
        return None
    
    try:
        # Set environment variables for React
        env = os.environ.copy()
        env['PORT'] = str(port)
        env['REACT_APP_API_URL'] = f'http://localhost:5000/api'
        env['REACT_APP_SOCKET_URL'] = 'http://localhost:5000'
        
        process = subprocess.Popen([
            'npm', 'start'
        ], cwd=client_dir, env=env)
        
        # Wait for server to start
        time.sleep(5)
        
        print("   ✅ Frontend server started successfully")
        print(f"   🌐 Application running at http://localhost:{port}")
        print(f"   🔗 Connected to backend at http://localhost:5000")
        
        return process
    except Exception as e:
        print(f"   ❌ Failed to start frontend server: {e}")
        return None

def print_usage_instructions():
    """Print instructions for using the system"""
    print("""
╔══════════════════════════════════════════════════════════════╗
║                        🎉 Setup Complete!                    ║
╚══════════════════════════════════════════════════════════════╝

📱 Access the application:
   Web Interface: http://localhost:3000
   API Server:    http://localhost:5000/api/

🚀 Quick Start:
   1. Open http://localhost:3000 in your browser
   2. A demo user will be automatically created
   3. Create or select a repository from the sidebar
   4. Start coding in the collaborative editor
   5. Make commits and learn version control!

📚 Educational Features:
   • Interactive tutorials in the "Tutorials" section
   • Code quality analysis with educational feedback
   • Peer code review system
   • Assignment submission and grading
   • Real-time collaborative editing

🎯 Demo Scenarios:
   1. Student Workflow:
      - Create repository → Write code → Commit → Submit assignment
   2. Collaboration:
      - Multiple users can edit the same file simultaneously
   3. Code Review:
      - Create pull requests and review code with comments
   4. Quality Analysis:
      - Analyze code for educational feedback and improvements

⚠️  Tips:
   - Use the sidebar navigation to explore all features
   - Check the Student Dashboard for progress tracking
   - Try the Quality Analysis tool for code feedback
   - Complete tutorials to learn version control concepts

🔧 Development:
   - Backend code: src/server/api.py
   - Frontend code: src/client/educational-vcs-client/
   - Core VCS: src/core/

💡 Need Help?
   - Check the README.md for detailed documentation
   - All components include educational comments
   - Error messages provide learning hints

Happy learning and coding! 🎓✨
    """)

def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully"""
    print("\n\n🛑 Shutting down servers...")
    sys.exit(0)

def main():
    """Main setup and run function"""
    signal.signal(signal.SIGINT, signal_handler)
    
    print_banner()
    
    # Check prerequisites
    print("🔍 Checking prerequisites...")
    check_python_version()
    
    if not check_node_npm():
        print("\n⚠️  Warning: Node.js not found. Frontend won't be available.")
        print("   Install Node.js from https://nodejs.org/ for full functionality")
    
    # Install dependencies
    print("\n📦 Installing dependencies...")
    if not install_python_dependencies():
        print("❌ Failed to install Python dependencies")
        sys.exit(1)
    
    if check_node_npm():
        if not install_client_dependencies():
            print("❌ Failed to install client dependencies")
            sys.exit(1)
    
    # Setup demo environment
    print("\n🏗️  Setting up demo environment...")
    create_demo_directory()
    
    # Start servers
    print("\n" + "="*60)
    
    backend_process = None
    frontend_process = None
    
    try:
        # Start backend
        backend_process = run_backend_server()
        
        if check_node_npm():
            # Start frontend (wait a bit for backend to fully start)
            time.sleep(2)
            frontend_process = run_frontend_server()
        
        # Print usage instructions
        print_usage_instructions()
        
        # Keep the script running
        print("\n🟢 Servers are running... Press Ctrl+C to stop")
        print("\n" + "="*60)
        
        # Wait for processes
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            pass
            
    except Exception as e:
        print(f"\n❌ Error during execution: {e}")
    
    finally:
        # Clean up processes
        print("\n🛑 Cleaning up...")
        if backend_process:
            backend_process.terminate()
            backend_process.wait()
            print("   ✅ Backend server stopped")
        
        if frontend_process:
            frontend_process.terminate()
            frontend_process.wait()
            print("   ✅ Frontend server stopped")
        
        print("\n👋 Thanks for using Educational VCS!")

if __name__ == "__main__":
    main()
