#!/usr/bin/env python3
"""
Hardware Testing Framework Main Orchestrator
Coordinates all hardware testing components and provides unified interface
"""

import os
import sys
import json
import time
import logging
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import asdict
import threading

# Import all testing modules
from hardware_detector import HardwareDetector
from compatibility_testing import HardwareCompatibilityTester
from power_thermal_testing import PowerThermalTester, PowerTestConfig
from peripheral_testing import PeripheralTester
from multicore_scaling_test import MultiCoreScalingTester
from stability_testing import LongTermStabilityTester
from optimization_engine import OptimizationEngine, HardwareProfile

class HardwareTestOrchestrator:
    """Main orchestrator for hardware testing framework"""
    
    def __init__(self):
        self.logger = self._setup_logging()
        self.hardware_detector = HardwareDetector()
        self.compatibility_tester = HardwareCompatibilityTester()
        self.power_thermal_tester = PowerThermalTester()
        self.peripheral_tester = PeripheralTester()
        self.multicore_tester = MultiCoreScalingTester()
        self.stability_tester = LongTermStabilityTester()
        self.optimization_engine = OptimizationEngine()
        
        self.test_results = {}
        self.hardware_profile = None
        
    def _setup_logging(self):
        """Setup comprehensive logging"""
        log_format = '%(asctime)s - %(levelname)s - %(message)s'
        
        # Create logs directory
        os.makedirs('/workspace/testing/hardware_tests/logs', exist_ok=True)
        
        logging.basicConfig(
            level=logging.INFO,
            format=log_format,
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/logs/orchestrator.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def detect_hardware(self) -> Dict[str, Any]:
        """Detect and profile all hardware"""
        self.logger.info("Starting hardware detection and profiling")
        
        try:
            # Run hardware detection
            hardware_data = self.hardware_detector.run_full_detection()
            
            # Generate hardware profile
            profile_path = self.hardware_detector.generate_hardware_profile()
            
            # Export configuration
            config_path = self.hardware_detector.export_config()
            
            # Create HardwareProfile object for other components
            self.hardware_profile = HardwareProfile(
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
            
            self.logger.info("Hardware detection completed successfully")
            
            return {
                'status': 'success',
                'hardware_data': hardware_data,
                'profile_path': profile_path,
                'config_path': config_path,
                'summary': self._generate_hardware_summary(hardware_data)
            }
            
        except Exception as e:
            self.logger.error(f"Hardware detection failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def _generate_hardware_summary(self, hardware_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate hardware summary"""
        return {
            'cpu': {
                'model': hardware_data.get('cpu', {}).get('model', 'Unknown'),
                'cores_physical': hardware_data.get('cpu', {}).get('cores_physical', 0),
                'cores_logical': hardware_data.get('cpu', {}).get('cores_logical', 0),
                'max_frequency': hardware_data.get('cpu', {}).get('frequency_max', 0)
            },
            'memory': {
                'total_gb': hardware_data.get('memory', {}).get('total_gb', 0),
                'type': hardware_data.get('memory', {}).get('memory_type', 'Unknown'),
                'ecc_support': hardware_data.get('memory', {}).get('ecc_support', False)
            },
            'storage': {
                'device_count': len(hardware_data.get('storage', [])),
                'total_capacity_gb': sum(device.get('size_gb', 0) for device in hardware_data.get('storage', []))
            },
            'network': {
                'interface_count': len(hardware_data.get('network', [])),
                'max_speed_mbps': max([iface.get('speed_mbps', 100) for iface in hardware_data.get('network', [])] + [100])
            },
            'gpu': {
                'gpu_count': len(hardware_data.get('gpu', [])),
                'total_vram_gb': sum(gpu.get('memory_mb', 0) for gpu in hardware_data.get('gpu', [])) / 1024
            }
        }
    
    def run_compatibility_tests(self, test_categories: List[str] = None) -> Dict[str, Any]:
        """Run hardware compatibility tests"""
        self.logger.info("Starting hardware compatibility testing")
        
        try:
            # Set hardware profile if available
            if self.hardware_profile:
                self.compatibility_tester.current_hardware_profile = asdict(self.hardware_profile)
            
            # Run compatibility tests
            results = self.compatibility_tester.run_compatibility_suite(test_categories)
            
            # Generate report
            report_path = self.compatibility_tester.generate_compatibility_report()
            
            # Analyze results
            summary = {
                'total_tests': len(results),
                'passed': len([r for r in results if r.status.value == 'compatible']),
                'failed': len([r for r in results if r.status.value == 'incompatible']),
                'warnings': len([r for r in results if r.status.value in ['warning', 'partial']]),
                'needs_driver': len([r for r in results if r.status.value == 'needs_driver'])
            }
            
            self.logger.info(f"Compatibility testing completed: {summary['passed']}/{summary['total_tests']} passed")
            
            return {
                'status': 'success',
                'summary': summary,
                'detailed_results': results,
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Compatibility testing failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def run_peripheral_tests(self) -> Dict[str, Any]:
        """Run peripheral testing"""
        self.logger.info("Starting peripheral testing")
        
        try:
            # Run peripheral test suite
            results = self.peripheral_tester.run_peripheral_test_suite()
            
            # Generate report
            report_path = self.peripheral_tester.generate_peripheral_report()
            
            self.logger.info(f"Peripheral testing completed: {results['summary']['passed']}/{results['summary']['total_tests']} passed")
            
            return {
                'status': 'success',
                'summary': results['summary'],
                'detailed_results': results['detailed_results'],
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Peripheral testing failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def run_multicore_scaling_test(self, max_threads: int = None) -> Dict[str, Any]:
        """Run multi-core scaling tests"""
        self.logger.info("Starting multi-core scaling tests")
        
        try:
            # Detect CPU topology
            cpu_topology = self.multicore_tester.detect_cpu_topology()
            
            # Run scaling analysis
            scaling_result = self.multicore_tester.run_scaling_analysis(max_threads)
            
            # Generate report
            report_path = self.multicore_tester.generate_scaling_report(scaling_result)
            
            # Analyze results
            max_speedup = max([r.speedup_factor for r in scaling_result.scaling_results])
            
            summary = {
                'cpu_topology': cpu_topology,
                'optimal_thread_count': scaling_result.optimal_thread_count,
                'max_speedup_achieved': max_speedup,
                'hyperthreading_effectiveness': scaling_result.hyperthreading_effectiveness
            }
            
            self.logger.info(f"Multi-core scaling test completed: {max_speedup:.2f}x speedup")
            
            return {
                'status': 'success',
                'summary': summary,
                'detailed_results': scaling_result,
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Multi-core scaling test failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def run_power_thermal_tests(self, test_duration_minutes: int = 60) -> Dict[str, Any]:
        """Run power and thermal tests"""
        self.logger.info("Starting power and thermal tests")
        
        try:
            # Detect thermal sensors
            thermal_zones = self.power_thermal_tester.detect_thermal_sensors()
            
            # Run power stress test
            power_config = PowerTestConfig(duration_minutes=test_duration_minutes)
            self.power_thermal_tester.config = power_config
            
            power_results = self.power_thermal_tester.run_power_stress_test()
            
            # Generate report
            report_path = self.power_thermal_tester.generate_power_thermal_report()
            
            # Analyze results
            summary = {
                'thermal_zones_detected': len(thermal_zones),
                'avg_power_consumption_w': power_results['power_consumption']['avg_total_power_w'],
                'max_temperature_c': power_results['temperatures']['max_cpu_temp_c'],
                'power_efficiency_score': power_results['efficiency_metrics']['efficiency_score']
            }
            
            self.logger.info(f"Power/thermal testing completed: {summary['avg_power_consumption_w']:.1f}W avg")
            
            return {
                'status': 'success',
                'summary': summary,
                'detailed_results': power_results,
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Power/thermal testing failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def run_stability_tests(self, test_duration_hours: int = 24) -> Dict[str, Any]:
        """Run stability tests"""
        self.logger.info("Starting stability tests")
        
        try:
            # Run comprehensive stability test
            stability_result = self.stability_tester.run_comprehensive_stability_test(test_duration_hours)
            
            # Generate report
            report_path = self.stability_tester.generate_stability_report(stability_result)
            
            # Analyze results
            summary = {
                'test_duration_hours': stability_result.duration_hours,
                'stability_status': stability_result.status.value,
                'total_anomalies': stability_result.summary['total_anomalies_detected'],
                'critical_anomalies': stability_result.summary['critical_anomalies'],
                'metrics_collected': stability_result.summary['total_metrics_collected']
            }
            
            self.logger.info(f"Stability testing completed: {stability_result.status.value}")
            
            return {
                'status': 'success',
                'summary': summary,
                'detailed_results': stability_result,
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Stability testing failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def generate_optimization_recommendations(self) -> Dict[str, Any]:
        """Generate optimization recommendations"""
        self.logger.info("Generating optimization recommendations")
        
        try:
            # Set hardware profile
            if self.hardware_profile:
                self.optimization_engine.hardware_profile = self.hardware_profile
            
            # Analyze hardware capabilities
            analysis = self.optimization_engine.analyze_hardware_capabilities(self.hardware_profile)
            
            # Generate recommendations
            recommendations = self.optimization_engine.generate_optimization_recommendations()
            
            # Generate report
            report_path = self.optimization_engine.generate_optimization_report(recommendations)
            
            # Analyze recommendations
            summary = {
                'optimization_score': analysis['overall_optimization_score'],
                'total_recommendations': len(recommendations),
                'automatic_optimizations': len([r for r in recommendations if r.auto_applicable]),
                'manual_interventions': len([r for r in recommendations if r.manual_required])
            }
            
            self.logger.info(f"Optimization analysis completed: {len(recommendations)} recommendations generated")
            
            return {
                'status': 'success',
                'summary': summary,
                'detailed_results': recommendations,
                'hardware_analysis': analysis,
                'report_path': report_path
            }
            
        except Exception as e:
            self.logger.error(f"Optimization analysis failed: {e}")
            return {
                'status': 'failed',
                'error': str(e)
            }
    
    def run_full_test_suite(self, test_duration_hours: int = 24, skip_stability: bool = False) -> Dict[str, Any]:
        """Run complete hardware testing suite"""
        self.logger.info("Starting full hardware testing suite")
        
        start_time = time.time()
        test_results = {}
        
        try:
            # 1. Hardware detection
            self.logger.info("Step 1: Hardware Detection")
            test_results['hardware_detection'] = self.detect_hardware()
            if test_results['hardware_detection']['status'] != 'success':
                raise Exception("Hardware detection failed")
            
            # 2. Compatibility testing
            self.logger.info("Step 2: Compatibility Testing")
            test_results['compatibility'] = self.run_compatibility_tests()
            
            # 3. Peripheral testing
            self.logger.info("Step 3: Peripheral Testing")
            test_results['peripheral'] = self.run_peripheral_tests()
            
            # 4. Multi-core scaling test
            self.logger.info("Step 4: Multi-core Scaling Test")
            test_results['multicore'] = self.run_multicore_scaling_test()
            
            # 5. Power and thermal testing
            self.logger.info("Step 5: Power and Thermal Testing")
            power_test_duration = min(test_duration_hours * 60, 120)  # Max 2 hours for power tests
            test_results['power_thermal'] = self.run_power_thermal_tests(power_test_duration)
            
            # 6. Stability testing (optional, long duration)
            if not skip_stability and test_duration_hours > 1:
                self.logger.info("Step 6: Stability Testing")
                stability_test_duration = min(test_duration_hours, 12)  # Max 12 hours for stability tests
                test_results['stability'] = self.run_stability_tests(stability_test_duration)
            
            # 7. Optimization recommendations
            self.logger.info("Step 7: Optimization Analysis")
            test_results['optimization'] = self.generate_optimization_recommendations()
            
            end_time = time.time()
            total_duration = (end_time - start_time) / 3600  # Convert to hours
            
            # Generate comprehensive report
            final_report = self.generate_final_report(test_results, total_duration)
            
            self.logger.info(f"Full test suite completed in {total_duration:.1f} hours")
            
            return {
                'status': 'success',
                'duration_hours': total_duration,
                'test_results': test_results,
                'final_report': final_report
            }
            
        except Exception as e:
            self.logger.error(f"Full test suite failed: {e}")
            return {
                'status': 'failed',
                'error': str(e),
                'partial_results': test_results
            }
    
    def generate_final_report(self, test_results: Dict[str, Any], duration_hours: float) -> str:
        """Generate final comprehensive test report"""
        
        # Calculate overall system scores
        scores = {}
        
        if 'hardware_detection' in test_results and test_results['hardware_detection']['status'] == 'success':
            scores['hardware_completeness'] = 100  # If detection worked, hardware is properly recognized
        else:
            scores['hardware_completeness'] = 0
        
        if 'compatibility' in test_results and test_results['compatibility']['status'] == 'success':
            comp_summary = test_results['compatibility']['summary']
            scores['compatibility_score'] = (comp_summary['passed'] / comp_summary['total_tests']) * 100
        else:
            scores['compatibility_score'] = 0
        
        if 'peripheral' in test_results and test_results['peripheral']['status'] == 'success':
            periph_summary = test_results['peripheral']['summary']
            scores['peripheral_score'] = (periph_summary['passed'] / periph_summary['total_tests']) * 100
        else:
            scores['peripheral_score'] = 0
        
        if 'multicore' in test_results and test_results['multicore']['status'] == 'success':
            # Score based on scaling efficiency
            multicore_summary = test_results['multicore']['summary']
            speedup = multicore_summary['max_speedup_achieved']
            cores = multicore_summary['cpu_topology']['logical_cores']
            scores['multicore_score'] = min((speedup / cores) * 100, 100)
        else:
            scores['multicore_score'] = 0
        
        if 'power_thermal' in test_results and test_results['power_thermal']['status'] == 'success':
            # Score based on thermal stability
            power_summary = test_results['power_thermal']['summary']
            max_temp = power_summary.get('max_temperature_c', 100)
            if max_temp < 70:
                scores['thermal_score'] = 100
            elif max_temp < 80:
                scores['thermal_score'] = 80
            elif max_temp < 90:
                scores['thermal_score'] = 60
            else:
                scores['thermal_score'] = 40
        else:
            scores['thermal_score'] = 0
        
        if 'stability' in test_results and test_results['stability']['status'] == 'success':
            stability_summary = test_results['stability']['summary']
            if stability_summary['stability_status'] == 'stable':
                scores['stability_score'] = 100
            elif stability_summary['stability_status'] == 'degrading':
                scores['stability_score'] = 70
            else:
                scores['stability_score'] = 50
        else:
            scores['stability_score'] = 0  # Not tested
        
        if 'optimization' in test_results and test_results['optimization']['status'] == 'success':
            opt_summary = test_results['optimization']['summary']
            scores['optimization_score'] = opt_summary['optimization_score']
        else:
            scores['optimization_score'] = 0
        
        # Calculate overall system score
        overall_score = sum(scores.values()) / len(scores)
        
        # Generate report data
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'framework_version': '1.0',
                'test_duration_hours': duration_hours,
                'overall_system_score': overall_score
            },
            'test_summary': {
                'total_test_suites': len(test_results),
                'successful_tests': len([r for r in test_results.values() if r.get('status') == 'success']),
                'failed_tests': len([r for r in test_results.values() if r.get('status') == 'failed']),
                'individual_scores': scores
            },
            'hardware_summary': test_results.get('hardware_detection', {}).get('summary', {}),
            'test_results': test_results,
            'recommendations': self._generate_final_recommendations(test_results),
            'system_grade': self._calculate_system_grade(overall_score)
        }
        
        # Save report
        report_path = f"/workspace/testing/hardware_tests/results/final_hardware_test_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Final comprehensive report generated: {report_path}")
        return report_path
    
    def _generate_final_recommendations(self, test_results: Dict[str, Any]) -> List[str]:
        """Generate final system recommendations"""
        recommendations = []
        
        # Hardware completeness
        if 'hardware_detection' not in test_results or test_results['hardware_detection']['status'] != 'success':
            recommendations.append("Hardware detection failed - check system configuration")
        
        # Compatibility issues
        if 'compatibility' in test_results and test_results['compatibility']['status'] == 'success':
            comp_summary = test_results['compatibility']['summary']
            if comp_summary['failed'] > 0:
                recommendations.append("Hardware compatibility issues detected - check drivers and BIOS settings")
            if comp_summary['needs_driver'] > 0:
                recommendations.append("Driver installation required for some hardware components")
        
        # Performance issues
        if 'multicore' in test_results and test_results['multicore']['status'] == 'success':
            multicore_summary = test_results['multicore']['summary']
            if multicore_summary['max_speedup_achieved'] < multicore_summary['cpu_topology']['logical_cores'] * 0.7:
                recommendations.append("Multi-core performance below expectations - check CPU thermal and power settings")
        
        # Thermal issues
        if 'power_thermal' in test_results and test_results['power_thermal']['status'] == 'success':
            power_summary = test_results['power_thermal']['summary']
            if power_summary.get('max_temperature_c', 0) > 80:
                recommendations.append("High operating temperatures detected - check cooling system")
        
        # Stability issues
        if 'stability' in test_results and test_results['stability']['status'] == 'success':
            stability_summary = test_results['stability']['summary']
            if stability_summary['stability_status'] not in ['stable']:
                recommendations.append("System stability issues detected - monitor for memory leaks or thermal problems")
        
        # Optimization opportunities
        if 'optimization' in test_results and test_results['optimization']['status'] == 'success':
            opt_summary = test_results['optimization']['summary']
            if opt_summary['automatic_optimizations'] > 0:
                recommendations.append(f"Apply {opt_summary['automatic_optimizations']} automatic optimizations")
        
        if not recommendations:
            recommendations.append("System shows good overall hardware compatibility and performance")
        
        return recommendations
    
    def _calculate_system_grade(self, score: float) -> str:
        """Calculate letter grade for system"""
        if score >= 90:
            return "A+"
        elif score >= 85:
            return "A"
        elif score >= 80:
            return "A-"
        elif score >= 75:
            return "B+"
        elif score >= 70:
            return "B"
        elif score >= 65:
            return "B-"
        elif score >= 60:
            return "C+"
        elif score >= 55:
            return "C"
        elif score >= 50:
            return "C-"
        elif score >= 40:
            return "D"
        else:
            return "F"


def main():
    """Main function for standalone execution"""
    parser = argparse.ArgumentParser(description='Hardware Testing Framework Orchestrator')
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Full test suite
    full_parser = subparsers.add_parser('full', help='Run full hardware test suite')
    full_parser.add_argument('--duration', type=int, default=24, 
                           help='Test duration in hours (default: 24)')
    full_parser.add_argument('--skip-stability', action='store_true',
                           help='Skip stability tests (faster execution)')
    
    # Individual test suites
    detection_parser = subparsers.add_parser('detect', help='Detect hardware only')
    compatibility_parser = subparsers.add_parser('compatibility', help='Run compatibility tests')
    compatibility_parser.add_argument('--categories', nargs='+', 
                                    choices=['cpu', 'memory', 'storage', 'network', 'gpu', 'usb', 'system'],
                                    help='Test categories')
    
    peripheral_parser = subparsers.add_parser('peripheral', help='Run peripheral tests')
    multicore_parser = subparsers.add_parser('multicore', help='Run multi-core scaling tests')
    multicore_parser.add_argument('--max-threads', type=int, help='Maximum threads to test')
    
    power_parser = subparsers.add_parser('power', help='Run power and thermal tests')
    power_parser.add_argument('--duration', type=int, default=60,
                            help='Test duration in minutes (default: 60)')
    
    stability_parser = subparsers.add_parser('stability', help='Run stability tests')
    stability_parser.add_argument('--duration', type=int, default=24,
                                help='Test duration in hours (default: 24)')
    
    optimization_parser = subparsers.add_parser('optimize', help='Generate optimization recommendations')
    
    # Report generation
    report_parser = subparsers.add_parser('report', help='Generate test report from existing results')
    report_parser.add_argument('--results-dir', type=str, 
                             default='/workspace/testing/hardware_tests/results',
                             help='Directory containing test results')
    
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--output', type=str, help='Custom output directory')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Set custom output directory if specified
    if args.output:
        os.environ['HARDWARE_TEST_OUTPUT'] = args.output
    
    # Initialize orchestrator
    orchestrator = HardwareTestOrchestrator()
    
    # Execute requested command
    if args.command == 'full':
        print("Running full hardware test suite...")
        results = orchestrator.run_full_test_suite(args.duration, args.skip_stability)
        
        if results['status'] == 'success':
            print(f"\n✓ Full test suite completed successfully!")
            print(f"  Duration: {results['duration_hours']:.1f} hours")
            print(f"  Final report: {results['final_report']}")
            print(f"  Overall score: {results['test_results']['optimization']['summary']['optimization_score']:.1f}/100")
        else:
            print(f"✗ Test suite failed: {results['error']}")
    
    elif args.command == 'detect':
        print("Detecting hardware...")
        results = orchestrator.detect_hardware()
        print(f"Detection completed: {results['status']}")
        if results['status'] == 'success':
            print(f"Profile saved to: {results['profile_path']}")
            print(f"Config saved to: {results['config_path']}")
    
    elif args.command == 'compatibility':
        print("Running compatibility tests...")
        results = orchestrator.run_compatibility_tests(args.categories)
        print(f"Compatibility testing completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  {summary['passed']}/{summary['total_tests']} tests passed")
    
    elif args.command == 'peripheral':
        print("Running peripheral tests...")
        results = orchestrator.run_peripheral_tests()
        print(f"Peripheral testing completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  {summary['passed']}/{summary['total_tests']} tests passed")
    
    elif args.command == 'multicore':
        print("Running multi-core scaling tests...")
        results = orchestrator.run_multicore_scaling_test(args.max_threads)
        print(f"Multi-core testing completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  Optimal threads: {summary['optimal_thread_count']}")
            print(f"  Max speedup: {summary['max_speedup_achieved']:.2f}x")
    
    elif args.command == 'power':
        print("Running power and thermal tests...")
        results = orchestrator.run_power_thermal_tests(args.duration)
        print(f"Power/thermal testing completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  Avg power: {summary['avg_power_consumption_w']:.1f}W")
            print(f"  Max temperature: {summary['max_temperature_c']:.1f}°C")
    
    elif args.command == 'stability':
        print("Running stability tests...")
        results = orchestrator.run_stability_tests(args.duration)
        print(f"Stability testing completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  Status: {summary['stability_status']}")
            print(f"  Duration: {summary['test_duration_hours']:.1f} hours")
    
    elif args.command == 'optimize':
        print("Generating optimization recommendations...")
        results = orchestrator.generate_optimization_recommendations()
        print(f"Optimization analysis completed: {results['status']}")
        if results['status'] == 'success':
            summary = results['summary']
            print(f"  Optimization score: {summary['optimization_score']:.1f}/100")
            print(f"  Recommendations: {summary['total_recommendations']}")
    
    elif args.command == 'report':
        print("Generating comprehensive report...")
        # This would require loading existing results and generating a report
        print("Report generation from existing results not yet implemented")
    
    else:
        parser.print_help()


if __name__ == "__main__":
    main()