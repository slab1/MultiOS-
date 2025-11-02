"""
State Space Exploration Generator for MultiOS
Generates test cases for concurrent operations and state space exploration
"""

import random
import json
import logging
from typing import Dict, List, Any, Optional, Tuple, Union, Set
from dataclasses import dataclass
from enum import Enum
import itertools
from collections import defaultdict, deque
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

class StateTransition(Enum):
    """Types of state transitions"""
    CREATE = "create"
    READ = "read"
    UPDATE = "update"
    DELETE = "delete"
    LOCK = "lock"
    UNLOCK = "unlock"
    FORK = "fork"
    JOIN = "join"
    EXECUTE = "execute"
    SIGNAL = "signal"

class ConcurrencyPattern(Enum):
    """Concurrency patterns to explore"""
    READER_WRITER = "reader_writer"
    PRODUCER_CONSUMER = "producer_consumer"
    BARRIER_SYNCHRONIZATION = "barrier_sync"
    MUTEX_DEADLOCK = "mutex_deadlock"
    RACE_CONDITION = "race_condition"
    LIVELOCK = "livelock"
    RESOURCE_ALLOCATION = "resource_allocation"
    TIMEOUT_DEADLOCK = "timeout_deadlock"

@dataclass
class State:
    """System state representation"""
    id: str
    resources: Dict[str, Any]
    processes: Dict[str, Any]
    locks: Dict[str, bool]
    timestamps: Dict[str, float]
    
@dataclass
class Transition:
    """State transition representation"""
    from_state: str
    to_state: str
    operation: StateTransition
    resources: List[str]
    thread_id: str
    timestamp: float

