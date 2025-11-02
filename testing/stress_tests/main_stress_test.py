#!/usr/bin/env python3
"""
Comprehensive Stress Testing Suite
Advanced stress testing system for memory management and file systems under extreme conditions.
"""

import os
import sys
import json
import time
import signal
import logging
import threading
import multiprocessing
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
import traceback
import psutil
import subprocess

# Add the stress_tests directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

# Import stress testing modules
from memory.memory_stress import MemoryStressTester
from filesystem.fs_stress import FileSystemStressTester
from cpu.cpu_stress import CPUStressTester
from concurrent.concurrent_stress import ConcurrentStressTester
from resource_exhaustion.resource_exhaustion import ResourceExhaustionTester
from reporting.stress_report import StressReportGenerator
from utils.system_utils import SystemMonitor, StressTestConfig


@dataclass
class TestResult:
    """Individual test result data structure"""
    test_name: str
    category: str
    status: str  # PASS, FAIL, ERROR, TIMEOUT
    start_time: float
    end_time: float
    duration: float
    metrics: Dict[str, Any]
    errors: List[str]
    warnings: List[str]
    memory_usage: Dict[str, float]
    cpu_usage: Dict[str, float]
    disk_usage: Dict[str, float]


class ComprehensiveStressTester:
    """Main orchestrator for comprehensive stress testing suite"""
    
    def __init__(self, config_file: str = None):
        self.config = StressTestConfig.from_file(config_file) if config_file else StressTestConfig()
        self.results: List[TestResult] = []
        self.start_time = time.time()
        self.running = False
        self.threads = []
        
        # Setup logging
        self._setup_logging()
        
        # Initialize test modules
        self.memory_tester = MemoryStressTester(self.config)
        self.fs_tester = FileSystemStressTester(self.config)
        self.cpu_tester = CPUStressTester(self.config)
        self.concurrent_tester = ConcurrentStressTester(self.config)
        self.resource_tester = ResourceExhaustionTester(self.config)
        
        # Initialize system monitor and reporter
        self.monitor = SystemMonitor(self.config)
        self.reporter = StressReportGenerator(self.config)
        
        # Setup signal handlers
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
    
    def _setup_logging(self):
        """Configure logging for stress testing"""
        log_format = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        log_level = logging.DEBUG if self.config.verbose else logging.INFO
        
        # Create logs directory
        log_dir = Path(self.config.output_dir) / 'logs'
        log_dir.mkdir(parents=True, exist_ok=True)
        
        # Setup file and console logging
        logging.basicConfig(
            level=log_level,
            format=log_format,
            handlers=[
                logging.FileHandler(log_dir / f'stress_test_{datetime.now().strftime("%Y%m%d_%H%M%S")}.log'),
                logging.StreamHandler()
            ]
        )
        
        self.logger = logging.getLogger(__name__)
        self.logger.info("Stress testing suite initialized")
    
    def _signal_handler(self, signum, frame):
        """Handle termination signals gracefully"""
        self.logger.warning(f"Received signal {signum}, initiating graceful shutdown...")
        self.running = False
    
    def run_all_tests(self) -> Dict[str, Any]:
        """Run all stress tests in the configured sequence"""
        self.logger.info("Starting comprehensive stress testing suite")
        self.running = True
        
        # Validate system requirements
        if not self._validate_system():
            return self._generate_final_report()
        
        # Run test categories
        test_suites = [
            ("Memory Tests", self._run_memory_tests),
            ("File System Tests", self._run_filesystem_tests),
            ("CPU Tests", self._run_cpu_tests),
            ("Concurrent Tests", self._run_concurrent_tests),
            ("Resource Exhaustion Tests", self._run_resource_exhaustion_tests)
        ]
        
        for suite_name, suite_func in test_suites:
            if not self.running:
                break
                
            self.logger.info(f"Running {suite_name}")
            try:
                suite_func()
            except Exception as e:
                self.logger.error(f"Error in {suite_name}: {str(e)}")
                self.logger.debug(traceback.format_exc())
        
        # Generate final report
        return self._generate_final_report()
    
    def _validate_system(self) -> bool:
        """Validate system requirements before testing"""
        try:
            # Check available memory
            memory = psutil.virtual_memory()
            if memory.available < self.config.min_available_memory_mb * 1024 * 1024:
                self.logger.warning(f"Low available memory: {memory.available / 1024**3:.1f}GB")
            
            # Check available disk space
            disk = psutil.disk_usage(self.config.test_dir)
            if disk.free < self.config.min_available_disk_gb * 1024**3:
                self.logger.warning(f"Low disk space: {disk.free / 1024**3:.1f}GB")
            
            # Check CPU count
            cpu_count = multiprocessing.cpu_count()
            if cpu_count < 2:
                self.logger.warning(f"Low CPU count: {cpu_count}")
            
            self.logger.info("System validation completed")
            return True
            
        except Exception as e:
            self.logger.error(f"System validation failed: {str(e)}")
            return False
    
    def _run_memory_tests(self):
        """Execute memory stress tests"""
        self.logger.info("Starting memory stress tests")
        
        memory_tests = [
            ("Memory Allocation", self.memory_tester.test_allocation_limits),
            ("Memory Leak Detection", self.memory_tester.test_memory_leaks),
            ("Memory Fragmentation", self.memory_tester.test_fragmentation),
            ("Memory Pressure", self.memory_tester.test_memory_pressure),
            ("Memory Overflow Protection", self.memory_tester.test_overflow_protection)
        ]
        
        self._run_test_suite("Memory", memory_tests)
    
    def _run_filesystem_tests(self):
        """Execute file system stress tests"""
        self.logger.info("Starting file system stress tests")
        
        fs_tests = [
            ("File I/O Limits", self.fs_tester.test_io_limits),
            ("Concurrent File Access", self.fs_tester.test_concurrent_access),
            ("File Corruption Simulation", self.fs_tester.test_corruption_scenarios),
            ("Disk Space Exhaustion", self.fs_tester.test_disk_exhaustion),
            ("File Handle Limits", self.fs_tester.test_file_handle_limits)
        ]
        
        self._run_test_suite("FileSystem", fs_tests)
    
    def _run_cpu_tests(self):
        """Execute CPU stress tests"""
        self.logger.info("Starting CPU stress tests")
        
        cpu_tests = [
            ("CPU Stress", self.cpu_tester.test_cpu_stress),
            ("CPU Thermal", self.cpu_tester.test_thermal_throttling),
            ("CPU Scheduling", self.cpu_tester.test_scheduling),
            ("CPU Cache Performance", self.cpu_tester.test_cache_performance)
        ]
        
        self._run_test_suite("CPU", cpu_tests)
    
    def _run_concurrent_tests(self):
        """Execute concurrent process and thread stress tests"""
        self.logger.info("Starting concurrent stress tests")
        
        concurrent_tests = [
            ("Thread Pool Stress", self.concurrent_tester.test_thread_pool),
            ("Process Pool Stress", self.concurrent_tester.test_process_pool),
            ("Thread Synchronization", self.concurrent_tester.test_sync),
            ("Process Communication", self.concurrent_tester.test_ipc),
            ("Resource Contention", self.concurrent_tester.test_resource_contention)
        ]
        
        self._run_test_suite("Concurrent", concurrent_tests)
    
    def _run_resource_exhaustion_tests(self):
        """Execute resource exhaustion tests"""
        self.logger.info("Starting resource exhaustion tests")
        
        resource_tests = [
            ("File Handle Exhaustion", self.resource_tester.test_file_handle_exhaustion),
            ("Memory Exhaustion", self.resource_tester.test_memory_exhaustion),
            ("Process Limit Exhaustion", self.resource_tester.test_process_limit),
            ("Network Connection Limits", self.resource_tester.test_network_limits),
            ("Resource Recovery", self.resource_tester.test_resource_recovery)
        ]
        
        self._run_test_suite("ResourceExhaustion", resource_tests)
    
    def _run_test_suite(self, category: str, tests: List[tuple]):
        """Run a suite of related tests with proper error handling"""
        for test_name, test_func in tests:
            if not self.running:
                break
                
            try:
                self.logger.info(f"Running {test_name} test")
                
                # Start system monitoring
                self.monitor.start_monitoring()
                
                # Run the test
                start_time = time.time()
                test_result = test_func()
                end_time = time.time()
                
                # Stop system monitoring
                monitor_data = self.monitor.stop_monitoring()
                
                # Create test result
                result = TestResult(
                    test_name=test_name,
                    category=category,
                    status=test_result.get("status", "PASS"),
                    start_time=start_time,
                    end_time=end_time,
                    duration=end_time - start_time,
                    metrics=test_result.get("metrics", {}),
                    errors=test_result.get("errors", []),
                    warnings=test_result.get("warnings", []),
                    memory_usage=monitor_data.get("memory", {}),
                    cpu_usage=monitor_data.get("cpu", {}),
                    disk_usage=monitor_data.get("disk", {})
                )
                
                self.results.append(result)
                
                # Log result
                status_symbol = "✓" if result.status == "PASS" else "✗"
                self.logger.info(f"{status_symbol} {test_name}: {result.status} ({result.duration:.2f}s)")
                
                # Recovery test if needed
                if result.status in ["FAIL", "ERROR"]:
                    self._run_recovery_test(test_name, category)
                    
            except Exception as e:
                self.logger.error(f"Error running {test_name}: {str(e)}")
                self.logger.debug(traceback.format_exc())
                
                # Create error result
                result = TestResult(
                    test_name=test_name,
                    category=category,
                    status="ERROR",
                    start_time=time.time(),
                    end_time=time.time(),
                    duration=0,
                    metrics={},
                    errors=[str(e)],
                    warnings=[],
                    memory_usage={},
                    cpu_usage={},
                    disk_usage={}
                )
                self.results.append(result)
    
    def _run_recovery_test(self, failed_test: str, category: str):
        """Run recovery test after a failed test"""
        try:
            self.logger.info(f"Running recovery test for {failed_test}")
            
            # System resource recovery test
            recovery_start = time.time()
            
            # Force garbage collection
            import gc
            gc.collect()
            
            # Wait for system stabilization
            time.sleep(2)
            
            # Check system health
            memory = psutil.virtual_memory()
            disk = psutil.disk_usage(self.config.test_dir)
            
            recovery_metrics = {
                "memory_available": memory.available / 1024**3,
                "disk_free": disk.free / 1024**3,
                "recovered": True
            }
            
            recovery_result = {
                "status": "PASS" if recovery_metrics["memory_available"] > 0.5 else "PARTIAL",
                "metrics": recovery_metrics,
                "errors": [],
                "warnings": []
            }
            
            recovery_end = time.time()
            
            # Create recovery test result
            result = TestResult(
                test_name=f"Recovery_{failed_test}",
                category=f"{category}_Recovery",
                status=recovery_result["status"],
                start_time=recovery_start,
                end_time=recovery_end,
                duration=recovery_end - recovery_start,
                metrics=recovery_result["metrics"],
                errors=recovery_result["errors"],
                warnings=recovery_result["warnings"],
                memory_usage={},
                cpu_usage={},
                disk_usage={}
            )
            
            self.results.append(result)
            
        except Exception as e:
            self.logger.error(f"Recovery test failed: {str(e)}")
    
    def _generate_final_report(self) -> Dict[str, Any]:
        """Generate comprehensive test report"""
        self.logger.info("Generating comprehensive test report")
        
        end_time = time.time()
        total_duration = end_time - self.start_time
        
        # Calculate summary statistics
        total_tests = len(self.results)
        passed_tests = len([r for r in self.results if r.status == "PASS"])
        failed_tests = len([r for r in self.results if r.status == "FAIL"])
        error_tests = len([r for r in self.results if r.status == "ERROR"])
        timeout_tests = len([r for r in self.results if r.status == "TIMEOUT"])
        
        summary = {
            "total_tests": total_tests,
            "passed_tests": passed_tests,
            "failed_tests": failed_tests,
            "error_tests": error_tests,
            "timeout_tests": timeout_tests,
            "success_rate": (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            "total_duration": total_duration,
            "start_time": datetime.fromtimestamp(self.start_time).isoformat(),
            "end_time": datetime.fromtimestamp(end_time).isoformat()
        }
        
        # Generate report data
        report_data = {
            "summary": summary,
            "test_results": [asdict(result) for result in self.results],
            "system_info": self.monitor.get_system_info(),
            "recommendations": self._generate_recommendations()
        }
        
        # Generate reports
        self.reporter.generate_html_report(report_data)
        self.reporter.generate_json_report(report_data)
        self.reporter.generate_csv_report(report_data)
        
        self.logger.info(f"Stress testing completed: {summary['passed_tests']}/{summary['total_tests']} tests passed")
        self.logger.info(f"Success rate: {summary['success_rate']:.1f}%")
        self.logger.info(f"Total duration: {total_duration:.2f} seconds")
        
        return report_data
    
    def _generate_recommendations(self) -> List[str]:
        """Generate recommendations based on test results"""
        recommendations = []
        
        # Analyze memory test results
        memory_results = [r for r in self.results if r.category == "Memory"]
        if any(r.status != "PASS" for r in memory_results):
            recommendations.append("Memory management issues detected. Consider optimizing memory allocation patterns and implementing better garbage collection.")
        
        # Analyze file system results
        fs_results = [r for r in self.results if r.category == "FileSystem"]
        if any(r.status != "PASS" for r in fs_results):
            recommendations.append("File system performance issues detected. Consider optimizing I/O operations and implementing better file handling.")
        
        # Analyze CPU results
        cpu_results = [r for r in self.results if r.category == "CPU"]
        if any(r.status != "PASS" for r in cpu_results):
            recommendations.append("CPU performance issues detected. Consider optimizing CPU-intensive operations and implementing better load balancing.")
        
        # Analyze concurrent results
        concurrent_results = [r for r in self.results if r.category == "Concurrent"]
        if any(r.status != "PASS" for r in concurrent_results):
            recommendations.append("Concurrent processing issues detected. Consider improving synchronization mechanisms and thread management.")
        
        # General recommendations
        if len(recommendations) == 0:
            recommendations.append("All stress tests passed successfully. System demonstrates good resilience under extreme conditions.")
        
        return recommendations


def main():
    """Main entry point for stress testing suite"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Comprehensive Stress Testing Suite")
    parser.add_argument("--config", type=str, help="Configuration file path")
    parser.add_argument("--output", type=str, default="./stress_test_results", help="Output directory")
    parser.add_argument("--duration", type=int, default=300, help="Test duration in seconds")
    parser.add_argument("--parallel", type=int, default=4, help="Number of parallel test threads")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    
    args = parser.parse_args()
    
    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize and run stress tester
    tester = ComprehensiveStressTester(args.config)
    tester.config.output_dir = str(output_dir)
    tester.config.test_duration = args.duration
    tester.config.parallel_threads = args.parallel
    tester.config.verbose = args.verbose
    
    try:
        results = tester.run_all_tests()
        
        # Exit with appropriate code
        success_rate = results["summary"]["success_rate"]
        if success_rate >= 90:
            sys.exit(0)
        elif success_rate >= 70:
            sys.exit(1)
        else:
            sys.exit(2)
            
    except KeyboardInterrupt:
        print("\nStress testing interrupted by user")
        sys.exit(130)
    except Exception as e:
        print(f"Stress testing failed: {str(e)}")
        sys.exit(1)


if __name__ == "__main__":
    main()