#!/usr/bin/env python3
"""
Multi-Core Scaling Test Framework for Real Hardware
Comprehensive testing of CPU multi-core performance, thread scaling, and parallel processing capabilities
"""

import os
import sys
import json
import time
import threading
import logging
import subprocess
import statistics
import multiprocessing as mp
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import psutil
import concurrent.futures
import numpy as np

class ScalingTestStatus(Enum):
    NOT_STARTED = "not_started"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"

@dataclass
class ThreadTestResult:
    """Result of a single thread count test"""
    thread_count: int
    execution_time: float
    operations_per_second: float
    speedup_factor: float
    efficiency_percent: float
    cpu_utilization_percent: List[float]
    memory_usage_mb: float
    frequency_ghz: Optional[float]

@dataclass
class CoreScalingResult:
    """Complete core scaling test result"""
    test_config: Dict[str, Any]
    baseline_single_thread: ThreadTestResult
    scaling_results: List[ThreadTestResult]
    max_threads_tested: int
    cpu_topology: Dict[str, Any]
    hyperthreading_effectiveness: float
    optimal_thread_count: int
    cpu_utilization_patterns: Dict[int, List[float]]

class MultiCoreScalingTester:
    """Main class for multi-core scaling tests"""
    
    def __init__(self):
        self.logger = self._setup_logging()
        self.cpu_info = {}
        self.test_results = []
        self.is_running = False
        self.stop_event = threading.Event()
        
    def _setup_logging(self):
        """Setup logging for multi-core testing"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/multicore_scaling_test.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def detect_cpu_topology(self) -> Dict[str, Any]:
        """Detect CPU topology information"""
        cpu_topology = {
            'physical_cores': psutil.cpu_count(logical=False),
            'logical_cores': psutil.cpu_count(logical=True),
            'sockets': 1,
            'cores_per_socket': 0,
            'threads_per_core': 1,
            'cpu_model': 'Unknown',
            'cpu_vendor': 'Unknown',
            'cpu_features': [],
            'current_frequency_ghz': 0,
            'max_frequency_ghz': 0,
            'min_frequency_ghz': 0,
            'cpu_family': None,
            'cpu_model_number': None,
            'cpu_stepping': None
        }
        
        try:
            # Get detailed CPU information from /proc/cpuinfo
            if os.path.exists('/proc/cpuinfo'):
                with open('/proc/cpuinfo', 'r') as f:
                    cpuinfo_content = f.read()
                
                # Extract basic information
                if 'model name' in cpuinfo_content:
                    model_match = re.search(r'model name\s*:\s*(.+)', cpuinfo_content)
                    if model_match:
                        cpu_topology['cpu_model'] = model_match.group(1).strip()
                
                # Determine vendor
                if 'GenuineIntel' in cpuinfo_content:
                    cpu_topology['cpu_vendor'] = 'Intel'
                elif 'AuthenticAMD' in cpuinfo_content:
                    cpu_topology['cpu_vendor'] = 'AMD'
                
                # Extract frequency information
                freq_match = re.search(r'cpu MHz\s*:\s*([\d.]+)', cpuinfo_content)
                if freq_match:
                    cpu_topology['current_frequency_ghz'] = float(freq_match.group(1)) / 1000
                
                # Extract cache sizes
                cache_match = re.search(r'cache size\s*:\s*(\d+)\s*KB', cpuinfo_content)
                if cache_match:
                    cpu_topology['cache_size_kb'] = int(cache_match.group(1))
                
                # Extract CPU family, model, stepping
                family_match = re.search(r'cpu family\s*:\s*(\d+)', cpuinfo_content)
                if family_match:
                    cpu_topology['cpu_family'] = int(family_match.group(1))
                
                model_num_match = re.search(r'Model\s*:\s*(\d+)', cpuinfo_content)
                if model_num_match:
                    cpu_topology['cpu_model_number'] = int(model_num_match.group(1))
                
                stepping_match = re.search(r'Stepping\s*:\s*(\d+)', cpuinfo_content)
                if stepping_match:
                    cpu_topology['cpu_stepping'] = int(stepping_match.group(1))
                
                # Extract features/flags
                flags_match = re.search(r'flags\s*:\s*(.+)', cpuinfo_content)
                if flags_match:
                    cpu_topology['cpu_features'] = flags_match.group(1).split()
            
            # Use lscpu for more detailed topology if available
            try:
                result = subprocess.run(['lscpu'], capture_output=True, text=True, timeout=10)
                if result.returncode == 0:
                    lines = result.stdout.split('\n')
                    
                    for line in lines:
                        if 'CPU(s):' in line and 'Thread' not in line and 'Core' not in line:
                            cpu_topology['logical_cores'] = int(line.split(':')[1].strip())
                        elif 'Core(s) per socket:' in line:
                            cpu_topology['cores_per_socket'] = int(line.split(':')[1].strip())
                        elif 'Socket(s):' in line:
                            cpu_topology['sockets'] = int(line.split(':')[1].strip())
                        elif 'Thread(s) per core:' in line:
                            cpu_topology['threads_per_core'] = int(line.split(':')[1].strip())
                        elif 'Model name:' in line:
                            cpu_topology['cpu_model'] = line.split(':')[1].strip()
                        elif 'CPU family:' in line:
                            cpu_topology['cpu_family'] = int(line.split(':')[1].strip())
                        elif 'Model:' in line:
                            cpu_topology['cpu_model_number'] = int(line.split(':')[1].strip())
                        elif 'Stepping:' in line:
                            cpu_topology['cpu_stepping'] = int(line.split(':')[1].strip())
                        elif 'CPU max MHz:' in line:
                            cpu_topology['max_frequency_ghz'] = float(line.split(':')[1].strip()) / 1000
                        elif 'CPU min MHz:' in line:
                            cpu_topology['min_frequency_ghz'] = float(line.split(':')[1].strip()) / 1000
                        
            except Exception as e:
                self.logger.warning(f"Could not get detailed CPU topology: {e}")
            
            # Calculate derived values
            if cpu_topology['cores_per_socket'] == 0:
                cpu_topology['cores_per_socket'] = cpu_topology['physical_cores'] // cpu_topology['sockets']
            
            if cpu_topology['threads_per_core'] == 1 and cpu_topology['physical_cores'] != cpu_topology['logical_cores']:
                cpu_topology['threads_per_core'] = cpu_topology['logical_cores'] // cpu_topology['physical_cores']
            
        except Exception as e:
            self.logger.error(f"Error detecting CPU topology: {e}")
        
        self.cpu_info = cpu_topology
        return cpu_topology
    
    def _cpu_intensive_task(self, duration_seconds: float = 5.0) -> int:
        """CPU-intensive task for testing"""
        end_time = time.time() + duration_seconds
        operations = 0
        
        while time.time() < end_time:
            # Prime number calculation (CPU intensive)
            n = 10000
            primes = []
            for num in range(2, n + 1):
                is_prime = True
                for i in range(2, int(num ** 0.5) + 1):
                    if (num % i) == 0:
                        is_prime = False
                        break
                if is_prime:
                    primes.append(num)
            operations += len(primes)
        
        return operations
    
    def _memory_intensive_task(self, duration_seconds: float = 5.0, memory_size_mb: int = 100) -> int:
        """Memory-intensive task for testing"""
        end_time = time.time() + duration_seconds
        operations = 0
        
        # Allocate memory
        memory_chunks = []
        chunk_size = memory_size_mb * 1024 * 1024 // 8  # Size in integers
        
        while time.time() < end_time:
            # Fill memory with data and perform operations
            chunk = list(range(chunk_size))
            
            # Perform some operations on the memory
            for i in range(0, len(chunk), 1000):
                if i < len(chunk):
                    chunk[i] = chunk[i] * 2
            
            operations += len(chunk)
            memory_chunks.append(chunk)
            
            # Limit memory usage
            if len(memory_chunks) > 10:
                memory_chunks = memory_chunks[-5:]  # Keep only last 5 chunks
        
        return operations
    
    def _mixed_workload_task(self, duration_seconds: float = 5.0) -> Dict[str, int]:
        """Mixed CPU/memory workload"""
        end_time = time.time() + duration_seconds
        operations = {'cpu': 0, 'memory': 0, 'io': 0}
        
        while time.time() < end_time:
            # CPU work
            cpu_result = sum(range(10000))
            operations['cpu'] += 1
            
            # Memory work
            memory_data = list(range(1000))
            for i in range(0, len(memory_data), 10):
                if i < len(memory_data):
                    memory_data[i] = memory_data[i] * 2
            operations['memory'] += 1
            
            # Simple I/O simulation
            temp_file = '/tmp/multicore_test.tmp'
            with open(temp_file, 'w') as f:
                f.write('x' * 1000)
            with open(temp_file, 'r') as f:
                data = f.read()
            operations['io'] += 1
            os.remove(temp_file)
        
        return operations
    
    def _measure_cpu_utilization(self, thread_count: int) -> List[float]:
        """Measure CPU utilization during multi-threaded work"""
        utilization_samples = []
        
        def cpu_worker(duration: float = 10.0):
            """Worker function that consumes CPU"""
            end_time = time.time() + duration
            while time.time() < end_time:
                # Simple CPU-intensive calculation
                sum(range(1000))
        
        # Start worker threads
        workers = []
        for _ in range(thread_count):
            thread = threading.Thread(target=cpu_worker)
            thread.start()
            workers.append(thread)
        
        # Measure utilization for a short period
        start_time = time.time()
        while time.time() - start_time < 3.0:
            cpu_percent = psutil.cpu_percent(interval=0.5)
            utilization_samples.append(cpu_percent)
        
        # Wait for workers to complete
        for worker in workers:
            worker.join()
        
        return utilization_samples
    
    def _measure_memory_usage(self) -> float:
        """Measure current memory usage"""
        process = psutil.Process()
        return process.memory_info().rss / (1024 * 1024)  # MB
    
    def _get_current_cpu_frequency(self) -> Optional[float]:
        """Get current CPU frequency"""
        try:
            cpu_freq = psutil.cpu_freq()
            if cpu_freq:
                return cpu_freq.current / 1000.0  # Convert to GHz
        except:
            pass
        return None
    
    def run_single_thread_benchmark(self, test_type: str = 'cpu') -> ThreadTestResult:
        """Run single-threaded benchmark"""
        self.logger.info("Running single-threaded benchmark")
        
        start_time = time.time()
        
        if test_type == 'cpu':
            operations = self._cpu_intensive_task(10.0)  # 10 seconds
        elif test_type == 'memory':
            operations = self._memory_intensive_task(10.0, 100)
        else:  # mixed
            operations_result = self._mixed_workload_task(10.0)
            operations = operations_result['cpu'] + operations_result['memory'] + operations_result['io']
        
        end_time = time.time()
        execution_time = end_time - start_time
        operations_per_second = operations / execution_time if execution_time > 0 else 0
        
        return ThreadTestResult(
            thread_count=1,
            execution_time=execution_time,
            operations_per_second=operations_per_second,
            speedup_factor=1.0,
            efficiency_percent=100.0,
            cpu_utilization_percent=[psutil.cpu_percent(interval=1)],
            memory_usage_mb=self._measure_memory_usage(),
            frequency_ghz=self._get_current_cpu_frequency()
        )
    
    def run_multi_thread_benchmark(self, thread_count: int, test_type: str = 'cpu') -> ThreadTestResult:
        """Run multi-threaded benchmark"""
        self.logger.info(f"Running multi-threaded benchmark with {thread_count} threads")
        
        start_time = time.time()
        
        # Start threads
        workers = []
        
        if test_type == 'cpu':
            for _ in range(thread_count):
                worker = threading.Thread(target=self._cpu_intensive_task, args=(5.0,))
                worker.start()
                workers.append(worker)
        elif test_type == 'memory':
            for _ in range(thread_count):
                worker = threading.Thread(target=self._memory_intensive_task, args=(5.0, 100))
                worker.start()
                workers.append(worker)
        else:  # mixed
            for _ in range(thread_count):
                worker = threading.Thread(target=self._mixed_workload_task, args=(5.0,))
                worker.start()
                workers.append(worker)
        
        # Wait for all workers
        for worker in workers:
            worker.join()
        
        end_time = time.time()
        execution_time = end_time - start_time
        
        # For multi-threaded tasks, we can't easily count total operations
        # We'll use execution time as the metric
        operations_per_second = 1.0 / execution_time if execution_time > 0 else 0
        
        # Measure CPU utilization
        cpu_utilization = self._measure_cpu_utilization(min(thread_count, self.cpu_info['logical_cores']))
        
        return ThreadTestResult(
            thread_count=thread_count,
            execution_time=execution_time,
            operations_per_second=operations_per_second,
            speedup_factor=0.0,  # Will be calculated later
            efficiency_percent=0.0,  # Will be calculated later
            cpu_utilization_percent=cpu_utilization,
            memory_usage_mb=self._measure_memory_usage(),
            frequency_ghz=self._get_current_cpu_frequency()
        )
    
    def run_scaling_analysis(self, max_threads: int = None, test_type: str = 'cpu') -> CoreScalingResult:
        """Run complete scaling analysis"""
        self.logger.info("Starting multi-core scaling analysis")
        
        # Detect CPU topology
        cpu_topology = self.detect_cpu_topology()
        
        if not max_threads:
            max_threads = min(cpu_topology['logical_cores'], 16)  # Limit to 16 threads for testing
        
        self.logger.info(f"Testing scaling from 1 to {max_threads} threads")
        
        # Run baseline single-thread benchmark
        baseline = self.run_single_thread_benchmark(test_type)
        
        scaling_results = []
        utilization_patterns = {}
        
        # Test different thread counts
        thread_counts = [1]
        if max_threads > 1:
            # Add key thread counts to test
            half_cores = cpu_topology['physical_cores'] // 2
            full_cores = cpu_topology['physical_cores']
            half_logical = cpu_topology['logical_cores'] // 2
            full_logical = cpu_topology['logical_cores']
            
            # Add important thread counts
            important_counts = set()
            important_counts.add(1)
            important_counts.add(2)
            if half_cores > 1:
                important_counts.add(half_cores)
            if full_cores > 1:
                important_counts.add(full_cores)
            if half_logical > full_cores:
                important_counts.add(half_logical)
            if full_logical > full_cores:
                important_counts.add(full_logical)
            if max_threads > full_logical:
                important_counts.add(max_threads)
            
            # Fill in remaining counts
            for i in range(1, min(max_threads + 1, 9)):
                important_counts.add(i)
            
            thread_counts = sorted([count for count in important_counts if count <= max_threads])
        
        # Run benchmarks for each thread count
        for thread_count in thread_counts:
            try:
                result = self.run_multi_thread_benchmark(thread_count, test_type)
                
                # Calculate speedup and efficiency relative to baseline
                if baseline.execution_time > 0:
                    speedup = baseline.execution_time / result.execution_time
                    efficiency = (speedup / thread_count) * 100
                else:
                    speedup = 0
                    efficiency = 0
                
                result.speedup_factor = speedup
                result.efficiency_percent = efficiency
                
                scaling_results.append(result)
                utilization_patterns[thread_count] = result.cpu_utilization_percent
                
                self.logger.info(f"Threads: {thread_count}, Speedup: {speedup:.2f}x, Efficiency: {efficiency:.1f}%")
                
            except Exception as e:
                self.logger.error(f"Failed to run benchmark with {thread_count} threads: {e}")
                continue
        
        # Find optimal thread count
        best_result = max(scaling_results, key=lambda r: r.speedup_factor)
        optimal_thread_count = best_result.thread_count
        
        # Calculate hyperthreading effectiveness
        physical_cores = cpu_topology['physical_cores']
        logical_cores = cpu_topology['logical_cores']
        
        if logical_cores > physical_cores:
            # Find performance at physical cores vs logical cores
            physical_performance = None
            logical_performance = None
            
            for result in scaling_results:
                if result.thread_count == physical_cores:
                    physical_performance = result.speedup_factor
                elif result.thread_count == logical_cores:
                    logical_performance = result.speedup_factor
            
            if physical_performance and logical_performance and physical_performance > 0:
                ht_effectiveness = (logical_performance / physical_performance) * 100
            else:
                ht_effectiveness = 0
        else:
            ht_effectiveness = 100  # No hyperthreading
        
        scaling_result = CoreScalingResult(
            test_config={
                'max_threads_tested': max_threads,
                'test_type': test_type,
                'test_duration_seconds': 10.0
            },
            baseline_single_thread=baseline,
            scaling_results=scaling_results,
            max_threads_tested=max(scaling_results, key=lambda r: r.thread_count).thread_count,
            cpu_topology=cpu_topology,
            hyperthreading_effectiveness=ht_effectiveness,
            optimal_thread_count=optimal_thread_count,
            cpu_utilization_patterns=utilization_patterns
        )
        
        return scaling_result
    
    def run_stress_scaling_test(self, duration_minutes: int = 30, thread_counts: List[int] = None) -> Dict[str, Any]:
        """Run extended stress test with multiple thread counts"""
        self.logger.info(f"Starting stress scaling test for {duration_minutes} minutes")
        
        if not thread_counts:
            thread_counts = [1, 2, 4, 8, 16][:self.cpu_info['logical_cores']]
        
        stress_results = {}
        
        for thread_count in thread_counts:
            self.logger.info(f"Running stress test with {thread_count} threads")
            
            start_time = time.time()
            end_time = start_time + (duration_minutes * 60)
            
            # Monitor system during stress test
            measurements = []
            
            while time.time() < end_time:
                # Start worker threads
                workers = []
                for _ in range(thread_count):
                    worker = threading.Thread(target=self._cpu_intensive_task, args=(10.0,))
                    worker.start()
                    workers.append(worker)
                
                # Collect measurements
                measurement = {
                    'timestamp': time.time(),
                    'cpu_usage': psutil.cpu_percent(interval=1),
                    'memory_usage': psutil.virtual_memory().percent,
                    'thread_count': thread_count,
                    'cpu_frequency': self._get_current_cpu_frequency() or 0,
                    'temperature': self._get_cpu_temperature()
                }
                measurements.append(measurement)
                
                # Wait for workers to complete
                for worker in workers:
                    worker.join()
            
            stress_results[thread_count] = {
                'duration_minutes': duration_minutes,
                'measurements': measurements,
                'avg_cpu_usage': statistics.mean([m['cpu_usage'] for m in measurements]),
                'max_cpu_usage': max([m['cpu_usage'] for m in measurements]),
                'avg_memory_usage': statistics.mean([m['memory_usage'] for m in measurements]),
                'avg_cpu_frequency': statistics.mean([m['cpu_frequency'] for m in measurements if m['cpu_frequency'] > 0]),
                'max_temperature': max([m['temperature'] for m in measurements])
            }
        
        return stress_results
    
    def _get_cpu_temperature(self) -> float:
        """Get current CPU temperature"""
        try:
            # Try thermal zones
            thermal_files = glob.glob('/sys/class/thermal/thermal_zone*/temp')
            if thermal_files:
                temps = []
                for temp_file in thermal_files:
                    try:
                        with open(temp_file, 'r') as f:
                            temp_millidegrees = int(f.read().strip())
                            temps.append(temp_millidegrees / 1000.0)
                    except:
                        continue
                if temps:
                    return max(temps)
            
            # Try lm-sensors
            try:
                result = subprocess.run(['sensors'], capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    temp_match = re.search(r'Core 0.*?\+([\d.]+)°C', result.stdout)
                    if temp_match:
                        return float(temp_match.group(1))
            except:
                pass
                
        except Exception:
            pass
        
        return 0.0
    
    def run_affinity_scaling_test(self, test_thread_count: int = 4) -> Dict[str, Any]:
        """Test CPU affinity effects on scaling"""
        self.logger.info(f"Running CPU affinity test with {test_thread_count} threads")
        
        affinity_results = {}
        available_cores = list(range(self.cpu_info['logical_cores']))
        
        # Test different affinity configurations
        affinity_configs = [
            {'name': 'any_cpu', 'cpus': None},
            {'name': 'physical_cores_only', 'cpus': available_cores[:self.cpu_info['physical_cores']]},
            {'name': 'spread_across_cores', 'cpus': [i * 2 for i in range(test_thread_count)] if test_thread_count <= self.cpu_info['physical_cores'] else available_cores[:test_thread_count]},
            {'name': 'contiguous_cores', 'cpus': available_cores[:test_thread_count]}
        ]
        
        for config in affinity_configs:
            try:
                start_time = time.time()
                
                if config['cpus']:
                    # Test with CPU affinity
                    workers = []
                    for i in range(test_thread_count):
                        cpu_id = config['cpus'][i % len(config['cpus'])]
                        worker = threading.Thread(target=self._cpu_intensive_task, args=(5.0,))
                        worker.start()
                        workers.append(worker)
                else:
                    # Test without affinity
                    workers = []
                    for _ in range(test_thread_count):
                        worker = threading.Thread(target=self._cpu_intensive_task, args=(5.0,))
                        worker.start()
                        workers.append(worker)
                
                # Measure performance
                execution_time = time.time() - start_time
                for worker in workers:
                    worker.join()
                
                affinity_results[config['name']] = {
                    'execution_time': execution_time,
                    'cpu_affinity': config['cpus'],
                    'performance_score': 1.0 / execution_time if execution_time > 0 else 0
                }
                
            except Exception as e:
                self.logger.error(f"Failed affinity test {config['name']}: {e}")
                affinity_results[config['name']] = {'error': str(e)}
        
        return affinity_results
    
    def generate_scaling_report(self, scaling_result: CoreScalingResult) -> str:
        """Generate comprehensive scaling analysis report"""
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'test_suite_version': '1.0'
            },
            'cpu_topology': asdict(scaling_result.cpu_topology),
            'test_configuration': scaling_result.test_config,
            'baseline_benchmark': asdict(scaling_result.baseline_single_thread),
            'scaling_results': [asdict(result) for result in scaling_result.scaling_results],
            'analysis': {
                'hyperthreading_effectiveness_percent': scaling_result.hyperthreading_effectiveness,
                'optimal_thread_count': scaling_result.optimal_thread_count,
                'max_speedup_achieved': max([r.speedup_factor for r in scaling_result.scaling_results]),
                'average_efficiency': statistics.mean([r.efficiency_percent for r in scaling_result.scaling_results])
            },
            'performance_recommendations': self._generate_scaling_recommendations(scaling_result)
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/multicore_scaling_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Multi-core scaling report generated: {report_path}")
        return report_path
    
    def _generate_scaling_recommendations(self, scaling_result: CoreScalingResult) -> Dict[str, List[str]]:
        """Generate performance recommendations based on scaling results"""
        recommendations = {
            'thread_optimization': [],
            'affinity_recommendations': [],
            'power_optimization': [],
            'workload_specific': []
        }
        
        # Analyze scaling results
        max_speedup = max([r.speedup_factor for r in scaling_result.scaling_results])
        best_efficiency = max([r.efficiency_percent for r in scaling_result.scaling_results])
        optimal_threads = scaling_result.optimal_thread_count
        
        # Thread count recommendations
        if optimal_threads < scaling_result.cpu_topology['logical_cores']:
            recommendations['thread_optimization'].append(
                f"Consider using {optimal_threads} threads for optimal performance"
            )
        else:
            recommendations['thread_optimization'].append(
                f"All {scaling_result.cpu_topology['logical_cores']} logical cores can be utilized effectively"
            )
        
        # Hyperthreading recommendations
        if scaling_result.hyperthreading_effectiveness < 50:
            recommendations['thread_optimization'].append(
                "Hyperthreading provides limited benefit - consider focusing on physical cores"
            )
        elif scaling_result.hyperthreading_effectiveness > 80:
            recommendations['thread_optimization'].append(
                "Hyperthreading is highly effective - can utilize all logical cores"
            )
        
        # Efficiency recommendations
        avg_efficiency = statistics.mean([r.efficiency_percent for r in scaling_result.scaling_results])
        if avg_efficiency < 70:
            recommendations['workload_specific'].append(
                "Consider workload optimization - scaling efficiency is lower than expected"
            )
        
        # CPU topology recommendations
        if scaling_result.cpu_topology['physical_cores'] != scaling_result.cpu_topology['logical_cores']:
            recommendations['affinity_recommendations'].append(
                "This CPU has hyperthreading - test thread placement for optimal performance"
            )
        
        return recommendations


def main():
    """Main function for standalone execution"""
    import argparse
    import re
    
    parser = argparse.ArgumentParser(description='Multi-Core Scaling Testing')
    parser.add_argument('--test', choices=['scaling', 'stress', 'affinity', 'all'],
                       default='all', help='Test type to run')
    parser.add_argument('--max-threads', type=int, help='Maximum threads to test')
    parser.add_argument('--duration', type=int, default=30, 
                       help='Stress test duration in minutes')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    tester = MultiCoreScalingTester()
    
    # Detect CPU topology
    cpu_topology = tester.detect_cpu_topology()
    print(f"CPU: {cpu_topology['cpu_model']}")
    print(f"Physical cores: {cpu_topology['physical_cores']}, Logical cores: {cpu_topology['logical_cores']}")
    
    results = {}
    
    if args.test in ['scaling', 'all']:
        print("Running scaling analysis...")
        scaling_result = tester.run_scaling_analysis(args.max_threads)
        report_path = tester.generate_scaling_report(scaling_result)
        results['scaling'] = {
            'report_path': report_path,
            'optimal_threads': scaling_result.optimal_thread_count,
            'max_speedup': max([r.speedup_factor for r in scaling_result.scaling_results]),
            'hyperthreading_effectiveness': scaling_result.hyperthreading_effectiveness
        }
    
    if args.test in ['stress', 'all']:
        print("Running stress scaling test...")
        stress_results = tester.run_stress_scaling_test(args.duration)
        results['stress'] = stress_results
    
    if args.test in ['affinity', 'all']:
        print("Running CPU affinity test...")
        affinity_results = tester.run_affinity_scaling_test(4)
        results['affinity'] = affinity_results
    
    # Print summary
    if 'scaling' in results:
        scaling_info = results['scaling']
        print(f"\nScaling Test Summary:")
        print(f"  Optimal thread count: {scaling_info['optimal_threads']}")
        print(f"  Maximum speedup: {scaling_info['max_speedup']:.2f}x")
        print(f"  Hyperthreading effectiveness: {scaling_info['hyperthreading_effectiveness']:.1f}%")
    
    print(f"\nReport generated: {results.get('scaling', {}).get('report_path', 'N/A')}")


if __name__ == "__main__":
    main()