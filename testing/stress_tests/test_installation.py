#!/usr/bin/env python3
"""
Test script to verify the stress testing suite installation
"""

import sys
import traceback
from pathlib import Path

def test_imports():
    """Test if all modules can be imported"""
    print("🔍 Testing module imports...")
    
    try:
        # Test main imports
        from main_stress_test import ComprehensiveStressTester, TestResult
        print("✅ Main stress test module imported successfully")
        
        from memory.memory_stress import MemoryStressTester
        print("✅ Memory stress test module imported successfully")
        
        from filesystem.fs_stress import FileSystemStressTester
        print("✅ File system stress test module imported successfully")
        
        from cpu.cpu_stress import CPUStressTester
        print("✅ CPU stress test module imported successfully")
        
        from concurrent.concurrent_stress import ConcurrentStressTester
        print("✅ Concurrent stress test module imported successfully")
        
        from resource_exhaustion.resource_exhaustion import ResourceExhaustionTester
        print("✅ Resource exhaustion test module imported successfully")
        
        from reporting.stress_report import StressReportGenerator
        print("✅ Reporting module imported successfully")
        
        from utils.system_utils import StressTestConfig, SystemMonitor
        print("✅ System utilities module imported successfully")
        
        return True
        
    except ImportError as e:
        print(f"❌ Import error: {str(e)}")
        traceback.print_exc()
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {str(e)}")
        traceback.print_exc()
        return False

def test_dependencies():
    """Test if required dependencies are available"""
    print("\n🔍 Testing dependencies...")
    
    required_deps = [
        'psutil', 'numpy', 'matplotlib', 'pandas', 
        'seaborn', 'jinja2', 'threading', 'multiprocessing'
    ]
    
    missing_deps = []
    
    for dep in required_deps:
        try:
            __import__(dep)
            print(f"✅ {dep} is available")
        except ImportError:
            print(f"❌ {dep} is missing")
            missing_deps.append(dep)
    
    if missing_deps:
        print(f"\n❌ Missing dependencies: {', '.join(missing_deps)}")
        print("Please install with: pip install " + " ".join(missing_deps))
        return False
    
    return True

def test_basic_functionality():
    """Test basic functionality"""
    print("\n🔍 Testing basic functionality...")
    
    try:
        # Test configuration
        from utils.system_utils import StressTestConfig
        config = StressTestConfig()
        print("✅ Configuration system works")
        
        # Test system monitoring
        monitor = SystemMonitor(config)
        system_info = monitor.get_system_info()
        if system_info:
            print("✅ System monitoring works")
        else:
            print("⚠️ System monitoring returned empty result")
        
        # Test resource manager
        from utils.system_utils import ResourceManager
        rm = ResourceManager()
        print("✅ Resource manager works")
        
        return True
        
    except Exception as e:
        print(f"❌ Basic functionality test failed: {str(e)}")
        traceback.print_exc()
        return False

def main():
    """Main test function"""
    print("🧪 STRESS TESTING SUITE VERIFICATION")
    print("=" * 50)
    
    # Test imports
    if not test_imports():
        print("\n❌ Import tests failed")
        return 1
    
    # Test dependencies
    if not test_dependencies():
        print("\n❌ Dependency tests failed")
        return 1
    
    # Test basic functionality
    if not test_basic_functionality():
        print("\n❌ Basic functionality tests failed")
        return 1
    
    print("\n" + "=" * 50)
    print("✅ ALL TESTS PASSED!")
    print("🎉 Stress testing suite is ready to use!")
    print("\nNext steps:")
    print("1. Run: python run_stress_tests.py --create-config")
    print("2. Edit configuration file as needed")
    print("3. Run: python run_stress_tests.py")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())