#!/usr/bin/env python3
"""
Runner script for comprehensive stress testing suite
Easy-to-use interface for running stress tests
"""

import os
import sys
import argparse
from pathlib import Path

# Add the stress_tests directory to Python path
stress_tests_dir = Path(__file__).parent
sys.path.insert(0, str(stress_tests_dir))

from main_stress_test import ComprehensiveStressTester
from config import create_config_file, load_config
from utils.system_utils import validate_test_environment


def validate_environment():
    """Validate that the testing environment is suitable"""
    print("🔍 Validating test environment...")
    
    # Check required packages
    required_packages = ['psutil', 'numpy', 'matplotlib', 'pandas', 'seaborn', 'jinja2']
    missing_packages = []
    
    for package in required_packages:
        try:
            __import__(package)
        except ImportError:
            missing_packages.append(package)
    
    if missing_packages:
        print(f"❌ Missing required packages: {', '.join(missing_packages)}")
        print("Please install with: pip install " + " ".join(missing_packages))
        return False
    
    print("✅ All required packages are installed")
    return True


def run_stress_tests(args):
    """Run the stress tests with the given arguments"""
    
    # Validate environment
    if not validate_environment():
        return 1
    
    # Load configuration
    config_file = args.config
    if not config_file:
        config_file = "stress_test_config.json"
        create_config_file(config_file)
        print(f"📝 Created default configuration file: {config_file}")
    
    config = load_config(config_file)
    
    # Override config with command line arguments
    if args.output:
        config["output_dir"] = args.output
    if args.duration:
        config["test_duration"] = args.duration
    if args.parallel:
        config["parallel_threads"] = args.parallel
    if args.verbose:
        config["verbose"] = True
    
    # Validate test environment
    from utils.system_utils import StressTestConfig
    test_config = StressTestConfig(**config)
    
    validation_result = validate_test_environment(test_config)
    
    if not validation_result["valid"]:
        print("❌ Environment validation failed:")
        for issue in validation_result["issues"]:
            print(f"   • {issue}")
        return 1
    
    if validation_result["warnings"]:
        print("⚠️  Environment warnings:")
        for warning in validation_result["warnings"]:
            print(f"   • {warning}")
    
    print(f"✅ Environment validation passed")
    
    # Initialize and run stress tester
    print(f"🚀 Starting comprehensive stress testing suite...")
    print(f"📁 Test directory: {test_config.test_dir}")
    print(f"📊 Output directory: {test_config.output_dir}")
    print(f"⏱️  Test duration: {test_config.test_duration}s")
    print(f"🔧 Parallel threads: {test_config.parallel_threads}")
    
    try:
        tester = ComprehensiveStressTester(config_file)
        results = tester.run_all_tests()
        
        # Print summary
        print("\n" + "="*60)
        print("📈 STRESS TEST RESULTS SUMMARY")
        print("="*60)
        
        summary = results["summary"]
        print(f"✅ Tests Passed: {summary['passed_tests']}/{summary['total_tests']}")
        print(f"❌ Tests Failed: {summary['failed_tests']}")
        print(f"💥 Tests with Errors: {summary['error_tests']}")
        print(f"⏰ Timeout Tests: {summary['timeout_tests']}")
        print(f"📊 Success Rate: {summary['success_rate']:.1f}%")
        print(f"⏱️  Total Duration: {summary['total_duration']:.2f} seconds")
        
        # Print recommendations
        if results.get("recommendations"):
            print(f"\n💡 RECOMMENDATIONS:")
            for i, recommendation in enumerate(results["recommendations"], 1):
                print(f"   {i}. {recommendation}")
        
        # Print report locations
        print(f"\n📋 DETAILED REPORTS:")
        report_dir = Path(test_config.output_dir) / "reports"
        charts_dir = Path(test_config.output_dir) / "charts"
        print(f"   📄 Reports: {report_dir}")
        print(f"   📊 Charts: {charts_dir}")
        
        # Print system info
        if results.get("system_info"):
            system_info = results["system_info"]
            print(f"\n🖥️  SYSTEM INFORMATION:")
            if system_info.get("system"):
                sys_info = system_info["system"]
                print(f"   Platform: {sys_info.get('platform', 'Unknown')} {sys_info.get('release', '')}")
                print(f"   Hostname: {sys_info.get('hostname', 'Unknown')}")
            
            if system_info.get("cpu"):
                cpu_info = system_info["cpu"]
                print(f"   CPU: {cpu_info.get('count', 'Unknown')} cores @ {cpu_info.get('frequency_mhz', 0):.0f} MHz")
            
            if system_info.get("memory"):
                memory_info = system_info["memory"]
                print(f"   Memory: {memory_info.get('total_gb', 0):.1f} GB total, {memory_info.get('available_gb', 0):.1f} GB available")
            
            if system_info.get("disk"):
                disk_info = system_info["disk"]
                print(f"   Disk: {disk_info.get('total_gb', 0):.1f} GB total, {disk_info.get('free_gb', 0):.1f} GB free")
        
        # Exit with appropriate code
        success_rate = summary["success_rate"]
        if success_rate >= 90:
            print("\n🎉 STRESS TESTS COMPLETED SUCCESSFULLY!")
            return 0
        elif success_rate >= 70:
            print("\n⚠️  STRESS TESTS COMPLETED WITH WARNINGS")
            return 1
        else:
            print("\n💥 STRESS TESTS FAILED - SYSTEM ISSUES DETECTED")
            return 2
            
    except KeyboardInterrupt:
        print("\n\n⚠️  Stress testing interrupted by user")
        return 130
    except Exception as e:
        print(f"\n💥 Stress testing failed with error: {str(e)}")
        import traceback
        if args.verbose:
            traceback.print_exc()
        return 1


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="Comprehensive Stress Testing Suite",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run with default settings
  python run_stress_tests.py
  
  # Run with custom configuration
  python run_stress_tests.py --config my_config.json
  
  # Run with custom output directory and duration
  python run_stress_tests.py --output /tmp/stress_results --duration 600
  
  # Run with more parallel threads
  python run_stress_tests.py --parallel 8 --verbose
        """
    )
    
    parser.add_argument(
        "--config", "-c",
        type=str,
        help="Configuration file path (default: stress_test_config.json)"
    )
    
    parser.add_argument(
        "--output", "-o",
        type=str,
        help="Output directory for results (default: ./stress_test_results)"
    )
    
    parser.add_argument(
        "--duration", "-d",
        type=int,
        help="Test duration in seconds (default: 300)"
    )
    
    parser.add_argument(
        "--parallel", "-p",
        type=int,
        help="Number of parallel threads (default: 4)"
    )
    
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose logging"
    )
    
    parser.add_argument(
        "--create-config",
        action="store_true",
        help="Create default configuration file and exit"
    )
    
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Validate environment and exit"
    )
    
    args = parser.parse_args()
    
    # Handle special commands
    if args.create_config:
        config_file = args.config or "stress_test_config.json"
        create_config_file(config_file)
        print(f"✅ Configuration file created: {config_file}")
        print(f"You can edit this file to customize test parameters.")
        print(f"Then run: python {sys.argv[0]} --config {config_file}")
        return 0
    
    if args.validate:
        if validate_environment():
            print("✅ Environment validation passed")
            return 0
        else:
            print("❌ Environment validation failed")
            return 1
    
    # Run stress tests
    return run_stress_tests(args)


if __name__ == "__main__":
    sys.exit(main())