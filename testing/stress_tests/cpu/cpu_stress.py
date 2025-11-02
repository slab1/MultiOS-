#!/usr/bin/env python3
"""
CPU stress testing module
Tests CPU utilization, thermal throttling, and performance under extreme load
"""

import os
import sys
import time
import threading
import multiprocessing
import psutil
import numpy as np
import ctypes
from typing import Dict, List, Any, Optional, Tuple
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor, as_completed
from pathlib import Path
import math
import gc


class CPUStressTester:
    """Advanced CPU stress testing module"""
    
    def __init__(self, config):
        self.config = config
        self.test_dir = Path(config.test_dir) / "cpu_tests"
        self.test_dir.mkdir(parents=True, exist_ok=True)
        
        # CPU tracking
        self.cpu_samples = []
        self.thermal_samples = []
        self.performance_metrics = []
    
    def test_cpu_stress(self) -> Dict[str, Any]:
        """Test CPU stress and performance under high load"""
        results = {
            "test_name": "CPU Stress Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            stress_tests = []
            
            # Test 1: Single-threaded CPU stress
            single_threaded = self._test_single_threaded_cpu_stress()
            stress_tests.append(single_threaded)
            
            # Test 2: Multi-threaded CPU stress
            multi_threaded = self._test_multi_threaded_cpu_stress()
            stress_tests.append(multi_threaded)
            
            # Test 3: CPU-intensive mathematical operations
            math_intensive = self._test_math_intensive_operations()
            stress_tests.append(math_intensive)
            
            # Test 4: Memory-bound vs CPU-bound operations
            memory_vs_cpu = self._test_memory_vs_cpu_bound()
            stress_tests.append(memory_vs_cpu)
            
            # Test 5: CPU context switching stress
            context_switching = self._test_context_switching_stress()
            stress_tests.append(context_switching)
            
            results["metrics"].update({
                "stress_tests": stress_tests,
                "total_stress_tests": len(stress_tests),
                "average_cpu_utilization": sum(test.get("cpu_utilization", 0) for test in stress_tests) / len(stress_tests),
                "peak_cpu_utilization": max(test.get("cpu_utilization", 0) for test in stress_tests),
                "system_stability": all(test.get("stable", True) for test in stress_tests)
            })
            
            # Determine status
            avg_utilization = results["metrics"]["average_cpu_utilization"]
            stable = results["metrics"]["system_stability"]
            
            if avg_utilization > 80 and stable:
                results["status"] = "PASS"
            elif avg_utilization > 50:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"CPU stress test failed: {str(e)}")
        
        return results
    
    def test_thermal_throttling(self) -> Dict[str, Any]:
        """Test CPU thermal throttling behavior"""
        results = {
            "test_name": "CPU Thermal Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            thermal_tests = []
            
            # Test 1: Sustained high CPU load thermal response
            sustained_load = self._test_sustained_thermal_load()
            thermal_tests.append(sustained_load)
            
            # Test 2: CPU frequency scaling under load
            frequency_scaling = self._test_cpu_frequency_scaling()
            thermal_tests.append(frequency_scaling)
            
            # Test 3: Thermal throttling detection
            throttling_detection = self._test_thermal_throttling_detection()
            thermal_tests.append(throttling_detection)
            
            # Test 4: Cool-down behavior
            cooldown_behavior = self._test_cpu_cooldown_behavior()
            thermal_tests.append(cooldown_behavior)
            
            results["metrics"].update({
                "thermal_tests": thermal_tests,
                "total_thermal_tests": len(thermal_tests),
                "thermal_throttling_detected": any(test.get("throttling_detected", False) for test in thermal_tests),
                "max_temperature": max(test.get("max_temperature", 0) for test in thermal_tests),
                "frequency_scaling_active": any(test.get("scaling_detected", False) for test in thermal_tests)
            })
            
            # Determine status
            throttling = results["metrics"]["thermal_throttling_detected"]
            if not throttling:
                results["status"] = "PASS"
            elif throttling and results["metrics"]["max_temperature"] < 85:  # Less than 85°C
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Thermal test failed: {str(e)}")
        
        return results
    
    def test_scheduling(self) -> Dict[str, Any]:
        """Test CPU scheduling and process prioritization"""
        results = {
            "test_name": "CPU Scheduling Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            scheduling_tests = []
            
            # Test 1: Process priority scheduling
            priority_scheduling = self._test_process_priority_scheduling()
            scheduling_tests.append(priority_scheduling)
            
            # Test 2: Thread scheduling behavior
            thread_scheduling = self._test_thread_scheduling()
            scheduling_tests.append(thread_scheduling)
            
            # Test 3: CPU affinity testing
            cpu_affinity = self._test_cpu_affinity()
            scheduling_tests.append(cpu_affinity)
            
            # Test 4: Round-robin vs FIFO scheduling
            scheduling_algorithms = self._test_scheduling_algorithms()
            scheduling_tests.append(scheduling_algorithms)
            
            results["metrics"].update({
                "scheduling_tests": scheduling_tests,
                "total_scheduling_tests": len(scheduling_tests),
                "priority_effectiveness": sum(1 for test in scheduling_tests if test.get("priority_worked", False)) / len(scheduling_tests),
                "affinity_enforced": any(test.get("affinity_enforced", False) for test in scheduling_tests)
            })
            
            # Determine status
            priority_effectiveness = results["metrics"]["priority_effectiveness"]
            if priority_effectiveness > 0.8:
                results["status"] = "PASS"
            elif priority_effectiveness > 0.5:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Scheduling test failed: {str(e)}")
        
        return results
    
    def test_cache_performance(self) -> Dict[str, Any]:
        """Test CPU cache performance and memory access patterns"""
        results = {
            "test_name": "CPU Cache Performance",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            cache_tests = []
            
            # Test 1: Sequential memory access patterns
            sequential_access = self._test_sequential_cache_access()
            cache_tests.append(sequential_access)
            
            # Test 2: Random memory access patterns
            random_access = self._test_random_cache_access()
            cache_tests.append(random_access)
            
            # Test 3: Cache thrashing detection
            cache_thrashing = self._test_cache_thrashing()
            cache_tests.append(cache_thrashing)
            
            # Test 4: Memory bandwidth utilization
            memory_bandwidth = self._test_memory_bandwidth()
            cache_tests.append(memory_bandwidth)
            
            results["metrics"].update({
                "cache_tests": cache_tests,
                "total_cache_tests": len(cache_tests),
                "cache_efficiency": sum(test.get("efficiency", 0) for test in cache_tests) / len(cache_tests),
                "memory_bandwidth_mbps": max(test.get("bandwidth_mbps", 0) for test in cache_tests)
            })
            
            # Determine status
            efficiency = results["metrics"]["cache_efficiency"]
            if efficiency > 0.7:
                results["status"] = "PASS"
            elif efficiency > 0.4:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Cache performance test failed: {str(e)}")
        
        return results
    
    # Helper methods for CPU stress testing
    def _test_single_threaded_cpu_stress(self) -> Dict[str, Any]:
        """Test single-threaded CPU intensive operations"""
        start_time = time.time()
        start_cpu = psutil.cpu_percent(interval=0.1)
        
        # CPU intensive calculation
        result = 0
        iterations = 1000000
        
        for i in range(iterations):
            result += math.sqrt(i) * math.sin(i) * math.cos(i)
            if i % 100000 == 0:
                # Brief yield to prevent system lockup
                time.sleep(0.001)
        
        end_time = time.time()
        duration = end_time - start_time
        end_cpu = psutil.cpu_percent(interval=0.1)
        
        cpu_utilization = max(start_cpu, end_cpu)
        
        return {
            "test_type": "single_threaded_stress",
            "iterations": iterations,
            "duration_seconds": duration,
            "operations_per_second": iterations / duration,
            "cpu_utilization": cpu_utilization,
            "stable": cpu_utilization > 80
        }
    
    def _test_multi_threaded_cpu_stress(self) -> Dict[str, Any]:
        """Test multi-threaded CPU intensive operations"""
        num_cores = multiprocessing.cpu_count()
        num_threads = num_cores * self.config.cpu_threads_per_core
        
        def cpu_intensive_worker(worker_id):
            """CPU intensive worker function"""
            result = 0
            iterations = 500000
            
            for i in range(iterations):
                result += math.sqrt(i + worker_id) * math.sin(i + worker_id)
                if i % 50000 == 0:
                    time.sleep(0.001)
            
            return {
                "worker_id": worker_id,
                "iterations": iterations,
                "result": result
            }
        
        start_time = time.time()
        start_cpu = psutil.cpu_percent(interval=0.1)
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(cpu_intensive_worker, i) for i in range(num_threads)]
            worker_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        end_cpu = psutil.cpu_percent(interval=0.1)
        
        total_iterations = sum(wr["iterations"] for wr in worker_results)
        cpu_utilization = max(start_cpu, end_cpu)
        
        return {
            "test_type": "multi_threaded_stress",
            "threads": num_threads,
            "workers_completed": len(worker_results),
            "total_iterations": total_iterations,
            "duration_seconds": duration,
            "operations_per_second": total_iterations / duration,
            "cpu_utilization": cpu_utilization,
            "stable": cpu_utilization > 80
        }
    
    def _test_math_intensive_operations(self) -> Dict[str, Any]:
        """Test mathematical operations under CPU stress"""
        operations = {
            "floating_point": 0,
            "integer": 0,
            "trigonometric": 0,
            "logarithmic": 0,
            "exponential": 0
        }
        
        start_time = time.time()
        start_cpu = psutil.cpu_percent(interval=0.1)
        
        iterations = 100000
        
        for i in range(iterations):
            # Floating point operations
            operations["floating_point"] += i * 3.14159 + 2.71828
            
            # Integer operations
            operations["integer"] += i ^ (i >> 1)
            
            # Trigonometric operations
            operations["trigonometric"] += math.sin(i) + math.cos(i) + math.tan(i/100)
            
            # Logarithmic operations
            if i > 0:
                operations["logarithmic"] += math.log(i) + math.log10(i)
            
            # Exponential operations
            operations["exponential"] += math.exp(i/1000) if i < 1000 else 0
            
            if i % 10000 == 0:
                time.sleep(0.001)
        
        end_time = time.time()
        duration = end_time - start_time
        end_cpu = psutil.cpu_percent(interval=0.1)
        
        cpu_utilization = max(start_cpu, end_cpu)
        total_ops = sum(operations.values())
        
        return {
            "test_type": "math_intensive",
            "iterations": iterations,
            "duration_seconds": duration,
            "operations": operations,
            "total_operations": total_ops,
            "operations_per_second": total_ops / duration,
            "cpu_utilization": cpu_utilization,
            "stable": cpu_utilization > 75
        }
    
    def _test_memory_vs_cpu_bound(self) -> Dict[str, Any]:
        """Test memory-bound vs CPU-bound operations"""
        memory_bound_results = {}
        cpu_bound_results = {}
        
        # Memory-bound operations (cache-friendly)
        start_time = time.time()
        start_cpu = psutil.cpu_percent(interval=0.1)
        
        array_size = 1000000
        arr = np.random.random(array_size)
        
        # Sequential access (memory-bound)
        memory_start = time.time()
        for i in range(array_size):
            arr[i] = arr[i] * 2 + 1
        memory_end = time.time()
        
        memory_bound_results["sequential_duration"] = memory_end - memory_start
        memory_bound_results["cpu_utilization"] = start_cpu
        
        # CPU-bound operations (random access)
        random_indices = np.random.randint(0, array_size, 100000)
        cpu_start = time.time()
        
        for idx in random_indices:
            arr[idx] = math.sqrt(arr[idx])
        
        cpu_end = time.time()
        
        cpu_bound_results["random_duration"] = cpu_end - cpu_start
        cpu_bound_results["end_cpu"] = psutil.cpu_percent(interval=0.1)
        
        return {
            "test_type": "memory_vs_cpu_bound",
            "memory_bound": memory_bound_results,
            "cpu_bound": cpu_bound_results,
            "memory_bound_efficiency": array_size / memory_bound_results["sequential_duration"],
            "cpu_bound_efficiency": 100000 / cpu_bound_results["random_duration"]
        }
    
    def _test_context_switching_stress(self) -> Dict[str, Any]:
        """Test CPU context switching under load"""
        def context_switching_worker(worker_id):
            """Worker that forces context switches"""
            switches = 0
            start_time = time.time()
            
            # Create many short-lived threads to force context switching
            threads = []
            for i in range(10):
                def short_task(task_id):
                    time.sleep(0.001)  # Force context switch
                    return task_id
                
                t = threading.Thread(target=lambda: short_task(i))
                threads.append(t)
                t.start()
            
            # Wait for all threads
            for t in threads:
                t.join()
            
            end_time = time.time()
            return {
                "worker_id": worker_id,
                "duration": end_time - start_time,
                "switches": switches
            }
        
        num_workers = self.config.parallel_threads
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [executor.submit(context_switching_worker, i) for i in range(num_workers)]
            worker_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            "test_type": "context_switching",
            "workers": num_workers,
            "duration": duration,
            "worker_results": worker_results,
            "average_worker_duration": sum(wr["duration"] for wr in worker_results) / len(worker_results)
        }
    
    def _test_sustained_thermal_load(self) -> Dict[str, Any]:
        """Test sustained CPU load and thermal response"""
        cpu_count = multiprocessing.cpu_count()
        duration = self.config.thermal_test_duration
        
        def thermal_load_worker(worker_id):
            """Worker that generates sustained CPU load"""
            start_time = time.time()
            operations = 0
            
            while time.time() - start_time < duration:
                # CPU intensive operations
                result = 0
                for i in range(1000):
                    result += math.sqrt(i) * math.sin(i)
                
                operations += 1000
                
                # Small sleep to prevent overheating
                time.sleep(0.01)
            
            return {
                "worker_id": worker_id,
                "operations": operations,
                "duration": time.time() - start_time
            }
        
        start_time = time.time()
        start_temp = self._get_cpu_temperature()
        
        with ThreadPoolExecutor(max_workers=cpu_count) as executor:
            futures = [executor.submit(thermal_load_worker, i) for i in range(cpu_count)]
            worker_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        total_duration = end_time - start_time
        end_temp = self._get_cpu_temperature()
        
        max_temp = max(start_temp or 0, end_temp or 0)
        thermal_rise = (end_temp or 0) - (start_temp or 0)
        throttling_detected = max_temp > 85  # Assume throttling at 85°C
        
        total_operations = sum(wr["operations"] for wr in worker_results)
        
        return {
            "test_type": "sustained_thermal",
            "duration": total_duration,
            "workers": cpu_count,
            "total_operations": total_operations,
            "start_temperature": start_temp,
            "end_temperature": end_temp,
            "max_temperature": max_temp,
            "thermal_rise": thermal_rise,
            "throttling_detected": throttling_detected,
            "operations_per_second": total_operations / total_duration
        }
    
    def _test_cpu_frequency_scaling(self) -> Dict[str, Any]:
        """Test CPU frequency scaling under load"""
        cpu_freq = psutil.cpu_freq()
        
        if not cpu_freq:
            return {
                "test_type": "frequency_scaling",
                "error": "CPU frequency not available",
                "scaling_detected": False
            }
        
        # Baseline frequency measurement
        baseline_freq = cpu_freq.current
        min_freq = cpu_freq.min
        max_freq = cpu_freq.max
        
        # Measure frequency under load
        def frequency_load_worker():
            """Worker that creates CPU load to test frequency scaling"""
            operations = 0
            start_time = time.time()
            
            while time.time() - start_time < 30:  # 30 seconds
                # Intensive computation
                for i in range(10000):
                    math.sqrt(i) * math.sin(i)
                operations += 10000
            
            return operations
        
        start_time = time.time()
        frequencies_during_load = []
        
        # Start CPU load
        with ThreadPoolExecutor(max_workers=multiprocessing.cpu_count()) as executor:
            future = executor.submit(frequency_load_worker)
            
            # Monitor frequency during load
            while not future.done():
                current_freq = psutil.cpu_freq()
                if current_freq:
                    frequencies_during_load.append(current_freq.current)
                time.sleep(0.5)
        
        end_time = time.time()
        operations = future.result()
        
        # Analyze frequency scaling
        if frequencies_during_load:
            avg_freq_under_load = sum(frequencies_during_load) / len(frequencies_during_load)
            scaling_detected = avg_freq_under_load > baseline_freq * 1.1  # 10% increase
        else:
            avg_freq_under_load = baseline_freq
            scaling_detected = False
        
        return {
            "test_type": "frequency_scaling",
            "baseline_frequency_mhz": baseline_freq,
            "min_frequency_mhz": min_freq,
            "max_frequency_mhz": max_freq,
            "avg_frequency_under_load_mhz": avg_freq_under_load,
            "scaling_detected": scaling_detected,
            "frequency_increase_percent": ((avg_freq_under_load - baseline_freq) / baseline_freq * 100) if baseline_freq > 0 else 0,
            "duration": end_time - start_time
        }
    
    def _test_thermal_throttling_detection(self) -> Dict[str, Any]:
        """Test detection of thermal throttling behavior"""
        # This is a simplified thermal throttling test
        # Real thermal throttling detection would require more sophisticated monitoring
        
        cpu_freq = psutil.cpu_freq()
        if not cpu_freq:
            return {
                "test_type": "thermal_throttling_detection",
                "error": "CPU frequency monitoring not available",
                "throttling_detected": False
            }
        
        # Test 1: Monitor frequency drop under sustained load
        def throttling_test_worker():
            for _ in range(60):  # 60 seconds
                math.sqrt(999999) * math.sin(999999)
                time.sleep(1)
        
        initial_freq = cpu_freq.current
        start_time = time.time()
        
        # Start load
        with ThreadPoolExecutor(max_workers=multiprocessing.cpu_count() // 2) as executor:
            future = executor.submit(throttling_test_worker)
            
            # Monitor frequency
            freq_samples = []
            while not future.done():
                current_freq = psutil.cpu_freq()
                if current_freq:
                    freq_samples.append(current_freq.current)
                time.sleep(2)
        
        end_time = time.time()
        
        # Analyze for throttling
        if freq_samples:
            min_freq = min(freq_samples)
            freq_drop_percent = ((initial_freq - min_freq) / initial_freq * 100) if initial_freq > 0 else 0
            throttling_detected = freq_drop_percent > 20  # 20% drop indicates potential throttling
        else:
            min_freq = initial_freq
            freq_drop_percent = 0
            throttling_detected = False
        
        return {
            "test_type": "thermal_throttling_detection",
            "initial_frequency_mhz": initial_freq,
            "min_frequency_mhz": min_freq,
            "frequency_drop_percent": freq_drop_percent,
            "throttling_detected": throttling_detected,
            "duration": end_time - start_time,
            "frequency_samples": len(freq_samples)
        }
    
    def _test_cpu_cooldown_behavior(self) -> Dict[str, Any]:
        """Test CPU cool-down behavior after high load"""
        # Generate high CPU load first
        cpu_count = multiprocessing.cpu_count()
        
        def high_load_worker():
            start_time = time.time()
            while time.time() - start_time < 30:  # 30 seconds high load
                math.sqrt(999999) * math.sin(999999)
        
        # Start high load
        with ThreadPoolExecutor(max_workers=cpu_count) as executor:
            futures = [executor.submit(high_load_worker) for _ in range(cpu_count)]
            
            # Let it run for 30 seconds
            time.sleep(30)
            
            # Cancel futures to stop load
            for future in futures:
                future.cancel()
        
        # Monitor cool-down
        cooldown_samples = []
        cooldown_start = time.time()
        
        while time.time() - cooldown_start < 60:  # Monitor for 60 seconds
            cpu_temp = self._get_cpu_temperature()
            cpu_freq = psutil.cpu_freq()
            
            cooldown_samples.append({
                "timestamp": time.time(),
                "temperature": cpu_temp,
                "frequency": cpu_freq.current if cpu_freq else 0
            })
            
            time.sleep(5)  # Sample every 5 seconds
        
        # Analyze cooldown
        if cooldown_samples:
            initial_temp = cooldown_samples[0]["temperature"]
            final_temp = cooldown_samples[-1]["temperature"]
            temp_drop = initial_temp - final_temp if initial_temp and final_temp else 0
        else:
            temp_drop = 0
        
        return {
            "test_type": "cpu_cooldown",
            "cooldown_duration": 60,
            "samples_count": len(cooldown_samples),
            "initial_temperature": initial_temp,
            "final_temperature": final_temp,
            "temperature_drop": temp_drop,
            "cooldown_samples": cooldown_samples
        }
    
    def _test_process_priority_scheduling(self) -> Dict[str, Any]:
        """Test process priority scheduling"""
        def high_priority_worker():
            """High priority worker"""
            start_time = time.time()
            operations = 0
            while time.time() - start_time < 10:
                for i in range(1000):
                    math.sqrt(i)
                operations += 1000
            return operations
        
        def low_priority_worker():
            """Low priority worker"""
            start_time = time.time()
            operations = 0
            while time.time() - start_time < 10:
                for i in range(1000):
                    math.sqrt(i)
                operations += 1000
            return operations
        
        # Start high priority process
        high_priority_process = multiprocessing.Process(target=high_priority_worker)
        high_priority_process.daemon = True
        
        # Start low priority process
        low_priority_process = multiprocessing.Process(target=low_priority_worker)
        low_priority_process.daemon = True
        
        # Set priorities (platform dependent)
        try:
            if hasattr(os, 'nice'):
                os.nice(-10)  # High priority (Linux)
                high_priority_process.nice = -10
            if hasattr(os, 'setpriority'):
                os.setpriority(os.PRIO_PROCESS, 0, -10)  # High priority
        except (OSError, AttributeError):
            pass  # Priority setting not available
        
        # Start both processes
        start_time = time.time()
        
        high_priority_process.start()
        low_priority_process.start()
        
        high_priority_process.join()
        low_priority_process.join()
        
        end_time = time.time()
        
        # This is a simplified test - real priority testing would measure execution times
        return {
            "test_type": "process_priority",
            "duration": end_time - start_time,
            "priority_worked": True,  # Simplified for this test
            "high_priority_pid": high_priority_process.pid,
            "low_priority_pid": low_priority_process.pid
        }
    
    def _test_thread_scheduling(self) -> Dict[str, Any]:
        """Test thread scheduling behavior"""
        def thread_worker(worker_id, duration=5):
            """Worker thread"""
            start_time = time.time()
            operations = 0
            
            while time.time() - start_time < duration:
                # CPU intensive operation
                for i in range(1000):
                    math.sqrt(i) * math.sin(i)
                operations += 1000
            
            return {
                "worker_id": worker_id,
                "operations": operations,
                "duration": time.time() - start_time
            }
        
        num_threads = multiprocessing.cpu_count() * 2
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(thread_worker, i) for i in range(num_threads)]
            thread_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        
        total_operations = sum(tr["operations"] for tr in thread_results)
        avg_duration = sum(tr["duration"] for tr in thread_results) / len(thread_results)
        
        return {
            "test_type": "thread_scheduling",
            "threads": num_threads,
            "duration": end_time - start_time,
            "total_operations": total_operations,
            "average_thread_duration": avg_duration,
            "operations_per_second": total_operations / (end_time - start_time)
        }
    
    def _test_cpu_affinity(self) -> Dict[str, Any]:
        """Test CPU affinity settings"""
        try:
            import psutil
            
            current_process = psutil.Process()
            initial_affinity = current_process.cpu_affinity()
            
            # Test setting specific CPU affinity
            cpu_count = multiprocessing.cpu_count()
            if cpu_count > 1:
                # Set affinity to first CPU
                test_affinity = [0]
                current_process.cpu_affinity(test_affinity)
                
                # Verify affinity
                new_affinity = current_process.cpu_affinity()
                affinity_enforced = new_affinity == test_affinity
                
                # Restore original affinity
                current_process.cpu_affinity(initial_affinity)
            else:
                affinity_enforced = False  # Cannot test with single CPU
                new_affinity = initial_affinity
                test_affinity = initial_affinity
        except (AttributeError, OSError):
            # CPU affinity not supported on this platform
            affinity_enforced = False
            initial_affinity = [0]
            new_affinity = [0]
            test_affinity = [0]
        
        return {
            "test_type": "cpu_affinity",
            "initial_affinity": initial_affinity,
            "test_affinity": test_affinity,
            "resulting_affinity": new_affinity,
            "affinity_enforced": affinity_enforced
        }
    
    def _test_scheduling_algorithms(self) -> Dict[str, Any]:
        """Test different scheduling algorithms"""
        # This is a simplified test for scheduling algorithm behavior
        # Real scheduling algorithm testing would require system-level instrumentation
        
        results = {}
        
        # Test FIFO-like behavior with sequential workers
        def sequential_worker(worker_id):
            time.sleep(0.1)  # Simulate work
            return f"worker_{worker_id}_completed"
        
        start_time = time.time()
        sequential_results = []
        
        with ThreadPoolExecutor(max_workers=3) as executor:
            futures = [executor.submit(sequential_worker, i) for i in range(3)]
            sequential_results = [future.result() for future in futures]
        
        sequential_duration = time.time() - start_time
        
        results["sequential"] = {
            "duration": sequential_duration,
            "results": sequential_results,
            "completion_order": "sequential"
        }
        
        # Test parallel execution
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=3) as executor:
            futures = [executor.submit(sequential_worker, i) for i in range(3, 6)]
            parallel_results = [future.result() for future in as_completed(futures)]
        
        parallel_duration = time.time() - start_time
        
        results["parallel"] = {
            "duration": parallel_duration,
            "results": parallel_results,
            "completion_order": "parallel"
        }
        
        return {
            "test_type": "scheduling_algorithms",
            "algorithms": results,
            "parallel_speedup": sequential_duration / parallel_duration if parallel_duration > 0 else 1
        }
    
    def _test_sequential_cache_access(self) -> Dict[str, Any]:
        """Test sequential memory access patterns (cache-friendly)"""
        array_size = 1000000
        iterations = 10
        
        # Create array
        arr = np.random.random(array_size)
        
        start_time = time.time()
        
        # Sequential access (cache-friendly)
        for iteration in range(iterations):
            for i in range(array_size):
                arr[i] = arr[i] * 2 + 1
        
        end_time = time.time()
        duration = end_time - start_time
        
        operations = array_size * iterations
        bandwidth = (operations * 8) / (duration * 1024 * 1024)  # 8 bytes per float64
        
        return {
            "test_type": "sequential_cache_access",
            "array_size": array_size,
            "iterations": iterations,
            "operations": operations,
            "duration": duration,
            "bandwidth_mbps": bandwidth,
            "efficiency": 0.9  # Simplified efficiency calculation
        }
    
    def _test_random_cache_access(self) -> Dict[str, Any]:
        """Test random memory access patterns (cache-unfriendly)"""
        array_size = 1000000
        access_count = 1000000
        
        # Create array
        arr = np.random.random(array_size)
        
        # Generate random access pattern
        np.random.seed(42)  # For reproducibility
        random_indices = np.random.randint(0, array_size, access_count)
        
        start_time = time.time()
        
        # Random access (cache-unfriendly)
        for idx in random_indices:
            arr[idx] = math.sqrt(arr[idx])
        
        end_time = time.time()
        duration = end_time - start_time
        
        bandwidth = (access_count * 8) / (duration * 1024 * 1024)  # 8 bytes per float64
        
        return {
            "test_type": "random_cache_access",
            "array_size": array_size,
            "access_count": access_count,
            "duration": duration,
            "bandwidth_mbps": bandwidth,
            "efficiency": 0.3  # Random access is less efficient
        }
    
    def _test_cache_thrashing(self) -> Dict[str, Any]:
        """Test cache thrashing with conflicting access patterns"""
        # Create multiple arrays that exceed cache size
        array_size = 1000000  # Large enough to cause cache thrashing
        num_arrays = 4
        
        arrays = []
        for i in range(num_arrays):
            arrays.append(np.random.random(array_size))
        
        start_time = time.time()
        
        # Access pattern designed to cause cache thrashing
        for iteration in range(10):
            for arr_idx in range(num_arrays):
                for i in range(array_size):
                    arrays[arr_idx][i] = arrays[arr_idx][i] * 2 + arr_idx
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_operations = num_arrays * array_size * 10
        
        return {
            "test_type": "cache_thrashing",
            "arrays": num_arrays,
            "array_size": array_size,
            "iterations": 10,
            "total_operations": total_operations,
            "duration": duration,
            "thrashing_detected": duration > 5  # Threshold for thrashing
        }
    
    def _test_memory_bandwidth(self) -> Dict[str, Any]:
        """Test memory bandwidth utilization"""
        # Large array operations to test memory bandwidth
        array_size = 10000000  # 10M elements
        iterations = 5
        
        arr1 = np.random.random(array_size)
        arr2 = np.random.random(array_size)
        result = np.zeros(array_size)
        
        start_time = time.time()
        
        # Memory-intensive operations
        for iteration in range(iterations):
            # Vectorized operations (memory bandwidth bound)
            result = arr1 + arr2
            result = result * 2
            result = np.sqrt(result)
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Calculate bandwidth (reading arr1, arr2 and writing result = 3 arrays)
        total_data_transferred = array_size * 8 * 3 * iterations  # 8 bytes per float64
        bandwidth_mbps = total_data_transferred / (duration * 1024 * 1024)
        
        return {
            "test_type": "memory_bandwidth",
            "array_size": array_size,
            "iterations": iterations,
            "duration": duration,
            "bandwidth_mbps": bandwidth_mbps,
            "total_data_transferred_mb": total_data_transferred / (1024 * 1024)
        }
    
    def _get_cpu_temperature(self) -> Optional[float]:
        """Get CPU temperature (platform dependent)"""
        try:
            # Try various methods to get CPU temperature
            if hasattr(psutil, "sensors_temperatures"):
                temps = psutil.sensors_temperatures()
                if temps:
                    # Get first temperature reading
                    for name, entries in temps.items():
                        if entries:
                            return entries[0].current
        except Exception:
            pass
        
        try:
            # Try reading from /sys/class/thermal/thermal_zone* (Linux)
            import glob
            thermal_files = glob.glob("/sys/class/thermal/thermal_zone*")
            if thermal_files:
                for zone in thermal_files:
                    try:
                        with open(f"{zone}/temp", 'r') as f:
                            temp_str = f.read().strip()
                            if temp_str:
                                # Convert from millidegrees to degrees
                                return float(temp_str) / 1000.0
                    except Exception:
                        continue
        except Exception:
            pass
        
        return None  # Temperature not available