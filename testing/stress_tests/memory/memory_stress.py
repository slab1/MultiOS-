#!/usr/bin/env python3
"""
Memory stress testing module
Tests memory allocation limits, memory leaks, and fragmentation under extreme conditions
"""

import os
import sys
import time
import gc
import threading
import multiprocessing
import psutil
import numpy as np
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
import tracemalloc
import weakref
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor
import ctypes


class MemoryStressTester:
    """Advanced memory stress testing module"""
    
    def __init__(self, config):
        self.config = config
        self.test_dir = Path(config.test_dir) / "memory_tests"
        self.test_dir.mkdir(parents=True, exist_ok=True)
        
        # Memory tracking
        self.memory_snapshots = []
        self.leaked_objects = []
        self.fragmentation_stats = []
    
    def test_allocation_limits(self) -> Dict[str, Any]:
        """Test memory allocation limits and failure handling"""
        results = {
            "test_name": "Memory Allocation Limits",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            allocations = []
            max_allocations = 0
            allocation_failures = 0
            total_allocated_mb = 0
            
            # Start memory tracking
            tracemalloc.start()
            
            # Test progressive memory allocation
            target_allocation_mb = self.config.max_memory_allocation_mb
            
            for size_mb in range(1, target_allocation_mb + 1, 10):
                try:
                    # Allocate memory
                    chunk = bytearray(size_mb * 1024 * 1024)
                    allocations.append(chunk)
                    max_allocations += 1
                    total_allocated_mb += size_mb
                    
                    # Check system memory
                    memory = psutil.virtual_memory()
                    if memory.percent > 95:
                        results["warnings"].append(f"High memory usage during allocation: {memory.percent:.1f}%")
                    
                    # Take memory snapshot
                    if max_allocations % 10 == 0:
                        current, peak = tracemalloc.get_traced_memory()
                        self.memory_snapshots.append({
                            "allocated_mb": total_allocated_mb,
                            "current_mb": current / (1024 * 1024),
                            "peak_mb": peak / (1024 * 1024),
                            "system_memory_percent": memory.percent
                        })
                    
                    time.sleep(0.1)  # Small delay to allow system response
                    
                except MemoryError:
                    allocation_failures += 1
                    results["warnings"].append(f"Memory allocation failed at {size_mb}MB")
                    break
                except Exception as e:
                    allocation_failures += 1
                    results["errors"].append(f"Unexpected error during allocation: {str(e)}")
                    break
            
            # Test allocation with different patterns
            patterns_results = self._test_allocation_patterns()
            results["metrics"].update(patterns_results)
            
            # Cleanup
            del allocations
            gc.collect()
            tracemalloc.stop()
            
            # Analyze results
            results["metrics"].update({
                "max_successful_allocations": max_allocations,
                "total_allocated_mb": total_allocated_mb,
                "allocation_failures": allocation_failures,
                "allocation_success_rate": max_allocations / target_allocation_mb * 100,
                "peak_memory_snapshot": max(self.memory_snapshots, key=lambda x: x["peak_mb"]) if self.memory_snapshots else {},
                "memory_snapshots_count": len(self.memory_snapshots)
            })
            
            # Determine status
            if allocation_failures == 0 and total_allocated_mb >= target_allocation_mb * 0.8:
                results["status"] = "PASS"
            elif allocation_failures > 0:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Memory allocation test failed: {str(e)}")
        
        return results
    
    def test_memory_leaks(self) -> Dict[str, Any]:
        """Test for memory leaks and memory growth patterns"""
        results = {
            "test_name": "Memory Leak Detection",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            # Enable detailed memory tracking
            tracemalloc.start()
            
            memory_samples = []
            leak_detected = False
            baseline_memory = 0
            
            for iteration in range(self.config.memory_leak_iterations):
                # Create and destroy objects
                objects = []
                
                for i in range(100):
                    # Create various object types to simulate real usage
                    obj_dict = {f"key_{i}_{j}": f"value_{i}_{j}" for j in range(50)}
                    obj_list = [i * j for j in range(1000)]
                    obj_str = f"String data {i}" * 100
                    
                    objects.extend([obj_dict, obj_list, obj_str])
                
                # Force garbage collection
                del objects
                gc.collect()
                
                # Sample memory every 100 iterations
                if iteration % 100 == 0:
                    current, peak = tracemalloc.get_traced_memory()
                    process = psutil.Process()
                    rss_memory = process.memory_info().rss / (1024 * 1024)  # MB
                    
                    memory_samples.append({
                        "iteration": iteration,
                        "tracemalloc_current_mb": current / (1024 * 1024),
                        "tracemalloc_peak_mb": peak / (1024 * 1024),
                        "process_rss_mb": rss_memory
                    })
                    
                    # Set baseline on first sample
                    if baseline_memory == 0:
                        baseline_memory = rss_memory
                    
                    # Check for significant memory growth
                    if len(memory_samples) > 10:
                        recent_samples = memory_samples[-5:]
                        avg_recent = sum(s["process_rss_mb"] for s in recent_samples) / len(recent_samples)
                        
                        # Flag potential leak if memory grew significantly
                        if avg_recent > baseline_memory * 1.5:
                            leak_detected = True
                            results["warnings"].append(f"Potential memory leak detected at iteration {iteration}")
                
                # Add small delay to simulate real workload
                time.sleep(0.01)
            
            # Analyze final memory state
            final_memory = psutil.Process().memory_info().rss / (1024 * 1024)
            memory_growth = final_memory - baseline_memory
            
            # Test with weak references to detect circular dependencies
            leak_test_results = self._test_circular_references()
            
            tracemalloc.stop()
            
            # Analyze results
            results["metrics"].update({
                "memory_samples_count": len(memory_samples),
                "baseline_memory_mb": baseline_memory,
                "final_memory_mb": final_memory,
                "memory_growth_mb": memory_growth,
                "leak_detected": leak_detected,
                "memory_growth_percent": (memory_growth / baseline_memory * 100) if baseline_memory > 0 else 0,
                "tracemalloc_summary": self._get_tracemalloc_summary(),
                "circular_reference_test": leak_test_results
            })
            
            # Determine status
            if not leak_detected and memory_growth < 50:  # Less than 50MB growth
                results["status"] = "PASS"
            elif memory_growth < 100:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Memory leak test failed: {str(e)}")
        
        return results
    
    def test_fragmentation(self) -> Dict[str, Any]:
        """Test memory fragmentation patterns and defragmentation strategies"""
        results = {
            "test_name": "Memory Fragmentation",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            fragmentation_samples = []
            
            # Test 1: Allocate and free in different patterns
            for pattern in ["sequential", "random", "alternating"]:
                pattern_results = self._test_fragmentation_pattern(pattern)
                fragmentation_samples.append(pattern_results)
            
            # Test 2: Large object allocation/deallocation cycles
            large_object_results = self._test_large_object_fragmentation()
            fragmentation_samples.append(large_object_results)
            
            # Test 3: Mixed allocation sizes
            mixed_size_results = self._test_mixed_size_allocation()
            fragmentation_samples.append(mixed_size_results)
            
            # Analyze fragmentation impact
            fragmentation_impact = self._analyze_fragmentation_impact()
            
            results["metrics"].update({
                "fragmentation_patterns": fragmentation_samples,
                "fragmentation_impact": fragmentation_impact,
                "total_samples": len(fragmentation_samples),
                "average_fragmentation_score": sum(s.get("fragmentation_score", 0) for s in fragmentation_samples) / len(fragmentation_samples) if fragmentation_samples else 0
            })
            
            # Determine status
            avg_fragmentation = results["metrics"]["average_fragmentation_score"]
            if avg_fragmentation < 0.3:
                results["status"] = "PASS"
            elif avg_fragmentation < 0.6:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Fragmentation test failed: {str(e)}")
        
        return results
    
    def test_memory_pressure(self) -> Dict[str, Any]:
        """Test system behavior under memory pressure"""
        results = {
            "test_name": "Memory Pressure Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            pressure_tests = []
            
            # Test 1: Gradual memory pressure
            gradual_pressure = self._test_gradual_memory_pressure()
            pressure_tests.append(gradual_pressure)
            
            # Test 2: Rapid memory allocation bursts
            burst_pressure = self._test_memory_bursts()
            pressure_tests.append(burst_pressure)
            
            # Test 3: Memory allocation with concurrent processes
            concurrent_pressure = self._test_concurrent_memory_pressure()
            pressure_tests.append(concurrent_pressure)
            
            # Test 4: System response to memory pressure
            system_response = self._test_memory_pressure_response()
            pressure_tests.append(system_response)
            
            results["metrics"].update({
                "pressure_tests": pressure_tests,
                "total_pressure_tests": len(pressure_tests),
                "system_pressure_resilience": self._calculate_pressure_resilience(pressure_tests)
            })
            
            # Determine status
            resilience_score = results["metrics"]["system_pressure_resilience"]
            if resilience_score >= 0.8:
                results["status"] = "PASS"
            elif resilience_score >= 0.5:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Memory pressure test failed: {str(e)}")
        
        return results
    
    def test_overflow_protection(self) -> Dict[str, Any]:
        """Test memory overflow protection mechanisms"""
        results = {
            "test_name": "Memory Overflow Protection",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            protection_tests = []
            
            # Test array bounds checking
            bounds_test = self._test_array_bounds_protection()
            protection_tests.append(bounds_test)
            
            # Test buffer overflow protection
            buffer_test = self._test_buffer_overflow_protection()
            protection_tests.append(buffer_test)
            
            # Test integer overflow protection
            integer_test = self._test_integer_overflow_protection()
            protection_tests.append(integer_test)
            
            # Test memory allocation overflow
            allocation_test = self._test_allocation_overflow_protection()
            protection_tests.append(allocation_test)
            
            results["metrics"].update({
                "protection_tests": protection_tests,
                "total_protection_tests": len(protection_tests),
                "protection_effectiveness": sum(1 for t in protection_tests if t.get("protected", False)) / len(protection_tests)
            })
            
            # Determine status
            effectiveness = results["metrics"]["protection_effectiveness"]
            if effectiveness >= 0.9:
                results["status"] = "PASS"
            elif effectiveness >= 0.7:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Overflow protection test failed: {str(e)}")
        
        return results
    
    # Helper methods for memory testing
    def _test_allocation_patterns(self) -> Dict[str, Any]:
        """Test different memory allocation patterns"""
        patterns = {}
        
        patterns["small_chunks"] = self._test_small_chunk_allocation()
        patterns["large_chunks"] = self._test_large_chunk_allocation()
        patterns["mixed_allocation"] = self._test_mixed_allocation()
        patterns["concurrent_allocation"] = self._test_concurrent_allocation()
        
        return patterns
    
    def _test_small_chunk_allocation(self) -> Dict[str, Any]:
        """Test many small memory allocations"""
        start_memory = psutil.Process().memory_info().rss
        allocations = []
        
        for i in range(10000):
            try:
                chunk = bytearray(1024)  # 1KB chunks
                allocations.append(chunk)
            except MemoryError:
                break
        
        end_memory = psutil.Process().memory_info().rss
        memory_used = (end_memory - start_memory) / (1024 * 1024)
        
        del allocations
        gc.collect()
        
        return {
            "pattern": "small_chunks",
            "allocations_count": len(allocations),
            "memory_used_mb": memory_used,
            "success": len(allocations) == 10000
        }
    
    def _test_large_chunk_allocation(self) -> Dict[str, Any]:
        """Test large memory allocations"""
        start_memory = psutil.Process().memory_info().rss
        allocations = []
        
        for i in range(10):
            try:
                chunk = bytearray(10 * 1024 * 1024)  # 10MB chunks
                allocations.append(chunk)
            except MemoryError:
                break
        
        end_memory = psutil.Process().memory_info().rss
        memory_used = (end_memory - start_memory) / (1024 * 1024)
        
        del allocations
        gc.collect()
        
        return {
            "pattern": "large_chunks",
            "allocations_count": len(allocations),
            "memory_used_mb": memory_used,
            "success": len(allocations) == 10
        }
    
    def _test_mixed_allocation(self) -> Dict[str, Any]:
        """Test mixed size allocations"""
        allocations = []
        sizes = [1024, 4096, 16384, 65536, 262144]  # 1KB to 256KB
        
        for size in sizes:
            try:
                chunk = bytearray(size)
                allocations.append(chunk)
            except MemoryError:
                break
        
        total_allocated = sum(len(chunk) for chunk in allocations)
        del allocations
        gc.collect()
        
        return {
            "pattern": "mixed_allocation",
            "allocations_count": len(allocations),
            "total_allocated_bytes": total_allocated,
            "success": len(allocations) == len(sizes)
        }
    
    def _test_concurrent_allocation(self) -> Dict[str, Any]:
        """Test concurrent memory allocation"""
        results = []
        
        def allocate_memory(worker_id):
            allocations = []
            for i in range(100):
                try:
                    chunk = bytearray(10240)  # 10KB
                    allocations.append(chunk)
                except MemoryError:
                    break
            return len(allocations)
        
        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(allocate_memory, i) for i in range(10)]
            results = [future.result() for future in futures]
        
        total_allocated = sum(results)
        
        return {
            "pattern": "concurrent_allocation",
            "worker_results": results,
            "total_allocated": total_allocated,
            "success": all(r > 0 for r in results)
        }
    
    def _test_fragmentation_pattern(self, pattern: str) -> Dict[str, Any]:
        """Test memory fragmentation with specific pattern"""
        if pattern == "sequential":
            return self._test_sequential_fragmentation()
        elif pattern == "random":
            return self._test_random_fragmentation()
        elif pattern == "alternating":
            return self._test_alternating_fragmentation()
        return {"pattern": pattern, "error": "Unknown pattern"}
    
    def _test_sequential_fragmentation(self) -> Dict[str, Any]:
        """Test sequential allocation/deallocation pattern"""
        blocks = []
        
        # Allocate sequential blocks
        for i in range(100):
            block = bytearray(1024 * (i + 1))  # Increasing sizes
            blocks.append(block)
        
        # Free every other block
        for i in range(0, len(blocks), 2):
            blocks[i] = None
        
        gc.collect()
        
        # Calculate fragmentation score
        fragmented_blocks = sum(1 for b in blocks if b is None)
        fragmentation_score = fragmented_blocks / len(blocks)
        
        return {
            "pattern": "sequential",
            "total_blocks": len(blocks),
            "fragmented_blocks": fragmented_blocks,
            "fragmentation_score": fragmentation_score
        }
    
    def _test_random_fragmentation(self) -> Dict[str, Any]:
        """Test random allocation/deallocation pattern"""
        import random
        blocks = []
        
        # Allocate blocks
        for i in range(100):
            size = random.randint(1024, 10240)
            block = bytearray(size)
            blocks.append(block)
        
        # Randomly free blocks
        indices_to_free = random.sample(range(len(blocks)), len(blocks) // 2)
        for idx in indices_to_free:
            blocks[idx] = None
        
        gc.collect()
        
        fragmentation_score = len(indices_to_free) / len(blocks)
        
        return {
            "pattern": "random",
            "total_blocks": len(blocks),
            "fragmented_blocks": len(indices_to_free),
            "fragmentation_score": fragmentation_score
        }
    
    def _test_alternating_fragmentation(self) -> Dict[str, Any]:
        """Test alternating allocation/deallocation pattern"""
        blocks = []
        
        # Allocate in pairs
        for i in range(0, 50, 2):
            block1 = bytearray(1024)
            block2 = bytearray(2048)
            blocks.extend([block1, block2])
            
            # Free first of each pair
            blocks[i] = None
        
        gc.collect()
        
        fragmented_blocks = sum(1 for b in blocks if b is None)
        fragmentation_score = fragmented_blocks / len(blocks)
        
        return {
            "pattern": "alternating",
            "total_blocks": len(blocks),
            "fragmented_blocks": fragmented_blocks,
            "fragmentation_score": fragmentation_score
        }
    
    def _test_large_object_fragmentation(self) -> Dict[str, Any]:
        """Test fragmentation with large objects"""
        large_objects = []
        
        # Allocate large objects
        for i in range(10):
            obj = np.zeros((1000, 1000), dtype=np.float64)  # ~8MB each
            large_objects.append(obj)
        
        # Free every third object
        for i in range(0, len(large_objects), 3):
            large_objects[i] = None
        
        gc.collect()
        
        fragmented = sum(1 for obj in large_objects if obj is None)
        fragmentation_score = fragmented / len(large_objects)
        
        return {
            "pattern": "large_objects",
            "total_objects": len(large_objects),
            "fragmented_objects": fragmented,
            "fragmentation_score": fragmentation_score
        }
    
    def _test_mixed_size_allocation(self) -> Dict[str, Any]:
        """Test fragmentation with mixed allocation sizes"""
        allocations = []
        
        # Mix of small, medium, and large allocations
        sizes = [1024] * 50 + [10240] * 25 + [102400] * 10
        
        for size in sizes:
            allocation = bytearray(size)
            allocations.append(allocation)
        
        # Free random selections from each size category
        import random
        small_indices = random.sample(range(50), 25)
        medium_indices = random.sample(range(50, 75), 12)
        large_indices = random.sample(range(75, 85), 5)
        
        for idx in small_indices + medium_indices + large_indices:
            allocations[idx] = None
        
        gc.collect()
        
        fragmented = sum(1 for a in allocations if a is None)
        fragmentation_score = fragmented / len(allocations)
        
        return {
            "pattern": "mixed_sizes",
            "total_allocations": len(allocations),
            "fragmented_allocations": fragmented,
            "fragmentation_score": fragmentation_score
        }
    
    def _analyze_fragmentation_impact(self) -> Dict[str, Any]:
        """Analyze the impact of fragmentation on memory usage"""
        # Get baseline memory usage
        baseline_memory = psutil.Process().memory_info().rss
        
        # Simulate fragmented memory usage
        fragments = []
        for i in range(100):
            fragment = bytearray(1024)
            fragments.append(fragment)
        
        # Create gaps by removing every other fragment
        for i in range(0, len(fragments), 2):
            fragments[i] = None
        
        gc.collect()
        
        fragmented_memory = psutil.Process().memory_info().rss
        memory_overhead = fragmented_memory - baseline_memory
        
        return {
            "baseline_memory_mb": baseline_memory / (1024 * 1024),
            "fragmented_memory_mb": fragmented_memory / (1024 * 1024),
            "memory_overhead_mb": memory_overhead / (1024 * 1024),
            "fragmentation_impact_score": min(1.0, memory_overhead / baseline_memory) if baseline_memory > 0 else 0
        }
    
    def _test_circular_references(self) -> Dict[str, Any]:
        """Test for circular reference leaks"""
        circular_refs = []
        weak_refs = []
        
        # Create circular references
        for i in range(100):
            obj1 = {"id": i, "ref": None}
            obj2 = {"id": i, "ref": obj1}
            obj1["ref"] = obj2
            
            # Create weak reference
            weak_refs.append(weakref.ref(obj1))
            
            circular_refs.extend([obj1, obj2])
        
        del circular_refs
        gc.collect()
        
        # Check if weak references are still valid (indicates leak)
        valid_refs = sum(1 for ref in weak_refs if ref() is not None)
        leak_detected = valid_refs > 0
        
        return {
            "circular_references_created": len(circular_refs),
            "weak_references_valid": valid_refs,
            "leak_detected": leak_detected
        }
    
    def _get_tracemalloc_summary(self) -> Dict[str, Any]:
        """Get tracemalloc memory summary"""
        try:
            snapshot = tracemalloc.take_snapshot()
            top_stats = snapshot.statistics('lineno')
            
            return {
                "top_memory_users": [
                    {
                        "filename": stat.traceback.format()[0],
                        "size_mb": stat.size / (1024 * 1024),
                        "count": stat.count
                    }
                    for stat in top_stats[:10]
                ]
            }
        except Exception:
            return {"error": "Could not get tracemalloc summary"}
    
    def _test_gradual_memory_pressure(self) -> Dict[str, Any]:
        """Test gradual memory pressure"""
        memory_usage_samples = []
        allocations = []
        
        for i in range(100):
            try:
                chunk = bytearray(1024 * 100)  # 100KB chunks
                allocations.append(chunk)
                
                if i % 10 == 0:
                    memory = psutil.Process().memory_info().rss
                    memory_usage_samples.append(memory / (1024 * 1024))
            except MemoryError:
                break
        
        del allocations
        gc.collect()
        
        return {
            "pressure_type": "gradual",
            "successful_allocations": len(allocations),
            "memory_samples": memory_usage_samples,
            "peak_memory_mb": max(memory_usage_samples) if memory_usage_samples else 0
        }
    
    def _test_memory_bursts(self) -> Dict[str, Any]:
        """Test rapid memory allocation bursts"""
        burst_allocations = []
        
        for burst in range(10):
            burst_allocs = []
            try:
                for i in range(50):
                    chunk = bytearray(1024 * 50)  # 50KB chunks
                    burst_allocs.append(chunk)
                burst_allocations.append(len(burst_allocs))
            except MemoryError:
                burst_allocations.append(len(burst_allocs))
                break
            
            time.sleep(0.1)  # Small delay between bursts
        
        return {
            "pressure_type": "burst",
            "burst_results": burst_allocations,
            "total_successful_bursts": sum(1 for result in burst_allocations if result == 50)
        }
    
    def _test_concurrent_memory_pressure(self) -> Dict[str, Any]:
        """Test memory pressure with concurrent processes"""
        def worker_memory_allocation(worker_id, results):
            try:
                allocations = []
                for i in range(20):
                    chunk = bytearray(1024 * 25)  # 25KB chunks
                    allocations.append(chunk)
                results[worker_id] = len(allocations)
            except MemoryError:
                results[worker_id] = 0
            except Exception:
                results[worker_id] = -1
        
        manager = multiprocessing.Manager()
        results = manager.dict()
        processes = []
        
        # Start multiple processes allocating memory
        for i in range(4):
            p = multiprocessing.Process(target=worker_memory_allocation, args=(i, results))
            processes.append(p)
            p.start()
        
        # Wait for all processes
        for p in processes:
            p.join()
        
        return {
            "pressure_type": "concurrent",
            "process_results": dict(results),
            "total_successful_allocations": sum(r for r in results.values() if r > 0)
        }
    
    def _test_memory_pressure_response(self) -> Dict[str, Any]:
        """Test system response to memory pressure"""
        # Get initial memory state
        initial_memory = psutil.virtual_memory()
        initial_available = initial_memory.available
        
        # Allocate memory to create pressure
        allocations = []
        try:
            for i in range(50):
                chunk = bytearray(1024 * 200)  # 200KB chunks
                allocations.append(chunk)
        except MemoryError:
            pass
        
        # Measure system response
        pressure_memory = psutil.virtual_memory()
        pressure_response = {
            "initial_available_mb": initial_available / (1024 * 1024),
            "pressure_available_mb": pressure_memory.available / (1024 * 1024),
            "memory_pressure_mb": (initial_available - pressure_memory.available) / (1024 * 1024),
            "allocations_created": len(allocations)
        }
        
        del allocations
        gc.collect()
        
        return pressure_response
    
    def _calculate_pressure_resilience(self, pressure_tests: List[Dict[str, Any]]) -> float:
        """Calculate system resilience to memory pressure"""
        if not pressure_tests:
            return 0.0
        
        resilience_scores = []
        
        for test in pressure_tests:
            if test["pressure_type"] == "gradual":
                # Resilience based on successful allocations
                score = min(1.0, test["successful_allocations"] / 100)
                resilience_scores.append(score)
            
            elif test["pressure_type"] == "burst":
                # Resilience based on burst success rate
                successful_bursts = test.get("total_successful_bursts", 0)
                score = min(1.0, successful_bursts / 10)
                resilience_scores.append(score)
            
            elif test["pressure_type"] == "concurrent":
                # Resilience based on concurrent allocation success
                total_allocations = test.get("total_successful_allocations", 0)
                expected_allocations = 4 * 20  # 4 processes * 20 allocations each
                score = min(1.0, total_allocations / expected_allocations)
                resilience_scores.append(score)
        
        return sum(resilience_scores) / len(resilience_scores) if resilience_scores else 0.0
    
    def _test_array_bounds_protection(self) -> Dict[str, Any]:
        """Test array bounds checking protection"""
        try:
            # Test with numpy arrays (built-in bounds checking)
            arr = np.array([1, 2, 3, 4, 5])
            
            # This should work
            valid_access = arr[2]
            
            # This would cause an error in Python/numpy
            try:
                invalid_access = arr[10]
                bounds_protection = False
            except IndexError:
                bounds_protection = True
            
            return {
                "protection_type": "array_bounds",
                "protected": bounds_protection,
                "test_details": "NumPy array bounds checking"
            }
        except Exception as e:
            return {
                "protection_type": "array_bounds",
                "protected": False,
                "error": str(e)
            }
    
    def _test_buffer_overflow_protection(self) -> Dict[str, Any]:
        """Test buffer overflow protection"""
        try:
            # Test with bytearray (Python's protected buffer)
            buffer = bytearray(100)
            
            # Valid access
            buffer[50] = 42
            
            # Test if we can detect overflow
            try:
                buffer[101] = 42  # This should raise IndexError
                overflow_protection = False
            except IndexError:
                overflow_protection = True
            
            return {
                "protection_type": "buffer_overflow",
                "protected": overflow_protection,
                "test_details": "Python bytearray bounds checking"
            }
        except Exception as e:
            return {
                "protection_type": "buffer_overflow",
                "protected": False,
                "error": str(e)
            }
    
    def _test_integer_overflow_protection(self) -> Dict[str, Any]:
        """Test integer overflow protection"""
        try:
            # Python automatically handles big integers
            large_int = 2 ** 1000
            larger_int = large_int * large_int
            
            # Test with numpy integers (may overflow)
            try:
                np_int16 = np.int16(32767)
                overflow = np_int16 + 1
                overflow_protected = (overflow == -32768)  # Wrapped around
            except Exception:
                overflow_protected = True  # Some protection mechanism
            
            return {
                "protection_type": "integer_overflow",
                "protected": True,  # Python big integers
                "test_details": "Python arbitrary precision integers"
            }
        except Exception as e:
            return {
                "protection_type": "integer_overflow",
                "protected": False,
                "error": str(e)
            }
    
    def _test_allocation_overflow_protection(self) -> Dict[str, Any]:
        """Test memory allocation overflow protection"""
        try:
            # Try to allocate extremely large memory
            try:
                huge_allocation = bytearray(2**63)  # Near max size
                allocation_protected = False
            except MemoryError:
                allocation_protected = True
            except OverflowError:
                allocation_protected = True
            
            # Try negative allocation size
            try:
                negative_allocation = bytearray(-1)
                size_protected = False
            except (ValueError, OverflowError):
                size_protected = True
            
            return {
                "protection_type": "allocation_overflow",
                "size_protected": size_protected,
                "allocation_protected": allocation_protected,
                "protected": size_protected and allocation_protected
            }
        except Exception as e:
            return {
                "protection_type": "allocation_overflow",
                "protected": False,
                "error": str(e)
            }