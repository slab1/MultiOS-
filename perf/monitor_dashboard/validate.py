#!/usr/bin/env python3
"""
Performance Dashboard Validation Script
Validates the installation and configuration
"""

import os
import sys
import json
import time
from pathlib import Path

def print_header():
    """Print validation header"""
    print("=" * 60)
    print("Performance Monitoring Dashboard - Validation")
    print("=" * 60)

def check_directory_structure():
    """Check if required directories exist"""
    print("\n📁 Checking directory structure...")
    
    required_dirs = [
        'data',
        'logs', 
        'reports',
        'config',
        'backend',
        'frontend/perf-dashboard',
        'cli'
    ]
    
    missing_dirs = []
    for dir_path in required_dirs:
        if not Path(dir_path).exists():
            missing_dirs.append(dir_path)
            print(f"   ❌ Missing: {dir_path}")
        else:
            print(f"   ✅ Found: {dir_path}")
    
    return len(missing_dirs) == 0

def check_required_files():
    """Check if required files exist"""
    print("\n📄 Checking required files...")
    
    required_files = [
        'backend/system_monitor.py',
        'backend/web_dashboard.py',
        'backend/alert_manager.py',
        'backend/report_generator.py',
        'backend/config_manager.py',
        'backend/utils.py',
        'cli/monitor_cli.py',
        'start_dashboard.py',
        'setup.py',
        'config/config.yaml',
        'requirements.txt',
        'README.md'
    ]
    
    missing_files = []
    for file_path in required_files:
        if not Path(file_path).exists():
            missing_files.append(file_path)
            print(f"   ❌ Missing: {file_path}")
        else:
            print(f"   ✅ Found: {file_path}")
    
    return len(missing_files) == 0

def check_python_dependencies():
    """Check if required Python modules can be imported"""
    print("\n🐍 Checking Python dependencies...")
    
    required_modules = {
        'psutil': 'System monitoring',
        'flask': 'Web framework',
        'flask-cors': 'CORS support',
        'flask-socketio': 'WebSocket support',
        'sqlite3': 'Database',
        'json': 'JSON handling',
        'threading': 'Threading support',
        'datetime': 'Date/time handling'
    }
    
    missing_modules = []
    for module, description in required_modules.items():
        try:
            __import__(module)
            print(f"   ✅ {module}: {description}")
        except ImportError:
            missing_modules.append(module)
            print(f"   ❌ {module}: {description} - NOT FOUND")
    
    return len(missing_modules) == 0

def check_configuration():
    """Check configuration file"""
    print("\n⚙️  Checking configuration...")
    
    config_file = Path('config/config.yaml')
    if not config_file.exists():
        print("   ❌ Configuration file not found")
        return False
    
    try:
        import yaml
        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)
        
        required_sections = ['database', 'monitoring', 'thresholds', 'web_dashboard']
        missing_sections = []
        
        for section in required_sections:
            if section not in config:
                missing_sections.append(section)
                print(f"   ❌ Missing config section: {section}")
            else:
                print(f"   ✅ Config section: {section}")
        
        return len(missing_sections) == 0
        
    except Exception as e:
        print(f"   ❌ Error reading configuration: {e}")
        return False

def test_database_creation():
    """Test database creation"""
    print("\n🗄️  Testing database creation...")
    
    test_db_path = 'data/test_monitor.db'
    try:
        # Add backend to path
        backend_dir = Path('backend')
        sys.path.insert(0, str(backend_dir))
        
        from system_monitor import SystemMonitor
        
        # Create test monitor
        monitor = SystemMonitor(test_db_path, history_size=100)
        
        # Test metrics collection
        metrics = monitor.get_current_metrics()
        
        if metrics and 'timestamp' in metrics:
            print("   ✅ Database creation successful")
            print("   ✅ Metrics collection working")
            
            # Clean up
            if Path(test_db_path).exists():
                os.remove(test_db_path)
            
            return True
        else:
            print("   ❌ Metrics collection failed")
            return False
            
    except Exception as e:
        print(f"   ❌ Database test failed: {e}")
        return False

def test_web_components():
    """Test web dashboard components"""
    print("\n🌐 Testing web components...")
    
    try:
        # Add backend to path
        backend_dir = Path('backend')
        sys.path.insert(0, str(backend_dir))
        
        # Test imports
        from web_dashboard import app
        from system_monitor import SystemMonitor
        from alert_manager import AlertManager
        from report_generator import ReportGenerator
        
        print("   ✅ Web dashboard imports successful")
        print("   ✅ Core components imported")
        
        # Test app creation
        if app:
            print("   ✅ Flask app creation successful")
        
        return True
        
    except Exception as e:
        print(f"   ❌ Web component test failed: {e}")
        return False

def test_cli_tools():
    """Test CLI tools"""
    print("\n🛠️  Testing CLI tools...")
    
    try:
        cli_file = Path('cli/monitor_cli.py')
        if not cli_file.exists():
            print("   ❌ CLI file not found")
            return False
        
        # Add backend to path for imports
        backend_dir = Path('backend')
        sys.path.insert(0, str(backend_dir))
        
        # Test CLI import
        import cli.monitor_cli
        
        print("   ✅ CLI imports successful")
        print("   ✅ CLI tools available")
        
        return True
        
    except Exception as e:
        print(f"   ❌ CLI test failed: {e}")
        return False

def generate_validation_report(results):
    """Generate validation report"""
    print("\n" + "=" * 60)
    print("VALIDATION SUMMARY")
    print("=" * 60)
    
    total_tests = len(results)
    passed_tests = sum(1 for result in results.values() if result)
    failed_tests = total_tests - passed_tests
    
    print(f"Total Tests: {total_tests}")
    print(f"Passed: {passed_tests}")
    print(f"Failed: {failed_tests}")
    print(f"Success Rate: {(passed_tests/total_tests)*100:.1f}%")
    
    if failed_tests == 0:
        print("\n🎉 ALL TESTS PASSED!")
        print("The Performance Monitoring Dashboard is ready to use.")
        print("\nNext steps:")
        print("1. Run: python start_dashboard.py --mode web")
        print("2. Open: http://localhost:5000")
        print("3. Or use CLI: python cli/monitor_cli.py monitor")
    else:
        print(f"\n⚠️  {failed_tests} test(s) failed")
        print("Please check the errors above and run setup.py if needed")
    
    print("\nFor detailed usage instructions, see README.md")

def main():
    """Main validation function"""
    print_header()
    
    # Run validation tests
    tests = {
        'Directory Structure': check_directory_structure,
        'Required Files': check_required_files,
        'Python Dependencies': check_python_dependencies,
        'Configuration': check_configuration,
        'Database Creation': test_database_creation,
        'Web Components': test_web_components,
        'CLI Tools': test_cli_tools
    }
    
    results = {}
    
    for test_name, test_function in tests.items():
        try:
            results[test_name] = test_function()
        except Exception as e:
            print(f"   ❌ {test_name} failed with exception: {e}")
            results[test_name] = False
    
    # Generate report
    generate_validation_report(results)
    
    # Return exit code based on results
    if all(results.values()):
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()