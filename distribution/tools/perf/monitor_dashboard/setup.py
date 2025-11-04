#!/usr/bin/env python3
"""
Setup Script for Performance Monitoring Dashboard
Automatically sets up the environment and dependencies
"""

import os
import sys
import subprocess
import platform
from pathlib import Path

def print_banner():
    """Print setup banner"""
    banner = """
╔══════════════════════════════════════════════════════════════╗
║                Performance Monitoring Dashboard               ║
║                        Setup Wizard                          ║
╠══════════════════════════════════════════════════════════════╣
║  Comprehensive real-time performance monitoring system       ║
║  with web dashboard, CLI tools, and alerting capabilities   ║
╚══════════════════════════════════════════════════════════════╝
    """
    print(banner)

def check_python_version():
    """Check if Python version is compatible"""
    if sys.version_info < (3, 8):
        print("❌ Python 3.8 or higher is required")
        print(f"   Current version: {sys.version}")
        return False
    
    print(f"✅ Python {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro} detected")
    return True

def check_system():
    """Check system compatibility"""
    system = platform.system()
    if system == "Windows":
        print("⚠️  Windows detected - some features may work differently")
    elif system == "Darwin":
        print("✅ macOS detected")
    elif system == "Linux":
        print("✅ Linux detected")
    else:
        print(f"⚠️  Unknown system: {system}")
    
    return True

def create_directory_structure():
    """Create necessary directories"""
    directories = [
        'data',
        'logs',
        'reports',
        'config',
        'backups',
        'frontend/perf-dashboard/dist',
        'backend/templates',
        'backend/static'
    ]
    
    print("\n📁 Creating directory structure...")
    
    for directory in directories:
        path = Path(directory)
        path.mkdir(parents=True, exist_ok=True)
        print(f"   ✅ Created: {directory}")

