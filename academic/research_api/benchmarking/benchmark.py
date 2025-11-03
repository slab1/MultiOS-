"""
Performance Benchmarking Framework

Provides comprehensive performance benchmarking and measurement capabilities
for OS experimentation and research.
"""

import os
import time
import json
import psutil
import threading
import asyncio
from typing import Dict, List, Any, Optional, Callable, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime
import logging
import subprocess
import statistics
import numpy as np
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor

from .config import ResearchConfig


@dataclass
class BenchmarkMetric:
    """Definition of a benchmark metric."""
    name: str
    description: str
    unit: str
    category: str  # 'cpu', 'memory', 'disk', 'network', 'process', 'system'
    measurement_function: Callable
    aggregation_method: str = 'mean'  # 'mean', 'median', 'max', 'min', 'sum'
    lower_is_better: bool = True
    enabled: bool = True


@dataclass
class BenchmarkResult:
    """Result of a single benchmark measurement."""
    metric_name: str
    value: float
    unit: str
    timestamp: datetime
    additional_data: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.additional_data is None:
            self.additional_data = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert result to dictionary."""
        return {
            'metric_name': self.metric_name,
            'value': self.value,
            'unit': self.unit,
            'timestamp': self.timestamp.isoformat(),
            'additional_data': self.additional_data
        }


@dataclass
class BenchmarkRun:
    """Results from a complete benchmark run."""
    benchmark_name: str
    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None
    results: List[BenchmarkResult] = None
    system_info: Dict[str, Any] = None
    configuration: Dict[str, Any] = None
    status: str = 'running'  # 'running', 'completed', 'failed'
    error_message: Optional[str] = None
    
    def __post_init__(self):
        if self.results is None:
            self.results = []
        if self.system_info is None:
            self.system_info = {}
        if self.configuration is None:
            self.configuration = {}
    
    def add_result(self, result: BenchmarkResult):
        """Add a benchmark result."""
        self.results.append(result)
    
    def complete(self, status: str = 'completed', error_message: Optional[str] = None):
        """Mark benchmark as completed."""
        self.end_time = datetime.now()
        self.duration = (self.end_time - self.start_time).total_seconds()
        self.status = status
        self.error_message = error_message
    
    def get_aggregated_results(self) -> Dict[str, float]:
        """Get aggregated results for each metric."""
        aggregated = {}
        
        for result in self.results:
            if result.metric_name not in aggregated:
                aggregated[result.metric_name] = []
            
            aggregated[result.metric_name].append(result.value)
        
        # Aggregate using specified methods
        final_results = {}
        for metric_name, values in aggregated.items():
            final_results[metric_name] = statistics.mean(values)  # Default to mean
        
        return final_results
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert benchmark run to dictionary."""
        data = asdict(self)
        data['start_time'] = self.start_time.isoformat()
        if self.end_time:
            data['end_time'] = self.end_time.isoformat()
        
        # Convert results
        data['results'] = [result.to_dict() for result in self.results]
        
        return data


