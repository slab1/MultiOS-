#!/usr/bin/env python3
"""
Hardware Testing Framework Demo
Demonstrates key features and capabilities of the hardware testing framework
"""

import os
import sys
import time
import logging
from pathlib import Path

# Add the framework to the path
sys.path.insert(0, '/workspace/testing/hardware_tests')

from hardware_detector import HardwareDetector
from compatibility_testing import HardwareCompatibilityTester
from optimization_engine import OptimizationEngine, HardwareProfile

def setup_logging():
    """Setup demo logging"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    return logging.getLogger(__name__)

def demo_hardware_detection():
    """Demonstrate hardware detection capabilities"""
    print("\n" + "="*60)
    print("DEMO 1: Hardware Detection and Profiling")
    print("="*60)
    
    logger = logging.getLogger(__name__)
    logger.info("Starting hardware detection demo")
    
    # Initialize detector
    detector = HardwareDetector()
    
    # Run detection
    print("Detecting hardware components...")
    hardware_data = detector.run_full_detection()
    
    # Display results
    print(f"\nDetected Hardware:")
    print(f"  CPU: {hardware_data['cpu'].get('model', 'Unknown')}")
    print(f"  Cores: {hardware_data['cpu'].get('cores_physical', 0)} physical, {hardware_data['cpu'].get('cores_logical', 0)} logical")
    print(f"  Memory: {hardware_data['memory'].get('total_gb', 0)}GB {hardware_data['memory'].get('memory_type', 'Unknown')}")
    print(f"  Storage: {len(hardware_data.get('storage', []))} devices")
    print(f"  Network: {len(hardware_data.get('network', []))} interfaces")
    print(f"  GPU: {len(hardware_data.get('gpu', []))} devices")
    
    # Generate profile
    profile_path = detector.generate_hardware_profile()
    print(f"\nHardware profile saved to: {profile_path}")
    
    # Export configuration
    config_path = detector.export_config()
    print(f"Configuration exported to: {config_path}")
    
    return hardware_data

def demo_compatibility_testing(hardware_data):
    """Demonstrate compatibility testing"""
    print("\n" + "="*60)
    print("DEMO 2: Hardware Compatibility Testing")
    print("="*60)
    
    logger = logging.getLogger(__name__)
    logger.info("Starting compatibility testing demo")
    
    # Initialize tester
    tester = HardwareCompatibilityTester()
    
    # Set hardware profile
    tester.current_hardware_profile = hardware_data
    
    # Run compatibility tests (limited set for demo)
    print("Running compatibility tests...")
    results = tester.run_compatibility_suite(['cpu', 'memory'])
    
    # Display results
    print(f"\nCompatibility Test Results:")
    for result in results:
        status_symbol = {
            'compatible': '✓',
            'incompatible': '✗',
            'partial': '⚠',
            'warning': '⚠',
            'needs_driver': '⊘',
            'unknown': '?'
        }.get(result.status.value, '?')
        
        print(f"  {status_symbol} {result.test_id}: {result.message}")
    
    # Generate report
    report_path = tester.generate_compatibility_report()
    print(f"\nCompatibility report saved to: {report_path}")
    
    return results

def demo_optimization_analysis(hardware_data):
    """Demonstrate optimization recommendations"""
    print("\n" + "="*60)
    print("DEMO 3: Optimization Analysis")
    print("="*60)
    
    logger = logging.getLogger(__name__)
    logger.info("Starting optimization analysis demo")
    
    # Create hardware profile
    hardware_profile = HardwareProfile(
        system_info=hardware_data.get('system', {}),
        cpu_info=hardware_data.get('cpu', {}),
        memory_info=hardware_data.get('memory', {}),
        storage_info=hardware_data.get('storage', []),
        network_info=hardware_data.get('network', []),
        gpu_info=hardware_data.get('gpu', []),
        thermal_sensors=hardware_data.get('system', {}).get('temperature_sensors', []),
        performance_baseline={},
        detected_workloads=[],
        optimization_potential={}
    )
    
    # Initialize optimization engine
    engine = OptimizationEngine()
    engine.hardware_profile = hardware_profile
    
    # Analyze hardware capabilities
    print("Analyzing hardware capabilities...")
    analysis = engine.analyze_hardware_capabilities(hardware_profile)
    
    print(f"\nHardware Analysis:")
    print(f"  Overall Optimization Score: {analysis['overall_optimization_score']:.1f}/100")
    
    if 'cpu_analysis' in analysis:
        cpu_analysis = analysis['cpu_analysis']
        print(f"  CPU Performance Score: {cpu_analysis['performance_score']:.1f}/100")
    
    # Generate recommendations
    print("\nGenerating optimization recommendations...")
    recommendations = engine.generate_optimization_recommendations()
    
    print(f"\nGenerated {len(recommendations)} recommendations:")
    for i, rec in enumerate(recommendations[:5], 1):  # Show first 5
        priority_symbol = {
            'critical': '🔴',
            'high': '🟠',
            'medium': '🟡',
            'low': '🟢',
            'optional': '⚪'
        }.get(rec.priority.value, '⚪')
        
        print(f"  {i}. {priority_symbol} {rec.title}")
        print(f"     Category: {rec.category.value}, Priority: {rec.priority.value}")
        print(f"     Benefit: {rec.expected_benefit}")
        print()
    
    # Generate report
    report_path = engine.generate_optimization_report(recommendations)
    print(f"Optimization report saved to: {report_path}")
    
    return analysis, recommendations

def demo_quick_performance_test():
    """Demonstrate quick performance testing"""
    print("\n" + "="*60)
    print("DEMO 4: Quick Performance Test")
    print("="*60)
    
    import threading
    import subprocess
    
    logger = logging.getLogger(__name__)
    logger.info("Starting quick performance test demo")
    
    # CPU performance test
    print("Running CPU performance test...")
    cpu_start = time.time()
    
    def cpu_intensive_task():
        return sum(range(100000))
    
    # Single-threaded test
    start_time = time.time()
    result = cpu_intensive_task()
    single_thread_time = time.time() - start_time
    
    # Multi-threaded test
    start_time = time.time()
    threads = []
    for _ in range(4):
        thread = threading.Thread(target=cpu_intensive_task)
        thread.start()
        threads.append(thread)
    
    for thread in threads:
        thread.join()
    
    multi_thread_time = time.time() - start_time
    speedup = single_thread_time / multi_thread_time if multi_thread_time > 0 else 0
    
    print(f"  Single-thread performance: {single_thread_time:.3f}s")
    print(f"  4-thread performance: {multi_thread_time:.3f}s")
    print(f"  Speedup: {speedup:.2f}x")
    
    # Memory performance test
    print("\nRunning memory performance test...")
    import psutil
    
    memory = psutil.virtual_memory()
    print(f"  Total memory: {memory.total / (1024**3):.1f}GB")
    print(f"  Available memory: {memory.available / (1024**3):.1f}GB")
    print(f"  Memory usage: {memory.percent:.1f}%")
    
    # Storage performance test
    print("\nRunning storage performance test...")
    test_file = '/tmp/demo_performance_test.tmp'
    
    # Write test
    start_time = time.time()
    with open(test_file, 'w') as f:
        f.write('0' * (10 * 1024 * 1024))  # 10MB
    write_time = time.time() - start_time
    write_speed = (10 / write_time) if write_time > 0 else 0
    
    # Read test
    start_time = time.time()
    with open(test_file, 'r') as f:
        f.read()
    read_time = time.time() - start_time
    read_speed = (10 / read_time) if read_time > 0 else 0
    
    # Cleanup
    os.remove(test_file)
    
    print(f"  Write speed: {write_speed:.1f}MB/s")
    print(f"  Read speed: {read_speed:.1f}MB/s")
    
    return {
        'cpu_speedup': speedup,
        'memory_usage_percent': memory.percent,
        'storage_write_speed_mbps': write_speed,
        'storage_read_speed_mbps': read_speed
    }

def demo_report_generation():
    """Demonstrate report generation"""
    print("\n" + "="*60)
    print("DEMO 5: Comprehensive Report Generation")
    print("="*60)
    
    logger = logging.getLogger(__name__)
    logger.info("Starting report generation demo")
    
    # Generate sample comprehensive report
    report_data = {
        'report_info': {
            'generated_at': time.time(),
            'demo_version': '1.0'
        },
        'demo_summary': {
            'demos_completed': 5,
            'framework_version': '1.0',
            'total_features_tested': 8
        },
        'demo_results': {
            'hardware_detection': 'completed',
            'compatibility_testing': 'completed',
            'optimization_analysis': 'completed',
            'performance_testing': 'completed',
            'report_generation': 'completed'
        },
        'capabilities_demonstrated': [
            'Automatic hardware detection and profiling',
            'Comprehensive compatibility testing',
            'AI-powered optimization recommendations',
            'Performance benchmarking',
            'Report generation and analysis'
        ],
        'next_steps': [
            'Run full test suite',
            'Test on different hardware',
            'Implement custom test scenarios',
            'Integrate with CI/CD pipeline'
        ]
    }
    
    # Save report
    report_path = '/workspace/testing/hardware_tests/results/demo_report.json'
    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    
    import json
    with open(report_path, 'w') as f:
        json.dump(report_data, f, indent=2)
    
    print(f"Demo report saved to: {report_path}")
    print("\nDemo Report Contents:")
    print(f"  • Framework capabilities demonstrated: {len(report_data['capabilities_demonstrated'])}")
    print(f"  • Test suites completed: {report_data['demo_summary']['demos_completed']}")
    print(f"  • Next steps provided: {len(report_data['next_steps'])}")
    
    return report_path

def main():
    """Main demo function"""
    print("="*60)
    print("Hardware Testing Framework - Feature Demo")
    print("="*60)
    print("\nThis demo showcases the key capabilities of the hardware testing framework.")
    print("Each demo will run automatically and display results.")
    
    # Setup logging
    logger = setup_logging()
    
    try:
        # Run demos
        hardware_data = demo_hardware_detection()
        
        compatibility_results = demo_compatibility_testing(hardware_data)
        
        optimization_analysis, recommendations = demo_optimization_analysis(hardware_data)
        
        performance_results = demo_quick_performance_test()
        
        report_path = demo_report_generation()
        
        # Summary
        print("\n" + "="*60)
        print("DEMO SUMMARY")
        print("="*60)
        print("\n✓ All demos completed successfully!")
        print(f"\nFramework Features Demonstrated:")
        print("  1. ✓ Hardware Detection and Auto-Configuration")
        print("  2. ✓ Automated Hardware Compatibility Testing")
        print("  3. ✓ Hardware-Specific Optimization Recommendations")
        print("  4. ✓ Performance Benchmarking")
        print("  5. ✓ Report Generation and Analysis")
        
        print(f"\nGenerated Files:")
        print(f"  • Hardware profiles: /workspace/testing/hardware_tests/profiles/")
        print(f"  • Test results: /workspace/testing/hardware_tests/results/")
        print(f"  • Demo report: {report_path}")
        
        print(f"\nNext Steps:")
        print("  • Run full test suite: python3 hardware_test_orchestrator.py full")
        print("  • Quick tests: ./scripts/quick_test.sh")
        print("  • View documentation: README.md")
        
    except Exception as e:
        logger.error(f"Demo failed: {e}")
        print(f"\n❌ Demo failed: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)