def install_backend_dependencies():
    """Install Python dependencies"""
    print("\n🐍 Installing Python dependencies...")
    
    requirements_file = Path("backend/requirements.txt")
    if not requirements_file.exists():
        print("❌ Requirements file not found: backend/requirements.txt")
        return False
    
    try:
        cmd = [sys.executable, "-m", "pip", "install", "-r", str(requirements_file)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode == 0:
            print("✅ Python dependencies installed successfully")
            return True
        else:
            print("❌ Failed to install Python dependencies")
            print(f"Error: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"❌ Error installing dependencies: {e}")
        return False

def install_frontend_dependencies():
    """Install Node.js dependencies and build frontend"""
    print("\n🌐 Setting up frontend...")
    
    # Check if Node.js is available
    try:
        result = subprocess.run(['node', '--version'], capture_output=True, text=True)
        if result.returncode != 0:
            print("⚠️  Node.js not found - frontend will use simple HTML")
            return True
    except FileNotFoundError:
        print("⚠️  Node.js not found - frontend will use simple HTML")
        return True
    
    frontend_dir = Path("frontend/perf-dashboard")
    if not frontend_dir.exists():
        print("⚠️  Frontend directory not found")
        return True
    
    try:
        # Install dependencies
        print("   Installing npm dependencies...")
        result = subprocess.run(['npm', 'install'], cwd=frontend_dir, capture_output=True, text=True)
        
        if result.returncode != 0:
            print("⚠️  Failed to install npm dependencies, but continuing...")
            return True
        
        # Build the frontend
        print("   Building frontend...")
        result = subprocess.run(['npm', 'run', 'build'], cwd=frontend_dir, capture_output=True, text=True)
        
        if result.returncode != 0:
            print("⚠️  Frontend build failed, but continuing...")
            return True
        
        print("✅ Frontend built successfully")
        return True
        
    except Exception as e:
        print(f"⚠️  Frontend setup issue: {e}")
        print("   Dashboard will work with simple HTML fallback")
        return True

def create_default_config():
    """Create default configuration file"""
    print("\n⚙️  Creating default configuration...")
    
    config_file = Path("config/config.yaml")
    if config_file.exists():
        print("   Configuration file already exists, skipping...")
        return True
    
    try:
        import shutil
        default_config = Path("config/config.yaml")
        if default_config.exists():
            print("✅ Default configuration created")
        else:
            print("⚠️  Default configuration file not found")
        
        return True
        
    except Exception as e:
        print(f"⚠️  Configuration setup issue: {e}")
        return True

def create_startup_scripts():
    """Create convenient startup scripts"""
    print("\n📝 Creating startup scripts...")
    
    scripts = []
    
    # Windows batch file
    if platform.system() == "Windows":
        batch_content = """@echo off
cd /d "%~dp0"
echo Starting Performance Monitoring Dashboard...
python start_dashboard.py --mode web
pause
"""
        scripts.append(("start_dashboard.bat", batch_content))
    
    # Unix shell script
    shell_content = """#!/bin/bash
cd "$(dirname "$0")"
echo "Starting Performance Monitoring Dashboard..."
python3 start_dashboard.py --mode web
"""
    scripts.append(("start_dashboard.sh", shell_content))
    
    for script_name, content in scripts:
        script_path = Path(script_name)
        try:
            with open(script_path, 'w') as f:
                f.write(content)
            
            # Make executable on Unix systems
            if platform.system() != "Windows":
                os.chmod(script_path, 0o755)
            
            print(f"   ✅ Created: {script_name}")
        except Exception as e:
            print(f"   ⚠️  Failed to create {script_name}: {e}")
    
    return True

def test_installation():
    """Test the installation"""
    print("\n🧪 Testing installation...")
    
    # Test Python imports
    try:
        sys.path.insert(0, 'backend')
        import psutil
        print("   ✅ psutil import successful")
        
        import yaml
        print("   ✅ yaml import successful")
        
        import flask
        print("   ✅ flask import successful")
        
        print("✅ Core Python modules available")
        
    except ImportError as e:
        print(f"   ❌ Import error: {e}")
        return False
    
    # Test configuration
    try:
        from backend.config_manager import ConfigManager
        config = ConfigManager()
        print("   ✅ Configuration manager working")
    except Exception as e:
        print(f"   ⚠️  Configuration issue: {e}")
    
    print("✅ Installation test completed")
    return True

def print_usage_instructions():
    """Print usage instructions"""
    print("\n" + "="*60)
    print("🎉 INSTALLATION COMPLETE!")
    print("="*60)
    
    print("\n📚 Quick Start:")
    print("   1. Start the web dashboard:")
    print("      python start_dashboard.py --mode web")
    print("\n   2. Or use the startup script:")
    if platform.system() == "Windows":
        print("      start_dashboard.bat")
    else:
        print("      ./start_dashboard.sh")
    
    print("\n   3. Access the dashboard:")
    print("      http://localhost:5000")
    
    print("\n🛠️  CLI Tools:")
    print("   python cli/monitor_cli.py monitor --interval 2")
    print("   python cli/monitor_cli.py status")
    print("   python cli/monitor_cli.py alerts --hours 24")
    
    print("\n📁 Important Files:")
    print("   • config/config.yaml - Configuration")
    print("   • data/monitor.db - Database (created automatically)")
    print("   • logs/monitor.log - Log files")
    print("   • reports/ - Generated reports")
    
    print("\n🔧 Configuration:")
    print("   Edit config/config.yaml to customize:")
    print("   • Monitoring intervals")
    print("   • Alert thresholds")
    print("   • Notification settings")
    
    print("\n📖 Documentation:")
    print("   See README.md for detailed documentation")
    
    print("\n🆘 Need Help?")
    print("   • Check logs/monitor.log for errors")
    print("   • Run: python cli/monitor_cli.py status")
    print("   • Review configuration: python -c 'from backend.config_manager import ConfigManager; print(ConfigManager().config)'")

def main():
    """Main setup function"""
    print_banner()
    
    # Run setup steps
    steps = [
        ("Checking Python version", check_python_version),
        ("Checking system", check_system),
        ("Creating directories", create_directory_structure),
        ("Installing backend dependencies", install_backend_dependencies),
        ("Setting up frontend", install_frontend_dependencies),
        ("Creating configuration", create_default_config),
        ("Creating startup scripts", create_startup_scripts),
        ("Testing installation", test_installation)
    ]
    
    failed_steps = []
    
    for step_name, step_function in steps:
        print(f"\n🔧 {step_name}...")
        try:
            if not step_function():
                failed_steps.append(step_name)
                print(f"⚠️  {step_name} completed with warnings")
        except Exception as e:
            failed_steps.append(step_name)
            print(f"❌ {step_name} failed: {e}")
    
    # Summary
    print("\n" + "="*60)
    if failed_steps:
        print("⚠️  Setup completed with some issues:")
        for step in failed_steps:
            print(f"   • {step}")
        print("\nThe dashboard should still work, but some features may be limited.")
    else:
        print("✅ Setup completed successfully!")
    
    print_usage_instructions()

if __name__ == "__main__":
    main()