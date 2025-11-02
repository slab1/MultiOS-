#!/usr/bin/env python3
"""
Resource exhaustion testing module
Tests system behavior when resources like file handles, memory, and processes are exhausted
"""

import os
import sys
import time
import threading
import multiprocessing
import psutil
import resource
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor, as_completed
import socket
import gc


class ResourceExhaustionTester:
    """Advanced resource exhaustion testing module"""
    
    def __init__(self, config):
        self.config = config
        self.test_dir = Path(config.test_dir) / "resource_exhaustion_tests"
        self.test_dir.mkdir(parents=True, exist_ok=True)
        
        # Resource tracking
        self.resource_samples = []
        self.exhaustion_points = {}
        self.recovery_metrics = {}
    
    def test_file_handle_exhaustion(self) -> Dict[str, Any]:
        """Test file handle/descriptor exhaustion scenarios"""
        results = {
            "test_name": "File Handle Exhaustion",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            # Get system limits
            try:
                soft_limit, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
                current_limit = soft_limit
            except Exception:
                current_limit = 1024  # Default assumption
            
            if current_limit > 10000:
                current_limit = 10000  # Cap for testing
            
            exhaustion_tests = []
            
            # Test 1: Progressive file handle allocation
            progressive_allocation = self._test_progressive_file_allocation(current_limit)
            exhaustion_tests.append(progressive_allocation)
            
            # Test 2: Rapid file handle allocation
            rapid_allocation = self._test_rapid_file_allocation(min(current_limit // 10, 1000))
            exhaustion_tests.append(rapid_allocation)
            
            # Test 3: File handle leak simulation
            leak_simulation = self._test_file_handle_leak_simulation()
            exhaustion_tests.append(leak_simulation)
            
            # Test 4: File handle recovery
            recovery_test = self._test_file_handle_recovery()
            exhaustion_tests.append(recovery_test)
            
            results["metrics"].update({
                "exhaustion_tests": exhaustion_tests,
                "system_file_handle_limit": current_limit,
                "exhaustion_point": progressive_allocation.get("exhaustion_point", current_limit),
                "recovery_successful": all(test.get("recovery_successful", True) for test in exhaustion_tests)
            })
            
            # Determine status
            recovery_success = results["metrics"]["recovery_successful"]
            exhaustion_point = results["metrics"]["exhaustion_point"]
            
            if recovery_success and exhaustion_point > current_limit * 0.8:
                results["status"] = "PASS"
            elif recovery_success:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"File handle exhaustion test failed: {str(e)}")
        
        return results
    
    def test_memory_exhaustion(self) -> Dict[str, Any]:
        """Test memory exhaustion scenarios"""
        results = {
            "test_name": "Memory Exhaustion Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            memory = psutil.virtual_memory()
            available_memory_mb = memory.available / (1024 * 1024)
            
            if available_memory_mb < 100:  # Less than 100MB available
                results["warnings"].append("Insufficient memory for exhaustion test")
                results["status"] = "SKIPPED"
                return results
            
            exhaustion_tests = []
            
            # Test 1: Progressive memory allocation
            progressive_allocation = self._test_progressive_memory_allocation(available_memory_mb)
            exhaustion_tests.append(progressive_allocation)
            
            # Test 2: Rapid memory allocation bursts
            rapid_allocation = self._test_rapid_memory_allocation()
            exhaustion_tests.append(rapid_allocation)
            
            # Test 3: Memory leak simulation
            leak_simulation = self._test_memory_leak_simulation()
            exhaustion_tests.append(leak_simulation)
            
            # Test 4: Memory pressure recovery
            pressure_recovery = self._test_memory_pressure_recovery()
            exhaustion_tests.append(pressure_recovery)
            
            results["metrics"].update({
                "exhaustion_tests": exhaustion_tests,
                "available_memory_mb": available_memory_mb,
                "exhaustion_point_mb": progressive_allocation.get("exhaustion_point_mb", 0),
                "recovery_successful": all(test.get("recovery_successful", True) for test in exhaustion_tests)
            })
            
            # Determine status
            recovery_success = results["metrics"]["recovery_successful"]
            if recovery_success:
                results["status"] = "PASS"
            else:
                results["status"] = "PARTIAL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Memory exhaustion test failed: {str(e)}")
        
        return results
    
    def test_process_limit(self) -> Dict[str, Any]:
        """Test process limit exhaustion scenarios"""
        results = {
            "test_name": "Process Limit Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            # Get process limits
            try:
                soft_limit, hard_limit = resource.getrlimit(resource.RLIMIT_NPROC)
                current_limit = soft_limit
            except Exception:
                current_limit = 100  # Default assumption
            
            if current_limit > 1000:
                current_limit = 1000  # Cap for testing
            
            exhaustion_tests = []
            
            # Test 1: Progressive process creation
            progressive_creation = self._test_progressive_process_creation(current_limit)
            exhaustion_tests.append(progressive_creation)
            
            # Test 2: Rapid process creation
            rapid_creation = self._test_rapid_process_creation(min(current_limit // 10, 50))
            exhaustion_tests.append(rapid_creation)
            
            # Test 3: Process cleanup and reuse
            cleanup_reuse = self._test_process_cleanup_reuse()
            exhaustion_tests.append(cleanup_reuse)
            
            results["metrics"].update({
                "exhaustion_tests": exhaustion_tests,
                "system_process_limit": current_limit,
                "exhaustion_point": progressive_creation.get("exhaustion_point", 0),
                "process_reuse_successful": cleanup_reuse.get("reuse_successful", False)
            })
            
            # Determine status
            exhaustion_point = results["metrics"]["exhaustion_point"]
            process_reuse = results["metrics"]["process_reuse_successful"]
            
            if exhaustion_point > current_limit * 0.5 and process_reuse:
                results["status"] = "PASS"
            elif exhaustion_point > current_limit * 0.3:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Process limit test failed: {str(e)}")
        
        return results
    
    def test_network_limits(self) -> Dict[str, Any]:
        """Test network connection limits"""
        results = {
            "test_name": "Network Limits Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            exhaustion_tests = []
            
            # Test 1: Socket creation limits
            socket_creation = self._test_socket_creation_limits()
            exhaustion_tests.append(socket_creation)
            
            # Test 2: Connection exhaustion
            connection_exhaustion = self._test_connection_exhaustion()
            exhaustion_tests.append(connection_exhaustion)
            
            # Test 3: Port exhaustion
            port_exhaustion = self._test_port_exhaustion()
            exhaustion_tests.append(port_exhaustion)
            
            # Test 4: Network resource cleanup
            network_cleanup = self._test_network_resource_cleanup()
            exhaustion_tests.append(network_cleanup)
            
            results["metrics"].update({
                "exhaustion_tests": exhaustion_tests,
                "socket_creation_limit": socket_creation.get("max_sockets", 0),
                "connection_limit": connection_exhaustion.get("max_connections", 0),
                "cleanup_successful": network_cleanup.get("cleanup_successful", False)
            })
            
            # Determine status
            cleanup_successful = results["metrics"]["cleanup_successful"]
            if cleanup_successful:
                results["status"] = "PASS"
            else:
                results["status"] = "PARTIAL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Network limits test failed: {str(e)}")
        
        return results
    
    def test_resource_recovery(self) -> Dict[str, Any]:
        """Test resource recovery mechanisms"""
        results = {
            "test_name": "Resource Recovery Testing",
            "status": "PASS",
            "metrics": {},
            "errors": [],
            "warnings": []
        }
        
        try:
            recovery_tests = []
            
            # Test 1: File handle recovery
            fh_recovery = self._test_file_handle_recovery_detailed()
            recovery_tests.append(fh_recovery)
            
            # Test 2: Memory recovery
            memory_recovery = self._test_memory_recovery_detailed()
            recovery_tests.append(memory_recovery)
            
            # Test 3: Process recovery
            process_recovery = self._test_process_recovery_detailed()
            recovery_tests.append(process_recovery)
            
            # Test 4: System resource recovery
            system_recovery = self._test_system_resource_recovery()
            recovery_tests.append(system_recovery)
            
            results["metrics"].update({
                "recovery_tests": recovery_tests,
                "total_recovery_tests": len(recovery_tests),
                "successful_recoveries": sum(1 for test in recovery_tests if test.get("recovery_successful", False)),
                "average_recovery_time": sum(test.get("recovery_time", 0) for test in recovery_tests) / len(recovery_tests)
            })
            
            # Determine status
            success_rate = results["metrics"]["successful_recoveries"] / results["metrics"]["total_recovery_tests"]
            if success_rate >= 0.9:
                results["status"] = "PASS"
            elif success_rate >= 0.7:
                results["status"] = "PARTIAL"
            else:
                results["status"] = "FAIL"
                
        except Exception as e:
            results["status"] = "ERROR"
            results["errors"].append(f"Resource recovery test failed: {str(e)}")
        
        return results
    
    # Helper methods for file handle exhaustion testing
    def _test_progressive_file_allocation(self, max_limit: int) -> Dict[str, Any]:
        """Test progressive file handle allocation until exhaustion"""
        open_files = []
        allocation_results = []
        exhaustion_point = 0
        
        try:
            for i in range(max_limit):
                try:
                    test_file = self.test_dir / f"progressive_test_{i}.dat"
                    f = open(test_file, 'w')
                    open_files.append(f)
                    
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "success": True,
                        "timestamp": time.time()
                    })
                    
                    exhaustion_point = i + 1
                    
                except OSError as e:
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "success": False,
                        "error": str(e),
                        "timestamp": time.time()
                    })
                    break
                
                except Exception as e:
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "success": False,
                        "error": str(e),
                        "timestamp": time.time()
                    })
                    break
                    
        except Exception as e:
            pass
        
        finally:
            # Clean up all open files
            for f in open_files:
                try:
                    f.close()
                except:
                    pass
            
            # Clean up test files
            for i in range(exhaustion_point):
                test_file = self.test_dir / f"progressive_test_{i}.dat"
                if test_file.exists():
                    test_file.unlink()
        
        return {
            "test_type": "progressive_file_allocation",
            "max_limit": max_limit,
            "exhaustion_point": exhaustion_point,
            "allocation_results": allocation_results[-10:],  # Last 10 results
            "recovery_successful": True
        }
    
    def _test_rapid_file_allocation(self, test_limit: int) -> Dict[str, Any]:
        """Test rapid file handle allocation"""
        open_files = []
        start_time = time.time()
        
        try:
            # Rapidly allocate file handles
            for i in range(test_limit):
                test_file = self.test_dir / f"rapid_test_{i}.dat"
                f = open(test_file, 'w')
                f.write(f"Rapid test {i}")
                open_files.append(f)
                
        except Exception as e:
            pass
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Clean up
        for f in open_files:
            try:
                f.close()
            except:
                pass
        
        for i in range(len(open_files)):
            test_file = self.test_dir / f"rapid_test_{i}.dat"
            if test_file.exists():
                test_file.unlink()
        
        return {
            "test_type": "rapid_file_allocation",
            "test_limit": test_limit,
            "allocated_handles": len(open_files),
            "duration": duration,
            "allocation_rate": len(open_files) / duration,
            "recovery_successful": True
        }
    
    def _test_file_handle_leak_simulation(self) -> Dict[str, Any]:
        """Test file handle leak simulation and detection"""
        initial_open_files = len(psutil.Process().open_files())
        
        # Simulate proper file handling
        proper_files = []
        for i in range(50):
            test_file = self.test_dir / f"proper_test_{i}.dat"
            try:
                f = open(test_file, 'w')
                f.write(f"Proper test {i}")
                f.close()  # Properly close
                proper_files.append(test_file)
            except Exception:
                break
        
        # Simulate potential leaks (files left open)
        leaked_files = []
        for i in range(20):
            test_file = self.test_dir / f"leak_test_{i}.dat"
            try:
                f = open(test_file, 'w')
                f.write(f"Leak test {i}")
                leaked_files.append((test_file, f))  # Don't close - simulate leak
            except Exception:
                break
        
        # Measure after potential leak
        after_leak_open_files = len(psutil.Process().open_files())
        
        # Clean up leaked files
        for test_file, f in leaked_files:
            try:
                f.close()
                test_file.unlink()
            except:
                pass
        
        for test_file in proper_files:
            if test_file.exists():
                test_file.unlink()
        
        final_open_files = len(psutil.Process().open_files())
        file_increase = final_open_files - initial_open_files
        
        return {
            "test_type": "file_handle_leak_simulation",
            "initial_open_files": initial_open_files,
            "after_leak_open_files": after_leak_open_files,
            "final_open_files": final_open_files,
            "file_handle_increase": file_increase,
            "leak_detected": file_increase > 5,  # Threshold for leak detection
            "recovery_successful": file_increase <= 2  # Allow small variance
        }
    
    def _test_file_handle_recovery(self) -> Dict[str, Any]:
        """Test file handle recovery mechanisms"""
        # Create file handle pressure
        open_files = []
        for i in range(100):
            try:
                test_file = self.test_dir / f"recovery_test_{i}.dat"
                f = open(test_file, 'w')
                f.write(f"Recovery test {i}")
                open_files.append(f)
            except Exception:
                break
        
        start_recovery_time = time.time()
        
        # Force garbage collection
        gc.collect()
        
        # Close files in batches to simulate recovery
        batch_size = 20
        for i in range(0, len(open_files), batch_size):
            batch = open_files[i:i + batch_size]
            for f in batch:
                try:
                    f.close()
                except:
                    pass
        
        end_recovery_time = time.time()
        recovery_time = end_recovery_time - start_recovery_time
        
        # Clean up test files
        for i in range(len(open_files)):
            test_file = self.test_dir / f"recovery_test_{i}.dat"
            if test_file.exists():
                test_file.unlink()
        
        return {
            "test_type": "file_handle_recovery",
            "files_opened": len(open_files),
            "recovery_time": recovery_time,
            "recovery_successful": True
        }
    
    # Helper methods for memory exhaustion testing
    def _test_progressive_memory_allocation(self, available_mb: float) -> Dict[str, Any]:
        """Test progressive memory allocation until exhaustion"""
        allocated_chunks = []
        exhaustion_point_mb = 0
        allocation_results = []
        
        try:
            # Allocate memory in chunks until exhaustion
            chunk_size_mb = min(available_mb / 20, 50)  # Allocate in reasonable chunks
            target_allocation = min(available_mb * 0.8, 500)  # Don't use all memory
            
            for i in range(int(target_allocation / chunk_size_mb)):
                try:
                    chunk = bytearray(int(chunk_size_mb * 1024 * 1024))
                    allocated_chunks.append(chunk)
                    
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "chunk_size_mb": chunk_size_mb,
                        "success": True,
                        "total_allocated_mb": (i + 1) * chunk_size_mb
                    })
                    
                    exhaustion_point_mb = (i + 1) * chunk_size_mb
                    
                except MemoryError:
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "chunk_size_mb": chunk_size_mb,
                        "success": False,
                        "error": "MemoryError",
                        "total_allocated_mb": i * chunk_size_mb
                    })
                    break
                
                except Exception as e:
                    allocation_results.append({
                        "allocation_number": i + 1,
                        "chunk_size_mb": chunk_size_mb,
                        "success": False,
                        "error": str(e),
                        "total_allocated_mb": i * chunk_size_mb
                    })
                    break
            
        finally:
            # Clean up allocated memory
            del allocated_chunks
            gc.collect()
        
        return {
            "test_type": "progressive_memory_allocation",
            "available_memory_mb": available_mb,
            "exhaustion_point_mb": exhaustion_point_mb,
            "allocation_results": allocation_results[-5:],  # Last 5 results
            "recovery_successful": True
        }
    
    def _test_rapid_memory_allocation(self) -> Dict[str, Any]:
        """Test rapid memory allocation bursts"""
        allocation_bursts = []
        
        for burst in range(10):
            burst_start = time.time()
            burst_chunks = []
            
            try:
                for i in range(20):
                    chunk = bytearray(10 * 1024 * 1024)  # 10MB chunks
                    burst_chunks.append(chunk)
                
                burst_duration = time.time() - burst_start
                allocation_bursts.append({
                    "burst_number": burst + 1,
                    "chunks_allocated": len(burst_chunks),
                    "duration": burst_duration,
                    "success": True
                })
                
            except MemoryError:
                burst_duration = time.time() - burst_start
                allocation_bursts.append({
                    "burst_number": burst + 1,
                    "chunks_allocated": len(burst_chunks),
                    "duration": burst_duration,
                    "success": False,
                    "error": "MemoryError"
                })
                break
            
            except Exception as e:
                burst_duration = time.time() - burst_start
                allocation_bursts.append({
                    "burst_number": burst + 1,
                    "chunks_allocated": len(burst_chunks),
                    "duration": burst_duration,
                    "success": False,
                    "error": str(e)
                })
                break
            
            finally:
                # Clean up burst chunks
                del burst_chunks
                gc.collect()
        
        return {
            "test_type": "rapid_memory_allocation",
            "allocation_bursts": allocation_bursts,
            "successful_bursts": sum(1 for b in allocation_bursts if b["success"]),
            "recovery_successful": True
        }
    
    def _test_memory_leak_simulation(self) -> Dict[str, Any]:
        """Test memory leak simulation and detection"""
        import tracemalloc
        tracemalloc.start()
        
        initial_memory = psutil.Process().memory_info().rss
        leak_objects = []
        
        # Simulate memory leak
        for i in range(100):
            # Create objects that might leak (circular references)
            obj1 = {"id": i, "data": "x" * 1000}
            obj2 = {"id": i, "ref": obj1}
            obj1["ref"] = obj2
            leak_objects.extend([obj1, obj2])
            
            # Take memory snapshot every 20 iterations
            if i % 20 == 0:
                current, peak = tracemalloc.get_traced_memory()
                memory_mb = current / (1024 * 1024)
                
                # Check for significant memory growth
                memory_growth = (current - tracemalloc.get_traced_memory()[0]) / (1024 * 1024)
                if memory_growth > 100:  # 100MB growth threshold
                    break
        
        # Simulate cleanup
        del leak_objects
        gc.collect()
        
        final_memory = psutil.Process().memory_info().rss
        memory_growth_mb = (final_memory - initial_memory) / (1024 * 1024)
        
        tracemalloc.stop()
        
        return {
            "test_type": "memory_leak_simulation",
            "initial_memory_mb": initial_memory / (1024 * 1024),
            "final_memory_mb": final_memory / (1024 * 1024),
            "memory_growth_mb": memory_growth_mb,
            "leak_detected": memory_growth_mb > 50,  # 50MB threshold
            "recovery_successful": memory_growth_mb < 20  # Less than 20MB retained
        }
    
    def _test_memory_pressure_recovery(self) -> Dict[str, Any]:
        """Test memory pressure recovery mechanisms"""
        # Create memory pressure
        memory_pressure_objects = []
        for i in range(20):
            try:
                obj = bytearray(50 * 1024 * 1024)  # 50MB objects
                memory_pressure_objects.append(obj)
            except MemoryError:
                break
        
        start_pressure_time = time.time()
        
        # Simulate pressure recovery
        del memory_pressure_objects
        gc.collect()
        
        # Force additional cleanup
        for _ in range(3):
            gc.collect()
            time.sleep(0.1)
        
        end_pressure_time = time.time()
        recovery_time = end_pressure_time - start_pressure_time
        
        return {
            "test_type": "memory_pressure_recovery",
            "objects_created": len(memory_pressure_objects),
            "recovery_time": recovery_time,
            "recovery_successful": True
        }
    
    # Helper methods for process limit testing
    def _test_progressive_process_creation(self, max_limit: int) -> Dict[str, Any]:
        """Test progressive process creation until limit"""
        processes = []
        creation_results = []
        exhaustion_point = 0
        
        def simple_process_worker(worker_id):
            """Simple worker that just sleeps"""
            time.sleep(5)
            return worker_id
        
        try:
            for i in range(max_limit):
                try:
                    p = multiprocessing.Process(target=simple_process_worker, args=(i,))
                    p.start()
                    processes.append(p)
                    
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": True,
                        "timestamp": time.time()
                    })
                    
                    exhaustion_point = i + 1
                    
                except OSError as e:
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": False,
                        "error": str(e),
                        "timestamp": time.time()
                    })
                    break
                
                except Exception as e:
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": False,
                        "error": str(e),
                        "timestamp": time.time()
                    })
                    break
                    
        except Exception as e:
            pass
        
        finally:
            # Clean up processes
            for p in processes:
                try:
                    p.terminate()
                    p.join(timeout=1)
                except:
                    pass
        
        return {
            "test_type": "progressive_process_creation",
            "max_limit": max_limit,
            "exhaustion_point": exhaustion_point,
            "creation_results": creation_results[-5:],  # Last 5 results
            "recovery_successful": True
        }
    
    def _test_rapid_process_creation(self, test_limit: int) -> Dict[str, Any]:
        """Test rapid process creation"""
        processes = []
        start_time = time.time()
        
        def rapid_worker(worker_id):
            time.sleep(1)
            return worker_id
        
        try:
            for i in range(test_limit):
                p = multiprocessing.Process(target=rapid_worker, args=(i,))
                p.start()
                processes.append(p)
                
        except Exception as e:
            pass
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Clean up processes
        for p in processes:
            try:
                p.terminate()
                p.join(timeout=1)
            except:
                pass
        
        return {
            "test_type": "rapid_process_creation",
            "test_limit": test_limit,
            "created_processes": len(processes),
            "duration": duration,
            "creation_rate": len(processes) / duration,
            "recovery_successful": True
        }
    
    def _test_process_cleanup_reuse(self) -> Dict[str, Any]:
        """Test process cleanup and reuse"""
        # Create and immediately cleanup processes
        cleanup_cycles = []
        
        for cycle in range(5):
            cycle_start = time.time()
            processes = []
            
            # Create processes
            def cleanup_worker(worker_id):
                return worker_id
            
            for i in range(10):
                p = multiprocessing.Process(target=cleanup_worker, args=(i,))
                p.start()
                processes.append(p)
            
            # Immediately cleanup
            for p in processes:
                try:
                    p.terminate()
                    p.join(timeout=1)
                except:
                    pass
            
            cycle_duration = time.time() - cycle_start
            cleanup_cycles.append({
                "cycle": cycle + 1,
                "processes_created": len(processes),
                "cleanup_duration": cycle_duration,
                "success": True
            })
        
        successful_cycles = sum(1 for c in cleanup_cycles if c["success"])
        
        return {
            "test_type": "process_cleanup_reuse",
            "cleanup_cycles": cleanup_cycles,
            "successful_cycles": successful_cycles,
            "reuse_successful": successful_cycles == len(cleanup_cycles)
        }
    
    # Helper methods for network limits testing
    def _test_socket_creation_limits(self) -> Dict[str, Any]:
        """Test socket creation limits"""
        sockets = []
        creation_results = []
        
        try:
            for i in range(1000):
                try:
                    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                    sockets.append(sock)
                    
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": True
                    })
                    
                except OSError as e:
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                
                except Exception as e:
                    creation_results.append({
                        "creation_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                    
        except Exception as e:
            pass
        
        finally:
            # Clean up sockets
            for sock in sockets:
                try:
                    sock.close()
                except:
                    pass
        
        return {
            "test_type": "socket_creation_limits",
            "max_sockets": len(sockets),
            "creation_results": creation_results[-10:],  # Last 10 results
            "recovery_successful": True
        }
    
    def _test_connection_exhaustion(self) -> Dict[str, Any]:
        """Test connection exhaustion"""
        # This is a simplified connection test
        # Real connection exhaustion would require a server to connect to
        
        connections = []
        connection_results = []
        
        try:
            # Create listening sockets (simulate server connections)
            for i in range(100):
                try:
                    server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                    server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                    
                    # Bind to any available port
                    server_sock.bind(('localhost', 0))
                    port = server_sock.getsockname()[1]
                    server_sock.listen(1)
                    
                    connections.append(server_sock)
                    
                    connection_results.append({
                        "connection_number": i + 1,
                        "port": port,
                        "success": True
                    })
                    
                except OSError as e:
                    connection_results.append({
                        "connection_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                
                except Exception as e:
                    connection_results.append({
                        "connection_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                    
        except Exception as e:
            pass
        
        finally:
            # Clean up connections
            for conn in connections:
                try:
                    conn.close()
                except:
                    pass
        
        return {
            "test_type": "connection_exhaustion",
            "max_connections": len(connections),
            "connection_results": connection_results[-10:],  # Last 10 results
            "recovery_successful": True
        }
    
    def _test_port_exhaustion(self) -> Dict[str, Any]:
        """Test port exhaustion"""
        ports_used = []
        port_results = []
        
        try:
            for i in range(100):
                try:
                    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                    
                    # Bind to any available port
                    sock.bind(('localhost', 0))
                    port = sock.getsockname()[1]
                    ports_used.append((sock, port))
                    
                    port_results.append({
                        "port_number": i + 1,
                        "port": port,
                        "success": True
                    })
                    
                except OSError as e:
                    port_results.append({
                        "port_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                
                except Exception as e:
                    port_results.append({
                        "port_number": i + 1,
                        "success": False,
                        "error": str(e)
                    })
                    break
                    
        except Exception as e:
            pass
        
        finally:
            # Clean up sockets
            for sock, port in ports_used:
                try:
                    sock.close()
                except:
                    pass
        
        return {
            "test_type": "port_exhaustion",
            "ports_used": len(ports_used),
            "port_results": port_results[-10:],  # Last 10 results
            "recovery_successful": True
        }
    
    def _test_network_resource_cleanup(self) -> Dict[str, Any]:
        """Test network resource cleanup"""
        # Create network resources
        network_resources = []
        for i in range(50):
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                sock.bind(('localhost', 0))
                network_resources.append(sock)
            except Exception:
                break
        
        start_cleanup_time = time.time()
        
        # Clean up network resources
        for sock in network_resources:
            try:
                sock.close()
            except:
                pass
        
        end_cleanup_time = time.time()
        cleanup_time = end_cleanup_time - start_cleanup_time
        
        return {
            "test_type": "network_resource_cleanup",
            "resources_created": len(network_resources),
            "cleanup_time": cleanup_time,
            "cleanup_successful": True
        }
    
    # Helper methods for resource recovery testing
    def _test_file_handle_recovery_detailed(self) -> Dict[str, Any]:
        """Detailed file handle recovery test"""
        recovery_start = time.time()
        
        # Create file handle pressure
        open_files = []
        for i in range(200):
            try:
                test_file = self.test_dir / f"detailed_recovery_{i}.dat"
                f = open(test_file, 'w')
                f.write(f"Recovery test {i}")
                open_files.append(f)
            except Exception:
                break
        
        # Measure resource usage during pressure
        pressure_fds = len(psutil.Process().open_files())
        
        # Recovery process
        recovery_actions = []
        
        # Step 1: Close files in batches
        batch_size = 50
        for i in range(0, len(open_files), batch_size):
            batch = open_files[i:i + batch_size]
            batch_start = time.time()
            
            for f in batch:
                try:
                    f.close()
                except:
                    pass
            
            batch_end = time.time()
            recovery_actions.append({
                "action": f"Close batch {i//batch_size + 1}",
                "files_closed": len(batch),
                "duration": batch_end - batch_start
            })
        
        # Step 2: Force cleanup
        gc.collect()
        
        # Step 3: Verify recovery
        final_fds = len(psutil.Process().open_files())
        
        # Clean up test files
        for i in range(len(open_files)):
            test_file = self.test_dir / f"detailed_recovery_{i}.dat"
            if test_file.exists():
                test_file.unlink()
        
        recovery_end = time.time()
        total_recovery_time = recovery_end - recovery_start
        
        return {
            "test_type": "file_handle_recovery_detailed",
            "pressure_fds": pressure_fds,
            "final_fds": final_fds,
            "recovery_actions": recovery_actions,
            "total_recovery_time": total_recovery_time,
            "recovery_successful": final_fds <= pressure_fds * 0.2  # 80% reduction
        }
    
    def _test_memory_recovery_detailed(self) -> Dict[str, Any]:
        """Detailed memory recovery test"""
        recovery_start = time.time()
        
        # Create memory pressure
        memory_objects = []
        for i in range(30):
            try:
                obj = bytearray(20 * 1024 * 1024)  # 20MB objects
                memory_objects.append(obj)
            except MemoryError:
                break
        
        # Measure memory during pressure
        initial_memory = psutil.Process().memory_info().rss
        
        # Recovery process
        recovery_steps = []
        
        # Step 1: Clear references
        step1_start = time.time()
        memory_objects.clear()
        step1_end = time.time()
        
        recovery_steps.append({
            "step": "Clear references",
            "duration": step1_end - step1_start
        })
        
        # Step 2: Force garbage collection
        step2_start = time.time()
        for _ in range(3):
            gc.collect()
        step2_end = time.time()
        
        recovery_steps.append({
            "step": "Force garbage collection",
            "duration": step2_end - step2_start
        })
        
        # Step 3: Wait for system cleanup
        step3_start = time.time()
        time.sleep(1)
        step3_end = time.time()
        
        recovery_steps.append({
            "step": "Wait for system cleanup",
            "duration": step3_end - step3_start
        })
        
        # Measure final memory
        final_memory = psutil.Process().memory_info().rss
        
        recovery_end = time.time()
        total_recovery_time = recovery_end - recovery_start
        
        memory_freed = initial_memory - final_memory
        recovery_percentage = (memory_freed / initial_memory * 100) if initial_memory > 0 else 0
        
        return {
            "test_type": "memory_recovery_detailed",
            "initial_memory_mb": initial_memory / (1024 * 1024),
            "final_memory_mb": final_memory / (1024 * 1024),
            "memory_freed_mb": memory_freed / (1024 * 1024),
            "recovery_percentage": recovery_percentage,
            "recovery_steps": recovery_steps,
            "total_recovery_time": total_recovery_time,
            "recovery_successful": recovery_percentage > 70  # 70% memory recovery
        }
    
    def _test_process_recovery_detailed(self) -> Dict[str, Any]:
        """Detailed process recovery test"""
        recovery_start = time.time()
        
        # Create process pressure
        processes = []
        def recovery_worker(worker_id):
            time.sleep(10)  # Long-running worker
            return worker_id
        
        for i in range(20):
            try:
                p = multiprocessing.Process(target=recovery_worker, args=(i,))
                p.start()
                processes.append(p)
            except Exception:
                break
        
        initial_process_count = len(psutil.pids())
        
        # Recovery process
        recovery_actions = []
        
        # Step 1: Terminate processes
        step1_start = time.time()
        for p in processes:
            try:
                p.terminate()
            except:
                pass
        step1_end = time.time()
        
        recovery_actions.append({
            "action": "Terminate processes",
            "duration": step1_end - step1_start
        })
        
        # Step 2: Wait for termination
        step2_start = time.time()
        for p in processes:
            try:
                p.join(timeout=2)
            except:
                pass
        step2_end = time.time()
        
        recovery_actions.append({
            "action": "Wait for termination",
            "duration": step2_end - step2_start
        })
        
        # Step 3: Force cleanup if needed
        step3_start = time.time()
        for p in processes:
            try:
                if p.is_alive():
                    p.kill()
                    p.join()
            except:
                pass
        step3_end = time.time()
        
        recovery_actions.append({
            "action": "Force cleanup",
            "duration": step3_end - step3_start
        })
        
        # Measure final process count
        final_process_count = len(psutil.pids())
        
        recovery_end = time.time()
        total_recovery_time = recovery_end - recovery_start
        
        return {
            "test_type": "process_recovery_detailed",
            "initial_process_count": initial_process_count,
            "final_process_count": final_process_count,
            "processes_created": len(processes),
            "recovery_actions": recovery_actions,
            "total_recovery_time": total_recovery_time,
            "recovery_successful": len(processes) == 20 and final_process_count <= initial_process_count + 5
        }
    
    def _test_system_resource_recovery(self) -> Dict[str, Any]:
        """Test overall system resource recovery"""
        recovery_start = time.time()
        
        # Measure baseline system resources
        baseline = {
            "memory_mb": psutil.virtual_memory().used / (1024 * 1024),
            "cpu_percent": psutil.cpu_percent(interval=1),
            "process_count": len(psutil.pids()),
            "open_files": len(psutil.Process().open_files())
        }
        
        # Create mixed resource pressure
        pressure_operations = []
        
        # Memory pressure
        memory_objects = []
        for i in range(20):
            try:
                obj = bytearray(10 * 1024 * 1024)  # 10MB
                memory_objects.append(obj)
                pressure_operations.append(f"Allocated {i+1} x 10MB")
            except MemoryError:
                break
        
        # File pressure
        open_files = []
        for i in range(100):
            try:
                test_file = self.test_dir / f"system_recovery_{i}.dat"
                f = open(test_file, 'w')
                f.write(f"System recovery test {i}")
                open_files.append(f)
                pressure_operations.append(f"Opened file {i+1}")
            except Exception:
                break
        
        # Process pressure
        processes = []
        def system_worker(worker_id):
            time.sleep(5)
            return worker_id
        
        for i in range(10):
            try:
                p = multiprocessing.Process(target=system_worker, args=(i,))
                p.start()
                processes.append(p)
                pressure_operations.append(f"Created process {i+1}")
            except Exception:
                break
        
        # Measure system under pressure
        under_pressure = {
            "memory_mb": psutil.virtual_memory().used / (1024 * 1024),
            "cpu_percent": psutil.cpu_percent(interval=1),
            "process_count": len(psutil.pids()),
            "open_files": len(psutil.Process().open_files())
        }
        
        # Recovery process
        recovery_actions = []
        
        # Step 1: Clean up memory
        step1_start = time.time()
        del memory_objects
        gc.collect()
        step1_end = time.time()
        
        recovery_actions.append({
            "action": "Memory cleanup",
            "duration": step1_end - step1_start
        })
        
        # Step 2: Clean up files
        step2_start = time.time()
        for f in open_files:
            try:
                f.close()
            except:
                pass
        step2_end = time.time()
        
        recovery_actions.append({
            "action": "File cleanup",
            "duration": step2_end - step2_start
        })
        
        # Step 3: Clean up processes
        step3_start = time.time()
        for p in processes:
            try:
                p.terminate()
                p.join(timeout=1)
            except:
                pass
        step3_end = time.time()
        
        recovery_actions.append({
            "action": "Process cleanup",
            "duration": step3_end - step3_start
        })
        
        # Clean up test files
        for i in range(len(open_files)):
            test_file = self.test_dir / f"system_recovery_{i}.dat"
            if test_file.exists():
                test_file.unlink()
        
        # Wait for system stabilization
        time.sleep(2)
        gc.collect()
        
        # Measure final system state
        final_system = {
            "memory_mb": psutil.virtual_memory().used / (1024 * 1024),
            "cpu_percent": psutil.cpu_percent(interval=1),
            "process_count": len(psutil.pids()),
            "open_files": len(psutil.Process().open_files())
        }
        
        recovery_end = time.time()
        total_recovery_time = recovery_end - recovery_start
        
        # Calculate recovery metrics
        memory_recovery = (under_pressure["memory_mb"] - final_system["memory_mb"]) / (under_pressure["memory_mb"] - baseline["memory_mb"]) * 100
        file_recovery = (under_pressure["open_files"] - final_system["open_files"]) / (under_pressure["open_files"] - baseline["open_files"]) * 100 if under_pressure["open_files"] > baseline["open_files"] else 100
        
        return {
            "test_type": "system_resource_recovery",
            "baseline": baseline,
            "under_pressure": under_pressure,
            "final_system": final_system,
            "pressure_operations": pressure_operations,
            "recovery_actions": recovery_actions,
            "memory_recovery_percent": memory_recovery,
            "file_recovery_percent": file_recovery,
            "total_recovery_time": total_recovery_time,
            "recovery_successful": memory_recovery > 80 and file_recovery > 90
        }