class SystemProfiler:
    """Profiling utilities for system metrics collection."""
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize system profiler.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Baseline measurements
        self.baseline = {}
        self._collect_baseline()
    
    def _collect_baseline(self):
        """Collect baseline system measurements."""
        self.baseline = {
            'cpu_count': psutil.cpu_count(),
            'cpu_freq': psutil.cpu_freq()._asdict() if psutil.cpu_freq() else {},
            'memory_total': psutil.virtual_memory().total,
            'disk_total': psutil.disk_usage('/').total,
            'network_interfaces': list(psutil.net_if_addrs().keys()),
            'boot_time': datetime.fromtimestamp(psutil.boot_time()),
            'platform': {
                'system': os.uname().sysname,
                'release': os.uname().release,
                'version': os.uname().version,
                'machine': os.uname().machine,
                'processor': os.uname().processor
            }
        }
    
    def get_cpu_usage(self, interval: float = 1.0) -> float:
        """
        Get CPU usage percentage.
        
        Args:
            interval: Measurement interval in seconds
            
        Returns:
            CPU usage percentage (0-100)
        """
        return psutil.cpu_percent(interval=interval)
    
    def get_memory_usage(self) -> Dict[str, float]:
        """
        Get memory usage information.
        
        Returns:
            Dictionary with memory metrics
        """
        memory = psutil.virtual_memory()
        return {
            'total_gb': memory.total / (1024**3),
            'available_gb': memory.available / (1024**3),
            'used_gb': memory.used / (1024**3),
            'percent': memory.percent,
            'free_gb': memory.free / (1024**3),
            'buffers_gb': memory.buffers / (1024**3) if hasattr(memory, 'buffers') else 0,
            'cached_gb': memory.cached / (1024**3) if hasattr(memory, 'cached') else 0
        }
    
    def get_disk_usage(self, path: str = '/') -> Dict[str, float]:
        """
        Get disk usage information.
        
        Args:
            path: Disk path to check
            
        Returns:
            Dictionary with disk metrics
        """
        disk = psutil.disk_usage(path)
        return {
            'total_gb': disk.total / (1024**3),
            'used_gb': disk.used / (1024**3),
            'free_gb': disk.free / (1024**3),
            'percent': (disk.used / disk.total) * 100
        }
    
    def get_network_io(self) -> Dict[str, int]:
        """
        Get network I/O statistics.
        
        Returns:
            Dictionary with network metrics
        """
        net_io = psutil.net_io_counters()
        return {
            'bytes_sent': net_io.bytes_sent,
            'bytes_recv': net_io.bytes_recv,
            'packets_sent': net_io.packets_sent,
            'packets_recv': net_io.packets_recv,
            'errin': net_io.errin,
            'errout': net_io.errout,
            'dropin': net_io.dropin,
            'dropout': net_io.dropout
        }
    
    def get_process_count(self) -> int:
        """Get total number of processes."""
        return len(psutil.pids())
    
    def get_thread_count(self) -> int:
        """Get total number of threads."""
        thread_count = 0
        for proc in psutil.process_iter(['num_threads']):
            try:
                thread_count += proc.info['num_threads']
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        return thread_count
    
    def get_load_average(self) -> Optional[float]:
        """Get system load average."""
        if hasattr(os, 'getloadavg'):
            load1, load5, load15 = os.getloadavg()
            return {
                '1min': load1,
                '5min': load5,
                '15min': load15
            }
        return None
    
    def get_context_switches(self) -> int:
        """Get context switches per second."""
        # This would require a more complex implementation
        # For now, return process-specific data
        total_switches = 0
        for proc in psutil.process_iter(['num_ctx_switches']):
            try:
                ctx_switches = proc.info['num_ctx_switches']
                if ctx_switches:
                    total_switches += ctx_switches.voluntary + ctx_switches.involuntary
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        return total_switches
    
    def get_io_stats(self) -> Dict[str, Any]:
        """Get I/O statistics for all disks."""
        io_counters = psutil.disk_io_counters()
        if io_counters:
            return {
                'read_count': io_counters.read_count,
                'write_count': io_counters.write_count,
                'read_bytes': io_counters.read_bytes,
                'write_bytes': io_counters.write_bytes,
                'read_time': io_counters.read_time,
                'write_time': io_counters.write_time
            }
        return {}
    
    def run_command_benchmark(self, command: str, iterations: int = 5) -> Dict[str, float]:
        """
        Run a command multiple times and measure execution time.
        
        Args:
            command: Command to execute
            iterations: Number of iterations
            
        Returns:
            Dictionary with timing statistics
        """
        execution_times = []
        
        for _ in range(iterations):
            start_time = time.time()
            result = subprocess.run(command, shell=True, capture_output=True, text=True)
            end_time = time.time()
            
            execution_times.append(end_time - start_time)
            
            # Check if command succeeded
            if result.returncode != 0:
                self.logger.warning(f"Command failed: {command}")
        
        return {
            'mean': statistics.mean(execution_times),
            'median': statistics.median(execution_times),
            'min': min(execution_times),
            'max': max(execution_times),
            'std_dev': statistics.stdev(execution_times) if len(execution_times) > 1 else 0,
            'iterations': len(execution_times)
        }
    
    def measure_file_io(self, file_path: str, size_mb: int = 100, operation: str = 'write') -> Dict[str, float]:
        """
        Measure file I/O performance.
        
        Args:
            file_path: Path to test file
            size_mb: Size of test file in MB
            operation: Operation type ('write', 'read', 'copy')
            
        Returns:
            Dictionary with I/O performance metrics
        """
        size_bytes = size_mb * 1024 * 1024
        data = b'0' * size_bytes
        
        start_time = time.time()
        
        if operation == 'write':
            with open(file_path, 'wb') as f:
                f.write(data)
        elif operation == 'read':
            with open(file_path, 'rb') as f:
                data = f.read()
        elif operation == 'copy':
            import shutil
            shutil.copyfile(file_path, f"{file_path}.copy")
            os.remove(f"{file_path}.copy")
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            'duration': duration,
            'throughput_mbps': size_mb / duration,
            'size_mb': size_mb,
            'operation': operation
        }


