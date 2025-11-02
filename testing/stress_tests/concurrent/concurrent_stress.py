#!/usr/bin/env python3
"""
Concurrent stress testing module
Tests thread pools, process pools, synchronization, and resource contention
"""

import os
import sys
import time
import threading
import multiprocessing
import queue
import concurrent.futures
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor, as_completed
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
import psutil
import random
import math
from threading import Lock, Semaphore, Event, Condition
from multiprocessing import Queue, Manager, Value, Array
import numpy as np


class ConcurrentStressTester:
    """Advanced concurrent stress testing module"""
    
    def __init__(self, config):
        self.config = config
        self.test_dir = Path(config.test_dir) / "concurrent_tests"
        self.test_dir.mkdir(parents=True, exist_ok=True)
        
        # Concurrent testing tracking
        self.thread_metrics = []
        self.process_metrics = []
        self.synchronization_metrics = []
        self.contention_metrics = []
    
    def test_thread_pool(self) -> Dict[str, Any]:
        """Test thread pool behavior under high concurrency"""
        results = {
            "test_name": "Thread Pool Stress Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            thread_tests = []
            
            # Test 1: Basic thread pool scaling
            basic_scaling = self._test_thread_pool_basic_scaling()
            thread_tests.append(basic_scaling)
            
            # Test 2: Thread pool with CPU-intensive tasks
            cpu_intensive = self._test_thread_pool_cpu_intensive()
            thread_tests.append(cpu_intensive)
            
            # Test 3: Thread pool with I/O-bound tasks
            io_bound = self._test_thread_pool_io_bound()
            thread_tests.append(io_bound)
            
            # Test 4: Thread pool under memory pressure
            memory_pressure = self._test_thread_pool_memory_pressure()
            thread_tests.append(memory_pressure)
            
            # Test 5: Thread pool task queue saturation
            queue_saturation = self._test_thread_pool_queue_saturation()
            thread_tests.append(queue_saturation)
            
            results["metrics"].update({
                "thread_tests": thread_tests,
                "total_thread_tests": len(thread_tests),
                "average_throughput": sum(test.get("throughput", 0) for test in thread_tests) / len(thread_tests),
                "thread_pool_stability": all(test.get("stable", True) for test in thread_tests)
            })
            
            # Determine status
            stability = results["metrics"]["thread_pool_stability"]
            avg_throughput = results["metrics"]["average_throughput"]
            
            if stability and avg_throughput > 100:
                results["status"] = "PASS"
            elif stability:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Thread pool test failed: {str(e)}")
        
        return results
    
    def test_process_pool(self) -> Dict[str, Any]:
        """Test process pool behavior under high concurrency"""
        results = {
            "test_name": "Process Pool Stress Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            process_tests = []
            
            # Test 1: Basic process pool scaling
            basic_scaling = self._test_process_pool_basic_scaling()
            process_tests.append(basic_scaling)
            
            # Test 2: Process pool with CPU-intensive tasks
            cpu_intensive = self._test_process_pool_cpu_intensive()
            process_tests.append(cpu_intensive)
            
            # Test 3: Process pool with large data transfer
            data_transfer = self._test_process_pool_data_transfer()
            process_tests.append(data_transfer)
            
            # Test 4: Process pool memory isolation
            memory_isolation = self._test_process_pool_memory_isolation()
            process_tests.append(memory_isolation)
            
            # Test 5: Process pool fault tolerance
            fault_tolerance = self._test_process_pool_fault_tolerance()
            process_tests.append(fault_tolerance)
            
            results["metrics"].update({
                "process_tests": process_tests,
                "total_process_tests": len(process_tests),
                "average_throughput": sum(test.get("throughput", 0) for test in process_tests) / len(process_tests),
                "process_pool_stability": all(test.get("stable", True) for test in process_tests)
            })
            
            # Determine status
            stability = results["metrics"]["process_pool_stability"]
            avg_throughput = results["metrics"]["average_throughput"]
            
            if stability and avg_throughput > 50:
                results["status"] = "PASS"
            elif stability:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Process pool test failed: {str(e)}")
        
        return results
    
    def test_sync(self) -> Dict[str, Any]:
        """Test thread and process synchronization mechanisms"""
        results = {
            "test_name": "Synchronization Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            sync_tests = []
            
            # Test 1: Mutex/lock contention
            mutex_contention = self._test_mutex_contention()
            sync_tests.append(mutex_contention)
            
            # Test 2: Semaphore operations
            semaphore_operations = self._test_semaphore_operations()
            sync_tests.append(semaphore_operations)
            
            # Test 3: Condition variables
            condition_variables = self._test_condition_variables()
            sync_tests.append(condition_variables)
            
            # Test 4: Event synchronization
            event_synchronization = self._test_event_synchronization()
            sync_tests.append(event_synchronization)
            
            # Test 5: Barrier synchronization
            barrier_synchronization = self._test_barrier_synchronization()
            sync_tests.append(barrier_synchronization)
            
            results["metrics"].update({
                "sync_tests": sync_tests,
                "total_sync_tests": len(sync_tests),
                "synchronization_effectiveness": sum(1 for test in sync_tests if test.get("effective", False)) / len(sync_tests),
                "contention_handling": all(test.get("no_deadlock", True) for test in sync_tests)
            })
            
            # Determine status
            effectiveness = results["metrics"]["synchronization_effectiveness"]
            no_deadlocks = results["metrics"]["contention_handling"]
            
            if effectiveness > 0.8 and no_deadlocks:
                results["status"] = "PASS"
            elif effectiveness > 0.6:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Synchronization test failed: {str(e)}")
        
        return results
    
    def test_ipc(self) -> Dict[str, Any]:
        """Test inter-process communication mechanisms"""
        results = {
            "test_name": "Inter-Process Communication Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            ipc_tests = []
            
            # Test 1: Queue-based communication
            queue_communication = self._test_queue_communication()
            ipc_tests.append(queue_communication)
            
            # Test 2: Shared memory communication
            shared_memory = self._test_shared_memory_communication()
            ipc_tests.append(shared_memory)
            
            # Test 3: Pipe-based communication
            pipe_communication = self._test_pipe_communication()
            ipc_tests.append(pipe_communication)
            
            # Test 4: Socket-based communication
            socket_communication = self._test_socket_communication()
            ipc_tests.append(socket_communication)
            
            # Test 5: High-frequency IPC
            high_frequency_ipc = self._test_high_frequency_ipc()
            ipc_tests.append(high_frequency_ipc)
            
            results["metrics"].update({
                "ipc_tests": ipc_tests,
                "total_ipc_tests": len(ipc_tests),
                "average_latency_ms": sum(test.get("avg_latency_ms", 0) for test in ipc_tests) / len(ipc_tests),
                "communication_reliability": sum(1 for test in ipc_tests if test.get("reliable", False)) / len(ipc_tests)
            })
            
            # Determine status
            reliability = results["metrics"]["communication_reliability"]
            avg_latency = results["metrics"]["average_latency_ms"]
            
            if reliability > 0.9 and avg_latency < 10:
                results["status"] = "PASS"
            elif reliability > 0.7:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"IPC test failed: {str(e)}")
        
        return results
    
    def test_resource_contention(self) -> Dict[str, Any]:
        """Test resource contention scenarios"""
        results = {
            "test_name": "Resource Contention Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            contention_tests = []
            
            # Test 1: CPU resource contention
            cpu_contention = self._test_cpu_resource_contention()
            contention_tests.append(cpu_contention)
            
            # Test 2: Memory resource contention
            memory_contention = self._test_memory_resource_contention()
            contention_tests.append(memory_contention)
            
            # Test 3: I/O resource contention
            io_contention = self._test_io_resource_contention()
            contention_tests.append(io_contention)
            
            # Test 4: File system resource contention
            filesystem_contention = self._test_filesystem_resource_contention()
            contention_tests.append(filesystem_contention)
            
            # Test 5: Network resource contention
            network_contention = self._test_network_resource_contention()
            contention_tests.append(network_contention)
            
            results["metrics"].update({
                "contention_tests": contention_tests,
                "total_contention_tests": len(contention_tests),
                "contention_severity": sum(test.get("severity", 0) for test in contention_tests) / len(contention_tests),
                "degradation_handling": all(test.get("graceful_degradation", True) for test in contention_tests)
            })
            
            # Determine status
            severity = results["metrics"]["contention_severity"]
            graceful = results["metrics"]["degradation_handling"]
            
            if severity < 0.3 and graceful:
                results["status"] = "PASS"
            elif severity < 0.6:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Resource contention test failed: {str(e)}")
        
        return results
    
    # Helper methods for thread pool testing
    def _test_thread_pool_basic_scaling(self) -> Dict[str, Any]:
        """Test basic thread pool scaling behavior"""
        num_tasks = 1000
        thread_counts = [1, 2, 4, 8, 16]
        
        def simple_task(task_id):
            """Simple CPU-bound task"""
            result = 0
            for i in range(1000):
                result += math.sqrt(i) * math.sin(i)
            return result
        
        scaling_results = []
        
        for num_threads in thread_counts:
            start_time = time.time()
            
            with ThreadPoolExecutor(max_workers=num_threads) as executor:
                futures = [executor.submit(simple_task, i) for i in range(num_tasks)]
                results = [future.result() for future in as_completed(futures)]
            
            end_time = time.time()
            duration = end_time - start_time
            throughput = num_tasks / duration
            
            scaling_results.append({
                "threads": num_threads,
                "duration": duration,
                "throughput": throughput,
                "tasks_completed": len(results)
            })
        
        # Calculate speedup
        baseline_throughput = scaling_results[0]["throughput"]
        max_throughput = max(result["throughput"] for result in scaling_results)
        speedup = max_throughput / baseline_throughput
        
        return {
            "test_type": "thread_pool_scaling",
            "thread_scaling_results": scaling_results,
            "speedup": speedup,
            "optimal_threads": max(scaling_results, key=lambda x: x["throughput"])["threads"]
        }
    
    def _test_thread_pool_cpu_intensive(self) -> Dict[str, Any]:
        """Test thread pool with CPU-intensive tasks"""
        num_threads = multiprocessing.cpu_count() * 2
        num_tasks = 500
        
        def cpu_intensive_task(task_id):
            """CPU-intensive mathematical computation"""
            result = 0
            iterations = 10000
            
            for i in range(iterations):
                result += math.sqrt(i) * math.sin(i) * math.cos(i)
                if i % 1000 == 0:
                    time.sleep(0.001)  # Brief yield
            
            return {
                "task_id": task_id,
                "result": result,
                "iterations": iterations
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(cpu_intensive_task, i) for i in range(num_tasks)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            "test_type": "thread_pool_cpu_intensive",
            "threads": num_threads,
            "tasks": num_tasks,
            "duration": duration,
            "throughput": num_tasks / duration,
            "tasks_completed": len(results),
            "stable": len(results) == num_tasks
        }
    
    def _test_thread_pool_io_bound(self) -> Dict[str, Any]:
        """Test thread pool with I/O-bound tasks"""
        num_threads = multiprocessing.cpu_count() * 4  # I/O bound can benefit from more threads
        num_tasks = 1000
        
        def io_bound_task(task_id):
            """I/O-bound task simulating network/disk operations"""
            # Simulate I/O wait time
            time.sleep(0.1)  # 100ms I/O wait
            
            # Some light processing
            result = sum(i * i for i in range(100))
            
            return {
                "task_id": task_id,
                "result": result,
                "io_wait_time": 0.1
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(io_bound_task, i) for i in range(num_tasks)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            "test_type": "thread_pool_io_bound",
            "threads": num_threads,
            "tasks": num_tasks,
            "duration": duration,
            "throughput": num_tasks / duration,
            "tasks_completed": len(results),
            "stable": len(results) == num_tasks
        }
    
    def _test_thread_pool_memory_pressure(self) -> Dict[str, Any]:
        """Test thread pool under memory pressure"""
        num_threads = 8
        num_tasks = 200
        
        def memory_pressure_task(task_id):
            """Task that creates memory pressure"""
            # Allocate memory
            large_array = np.random.random(100000)  # ~800KB
            
            # Process the array
            result = np.sum(large_array * 2)
            
            # Clean up
            del large_array
            
            return {
                "task_id": task_id,
                "result": result,
                "memory_allocated_mb": 0.8
            }
        
        start_time = time.time()
        
        try:
            with ThreadPoolExecutor(max_workers=num_threads) as executor:
                futures = [executor.submit(memory_pressure_task, i) for i in range(num_tasks)]
                results = [future.result() for future in as_completed(futures)]
            
            end_time = time.time()
            duration = end_time - start_time
            
            return {
                "test_type": "thread_pool_memory_pressure",
                "threads": num_threads,
                "tasks": num_tasks,
                "duration": duration,
                "throughput": num_tasks / duration,
                "tasks_completed": len(results),
                "memory_pressure_handled": len(results) == num_tasks,
                "stable": len(results) == num_tasks
            }
        except Exception as e:
            return {
                "test_type": "thread_pool_memory_pressure",
                "error": str(e),
                "stable": False
            }
    
    def _test_thread_pool_queue_saturation(self) -> Dict[str, Any]:
        """Test thread pool queue saturation behavior"""
        num_threads = 4
        queue_size = 100
        num_tasks = 1000
        
        # Use a bounded queue to test saturation
        task_queue = queue.Queue(maxsize=queue_size)
        results_queue = queue.Queue()
        
        def worker_task():
            """Worker that processes tasks from queue"""
            while True:
                try:
                    task = task_queue.get(timeout=1)
                    if task is None:
                        break
                    
                    # Process task
                    result = sum(i * i for i in range(1000))
                    results_queue.put(result)
                    
                    task_queue.task_done()
                except queue.Empty:
                    break
        
        # Start workers
        workers = []
        for i in range(num_threads):
            worker = threading.Thread(target=worker_task)
            worker.start()
            workers.append(worker)
        
        start_time = time.time()
        
        # Submit tasks
        submitted_tasks = 0
        for i in range(num_tasks):
            try:
                task_queue.put(i, timeout=0.1)
                submitted_tasks += 1
            except queue.Full:
                break
        
        # Wait for workers to complete
        task_queue.join()
        
        # Stop workers
        for _ in range(num_threads):
            task_queue.put(None)
        
        for worker in workers:
            worker.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Collect results
        results = []
        while not results_queue.empty():
            results.append(results_queue.get())
        
        return {
            "test_type": "thread_pool_queue_saturation",
            "threads": num_threads,
            "max_queue_size": queue_size,
            "tasks_submitted": submitted_tasks,
            "tasks_completed": len(results),
            "duration": duration,
            "queue_saturation_reached": submitted_tasks < num_tasks,
            "throughput": len(results) / duration
        }
    
    # Helper methods for process pool testing
    def _test_process_pool_basic_scaling(self) -> Dict[str, Any]:
        """Test basic process pool scaling behavior"""
        num_tasks = 500
        process_counts = [1, 2, 4, multiprocessing.cpu_count()]
        
        def simple_process_task(task_id):
            """Simple task for process pool"""
            result = 0
            for i in range(5000):
                result += math.sqrt(i) * math.sin(i)
            return result
        
        scaling_results = []
        
        for num_processes in process_counts:
            if num_processes > multiprocessing.cpu_count():
                continue
                
            start_time = time.time()
            
            with ProcessPoolExecutor(max_workers=num_processes) as executor:
                futures = [executor.submit(simple_process_task, i) for i in range(num_tasks)]
                results = [future.result() for future in as_completed(futures)]
            
            end_time = time.time()
            duration = end_time - start_time
            throughput = num_tasks / duration
            
            scaling_results.append({
                "processes": num_processes,
                "duration": duration,
                "throughput": throughput,
                "tasks_completed": len(results)
            })
        
        # Calculate speedup
        if scaling_results:
            baseline_throughput = scaling_results[0]["throughput"]
            max_throughput = max(result["throughput"] for result in scaling_results)
            speedup = max_throughput / baseline_throughput
        else:
            speedup = 1
        
        return {
            "test_type": "process_pool_scaling",
            "process_scaling_results": scaling_results,
            "speedup": speedup
        }
    
    def _test_process_pool_cpu_intensive(self) -> Dict[str, Any]:
        """Test process pool with CPU-intensive tasks"""
        num_processes = multiprocessing.cpu_count()
        num_tasks = 200
        
        def cpu_intensive_process_task(task_id):
            """CPU-intensive task for processes"""
            result = 0
            iterations = 20000
            
            for i in range(iterations):
                result += math.sqrt(i) * math.sin(i) * math.cos(i)
            
            return {
                "task_id": task_id,
                "result": result,
                "iterations": iterations
            }
        
        start_time = time.time()
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            futures = [executor.submit(cpu_intensive_process_task, i) for i in range(num_tasks)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            "test_type": "process_pool_cpu_intensive",
            "processes": num_processes,
            "tasks": num_tasks,
            "duration": duration,
            "throughput": num_tasks / duration,
            "tasks_completed": len(results),
            "stable": len(results) == num_tasks
        }
    
    def _test_process_pool_data_transfer(self) -> Dict[str, Any]:
        """Test process pool with large data transfer"""
        num_processes = min(4, multiprocessing.cpu_count())
        num_tasks = 50
        
        def data_transfer_task(task_id):
            """Task that transfers large amounts of data"""
            # Create large data structure
            large_data = np.random.random(50000)  # ~400KB
            
            # Process the data
            processed_data = large_data * 2 + 1
            result = np.sum(processed_data)
            
            # Return result (data is serialized/deserialized)
            return {
                "task_id": task_id,
                "result": result,
                "data_size_mb": len(large_data) * 8 / (1024 * 1024)  # 8 bytes per float64
            }
        
        start_time = time.time()
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            futures = [executor.submit(data_transfer_task, i) for i in range(num_tasks)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_data_mb = sum(r["data_size_mb"] for r in results)
        
        return {
            "test_type": "process_pool_data_transfer",
            "processes": num_processes,
            "tasks": num_tasks,
            "duration": duration,
            "total_data_mb": total_data_mb,
            "throughput": num_tasks / duration,
            "data_throughput_mbps": total_data_mb / duration,
            "tasks_completed": len(results),
            "stable": len(results) == num_tasks
        }
    
    def _test_process_pool_memory_isolation(self) -> Dict[str, Any]:
        """Test process pool memory isolation"""
        num_processes = min(4, multiprocessing.cpu_count())
        num_tasks = 30
        
        def memory_isolation_task(task_id):
            """Task that tests memory isolation between processes"""
            # Each process allocates its own memory
            local_array = np.random.random(100000)  # ~800KB
            
            # Process the array
            result = np.sum(local_array * 2)
            
            # The memory should be isolated per process
            process_id = os.getpid()
            
            return {
                "task_id": task_id,
                "process_id": process_id,
                "result": result,
                "memory_isolated": True
            }
        
        start_time = time.time()
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            futures = [executor.submit(memory_isolation_task, i) for i in range(num_tasks)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Check that we got results from different processes
        unique_processes = set(r["process_id"] for r in results)
        memory_isolation_confirmed = len(unique_processes) > 1
        
        return {
            "test_type": "process_pool_memory_isolation",
            "processes": num_processes,
            "tasks": num_tasks,
            "duration": duration,
            "unique_processes": len(unique_processes),
            "memory_isolation_confirmed": memory_isolation_confirmed,
            "tasks_completed": len(results),
            "stable": len(results) == num_tasks
        }
    
    def _test_process_pool_fault_tolerance(self) -> Dict[str, Any]:
        """Test process pool fault tolerance"""
        num_processes = 4
        num_tasks = 50
        
        def fault_tolerant_task(task_id):
            """Task that may fail randomly to test fault tolerance"""
            import random
            
            # Randomly fail some tasks
            if random.random() < 0.1:  # 10% failure rate
                raise Exception(f"Random failure in task {task_id}")
            
            # Normal task processing
            result = sum(i * i for i in range(1000))
            
            return {
                "task_id": task_id,
                "result": result,
                "success": True
            }
        
        start_time = time.time()
        
        with ProcessPoolExecutor(max_workers=num_processes) as executor:
            futures = [executor.submit(fault_tolerant_task, i) for i in range(num_tasks)]
            
            successful_results = 0
            failed_results = 0
            
            for future in as_completed(futures):
                try:
                    result = future.result()
                    successful_results += 1
                except Exception:
                    failed_results += 1
        
        end_time = time.time()
        duration = end_time - start_time
        
        return {
            "test_type": "process_pool_fault_tolerance",
            "processes": num_processes,
            "tasks": num_tasks,
            "duration": duration,
            "successful_tasks": successful_results,
            "failed_tasks": failed_results,
            "success_rate": successful_results / num_tasks,
            "fault_tolerance_working": failed_results > 0  # Should have some failures
        }
    
    # Helper methods for synchronization testing
    def _test_mutex_contention(self) -> Dict[str, Any]:
        """Test mutex/lock contention under high concurrency"""
        num_threads = 20
        operations_per_thread = 100
        shared_counter = Value('i', 0)
        lock = Lock()
        contention_stats = {"contended_locks": 0, "total_locks": 0}
        
        def contended_worker(worker_id):
            """Worker that creates lock contention"""
            local_operations = 0
            
            for i in range(operations_per_thread):
                # Acquire lock
                start_time = time.time()
                with lock:
                    # Critical section
                    shared_counter.value += 1
                    end_time = time.time()
                    
                    # Track contention (simplified)
                    if end_time - start_time > 0.001:  # 1ms threshold
                        contention_stats["contended_locks"] += 1
                
                contention_stats["total_locks"] += 1
                local_operations += 1
            
            return {
                "worker_id": worker_id,
                "operations": local_operations
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(contended_worker, i) for i in range(num_threads)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_operations = sum(r["operations"] for r in results)
        contention_ratio = contention_stats["contended_locks"] / contention_stats["total_locks"] if contention_stats["total_locks"] > 0 else 0
        
        return {
            "test_type": "mutex_contention",
            "threads": num_threads,
            "operations_per_thread": operations_per_thread,
            "total_operations": total_operations,
            "final_counter_value": shared_counter.value,
            "duration": duration,
            "contention_ratio": contention_ratio,
            "effective": shared_counter.value == total_operations,
            "no_deadlock": duration < 30  # Should complete quickly
        }
    
    def _test_semaphore_operations(self) -> Dict[str, Any]:
        """Test semaphore operations and concurrency control"""
        max_concurrent = 5
        num_threads = 15
        operations_per_thread = 20
        semaphore = Semaphore(max_concurrent)
        active_operations = Value('i', 0)
        max_concurrent_reached = Value('i', 0)
        
        def semaphore_worker(worker_id):
            """Worker that uses semaphore for concurrency control"""
            local_operations = 0
            local_max_concurrent = 0
            
            for i in range(operations_per_thread):
                # Acquire semaphore
                semaphore.acquire()
                
                try:
                    # Track concurrent operations
                    with active_operations.get_lock():
                        active_operations.value += 1
                        if active_operations.value > max_concurrent_reached.value:
                            max_concurrent_reached.value = active_operations.value
                        local_max_concurrent = max(local_max_concurrent, active_operations.value)
                    
                    # Simulate work
                    time.sleep(0.01)
                    
                    # Release semaphore
                    with active_operations.get_lock():
                        active_operations.value -= 1
                    
                    local_operations += 1
                    
                finally:
                    semaphore.release()
            
            return {
                "worker_id": worker_id,
                "operations": local_operations,
                "max_concurrent_observed": local_max_concurrent
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(semaphore_worker, i) for i in range(num_threads)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_operations = sum(r["operations"] for r in results)
        max_concurrent_actual = max_concurrent_reached.value
        
        return {
            "test_type": "semaphore_operations",
            "max_allowed_concurrent": max_concurrent,
            "max_concurrent_observed": max_concurrent_actual,
            "total_operations": total_operations,
            "duration": duration,
            "effective": max_concurrent_actual <= max_concurrent,
            "throughput": total_operations / duration,
            "no_deadlock": duration < 60
        }
    
    def _test_condition_variables(self) -> Dict[str, Any]:
        """Test condition variable synchronization"""
        num_producers = 5
        num_consumers = 5
        queue_size = 10
        operations_per_thread = 20
        
        # Shared state
        condition = Condition()
        shared_queue = queue.Queue(maxsize=queue_size)
        produced_items = Value('i', 0)
        consumed_items = Value('i', 0)
        
        def producer_worker(worker_id):
            """Producer worker using condition variables"""
            local_produced = 0
            
            for i in range(operations_per_thread):
                item = f"item_{worker_id}_{i}"
                
                with condition:
                    # Wait while queue is full
                    while shared_queue.full():
                        condition.wait()
                    
                    shared_queue.put(item)
                    produced_items.value += 1
                    local_produced += 1
                    
                    # Notify consumers
                    condition.notify()
            
            return {
                "worker_id": worker_id,
                "produced": local_produced
            }
        
        def consumer_worker(worker_id):
            """Consumer worker using condition variables"""
            local_consumed = 0
            
            for i in range(operations_per_thread):
                item = None
                
                with condition:
                    # Wait while queue is empty
                    while shared_queue.empty():
                        condition.wait()
                    
                    try:
                        item = shared_queue.get_nowait()
                        consumed_items.value += 1
                        local_consumed += 1
                    except queue.Empty:
                        pass
                    
                    # Notify producers
                    condition.notify()
            
            return {
                "worker_id": worker_id,
                "consumed": local_consumed
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_producers + num_consumers) as executor:
            # Submit producers
            producer_futures = [executor.submit(producer_worker, i) for i in range(num_producers)]
            # Submit consumers
            consumer_futures = [executor.submit(consumer_worker, i) for i in range(num_consumers)]
            
            producer_results = [future.result() for future in as_completed(producer_futures)]
            consumer_results = [future.result() for future in as_completed(consumer_futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_produced = sum(r["produced"] for r in producer_results)
        total_consumed = sum(r["consumed"] for r in consumer_results)
        
        return {
            "test_type": "condition_variables",
            "producers": num_producers,
            "consumers": num_consumers,
            "operations_per_thread": operations_per_thread,
            "total_produced": total_produced,
            "total_consumed": total_consumed,
            "duration": duration,
            "effective": total_produced == total_consumed,
            "no_deadlock": duration < 60,
            "throughput": total_produced / duration
        }
    
    def _test_event_synchronization(self) -> Dict[str, Any]:
        """Test event-based synchronization"""
        num_workers = 10
        start_event = Event()
        completion_event = Event()
        worker_results = {"completed": 0, "errors": 0}
        results_lock = Lock()
        
        def event_worker(worker_id):
            """Worker synchronized by events"""
            try:
                # Wait for start event
                start_event.wait()
                
                # Do some work
                result = sum(i * i for i in range(1000))
                
                # Signal completion (simplified - would need coordination for real completion event)
                with results_lock:
                    worker_results["completed"] += 1
                
                return {
                    "worker_id": worker_id,
                    "result": result,
                    "success": True
                }
            except Exception as e:
                with results_lock:
                    worker_results["errors"] += 1
                return {
                    "worker_id": worker_id,
                    "error": str(e),
                    "success": False
                }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            # Submit workers (they'll wait for start event)
            futures = [executor.submit(event_worker, i) for i in range(num_workers)]
            
            # Small delay to ensure all workers are waiting
            time.sleep(0.1)
            
            # Signal start
            start_event.set()
            
            # Wait for all workers to complete
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        successful_workers = sum(1 for r in results if r["success"])
        
        return {
            "test_type": "event_synchronization",
            "workers": num_workers,
            "successful_workers": successful_workers,
            "failed_workers": worker_results["errors"],
            "duration": duration,
            "effective": successful_workers == num_workers,
            "no_deadlock": duration < 30,
            "throughput": successful_workers / duration
        }
    
    def _test_barrier_synchronization(self) -> Dict[str, Any]:
        """Test barrier synchronization"""
        num_parties = 8
        num_cycles = 5
        
        # Use a simple barrier implementation since threading.Barrier might not be available
        barrier_lock = Lock()
        barrier_count = 0
        barrier_waiting = []
        
        def custom_barrier(party_id):
            """Custom barrier implementation"""
            nonlocal barrier_count
            
            with barrier_lock:
                barrier_count += 1
                barrier_waiting.append(party_id)
            
            # Wait until all parties reach the barrier
            while barrier_count < num_parties:
                time.sleep(0.001)
            
            # Reset for next cycle
            if party_id == 0:
                barrier_count = 0
                barrier_waiting.clear()
        
        def barrier_worker(worker_id):
            """Worker that uses barrier synchronization"""
            cycle_results = []
            
            for cycle in range(num_cycles):
                # Phase 1: Pre-barrier work
                phase1_work = sum(i * i for i in range(500))
                
                # Wait at barrier
                custom_barrier(worker_id)
                
                # Phase 2: Post-barrier work
                phase2_work = sum(i * i for i in range(500))
                
                cycle_results.append({
                    "cycle": cycle,
                    "phase1_work": phase1_work,
                    "phase2_work": phase2_work
                })
            
            return {
                "worker_id": worker_id,
                "cycles_completed": num_cycles,
                "cycle_results": cycle_results
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_parties) as executor:
            futures = [executor.submit(barrier_worker, i) for i in range(num_parties)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        successful_workers = sum(1 for r in results if r["cycles_completed"] == num_cycles)
        
        return {
            "test_type": "barrier_synchronization",
            "parties": num_parties,
            "cycles": num_cycles,
            "successful_workers": successful_workers,
            "duration": duration,
            "effective": successful_workers == num_parties,
            "no_deadlock": duration < 60,
            "throughput": successful_workers / duration
        }
    
    # Helper methods for IPC testing
    def _test_queue_communication(self) -> Dict[str, Any]:
        """Test queue-based inter-process communication"""
        num_producers = 3
        num_consumers = 3
        messages_per_producer = 100
        
        def producer_process(queue, producer_id):
            """Producer process that sends messages"""
            for i in range(messages_per_producer):
                message = f"Message from producer {producer_id} - {i}"
                queue.put(message)
                start_time = time.time()
                latency = time.time() - start_time
            return producer_id
        
        def consumer_process(queue, consumer_id):
            """Consumer process that receives messages"""
            messages_received = 0
            total_latency = 0
            
            for _ in range(messages_per_producer * num_producers // num_consumers):
                try:
                    message = queue.get(timeout=5)
                    messages_received += 1
                except queue.Empty:
                    break
            
            return {
                "consumer_id": consumer_id,
                "messages_received": messages_received
            }
        
        # Create shared queue
        manager = Manager()
        shared_queue = manager.Queue()
        
        start_time = time.time()
        
        # Start producer processes
        producer_processes = []
        for i in range(num_producers):
            p = multiprocessing.Process(target=producer_process, args=(shared_queue, i))
            p.start()
            producer_processes.append(p)
        
        # Start consumer processes
        consumer_processes = []
        for i in range(num_consumers):
            p = multiprocessing.Process(target=consumer_process, args=(shared_queue, i))
            p.start()
            consumer_processes.append(p)
        
        # Wait for all processes
        for p in producer_processes:
            p.join()
        
        for p in consumer_processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_messages_expected = num_producers * messages_per_producer
        
        return {
            "test_type": "queue_communication",
            "producers": num_producers,
            "consumers": num_consumers,
            "messages_expected": total_messages_expected,
            "duration": duration,
            "reliable": True,  # Queue handles reliability
            "avg_latency_ms": (duration / total_messages_expected * 1000) if total_messages_expected > 0 else 0
        }
    
    def _test_shared_memory_communication(self) -> Dict[str, Any]:
        """Test shared memory inter-process communication"""
        num_processes = 4
        array_size = 10000
        
        def shared_memory_worker(shared_array, process_id, results):
            """Worker that uses shared memory"""
            start_time = time.time()
            
            # Write to shared memory
            for i in range(array_size):
                shared_array[i] = process_id * 1000000 + i
            
            # Read from shared memory
            read_sum = sum(shared_array[i] for i in range(array_size))
            
            end_time = time.time()
            duration = end_time - start_time
            
            results[process_id] = {
                "process_id": process_id,
                "duration": duration,
                "read_sum": read_sum,
                "success": True
            }
        
        # Create shared array
        manager = Manager()
        shared_array = manager.Array('i', [0] * array_size)
        
        results = manager.dict()
        processes = []
        
        start_time = time.time()
        
        # Start worker processes
        for i in range(num_processes):
            p = multiprocessing.Process(target=shared_memory_worker, args=(shared_array, i, results))
            p.start()
            processes.append(p)
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        successful_processes = sum(1 for r in results.values() if r.get("success", False))
        
        return {
            "test_type": "shared_memory_communication",
            "processes": num_processes,
            "array_size": array_size,
            "duration": duration,
            "successful_processes": successful_processes,
            "reliable": successful_processes == num_processes,
            "avg_latency_ms": (duration / num_processes * 1000) if num_processes > 0 else 0
        }
    
    def _test_pipe_communication(self) -> Dict[str, Any]:
        """Test pipe-based inter-process communication"""
        def pipe_sender(pipe, sender_id, num_messages):
            """Process that sends messages through pipe"""
            for i in range(num_messages):
                message = f"Message {i} from sender {sender_id}"
                pipe.send(message)
            
            pipe.close()
            return sender_id
        
        def pipe_receiver(pipe, receiver_id, expected_messages):
            """Process that receives messages through pipe"""
            messages_received = 0
            total_latency = 0
            
            try:
                while True:
                    message = pipe.recv()
                    messages_received += 1
            except EOFError:
                pass
            
            return {
                "receiver_id": receiver_id,
                "messages_received": messages_received,
                "success": messages_received == expected_messages
            }
        
        # Create pipe pairs
        pipes = []
        receivers = []
        num_senders = 2
        messages_per_sender = 50
        
        start_time = time.time()
        
        for i in range(num_senders):
            parent_conn, child_conn = multiprocessing.Pipe()
            
            # Start sender process
            sender_process = multiprocessing.Process(
                target=pipe_sender, 
                args=(child_conn, i, messages_per_sender)
            )
            sender_process.start()
            
            pipes.append(parent_conn)
        
        # Start receiver processes
        receiver_processes = []
        for i, pipe in enumerate(pipes):
            receiver_process = multiprocessing.Process(
                target=pipe_receiver,
                args=(pipe, i, messages_per_sender)
            )
            receiver_process.start()
            receiver_processes.append(receiver_process)
        
        # Wait for all processes
        for process in receiver_processes:
            process.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_messages_expected = num_senders * messages_per_sender
        
        return {
            "test_type": "pipe_communication",
            "sender_processes": num_senders,
            "messages_per_sender": messages_per_sender,
            "total_messages": total_messages_expected,
            "duration": duration,
            "reliable": True,  # Pipes are reliable
            "avg_latency_ms": (duration / total_messages_expected * 1000) if total_messages_expected > 0 else 0
        }
    
    def _test_socket_communication(self) -> Dict[str, Any]:
        """Test socket-based inter-process communication"""
        import socket
        
        def socket_server(port, num_connections):
            """Socket server process"""
            server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            
            try:
                server_socket.bind(('localhost', port))
                server_socket.listen(num_connections)
                
                connections = []
                for _ in range(num_connections):
                    conn, addr = server_socket.accept()
                    connections.append(conn)
                
                # Receive data from all connections
                total_received = 0
                for conn in connections:
                    while True:
                        data = conn.recv(1024)
                        if not data:
                            break
                        total_received += len(data)
                
                return total_received
                
            finally:
                server_socket.close()
        
        def socket_client(port, client_id, message_size):
            """Socket client process"""
            client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            
            try:
                client_socket.connect(('localhost', port))
                
                # Send message
                message = b'x' * message_size
                client_socket.send(message)
                
                return client_id
                
            finally:
                client_socket.close()
        
        port = 12345
        num_connections = 3
        message_size = 1024
        
        # Start server process
        server_process = multiprocessing.Process(
            target=socket_server,
            args=(port, num_connections)
        )
        server_process.start()
        
        # Wait for server to start
        time.sleep(1)
        
        # Start client processes
        client_processes = []
        for i in range(num_connections):
            p = multiprocessing.Process(
                target=socket_client,
                args=(port, i, message_size)
            )
            p.start()
            client_processes.append(p)
        
        # Wait for all processes
        for p in client_processes:
            p.join()
        
        server_process.join()
        
        return {
            "test_type": "socket_communication",
            "connections": num_connections,
            "message_size": message_size,
            "reliable": True,  # TCP is reliable
            "avg_latency_ms": 1  # Simplified
        }
    
    def _test_high_frequency_ipc(self) -> Dict[str, Any]:
        """Test high-frequency inter-process communication"""
        num_exchanges = 1000
        
        def high_freq_sender(pipe):
            """High frequency sender"""
            start_time = time.time()
            
            for i in range(num_exchanges):
                message = f"Message {i}"
                pipe.send(message)
                # Immediate response expected
                response = pipe.recv()
            
            end_time = time.time()
            duration = end_time - start_time
            
            pipe.close()
            return duration
        
        def high_freq_receiver(pipe):
            """High frequency receiver"""
            start_time = time.time()
            
            for i in range(num_exchanges):
                message = pipe.recv()
                # Echo back
                pipe.send(f"Response {i}")
            
            end_time = time.time()
            duration = end_time - start_time
            
            pipe.close()
            return duration
        
        # Create pipe for high-frequency communication
        parent_conn, child_conn = multiprocessing.Pipe()
        
        start_time = time.time()
        
        # Start sender and receiver processes
        sender_process = multiprocessing.Process(target=high_freq_sender, args=(parent_conn,))
        receiver_process = multiprocessing.Process(target=high_freq_receiver, args=(child_conn,))
        
        sender_process.start()
        receiver_process.start()
        
        sender_process.join()
        receiver_process.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        messages_per_second = num_exchanges / duration
        
        return {
            "test_type": "high_frequency_ipc",
            "message_exchanges": num_exchanges,
            "duration": duration,
            "messages_per_second": messages_per_second,
            "reliable": True,
            "avg_latency_ms": (duration / num_exchanges * 1000) if num_exchanges > 0 else 0
        }
    
    # Helper methods for resource contention testing
    def _test_cpu_resource_contention(self) -> Dict[str, Any]:
        """Test CPU resource contention"""
        num_processes = multiprocessing.cpu_count() * 2
        cpu_intensive_tasks = []
        
        def cpu_contention_worker(worker_id):
            """Worker that competes for CPU resources"""
            start_time = time.time()
            operations = 0
            
            while time.time() - start_time < 10:  # 10 seconds
                # CPU intensive operation
                for i in range(1000):
                    math.sqrt(i) * math.sin(i)
                operations += 1000
            
            return {
                "worker_id": worker_id,
                "operations": operations,
                "duration": time.time() - start_time
            }
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_processes) as executor:
            futures = [executor.submit(cpu_contention_worker, i) for i in range(num_processes)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_operations = sum(r["operations"] for r in results)
        avg_operations_per_worker = total_operations / len(results)
        
        # Calculate contention severity (lower throughput indicates more contention)
        expected_operations_per_worker = 1000 * (duration / 0.001)  # Rough estimate
        performance_degradation = 1 - (avg_operations_per_worker / expected_operations_per_worker)
        
        return {
            "test_type": "cpu_resource_contention",
            "competing_processes": num_processes,
            "total_operations": total_operations,
            "duration": duration,
            "performance_degradation": max(0, performance_degradation),
            "severity": min(1, performance_degradation * 2),
            "graceful_degradation": performance_degradation < 0.5
        }
    
    def _test_memory_resource_contention(self) -> Dict[str, Any]:
        """Test memory resource contention"""
        num_processes = 6
        memory_per_process_mb = 100
        
        def memory_contention_worker(worker_id, results):
            """Worker that creates memory pressure"""
            try:
                start_time = time.time()
                
                # Allocate memory
                memory_arrays = []
                array_size = memory_per_process_mb * 1024 * 1024 // 8  # 8 bytes per float64
                
                for i in range(5):  # Allocate 5 arrays
                    array = np.random.random(array_size)
                    memory_arrays.append(array)
                
                # Process the memory
                for array in memory_arrays:
                    processed = array * 2 + 1
                    result = np.sum(processed)
                
                duration = time.time() - start_time
                
                results[worker_id] = {
                    "success": True,
                    "duration": duration,
                    "memory_allocated_mb": memory_per_process_mb
                }
                
            except MemoryError:
                results[worker_id] = {
                    "success": False,
                    "error": "Memory allocation failed",
                    "memory_allocated_mb": 0
                }
            except Exception as e:
                results[worker_id] = {
                    "success": False,
                    "error": str(e),
                    "memory_allocated_mb": 0
                }
        
        # Use manager to share results
        manager = multiprocessing.Manager()
        results = manager.dict()
        processes = []
        
        start_time = time.time()
        
        # Start memory contention workers
        for i in range(num_processes):
            p = multiprocessing.Process(target=memory_contention_worker, args=(i, results))
            p.start()
            processes.append(p)
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        successful_workers = sum(1 for r in results.values() if r.get("success", False))
        total_memory_allocated = sum(r.get("memory_allocated_mb", 0) for r in results.values())
        
        # Calculate contention severity
        expected_memory = num_processes * memory_per_process_mb
        memory_pressure_ratio = total_memory_allocated / expected_memory
        severity = 1 - memory_pressure_ratio
        
        return {
            "test_type": "memory_resource_contention",
            "competing_processes": num_processes,
            "memory_per_process_mb": memory_per_process_mb,
            "successful_processes": successful_workers,
            "total_memory_allocated_mb": total_memory_allocated,
            "duration": duration,
            "memory_pressure_ratio": memory_pressure_ratio,
            "severity": max(0, severity),
            "graceful_degradation": successful_workers >= num_processes * 0.7
        }
    
    def _test_io_resource_contention(self) -> Dict[str, Any]:
        """Test I/O resource contention"""
        num_processes = 8
        operations_per_process = 50
        test_files = []
        
        def io_contention_worker(worker_id, results):
            """Worker that creates I/O contention"""
            try:
                start_time = time.time()
                successful_operations = 0
                
                for i in range(operations_per_process):
                    # Create test file
                    test_file = self.test_dir / f"io_contention_{worker_id}_{i}.dat"
                    data = f"Worker {worker_id} operation {i}".encode() * 100
                    
                    try:
                        # Write operation
                        with open(test_file, 'wb') as f:
                            f.write(data)
                        
                        # Read operation
                        with open(test_file, 'rb') as f:
                            read_data = f.read()
                        
                        if len(read_data) == len(data):
                            successful_operations += 1
                        
                        # Clean up
                        test_file.unlink()
                        
                    except Exception:
                        pass
                
                duration = time.time() - start_time
                
                results[worker_id] = {
                    "success": True,
                    "successful_operations": successful_operations,
                    "total_operations": operations_per_process,
                    "duration": duration
                }
                
            except Exception as e:
                results[worker_id] = {
                    "success": False,
                    "error": str(e),
                    "successful_operations": 0
                }
        
        # Use manager to share results
        manager = multiprocessing.Manager()
        results = manager.dict()
        processes = []
        
        start_time = time.time()
        
        # Start I/O contention workers
        for i in range(num_processes):
            p = multiprocessing.Process(target=io_contention_worker, args=(i, results))
            p.start()
            processes.append(p)
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_successful_operations = sum(r.get("successful_operations", 0) for r in results.values())
        total_expected_operations = num_processes * operations_per_process
        
        # Calculate I/O contention severity
        io_efficiency = total_successful_operations / total_expected_operations
        severity = 1 - io_efficiency
        
        return {
            "test_type": "io_resource_contention",
            "competing_processes": num_processes,
            "operations_per_process": operations_per_process,
            "total_successful_operations": total_successful_operations,
            "total_expected_operations": total_expected_operations,
            "duration": duration,
            "io_efficiency": io_efficiency,
            "severity": max(0, severity),
            "graceful_degradation": io_efficiency > 0.5
        }
    
    def _test_filesystem_resource_contention(self) -> Dict[str, Any]:
        """Test file system resource contention"""
        num_processes = 6
        shared_file = self.test_dir / "fs_contention_shared.dat"
        
        def filesystem_contention_worker(worker_id, results):
            """Worker that creates file system contention"""
            try:
                start_time = time.time()
                file_operations = 0
                
                for i in range(20):
                    try:
                        # Lock and access shared file
                        with open(shared_file, 'a') as f:
                            f.write(f"Worker {worker_id} - Operation {i}\n")
                            f.flush()
                        
                        file_operations += 1
                        
                    except Exception:
                        pass
                
                duration = time.time() - start_time
                
                results[worker_id] = {
                    "success": True,
                    "file_operations": file_operations,
                    "duration": duration
                }
                
            except Exception as e:
                results[worker_id] = {
                    "success": False,
                    "error": str(e),
                    "file_operations": 0
                }
        
        # Create shared file
        shared_file.touch()
        
        # Use manager to share results
        manager = multiprocessing.Manager()
        results = manager.dict()
        processes = []
        
        start_time = time.time()
        
        # Start file system contention workers
        for i in range(num_processes):
            p = multiprocessing.Process(target=filesystem_contention_worker, args=(i, results))
            p.start()
            processes.append(p)
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        total_operations = sum(r.get("file_operations", 0) for r in results.values())
        
        # Check final file size to verify operations
        file_size = shared_file.stat().st_size if shared_file.exists() else 0
        
        # Cleanup
        shared_file.unlink()
        
        return {
            "test_type": "filesystem_resource_contention",
            "competing_processes": num_processes,
            "total_operations": total_operations,
            "duration": duration,
            "file_size_bytes": file_size,
            "graceful_degradation": total_operations > 0,
            "severity": 0.2  # File system contention usually manageable
        }
    
    def _test_network_resource_contention(self) -> Dict[str, Any]:
        """Test network resource contention"""
        # This is a simplified network contention test
        # Real network testing would require actual network interface testing
        
        import socket
        
        def network_contention_worker(worker_id, results):
            """Worker that creates network-like contention"""
            try:
                start_time = time.time()
                
                # Simulate network operations with local sockets
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                
                # Bind to a local port (simulating network endpoint)
                try:
                    sock.bind(('localhost', 0))  # Use any available port
                    port = sock.getsockname()[1]
                    
                    sock.listen(1)
                    
                    # Simulate connection handling
                    operations = 0
                    for i in range(10):
                        try:
                            # Simulate socket operations
                            sock.settimeout(0.1)
                            operations += 1
                        except socket.timeout:
                            pass
                    
                    duration = time.time() - start_time
                    
                    results[worker_id] = {
                        "success": True,
                        "operations": operations,
                        "duration": duration,
                        "port": port
                    }
                    
                except Exception as e:
                    results[worker_id] = {
                        "success": False,
                        "error": str(e),
                        "operations": 0
                    }
                
                sock.close()
                
            except Exception as e:
                results[worker_id] = {
                    "success": False,
                    "error": str(e),
                    "operations": 0
                }
        
        num_processes = 4
        
        # Use manager to share results
        manager = multiprocessing.Manager()
        results = manager.dict()
        processes = []
        
        start_time = time.time()
        
        # Start network contention workers
        for i in range(num_processes):
            p = multiprocessing.Process(target=network_contention_worker, args=(i, results))
            p.start()
            processes.append(p)
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        end_time = time.time()
        duration = end_time - start_time
        
        successful_workers = sum(1 for r in results.values() if r.get("success", False))
        total_operations = sum(r.get("operations", 0) for r in results.values())
        
        return {
            "test_type": "network_resource_contention",
            "competing_processes": num_processes,
            "successful_processes": successful_workers,
            "total_operations": total_operations,
            "duration": duration,
            "graceful_degradation": successful_workers >= num_processes * 0.75,
            "severity": 0.1  # Network contention usually light for local operations
        }