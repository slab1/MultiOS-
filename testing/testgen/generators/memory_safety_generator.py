"""
Memory Safety Test Case Generator for MultiOS
Generates comprehensive memory safety test cases including buffer overflows, underflows, 
dangling pointers, and memory leaks
"""

import random
import json
import logging
from typing import Dict, List, Any, Optional, Tuple, Union
from dataclasses import dataclass
from enum import Enum

class MemoryViolationType(Enum):
    """Types of memory violations"""
    BUFFER_OVERFLOW = "buffer_overflow"
    BUFFER_UNDERFLOW = "buffer_underflow"
    DOUBLE_FREE = "double_free"
    USE_AFTER_FREE = "use_after_free"
    MEMORY_LEAK = "memory_leak"
    INVALID_POINTER = "invalid_pointer"
    STACK_OVERFLOW = "stack_overflow"
    NULL_POINTER_DEREFERENCE = "null_pointer_dereference"
    UNINITIALIZED_MEMORY = "uninitialized_memory"
    HEAP_CORRUPTION = "heap_corruption"
    OVERLAPPING_BUFFERS = "overlapping_buffers"

class MemorySafetyGenerator:
    """Generates memory safety test scenarios"""
    
    def __init__(self, seed: Optional[int] = None):
        self.logger = logging.getLogger(__name__)
        if seed:
            random.seed(seed)
        
        self.violation_types = list(MemoryViolationType)
        self.test_scenarios = self._initialize_scenarios()
        
    def _initialize_scenarios(self) -> Dict[str, List[MemoryViolationType]]:
        """Initialize memory safety test scenarios"""
        return {
            "allocation": [
                MemoryViolationType.BUFFER_OVERFLOW,
                MemoryViolationType.BUFFER_UNDERFLOW,
                MemoryViolationType.MEMORY_LEAK,
                MemoryViolationType.INVALID_POINTER
            ],
            "deallocation": [
                MemoryViolationType.DOUBLE_FREE,
                MemoryViolationType.USE_AFTER_FREE,
                MemoryViolationType.HEAP_CORRUPTION
            ],
            "access": [
                MemoryViolationType.NULL_POINTER_DEREFERENCE,
                MemoryViolationType.UNINITIALIZED_MEMORY,
                MemoryViolationType.OVERLAPPING_BUFFERS
            ],
            "bounds": [
                MemoryViolationType.BUFFER_OVERFLOW,
                MemoryViolationType.BUFFER_UNDERFLOW,
                MemoryViolationType.STACK_OVERFLOW
            ]
        }
    
    async def generate_memory_tests(self, component: str, iterations: int = 1000,
                                  parameters: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Generate memory safety test suite"""
        test_cases = []
        parameters = parameters or {}
        
        # Generate different types of memory tests
        test_cases.extend(await self._generate_buffer_overflow_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_memory_leak_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_double_free_tests(component, iterations // 4, parameters))
        test_cases.extend(await self._generate_use_after_free_tests(component, iterations // 6, parameters))
        test_cases.extend(await self._generate_pointer_safety_tests(component, iterations // 6, parameters))
        test_cases.extend(await self._generate_heap_corruption_tests(component, iterations // 8, parameters))
        
        return test_cases
    
    async def _generate_buffer_overflow_tests(self, component: str, count: int,
                                            parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate buffer overflow test cases"""
        test_cases = []
        
        overflow_types = [
            "write_beyond_end",
            "write_beyond_start",
            "read_beyond_end",
            "read_beyond_start",
            "array_index_overflow",
            "string_overflow"
        ]
        
        for i in range(count):
            overflow_type = overflow_types[i % len(overflow_types)]
            buffer_size = random.choice([4, 8, 16, 32, 64, 128])
            overflow_offset = random.randint(1, buffer_size * 2)
            
            test_case = {
                "id": f"buffer_overflow_{i}",
                "name": f"Buffer Overflow - {overflow_type}",
                "type": "memory_safety",
                "description": f"Test {component} buffer overflow scenario: {overflow_type}",
                "component": component,
                "violation_type": MemoryViolationType.BUFFER_OVERFLOW.value,
                "input_data": {
                    "overflow_type": overflow_type,
                    "buffer_size": buffer_size,
                    "buffer_type": random.choice(["char", "int", "struct", "array"]),
                    "overflow_offset": overflow_offset,
                    "operation": random.choice(["read", "write", "copy"]),
                    "data": "x" * (buffer_size + overflow_offset),
                    "access_pattern": random.choice(["sequential", "random", "indexed"])
                },
                "expected_behavior": "buffer overflow should be detected and prevented",
                "priority": 5,
                "category": "memory_safety",
                "preconditions": [
                    "memory allocation tracking enabled",
                    "bounds checking enabled",
                    "ASLR enabled"
                ],
                "test_steps": [
                    f"1. Allocate buffer of size {buffer_size}",
                    f"2. Perform {overflow_type} operation",
                    "3. Monitor for overflow detection",
                    "4. Check for memory corruption",
                    "5. Verify error handling"
                ],
                "assertions": [
                    "assert overflow is detected",
                    "assert no memory corruption",
                    "assert appropriate error returned",
                    "assert program remains stable"
                ],
                "memory_layout": {
                    "buffer_start": "0x1000",
                    "buffer_end": f"0x{1000 + buffer_size:04x}",
                    "overflow_target": f"0x{1000 + buffer_size + overflow_offset:04x}",
                    "guard_pages": True,
                    "stack_canaries": True
                },
                "detection_methods": [
                    "bounds_checking",
                    "stack_canaries",
                    "address_sanitizer",
                    "memory_tagging"
                ]
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_memory_leak_tests(self, component: str, count: int,
                                        parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate memory leak test cases"""
        test_cases = []
        
        leak_scenarios = [
            "forgotten_free",
            "exception_without_cleanup",
            "circular_reference",
            "cache_not_purged",
            "global_static_leak",
            "resource_handle_leak"
        ]
        
        for i in range(count):
            scenario = leak_scenarios[i % len(leak_scenarios)]
            allocation_size = random.choice([1024, 4096, 8192, 16384, 65536])
            allocation_count = random.randint(10, 100)
            
            test_case = {
                "id": f"memory_leak_{i}",
                "name": f"Memory Leak - {scenario}",
                "type": "memory_safety",
                "description": f"Test {component} memory leak scenario: {scenario}",
                "component": component,
                "violation_type": MemoryViolationType.MEMORY_LEAK.value,
                "input_data": {
                    "leak_scenario": scenario,
                    "allocation_size": allocation_size,
                    "allocation_count": allocation_count,
                    "total_allocated": allocation_size * allocation_count,
                    "expected_leak": random.randint(1, allocation_count // 2),
                    "leak_type": random.choice(["heap", "stack", "global"]),
                    "tracking_method": "reference_counting"
                },
                "expected_behavior": "memory leaks should be detected and measured",
                "priority": 4,
                "category": "memory_safety",
                "preconditions": [
                    "memory profiling enabled",
                    "leak detection instrumentation",
                    "reference counting"
                ],
                "test_steps": [
                    f"1. Allocate {allocation_count} blocks of {allocation_size} bytes each",
                    "2. Perform operations to create leak",
                    f"3. Verify {random.randint(1, allocation_count // 2)} blocks are leaked",
                    "4. Generate leak report",
                    "5. Verify leak detection accuracy"
                ],
                "assertions": [
                    "assert leaks are detected",
                    "assert leak size is measured correctly",
                    "assert leak location is identified",
                    "assert no false positives"
                ],
                "monitoring": {
                    "heap_profiling": True,
                    "allocation_tracking": True,
                    "leak_detection": True,
                    "reference_graph": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_double_free_tests(self, component: str, count: int,
                                        parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate double free test cases"""
        test_cases = []
        
        double_free_scenarios = [
            "same_pointer_free_twice",
            "freed_pointer_reuse",
            "pointer_aliasing",
            "function_return_free",
            "exception_during_free",
            "conditional_free_bug"
        ]
        
        for i in range(count):
            scenario = double_free_scenarios[i % len(double_free_scenarios)]
            memory_size = random.choice([16, 32, 64, 128, 256, 512])
            
            test_case = {
                "id": f"double_free_{i}",
                "name": f"Double Free - {scenario}",
                "type": "memory_safety",
                "description": f"Test {component} double free scenario: {scenario}",
                "component": component,
                "violation_type": MemoryViolationType.DOUBLE_FREE.value,
                "input_data": {
                    "scenario": scenario,
                    "memory_size": memory_size,
                    "allocation_count": random.randint(2, 5),
                    "free_count": random.randint(2, 5),
                    "pointer_handling": random.choice(["same_pointer", "copied_pointer", "aliased_pointer"]),
                    "timing": random.choice(["immediate", "delayed", "after_use"])
                },
                "expected_behavior": "double free should be detected and prevented",
                "priority": 5,
                "category": "memory_safety",
                "preconditions": [
                    "heap corruption detection",
                    "pointer tracking",
                    "free list validation"
                ],
                "test_steps": [
                    f"1. Allocate memory block of size {memory_size}",
                    "2. Free the memory block",
                    "3. Attempt to free same block again",
                    "4. Monitor for double free detection",
                    "5. Verify heap integrity"
                ],
                "assertions": [
                    "assert double free is detected",
                    "assert heap corruption is prevented",
                    "assert appropriate error handling",
                    "assert program terminates safely"
                ],
                "detection_mechanisms": {
                    "canary_values": True,
                    "free_list_sanitization": True,
                    "heap_consistency_checks": True,
                    "address_sanitizer": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_use_after_free_tests(self, component: str, count: int,
                                           parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate use-after-free test cases"""
        test_cases = []
        
        uaf_scenarios = [
            "read_after_free",
            "write_after_free",
            "pointer_dangling_after_free",
            "function_call_after_free",
            "pointer_arithmetic_after_free",
            "comparsion_after_free"
        ]
        
        for i in range(count):
            scenario = uaf_scenarios[i % len(uaf_scenarios)]
            block_size = random.choice([64, 128, 256, 512, 1024])
            use_delay = random.randint(1, 100)  # Operations before use
            
            test_case = {
                "id": f"use_after_free_{i}",
                "name": f"Use After Free - {scenario}",
                "type": "memory_safety",
                "description": f"Test {component} use-after-free scenario: {scenario}",
                "component": component,
                "violation_type": MemoryViolationType.USE_AFTER_FREE.value,
                "input_data": {
                    "scenario": scenario,
                    "block_size": block_size,
                    "operations_before_use": use_delay,
                    "use_type": random.choice(["read", "write", "call", "compare"]),
                    "pointer_state": "dangling",
                    "memory_reuse": random.choice([True, False])
                },
                "expected_behavior": "use-after-free should be detected and prevented",
                "priority": 5,
                "category": "memory_safety",
                "preconditions": [
                    "temporal memory safety checks",
                    "dangling pointer detection",
                    "memory reuse tracking"
                ],
                "test_steps": [
                    f"1. Allocate memory block of size {block_size}",
                    "2. Free the memory block",
                    f"3. Perform {use_delay} unrelated operations",
                    f"4. Attempt to {scenario.replace('_', ' ')}",
                    "5. Monitor for use-after-free detection"
                ],
                "assertions": [
                    "assert use-after-free is detected",
                    "assert no invalid memory access",
                    "assert program behavior is undefined or safe",
                    "assert detection is timely"
                ],
                "temporal_safety": {
                    "memory_tagging": True,
                    "bounds_temporal_checks": True,
                    "dangling_pointer_detection": True,
                    "memory_reuse_validation": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_pointer_safety_tests(self, component: str, count: int,
                                           parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate pointer safety test cases"""
        test_cases = []
        
        pointer_violations = [
            "null_pointer_dereference",
            "invalid_pointer_arithmetic",
            "wild_pointer_usage",
            "uninitialized_pointer",
            "cast_pointer_mismatch",
            "pointer_alignment_violation"
        ]
        
        for i in range(count):
            violation = pointer_violations[i % len(pointer_violations)]
            
            test_case = {
                "id": f"pointer_safety_{i}",
                "name": f"Pointer Safety - {violation}",
                "type": "memory_safety",
                "description": f"Test {component} pointer safety: {violation}",
                "component": component,
                "violation_type": MemoryViolationType.INVALID_POINTER.value,
                "input_data": {
                    "violation": violation,
                    "pointer_type": random.choice(["null", "uninitialized", "invalid", "misaligned"]),
                    "operation": random.choice(["dereference", "arithmetic", "comparison", "assignment"]),
                    "pointer_value": self._get_poisoned_pointer_value(violation),
                    "alignment_requirement": random.choice([4, 8, 16, 32])
                },
                "expected_behavior": "invalid pointer usage should be detected",
                "priority": 5,
                "category": "memory_safety",
                "preconditions": [
                    "pointer validation",
                    "null check enforcement",
                    "alignment checking"
                ],
                "test_steps": [
                    f"1. Create {violation.replace('_', ' ')} condition",
                    "2. Attempt pointer operation",
                    "3. Monitor for violation detection",
                    "4. Verify safety mechanisms",
                    "5. Check error handling"
                ],
                "assertions": [
                    "assert pointer violation is detected",
                    "assert no segmentation faults",
                    "assert appropriate error handling",
                    "assert memory safety maintained"
                ],
                "safety_mechanisms": {
                    "null_pointer_checks": True,
                    "pointer_validation": True,
                    "alignment_checks": True,
                    "control_flow_integrity": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_heap_corruption_tests(self, component: str, count: int,
                                            parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate heap corruption test cases"""
        test_cases = []
        
        corruption_types = [
            "heap_metadata_corruption",
            "chunk_size_inconsistency",
            "free_list_corruption",
            "heap_boundary_violation",
            "allignment_corruption",
            "chunk_coalescing_error"
        ]
        
        for i in range(count):
            corruption = corruption_types[i % len(corruption_types)]
            
            test_case = {
                "id": f"heap_corruption_{i}",
                "name": f"Heap Corruption - {corruption}",
                "type": "memory_safety",
                "description": f"Test {component} heap corruption: {corruption}",
                "component": component,
                "violation_type": MemoryViolationType.HEAP_CORRUPTION.value,
                "input_data": {
                    "corruption_type": corruption,
                    "corruption_location": random.choice(["metadata", "user_data", "boundary"]),
                    "allocation_count": random.randint(5, 20),
                    "corruption_pattern": random.choice(["bit_flip", "value_overwrite", "structure_misalignment"]),
                    "detection_method": "heap_consistency_check"
                },
                "expected_behavior": "heap corruption should be detected and reported",
                "priority": 5,
                "category": "memory_safety",
                "preconditions": [
                    "heap integrity checking",
                    "metadata protection",
                    "consistency validation"
                ],
                "test_steps": [
                    f"1. Perform {random.randint(5, 20)} allocations",
                    f"2. Introduce {corruption}",
                    "3. Perform memory operations",
                    "4. Trigger heap consistency check",
                    "5. Verify corruption detection"
                ],
                "assertions": [
                    "assert heap corruption is detected",
                    "assert corruption location is identified",
                    "assert heap integrity is maintained",
                    "assert safe recovery or termination"
                ],
                "integrity_checks": {
                    "metadata_validation": True,
                    "chunk_size_verification": True,
                    "boundary_protection": True,
                    "free_list_consistency": True
                }
            }
            
            test_cases.append(test_case)
        
        return test_cases
    
    def _get_poisoned_pointer_value(self, violation: str) -> str:
        """Get poisoned pointer values for testing"""
        poisoned_values = {
            "null_pointer_dereference": "0x0",
            "invalid_pointer_arithmetic": "0xDEADBEEF",
            "wild_pointer_usage": "0xFFFFFFFF",
            "uninitialized_pointer": "random",
            "cast_pointer_mismatch": "0x12345678",
            "pointer_alignment_violation": "0x1001"
        }
        return poisoned_values.get(violation, "0x0")
    
    async def generate_memory_safety_benchmark(self, component: str, iterations: int = 100,
                                             parameters: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Generate comprehensive memory safety benchmark"""
        all_violations = []
        
        for violation_type in MemoryViolationType:
            violation_tests = await self._generate_violation_specific_tests(
                component, violation_type, iterations // len(MemoryViolationType), parameters
            )
            all_violations.extend(violation_tests)
        
        return {
            "benchmark_name": f"{component}_memory_safety_benchmark",
            "component": component,
            "total_tests": len(all_violations),
            "violation_distribution": {
                vtype.value: sum(1 for tc in all_violations 
                               if tc.get("violation_type") == vtype.value)
                for vtype in MemoryViolationType
            },
            "test_cases": all_violations,
            "coverage_goals": {
                "buffer_overflow": "100%",
                "memory_leak": "100%",
                "use_after_free": "100%",
                "pointer_safety": "100%"
            }
        }
    
    async def _generate_violation_specific_tests(self, component: str, violation_type: MemoryViolationType,
                                               count: int, parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate tests for specific violation type"""
        if violation_type == MemoryViolationType.BUFFER_OVERFLOW:
            return await self._generate_buffer_overflow_tests(component, count, parameters)
        elif violation_type == MemoryViolationType.MEMORY_LEAK:
            return await self._generate_memory_leak_tests(component, count, parameters)
        elif violation_type == MemoryViolationType.DOUBLE_FREE:
            return await self._generate_double_free_tests(component, count, parameters)
        elif violation_type == MemoryViolationType.USE_AFTER_FREE:
            return await self._generate_use_after_free_tests(component, count, parameters)
        elif violation_type == MemoryViolationType.INVALID_POINTER:
            return await self._generate_pointer_safety_tests(component, count, parameters)
        elif violation_type == MemoryViolationType.HEAP_CORRUPTION:
            return await self._generate_heap_corruption_tests(component, count, parameters)
        else:
            return await self._generate_generic_violation_tests(component, violation_type, count, parameters)
    
    async def _generate_generic_violation_tests(self, component: str, violation_type: MemoryViolationType,
                                              count: int, parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate generic violation tests"""
        test_cases = []
        
        for i in range(count):
            test_case = {
                "id": f"generic_violation_{violation_type.value}_{i}",
                "name": f"Generic {violation_type.value} Test {i}",
                "type": "memory_safety",
                "description": f"Test {component} for {violation_type.value} vulnerabilities",
                "component": component,
                "violation_type": violation_type.value,
                "input_data": {
                    "violation_type": violation_type.value,
                    "severity": random.choice(["low", "medium", "high", "critical"]),
                    "detection_difficulty": random.choice(["easy", "medium", "hard"])
                },
                "expected_behavior": f"{violation_type.value} should be detected",
                "priority": 4,
                "category": "memory_safety"
            }
            test_cases.append(test_case)
        
        return test_cases