class ResourceMonitor:
    """Continuous resource monitoring with configurable sampling."""
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize resource monitor.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.profiler = SystemProfiler(config)
        self.logger = logging.getLogger(__name__)
        
        # Monitoring state
        self.monitoring = False
        self.monitor_thread = None
        self.sample_interval = config.performance.measurement_interval
        self.metrics_buffer = []
        
        # Available metrics
        self.available_metrics = self._define_metrics()
    
    def _define_metrics(self) -> List[BenchmarkMetric]:
        """Define available benchmark metrics."""
        return [
            BenchmarkMetric(
                name='cpu_usage',
                description='CPU usage percentage',
                unit='%',
                category='cpu',
                measurement_function=lambda: self.profiler.get_cpu_usage(),
                lower_is_better=False
            ),
            BenchmarkMetric(
                name='memory_usage',
                description='Memory usage percentage',
                unit='%',
                category='memory',
                measurement_function=lambda: self.profiler.get_memory_usage()['percent'],
                lower_is_better=False
            ),
            BenchmarkMetric(
                name='disk_usage',
                description='Disk usage percentage',
                unit='%',
                category='disk',
                measurement_function=lambda: self.profiler.get_disk_usage()['percent'],
                lower_is_better=False
            ),
            BenchmarkMetric(
                name='process_count',
                description='Number of running processes',
                unit='count',
                category='process',
                measurement_function=lambda: self.profiler.get_process_count(),
                lower_is_better=False
            ),
            BenchmarkMetric(
                name='thread_count',
                description='Number of running threads',
                unit='count',
                category='process',
                measurement_function=lambda: self.profiler.get_thread_count(),
                lower_is_better=False
            )
        ]
    
    def start_monitoring(self, 
                        metrics: Optional[List[str]] = None,
                        duration: Optional[float] = None,
                        sample_interval: Optional[float] = None):
        """
        Start continuous resource monitoring.
        
        Args:
            metrics: List of metrics to monitor
            duration: Monitoring duration in seconds
            sample_interval: Sampling interval in seconds
        """
        if self.monitoring:
            self.logger.warning("Monitoring already in progress")
            return
        
        # Setup monitoring parameters
        self.metrics_to_monitor = metrics or [m.name for m in self.available_metrics if m.enabled]
        self.sample_interval = sample_interval or self.config.performance.measurement_interval
        self.monitoring_duration = duration
        
        # Reset buffer
        self.metrics_buffer = []
        
        # Start monitoring thread
        self.monitoring = True
        self.monitor_thread = threading.Thread(target=self._monitoring_loop)
        self.monitor_thread.start()
        
        self.logger.info(f"Started monitoring {len(self.metrics_to_monitor)} metrics")
    
    def stop_monitoring(self) -> List[Dict[str, Any]]:
        """
        Stop monitoring and return collected data.
        
        Returns:
            List of monitoring samples
        """
        if not self.monitoring:
            self.logger.warning("No monitoring in progress")
            return []
        
        self.monitoring = False
        
        if self.monitor_thread:
            self.monitor_thread.join(timeout=5)
        
        self.logger.info("Stopped monitoring")
        return self.metrics_buffer.copy()
    
    def _monitoring_loop(self):
        """Main monitoring loop."""
        start_time = time.time()
        
        while self.monitoring:
            sample_start_time = time.time()
            
            # Collect metrics
            sample = {
                'timestamp': datetime.now(),
                'metrics': {}
            }
            
            for metric_name in self.metrics_to_monitor:
                try:
                    metric = next((m for m in self.available_metrics if m.name == metric_name), None)
                    if metric:
                        value = metric.measurement_function()
                        sample['metrics'][metric_name] = value
                except Exception as e:
                    self.logger.warning(f"Failed to collect metric {metric_name}: {e}")
                    sample['metrics'][metric_name] = None
            
            self.metrics_buffer.append(sample)
            
            # Check duration limit
            elapsed = time.time() - start_time
            if self.monitoring_duration and elapsed >= self.monitoring_duration:
                break
            
            # Wait for next sample
            elapsed_sample = time.time() - sample_start_time
            sleep_time = max(0, self.sample_interval - elapsed_sample)
            
            if sleep_time > 0:
                time.sleep(sleep_time)
    
    def collect_snapshot(self, metrics: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Collect a single snapshot of metrics.
        
        Args:
            metrics: List of metrics to collect
            
        Returns:
            Dictionary with metric values
        """
        metrics_to_collect = metrics or [m.name for m in self.available_metrics if m.enabled]
        snapshot = {'timestamp': datetime.now(), 'metrics': {}}
        
        for metric_name in metrics_to_collect:
            try:
                metric = next((m for m in self.available_metrics if m.name == metric_name), None)
                if metric:
                    value = metric.measurement_function()
                    snapshot['metrics'][metric_name] = value
            except Exception as e:
                self.logger.warning(f"Failed to collect metric {metric_name}: {e}")
                snapshot['metrics'][metric_name] = None
        
        return snapshot


class BenchmarkSuite:
    """Collection of benchmark tests and suites."""
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize benchmark suite.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.profiler = SystemProfiler(config)
        self.monitor = ResourceMonitor(config)
        self.logger = logging.getLogger(__name__)
        
        # Benchmark definitions
        self.benchmarks = self._define_benchmarks()
        self.suites = self._define_suites()
    
    def _define_benchmarks(self) -> Dict[str, Callable]:
        """Define available benchmark tests."""
        return {
            'cpu_intensive': self._benchmark_cpu_intensive,
            'memory_intensive': self._benchmark_memory_intensive,
            'disk_io': self._benchmark_disk_io,
            'network_io': self._benchmark_network_io,
            'file_operations': self._benchmark_file_operations,
            'process_creation': self._benchmark_process_creation,
            'thread_creation': self._benchmark_thread_creation,
            'context_switches': self._benchmark_context_switches,
            'system_calls': self._benchmark_system_calls,
            'memory_allocation': self._benchmark_memory_allocation
        }
    
    def _define_suites(self) -> Dict[str, List[str]]:
        """Define benchmark suites."""
        return {
            'basic': ['cpu_intensive', 'memory_intensive', 'disk_io'],
            'comprehensive': ['cpu_intensive', 'memory_intensive', 'disk_io', 
                            'network_io', 'file_operations', 'process_creation'],
            'stress': ['cpu_intensive', 'memory_intensive', 'process_creation', 
                      'thread_creation', 'memory_allocation'],
            'performance': ['cpu_intensive', 'disk_io', 'file_operations', 
                          'system_calls'],
            'system_analysis': ['process_creation', 'thread_creation', 
                              'context_switches', 'system_calls']
        }
    
    def run_benchmark(self, 
                     benchmark_name: str,
                     iterations: int = 3,
                     duration: Optional[float] = None) -> BenchmarkRun:
        """
        Run a single benchmark.
        
        Args:
            benchmark_name: Name of benchmark to run
            iterations: Number of iterations
            duration: Duration for timed benchmarks
            
        Returns:
            Benchmark run results
        """
        if benchmark_name not in self.benchmarks:
            raise ValueError(f"Unknown benchmark: {benchmark_name}")
        
        self.logger.info(f"Running benchmark: {benchmark_name}")
        
        run = BenchmarkRun(
            benchmark_name=benchmark_name,
            start_time=datetime.now(),
            system_info=self.profiler.baseline,
            configuration={
                'iterations': iterations,
                'duration': duration
            }
        )
        
        try:
            # Execute benchmark
            benchmark_function = self.benchmarks[benchmark_name]
            results = benchmark_function(iterations, duration)
            
            # Process results
            for result_data in results:
                if isinstance(result_data, dict):
                    result = BenchmarkResult(
                        metric_name=result_data['metric'],
                        value=result_data['value'],
                        unit=result_data['unit'],
                        timestamp=datetime.now(),
                        additional_data=result_data.get('additional_data', {})
                    )
                    run.add_result(result)
            
            run.complete('completed')
            
        except Exception as e:
            self.logger.error(f"Benchmark {benchmark_name} failed: {e}")
            run.complete('failed', str(e))
        
        return run
    
    def run_suite(self, suite_name: str, iterations: int = 3) -> Dict[str, BenchmarkRun]:
        """
        Run a benchmark suite.
        
        Args:
            suite_name: Name of suite to run
            iterations: Iterations per benchmark
            
        Returns:
            Dictionary of benchmark results
        """
        if suite_name not in self.suites:
            raise ValueError(f"Unknown suite: {suite_name}")
        
        self.logger.info(f"Running benchmark suite: {suite_name}")
        
        results = {}
        suite_benchmarks = self.suites[suite_name]
        
        for benchmark_name in suite_benchmarks:
            try:
                result = self.run_benchmark(benchmark_name, iterations)
                results[benchmark_name] = result
            except Exception as e:
                self.logger.error(f"Failed to run benchmark {benchmark_name}: {e}")
                # Create failed result
                failed_run = BenchmarkRun(
                    benchmark_name=benchmark_name,
                    start_time=datetime.now(),
                    status='failed',
                    error_message=str(e)
                )
                results[benchmark_name] = failed_run
        
        return results
    
    def _benchmark_cpu_intensive(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark CPU-intensive operations."""
        results = []
        
        # Prime number calculation
        def calculate_primes(n):
            primes = []
            for num in range(2, n):
                for i in range(2, int(num ** 0.5) + 1):
                    if (num % i) == 0:
                        break
                else:
                    primes.append(num)
            return primes
        
        for i in range(iterations):
            start_time = time.time()
            primes = calculate_primes(1000)
            end_time = time.time()
            
            results.append({
                'metric': 'prime_calculation_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'primes_found': len(primes)}
            })
        
        return results
    
    def _benchmark_memory_intensive(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark memory-intensive operations."""
        results = []
        
        for i in range(iterations):
            # Memory allocation and manipulation
            start_time = time.time()
            
            # Allocate large data structures
            data = []
            for j in range(10000):
                data.append([0] * 1000)
            
            # Perform operations
            total = sum(sum(row) for row in data)
            
            # Clear memory
            del data
            
            end_time = time.time()
            
            results.append({
                'metric': 'memory_allocation_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'data_size_mb': 80}  # Approximate
            })
        
        return results
    
    def _benchmark_disk_io(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark disk I/O operations."""
        results = []
        test_file = "/tmp/benchmark_test.dat"
        
        try:
            for i in range(iterations):
                # Write test
                start_time = time.time()
                file_size_mb = 50
                with open(test_file, 'wb') as f:
                    f.write(b'0' * (file_size_mb * 1024 * 1024))
                write_end_time = time.time()
                
                # Read test
                read_start_time = time.time()
                with open(test_file, 'rb') as f:
                    data = f.read()
                read_end_time = time.time()
                
                # Cleanup
                os.remove(test_file)
                
                results.append({
                    'metric': 'disk_write_throughput',
                    'value': file_size_mb / (write_end_time - start_time),
                    'unit': 'MB/s',
                    'additional_data': {'file_size_mb': file_size_mb}
                })
                
                results.append({
                    'metric': 'disk_read_throughput',
                    'value': file_size_mb / (read_end_time - read_start_time),
                    'unit': 'MB/s',
                    'additional_data': {'file_size_mb': file_size_mb}
                })
        
        except Exception as e:
            results.append({
                'metric': 'disk_io_error',
                'value': 0,
                'unit': 'error',
                'additional_data': {'error': str(e)}
            })
        
        return results
    
    def _benchmark_network_io(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark network I/O operations."""
        # This would require actual network testing
        # For now, return placeholder results
        
        results = []
        
        for i in range(iterations):
            start_time = time.time()
            
            # Simulate network operation
            time.sleep(0.1)  # Simulate network latency
            
            end_time = time.time()
            
            results.append({
                'metric': 'network_latency',
                'value': (end_time - start_time) * 1000,  # Convert to ms
                'unit': 'milliseconds',
                'additional_data': {'simulated': True}
            })
        
        return results
    
    def _benchmark_file_operations(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark file operations."""
        results = []
        test_dir = "/tmp/benchmark_files"
        
        try:
            # Create test directory
            os.makedirs(test_dir, exist_ok=True)
            
            for i in range(iterations):
                start_time = time.time()
                
                # Create files
                for j in range(100):
                    file_path = os.path.join(test_dir, f"test_{j}.txt")
                    with open(file_path, 'w') as f:
                        f.write("test data" * 100)
                
                # List files
                files = os.listdir(test_dir)
                
                # Delete files
                for j in range(100):
                    file_path = os.path.join(test_dir, f"test_{j}.txt")
                    os.remove(file_path)
                
                end_time = time.time()
                
                results.append({
                    'metric': 'file_operations_time',
                    'value': end_time - start_time,
                    'unit': 'seconds',
                    'additional_data': {'files_processed': 100}
                })
        
        except Exception as e:
            results.append({
                'metric': 'file_operations_error',
                'value': 0,
                'unit': 'error',
                'additional_data': {'error': str(e)}
            })
        
        finally:
            # Cleanup
            if os.path.exists(test_dir):
                import shutil
                shutil.rmtree(test_dir)
        
        return results
    
    def _benchmark_process_creation(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark process creation."""
        results = []
        
        for i in range(iterations):
            start_time = time.time()
            
            # Create and terminate processes
            processes = []
            for j in range(10):
                proc = subprocess.Popen(['sleep', '0.1'])
                processes.append(proc)
            
            # Wait for all processes
            for proc in processes:
                proc.wait()
            
            end_time = time.time()
            
            results.append({
                'metric': 'process_creation_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'processes_created': 10}
            })
        
        return results
    
    def _benchmark_thread_creation(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark thread creation."""
        results = []
        
        def thread_function():
            time.sleep(0.01)
        
        for i in range(iterations):
            start_time = time.time()
            
            # Create and join threads
            threads = []
            for j in range(50):
                thread = threading.Thread(target=thread_function)
                threads.append(thread)
                thread.start()
            
            # Wait for all threads
            for thread in threads:
                thread.join()
            
            end_time = time.time()
            
            results.append({
                'metric': 'thread_creation_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'threads_created': 50}
            })
        
        return results
    
    def _benchmark_context_switches(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark context switches."""
        results = []
        
        # This would require more sophisticated measurement
        for i in range(iterations):
            switch_count = self.profiler.get_context_switches()
            
            results.append({
                'metric': 'context_switches',
                'value': switch_count,
                'unit': 'count',
                'additional_data': {'measurement_period': 1}
            })
        
        return results
    
    def _benchmark_system_calls(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark system call performance."""
        results = []
        
        for i in range(iterations):
            start_time = time.time()
            
            # Perform various system calls
            for j in range(1000):
                os.getpid()
                os.getcwd()
                time.time()
            
            end_time = time.time()
            
            results.append({
                'metric': 'system_calls_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'calls_made': 1000}
            })
        
        return results
    
    def _benchmark_memory_allocation(self, iterations: int, duration: Optional[float]) -> List[Dict[str, Any]]:
        """Benchmark memory allocation."""
        results = []
        
        import gc
        
        for i in range(iterations):
            # Force garbage collection
            gc.collect()
            
            start_time = time.time()
            
            # Allocate and deallocate memory
            allocations = []
            for j in range(1000):
                allocation = bytearray(1024)  # 1KB allocation
                allocations.append(allocation)
            
            # Clear references
            allocations.clear()
            
            # Force garbage collection
            gc.collect()
            
            end_time = time.time()
            
            results.append({
                'metric': 'memory_allocation_time',
                'value': end_time - start_time,
                'unit': 'seconds',
                'additional_data': {'allocations': 1000, 'allocation_size_kb': 1}
            })
        
        return results
    
    def configure_metrics(self, metrics: List[str]):
        """
        Configure which metrics to collect.
        
        Args:
            metrics: List of metric names to enable
        """
        for metric in self.monitor.available_metrics:
            metric.enabled = metric.name in metrics
        
        self.logger.info(f"Configured metrics: {metrics}")
    
    def get_benchmark_summary(self, results: Dict[str, BenchmarkRun]) -> Dict[str, Any]:
        """
        Generate summary of benchmark results.
        
        Args:
            results: Dictionary of benchmark results
            
        Returns:
            Summary statistics
        """
        summary = {
            'total_benchmarks': len(results),
            'completed': 0,
            'failed': 0,
            'total_duration': 0,
            'metrics_summary': {}
        }
        
        for benchmark_name, result in results.items():
            if result.status == 'completed':
                summary['completed'] += 1
                summary['total_duration'] += result.duration or 0
                
                # Aggregate metrics for this benchmark
                aggregated = result.get_aggregated_results()
                for metric_name, value in aggregated.items():
                    if metric_name not in summary['metrics_summary']:
                        summary['metrics_summary'][metric_name] = []
                    summary['metrics_summary'][metric_name].append(value)
            else:
                summary['failed'] += 1
        
        # Calculate overall statistics
        for metric_name, values in summary['metrics_summary'].items():
            if values:
                summary['metrics_summary'][metric_name] = {
                    'mean': statistics.mean(values),
                    'median': statistics.median(values),
                    'min': min(values),
                    'max': max(values),
                    'std_dev': statistics.stdev(values) if len(values) > 1 else 0
                }
        
        return summary


@dataclass
class PerformanceMetrics:
    """Container for performance metric definitions."""
    cpu_usage: BenchmarkMetric
    memory_usage: BenchmarkMetric
    disk_io: BenchmarkMetric
    network_io: BenchmarkMetric
    process_count: BenchmarkMetric
    thread_count: BenchmarkMetric
    
    def to_list(self) -> List[BenchmarkMetric]:
        """Convert to list of metrics."""
        return [
            self.cpu_usage,
            self.memory_usage,
            self.disk_io,
            self.network_io,
            self.process_count,
            self.thread_count
        ]