class StateSpaceGenerator:
    """Generates state space exploration test scenarios"""
    
    def __init__(self, seed: Optional[int] = None):
        self.logger = logging.getLogger(__name__)
        if seed:
            random.seed(seed)
        
        self.concurrency_patterns = self._initialize_patterns()
        self.state_graphs: Dict[str, Dict[str, Any]] = {}
        
    def _initialize_patterns(self) -> Dict[str, ConcurrencyPattern]:
        """Initialize concurrency patterns"""
        return {
            "filesystem": [
                ConcurrencyPattern.READER_WRITER,
                ConcurrencyPattern.RACE_CONDITION,
                ConcurrencyPattern.MUTEX_DEADLOCK,
                ConcurrencyPattern.RESOURCE_ALLOCATION
            ],
            "memory": [
                ConcurrencyPattern.PRODUCER_CONSUMER,
                ConcurrencyPattern.MUTEX_DEADLOCK,
                ConcurrencyPattern.LIVELOCK,
                ConcurrencyPattern.RESOURCE_ALLOCATION
            ],
            "network": [
                ConcurrencyPattern.READER_WRITER,
                ConcurrencyPattern.PRODUCER_CONSUMER,
                ConcurrencyPattern.RESOURCE_ALLOCATION
            ],
            "process": [
                ConcurrencyPattern.BARRIER_SYNCHRONIZATION,
                ConcurrencyPattern.MUTEX_DEADLOCK,
                ConcurrencyPattern.TIMEOUT_DEADLOCK,
                ConcurrencyPattern.RESOURCE_ALLOCATION
            ]
        }
    
    async def generate_state_tests(self, component: str, iterations: int = 1000,
                                 parameters: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Generate state space exploration test suite"""
        test_cases = []
        parameters = parameters or {}
        
        # Get relevant patterns for component
        patterns = self.concurrency_patterns.get(component, list(ConcurrencyPattern))
        
        # Generate tests for each pattern
        tests_per_pattern = max(1, iterations // len(patterns))
        
        for pattern in patterns:
            pattern_tests = await self._generate_pattern_tests(
                component, pattern, tests_per_pattern, parameters
            )
            test_cases.extend(pattern_tests)
        
        return test_cases
    
    async def _generate_pattern_tests(self, component: str, pattern: ConcurrencyPattern,
                                    count: int, parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate test cases for specific concurrency pattern"""
        test_cases = []
        
        for i in range(count):
            if pattern == ConcurrencyPattern.READER_WRITER:
                test_case = await self._generate_reader_writer_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.PRODUCER_CONSUMER:
                test_case = await self._generate_producer_consumer_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.BARRIER_SYNCHRONIZATION:
                test_case = await self._generate_barrier_sync_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.MUTEX_DEADLOCK:
                test_case = await self._generate_mutex_deadlock_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.RACE_CONDITION:
                test_case = await self._generate_race_condition_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.LIVELOCK:
                test_case = await self._generate_livelock_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.RESOURCE_ALLOCATION:
                test_case = await self._generate_resource_allocation_test(component, i, parameters)
            elif pattern == ConcurrencyPattern.TIMEOUT_DEADLOCK:
                test_case = await self._generate_timeout_deadlock_test(component, i, parameters)
            else:
                test_case = await self._generate_generic_concurrency_test(component, i, pattern, parameters)
            
            test_cases.append(test_case)
        
        return test_cases
    
    async def _generate_reader_writer_test(self, component: str, test_id: int,
                                         parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate reader-writer concurrency test"""
        thread_count = parameters.get("thread_count", random.randint(3, 10))
        read_ratio = parameters.get("read_ratio", 0.7)  # 70% reads, 30% writes
        
        test_case = {
            "id": f"reader_writer_{test_id}",
            "name": f"Reader-Writer Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} with reader-writer pattern",
            "component": component,
            "pattern": ConcurrencyPattern.READER_WRITER.value,
            "input_data": {
                "thread_count": thread_count,
                "read_ratio": read_ratio,
                "shared_resource": f"{component}_resource_{test_id}",
                "operations": self._generate_reader_writer_operations(thread_count, read_ratio),
                "synchronization": "rwlock",
                "max_iterations": 100,
                "timeout": 30
            },
            "expected_behavior": "concurrent access should be safe and deadlock-free",
            "priority": 5,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {thread_count} threads (mix of readers and writers)",
                "2. Launch threads concurrently",
                "3. Monitor for deadlocks and race conditions",
                "4. Verify data consistency",
                "5. Check performance metrics"
            ],
            "assertions": [
                "assert no deadlocks occur",
                "assert no data corruption",
                "assert readers can access simultaneously",
                "assert writers get exclusive access",
                "assert all operations complete"
            ],
            "state_exploration": {
                "initial_state": {
                    "resource_locked": False,
                    "active_readers": 0,
                    "active_writers": 0,
                    "waiting_readers": 0,
                    "waiting_writers": 0
                },
                "state_transitions": self._generate_reader_writer_transitions(),
                "critical_states": [
                    "writer_waiting_for_readers",
                    "multiple_readers_active",
                    "writer_exclusive_access",
                    "all_threads_blocked"
                ]
            }
        }
        
        return test_case
    
    async def _generate_producer_consumer_test(self, component: str, test_id: int,
                                             parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate producer-consumer concurrency test"""
        producer_count = parameters.get("producer_count", random.randint(1, 5))
        consumer_count = parameters.get("consumer_count", random.randint(1, 5))
        buffer_size = parameters.get("buffer_size", random.randint(1, 10))
        
        test_case = {
            "id": f"producer_consumer_{test_id}",
            "name": f"Producer-Consumer Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} with producer-consumer pattern",
            "component": component,
            "pattern": ConcurrencyPattern.PRODUCER_CONSUMER.value,
            "input_data": {
                "producers": producer_count,
                "consumers": consumer_count,
                "buffer_size": buffer_size,
                "shared_buffer": f"{component}_buffer_{test_id}",
                "operations": self._generate_producer_consumer_operations(
                    producer_count, consumer_count
                ),
                "synchronization": ["mutex", "semaphore"],
                "items_to_produce": 50,
                "timeout": 60
            },
            "expected_behavior": "producers and consumers should coordinate without deadlocks",
            "priority": 5,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {producer_count} producer threads",
                f"2. Create {consumer_count} consumer threads",
                "3. Initialize bounded buffer",
                "4. Start all threads simultaneously",
                "5. Monitor buffer operations",
                "6. Verify all items are produced and consumed"
            ],
            "assertions": [
                "assert buffer never overflows",
                "assert buffer never underflows",
                "assert all items are eventually consumed",
                "assert no deadlocks occur",
                "assert thread synchronization works correctly"
            ],
            "state_exploration": {
                "initial_state": {
                    "buffer": [],
                    "buffer_count": 0,
                    "producers_active": producer_count,
                    "consumers_active": consumer_count
                },
                "state_transitions": self._generate_producer_consumer_transitions(),
                "critical_states": [
                    "buffer_full_producer_waiting",
                    "buffer_empty_consumer_waiting",
                    "all_producers_finished",
                    "all_consumers_finished"
                ]
            }
        }
        
        return test_case
    
    async def _generate_barrier_sync_test(self, component: str, test_id: int,
                                        parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate barrier synchronization test"""
        thread_count = parameters.get("thread_count", random.randint(4, 20))
        barrier_count = parameters.get("barrier_count", random.randint(2, 5))
        
        test_case = {
            "id": f"barrier_sync_{test_id}",
            "name": f"Barrier Synchronization Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} with barrier synchronization",
            "component": component,
            "pattern": ConcurrencyPattern.BARRIER_SYNCHRONIZATION.value,
            "input_data": {
                "thread_count": thread_count,
                "barrier_count": barrier_count,
                "barriers": [f"barrier_{i}" for i in range(barrier_count)],
                "work_per_phase": random.randint(10, 100),
                "synchronization": "barrier",
                "timeout": 45
            },
            "expected_behavior": "all threads should synchronize at each barrier",
            "priority": 4,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {thread_count} worker threads",
                f"2. Initialize {barrier_count} barriers",
                "3. Each thread performs work in phases",
                "4. Threads synchronize at each barrier",
                "5. Verify all threads reach barriers together"
            ],
            "assertions": [
                "assert no thread proceeds past barrier early",
                "assert all threads complete each phase",
                "assert no deadlock at barriers",
                "assert correct thread count at each barrier"
            ],
            "state_exploration": {
                "initial_state": {
                    "threads_at_barrier": [0] * barrier_count,
                    "phases_completed": 0,
                    "threads_active": thread_count
                },
                "state_transitions": self._generate_barrier_transitions(),
                "critical_states": [
                    "partial_barrier_reach",
                    "all_threads_at_barrier",
                    "phase_transition"
                ]
            }
        }
        
        return test_case
    
    async def _generate_mutex_deadlock_test(self, component: str, test_id: int,
                                          parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate mutex deadlock scenario test"""
        mutex_count = parameters.get("mutex_count", random.randint(2, 6))
        thread_count = parameters.get("thread_count", mutex_count * 2)
        
        test_case = {
            "id": f"mutex_deadlock_{test_id}",
            "name": f"Mutex Deadlock Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} for mutex deadlock scenarios",
            "component": component,
            "pattern": ConcurrencyPattern.MUTEX_DEADLOCK.value,
            "input_data": {
                "mutexes": [f"mutex_{i}" for i in range(mutex_count)],
                "threads": thread_count,
                "deadlock_potential": "high",
                "acquisition_order": "random",  # or "consistent"
                "timeout": 30,
                "resource_allocation_graph": self._generate_resource_graph(mutex_count)
            },
            "expected_behavior": "system should detect and handle deadlocks",
            "priority": 5,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {mutex_count} mutexes",
                f"2. Create {thread_count} threads",
                "3. Configure random acquisition order",
                "4. Execute concurrent mutex operations",
                "5. Monitor for deadlock detection",
                "6. Verify deadlock recovery"
            ],
            "assertions": [
                "assert deadlock detection works",
                "assert system recovers from deadlock",
                "assert no thread starvation",
                "assert resource allocation graph is safe",
                "assert timeout handling works"
            ],
            "state_exploration": {
                "initial_state": {
                    "mutex_holders": {},
                    "waiting_threads": [],
                    "resource_graph": {},
                    "deadlock_cycle": None
                },
                "state_transitions": self._generate_deadlock_transitions(),
                "critical_states": [
                    "potential_deadlock",
                    "actual_deadlock",
                    "deadlock_recovery",
                    "safe_state"
                ]
            }
        }
        
        return test_case
    
    async def _generate_race_condition_test(self, component: str, test_id: int,
                                          parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate race condition test"""
        operation_count = parameters.get("operation_count", random.randint(5, 20))
        thread_count = parameters.get("thread_count", random.randint(2, 10))
        
        test_case = {
            "id": f"race_condition_{test_id}",
            "name": f"Race Condition Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} for race conditions",
            "component": component,
            "pattern": ConcurrencyPattern.RACE_CONDITION.value,
            "input_data": {
                "shared_resource": f"{component}_race_resource_{test_id}",
                "operations": operation_count,
                "threads": thread_count,
                "operation_types": ["read", "write", "modify"],
                "timing_variance": random.uniform(0.001, 0.1),
                "synchronization": "none",  # Intentional lack of sync
                "expected_race_conditions": True,
                "timeout": 30
            },
            "expected_behavior": "race conditions should be detectable and reproducible",
            "priority": 4,
            "category": "concurrency",
            "test_steps": [
                f"1. Create shared resource without synchronization",
                f"2. Launch {thread_count} threads concurrently",
                "3. Perform competing operations rapidly",
                "4. Monitor for race conditions",
                "5. Verify data corruption",
                "6. Compare with synchronized version"
            ],
            "assertions": [
                "assert race conditions are reproducible",
                "assert data corruption is detected",
                "assert timing issues are visible",
                "assert race conditions are fixable with sync"
            ],
            "state_exploration": {
                "initial_state": {
                    "resource_value": 0,
                    "operation_sequence": [],
                    "race_window": 0
                },
                "state_transitions": self._generate_race_transitions(),
                "critical_states": [
                    "concurrent_modification",
                    "inconsistent_read",
                    "lost_update",
                    "torn_write"
                ]
            }
        }
        
        return test_case
    
    async def _generate_livelock_test(self, component: str, test_id: int,
                                    parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate livelock scenario test"""
        thread_count = parameters.get("thread_count", random.randint(2, 6))
        
        test_case = {
            "id": f"livelock_{test_id}",
            "name": f"Livelock Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} for livelock scenarios",
            "component": component,
            "pattern": ConcurrencyPattern.LIVELOCK.value,
            "input_data": {
                "threads": thread_count,
                "shared_resource": f"{component}_livelock_resource_{test_id}",
                "contention_strategy": "aggressive_retry",
                "retry_delay": 0.001,
                "max_retries": 100,
                "livelock_simulation": True,
                "timeout": 30
            },
            "expected_behavior": "system should detect and resolve livelock",
            "priority": 4,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {thread_count} competing threads",
                "2. Configure aggressive retry behavior",
                "3. Execute with high contention",
                "4. Monitor for livelock patterns",
                "5. Verify detection and recovery",
                "6. Test backoff strategies"
            ],
            "assertions": [
                "assert livelock is detected",
                "assert system recovers from livelock",
                "assert backoff strategies work",
                "assert progress is made eventually"
            ],
            "state_exploration": {
                "initial_state": {
                    "resource_state": "available",
                    "active_threads": thread_count,
                    "retry_count": 0,
                    "backoff_factor": 1
                },
                "state_transitions": self._generate_livelock_transitions(),
                "critical_states": [
                    "high_contention",
                    "livelock_detected",
                    "backoff_applied",
                    "progress_made"
                ]
            }
        }
        
        return test_case
    
    async def _generate_resource_allocation_test(self, component: str, test_id: int,
                                               parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate resource allocation test"""
        resource_count = parameters.get("resource_count", random.randint(3, 8))
        thread_count = parameters.get("thread_count", random.randint(4, 12))
        
        test_case = {
            "id": f"resource_allocation_{test_id}",
            "name": f"Resource Allocation Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} resource allocation patterns",
            "component": component,
            "pattern": ConcurrencyPattern.RESOURCE_ALLOCATION.value,
            "input_data": {
                "resources": [f"resource_{i}" for i in range(resource_count)],
                "threads": thread_count,
                "allocation_strategy": random.choice(["random", "greedy", "fair"]),
                "resource_types": ["exclusive", "shared"],
                "allocation_requests": self._generate_allocation_requests(thread_count),
                "deadline_policy": random.choice(["fifo", "priority", "earliest_deadline"]),
                "timeout": 45
            },
            "expected_behavior": "resources should be allocated efficiently without deadlocks",
            "priority": 4,
            "category": "concurrency",
            "test_steps": [
                f"1. Initialize {resource_count} resources",
                f"2. Create {thread_count} requesting threads",
                "3. Execute allocation requests",
                "4. Monitor allocation patterns",
                "5. Check for deadlocks",
                "6. Verify fair allocation"
            ],
            "assertions": [
                "assert no deadlocks in resource allocation",
                "assert all requests are eventually satisfied",
                "assert allocation strategy is followed",
                "assert no resource starvation",
                "assert allocation graph is safe"
            ],
            "state_exploration": {
                "initial_state": {
                    "allocated_resources": {},
                    "waiting_requests": [],
                    "resource_availability": {f"resource_{i}": True for i in range(resource_count)}
                },
                "state_transitions": self._generate_allocation_transitions(),
                "critical_states": [
                    "resource_requested",
                    "resource_allocated",
                    "resource_released",
                    "allocation_deadlock"
                ]
            }
        }
        
        return test_case
    
    async def _generate_timeout_deadlock_test(self, component: str, test_id: int,
                                            parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate timeout-based deadlock test"""
        timeout_value = parameters.get("timeout", random.randint(5, 30))
        thread_count = parameters.get("thread_count", random.randint(2, 6))
        
        test_case = {
            "id": f"timeout_deadlock_{test_id}",
            "name": f"Timeout Deadlock Test {test_id}",
            "type": "state_space_exploration",
            "description": f"Test {component} with timeout-based deadlock handling",
            "component": component,
            "pattern": ConcurrencyPattern.TIMEOUT_DEADLOCK.value,
            "input_data": {
                "threads": thread_count,
                "shared_resources": [f"resource_{i}" for i in range(thread_count)],
                "timeout_value": timeout_value,
                "acquisition_order": "circular",  # Classic deadlock scenario
                "force_deadlock": True,
                "timeout_handling": "abort_request",
                "timeout_granularity": 1  # seconds
            },
            "expected_behavior": "system should detect and timeout deadlocks",
            "priority": 5,
            "category": "concurrency",
            "test_steps": [
                f"1. Create {thread_count} threads with circular resource dependencies",
                f"2. Set timeout to {timeout_value} seconds",
                "3. Start all threads simultaneously",
                "4. Force circular wait condition",
                "5. Monitor for timeout detection",
                "6. Verify recovery from timeout"
            ],
            "assertions": [
                "assert deadlock is detected within timeout",
                "assert timed-out requests are aborted",
                "assert system recovers after timeout",
                "assert no indefinite blocking",
                "assert resource cleanup happens"
            ],
            "state_exploration": {
                "initial_state": {
                    "resource_locks": {},
                    "waiting_threads": [],
                    "deadlock_detected": False,
                    "timeout_triggered": False
                },
                "state_transitions": self._generate_timeout_transitions(),
                "critical_states": [
                    "circular_wait_formed",
                    "timeout_expired",
                    "deadlock_recovery",
                    "normal_operation"
                ]
            }
        }
        
        return test_case
    
    async def _generate_generic_concurrency_test(self, component: str, test_id: int,
                                               pattern: ConcurrencyPattern,
                                               parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Generate generic concurrency test"""
        return {
            "id": f"generic_concurrency_{test_id}",
            "name": f"Generic Concurrency {test_id} - {pattern.value}",
            "type": "state_space_exploration",
            "description": f"Generic concurrency test for {component}",
            "component": component,
            "pattern": pattern.value,
            "input_data": {
                "thread_count": random.randint(2, 8),
                "operation_count": random.randint(10, 50),
                "timeout": 30
            },
            "expected_behavior": "concurrent operations should complete safely",
            "priority": 3,
            "category": "concurrency"
        }
    
    # Helper methods for generating test data
    def _generate_reader_writer_operations(self, thread_count: int, read_ratio: float) -> List[Dict[str, Any]]:
        """Generate reader-writer operation sequence"""
        operations = []
        for i in range(thread_count):
            is_reader = random.random() < read_ratio
            operations.append({
                "thread_id": f"thread_{i}",
                "type": "read" if is_reader else "write",
                "resource": "shared_resource",
                "duration": random.uniform(0.01, 0.1)
            })
        return operations
    
    def _generate_producer_consumer_operations(self, producer_count: int, consumer_count: int) -> List[Dict[str, Any]]:
        """Generate producer-consumer operation sequence"""
        operations = []
        
        # Producers
        for i in range(producer_count):
            operations.append({
                "type": "produce",
                "thread_id": f"producer_{i}",
                "item": f"item_{i}",
                "priority": random.randint(1, 10)
            })
        
        # Consumers
        for i in range(consumer_count):
            operations.append({
                "type": "consume",
                "thread_id": f"consumer_{i}",
                "expected_item": f"item_{i}",
                "priority": random.randint(1, 10)
            })
        
        random.shuffle(operations)
        return operations
    
    def _generate_resource_graph(self, mutex_count: int) -> Dict[str, List[str]]:
        """Generate resource allocation graph"""
        graph = {}
        for i in range(mutex_count):
            graph[f"mutex_{i}"] = [f"mutex_{(i+1) % mutex_count}"]
        return graph
    
    def _generate_allocation_requests(self, thread_count: int) -> List[Dict[str, Any]]:
        """Generate resource allocation requests"""
        requests = []
        for i in range(thread_count):
            requests.append({
                "thread_id": f"thread_{i}",
                "resources": [f"resource_{j}" for j in range(i % 3 + 1)],
                "priority": random.randint(1, 10),
                "deadline": time.time() + random.randint(10, 60)
            })
        return requests
    
    # State transition generators
    def _generate_reader_writer_transitions(self) -> List[Dict[str, Any]]:
        """Generate reader-writer state transitions"""
        return [
            {"from": "idle", "to": "reader_waiting", "operation": "read_request"},
            {"from": "idle", "to": "writer_waiting", "operation": "write_request"},
            {"from": "reader_waiting", "to": "reading", "operation": "read_granted"},
            {"from": "writer_waiting", "to": "writing", "operation": "write_granted"},
            {"from": "reading", "to": "idle", "operation": "read_released"},
            {"from": "writing", "to": "idle", "operation": "write_released"}
        ]
    
    def _generate_producer_consumer_transitions(self) -> List[Dict[str, Any]]:
        """Generate producer-consumer state transitions"""
        return [
            {"from": "buffer_empty", "to": "buffer_has_items", "operation": "produce"},
            {"from": "buffer_has_items", "to": "buffer_empty", "operation": "consume"},
            {"from": "buffer_full", "to": "buffer_has_space", "operation": "consume"},
            {"from": "buffer_has_space", "to": "buffer_full", "operation": "produce"}
        ]
    
    def _generate_barrier_transitions(self) -> List[Dict[str, Any]]:
        """Generate barrier synchronization transitions"""
        return [
            {"from": "work_phase", "to": "barrier_wait", "operation": "finish_work"},
            {"from": "barrier_wait", "to": "work_phase", "operation": "all_threads_arrive"},
            {"from": "work_phase", "to": "completed", "operation": "all_phases_done"}
        ]
    
    def _generate_deadlock_transitions(self) -> List[Dict[str, Any]]:
        """Generate deadlock state transitions"""
        return [
            {"from": "safe", "to": "potential_deadlock", "operation": "acquire_mutex"},
            {"from": "potential_deadlock", "to": "deadlock", "operation": "circular_wait"},
            {"from": "deadlock", "to": "recovery", "operation": "timeout"},
            {"from": "recovery", "to": "safe", "operation": "cleanup"}
        ]
    
    def _generate_race_transitions(self) -> List[Dict[str, Any]]:
        """Generate race condition transitions"""
        return [
            {"from": "idle", "to": "reading", "operation": "start_read"},
            {"from": "idle", "to": "writing", "operation": "start_write"},
            {"from": "reading", "to": "modified", "operation": "concurrent_write"},
            {"from": "writing", "to": "consistent", "operation": "complete_write"},
            {"from": "modified", "to": "consistent", "operation": "read_after_write"}
        ]
    
    def _generate_livelock_transitions(self) -> List[Dict[str, Any]]:
        """Generate livelock state transitions"""
        return [
            {"from": "normal", "to": "contention", "operation": "increase_contention"},
            {"from": "contention", "to": "livelock", "operation": "continuous_retry"},
            {"from": "livelock", "to": "backoff", "operation": "detect_livelock"},
            {"from": "backoff", "to": "normal", "operation": "apply_backoff"}
        ]
    
    def _generate_allocation_transitions(self) -> List[Dict[str, Any]]:
        """Generate resource allocation transitions"""
        return [
            {"from": "available", "to": "requested", "operation": "request_resource"},
            {"from": "requested", "to": "allocated", "operation": "grant_resource"},
            {"from": "allocated", "to": "available", "operation": "release_resource"},
            {"from": "requested", "to": "starvation", "operation": "timeout_wait"}
        ]
    
    def _generate_timeout_transitions(self) -> List[Dict[str, Any]]:
        """Generate timeout-based deadlock transitions"""
        return [
            {"from": "normal", "to": "waiting", "operation": "acquire_with_timeout"},
            {"from": "waiting", "to": "deadlock", "operation": "timeout_expired"},
            {"from": "deadlock", "to": "recovery", "operation": "abort_request"},
            {"from": "recovery", "to": "normal", "operation": "cleanup"}
        ]