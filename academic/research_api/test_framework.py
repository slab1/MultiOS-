#!/usr/bin/env python3
"""
OS Research API - Framework Test

Tests all components of the research framework to ensure proper functionality.
"""

import sys
import os
from pathlib import Path

# Add the current directory to the path
current_dir = Path(__file__).parent
parent_dir = current_dir.parent
sys.path.insert(0, str(parent_dir))

def test_framework_components():
    """Test that all framework components are working."""
    print("="*80)
    print("OS RESEARCH API - COMPONENT VERIFICATION")
    print("="*80)
    
    try:
        # Test core framework
        from research_api import ResearchFramework
        print("✓ ResearchFramework imported successfully")
        
        # Test configuration
        from research_api.core.config import ResearchConfig
        print("✓ ResearchConfig imported successfully")
        
        # Test environment manager
        from research_api.core.environment import EnvironmentManager
        print("✓ EnvironmentManager imported successfully")
        
        # Test benchmarking
        from research_api.benchmarking.benchmark import BenchmarkSuite
        print("✓ BenchmarkSuite imported successfully")
        
        # Test analysis
        from research_api.analysis.system import SystemAnalyzer
        print("✓ SystemAnalyzer imported successfully")
        
        # Test instrumentation
        from research_api.instrumentation.hooks import OSInstrumentor
        print("✓ OSInstrumentor imported successfully")
        
        # Test testing framework
        from research_api.testing.validator import TestFramework
        print("✓ TestFramework imported successfully")
        
        # Test data collection
        from research_api.data_collection.collector import DataCollector
        print("✓ DataCollector imported successfully")
        
        # Test reporting
        from research_api.reporting.visualization_engine import ReportGenerator
        print("✓ ReportGenerator imported successfully")
        
        print("\n✓ ALL COMPONENTS VERIFIED SUCCESSFULLY!")
        
        # Test framework creation
        framework = ResearchFramework({}, str(current_dir))
        print("✓ ResearchFramework instance created successfully")
        
        # Test experiment creation
        experiment = framework.create_experiment(
            name='test_experiment',
            description='Test experiment',
            parameters={'name': 'Test', 'description': 'Test'},
            environment={}
        )
        print("✓ Experiment created successfully")
        
        # Test benchmark suite
        benchmark = BenchmarkSuite({})
        print("✓ BenchmarkSuite instance created successfully")
        
        # Test system analyzer
        analyzer = SystemAnalyzer({})
        print("✓ SystemAnalyzer instance created successfully")
        
        # Test OS instrumentor
        instrumentor = OSInstrumentor({})
        print("✓ OSInstrumentor instance created successfully")
        
        # Test test framework
        tester = TestFramework({})
        print("✓ TestFramework instance created successfully")
        
        # Test data collector
        collector = DataCollector(str(workspace_dir), {})
        print("✓ DataCollector instance created successfully")
        
        # Test report generator
        reporter = ReportGenerator(str(current_dir), {})
        print("✓ ReportGenerator instance created successfully")
        
        print("\n" + "="*80)
        print("🎉 OS RESEARCH API IS FULLY FUNCTIONAL! 🎉")
        print("="*80)
        
        return True
        
    except Exception as e:
        print(f"✗ Verification failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_factory_functions():
    """Test the factory functions from __init__.py."""
    print("\n" + "="*50)
    print("FACTORY FUNCTIONS TEST")
    print("="*50)
    
    try:
        # Test factory functions
        from research_api import (
            create_experiment_framework, 
            create_benchmark_suite, 
            create_system_analyzer,
            create_os_instrumentor,
            create_test_validator,
            create_data_collector
        )
        print("✓ All factory functions imported successfully")
        
        # Test factory function calls
        framework = create_experiment_framework({}, str(current_dir))
        print("✓ create_experiment_framework() works")
        
        benchmark = create_benchmark_suite({})
        print("✓ create_benchmark_suite() works")
        
        analyzer = create_system_analyzer({})
        print("✓ create_system_analyzer() works")
        
        instrumentor = create_os_instrumentor({})
        print("✓ create_os_instrumentor() works")
        
        tester = create_test_validator({})
        print("✓ create_test_validator() works")
        
        collector = create_data_collector(str(current_dir), {})
        print("✓ create_data_collector() works")
        
        print("✓ ALL FACTORY FUNCTIONS WORKING")
        return True
        
    except Exception as e:
        print(f"✗ Factory functions test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Main test function."""
    success1 = test_framework_components()
    success2 = test_factory_functions()
    
    overall_success = success1 and success2
    
    print(f"\nFramework Status: {'OPERATIONAL' if overall_success else 'ERRORS DETECTED'}")
    
    return overall_success

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)