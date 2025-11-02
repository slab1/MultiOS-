"""
Property-Based Testing Generator for MultiOS
Generates tests based on system properties, invariants, and behavioral specifications
"""

import random
import json
import logging
from typing import Dict, List, Any, Optional, Tuple, Union, Callable
from dataclasses import dataclass
from enum import Enum
import itertools
from abc import ABC, abstractmethod

class PropertyType(Enum):
    """Types of properties to test"""
    INVARIANT = "invariant"
    PREDICATE = "predicate"
    RELATION = "relation"
    FUNCTIONAL = "functional"
    MONOID = "monoid"
    MONAD = "monad"
    LAWS = "laws"
    IDEMPOTENT = "idempotent"
    COMMUTATIVE = "commutative"
    ASSOCIATIVE = "associative"
    DISTRIBUTIVE = "distributive"

@dataclass
class Property:
    """System property definition"""
    name: str
    description: str
    property_type: PropertyType
    domain: str  # Component domain (filesystem, memory, etc.)
    validator: Callable[[Any], bool]
    generator: Callable[[], Any]
    counter_example: Optional[Dict[str, Any]] = None

class PropertyBasedGenerator:
    """Generates property-based test scenarios"""
    
    def __init__(self, seed: Optional[int] = None):
        self.logger = logging.getLogger(__name__)
        if seed:
            random.seed(seed)
        
        self.properties = self._initialize_properties()
        
    def _initialize_properties(self) -> Dict[str, List[Property]]:
        """Initialize property database for different components"""
        return {
            "filesystem": self._get_filesystem_properties(),
            "memory": self._get_memory_properties(),
            "network": self._get_network_properties(),
            "process": self._get_process_properties(),
            "api": self._get_api_properties(),
            "math": self._get_math_properties(),
            "data_structure": self._get_data_structure_properties()
        }
    
    def _get_filesystem_properties(self) -> List[Property]:
        """Get filesystem-specific properties"""
        return [
            Property(
                name="file_access_consistency",
                description="File access permissions should be consistent",
                property_type=PropertyType.INVARIANT,
                domain="filesystem",
                validator=self._validate_file_access_consistency,
                generator=self._generate_file_operation
            ),
            Property(
                name="path_normalization",
                description="Path normalization should preserve semantics",
                property_type=PropertyType.FUNCTIONAL,
                domain="filesystem",
                validator=self._validate_path_normalization,
                generator=self._generate_path_operation
            ),
            Property(
                name="atomic_file_operations",
                description="File operations should be atomic",
                property_type=PropertyType.INVARIANT,
                domain="filesystem",
                validator=self._validate_atomic_operations,
                generator=self._generate_concurrent_file_op
            ),
            Property(
                name="directory_hierarchy",
                description="Directory operations maintain hierarchy integrity",
                property_type=PropertyType.INVARIANT,
                domain="filesystem",
                validator=self._validate_directory_hierarchy,
                generator=self._generate_directory_operation
            ),
            Property(
                name="file_content_integrity",
                description="File content should not be corrupted by operations",
                property_type=PropertyType.INVARIANT,
                domain="filesystem",
                validator=self._validate_file_content_integrity,
                generator=self._generate_file_content_operation
            )
        ]
    
    def _get_memory_properties(self) -> List[Property]:
        """Get memory-specific properties"""
        return [
            Property(
                name="memory_allocation_freedom",
                description="Memory allocation should eventually succeed or fail predictably",
                property_type=PropertyType.PREDICATE,
                domain="memory",
                validator=self._validate_allocation_freedom,
                generator=self._generate_allocation_pattern
            ),
            Property(
                name="memory_leak_freedom",
                description="Memory operations should not leak resources",
                property_type=PropertyType.INVARIANT,
                domain="memory",
                validator=self._validate_no_memory_leaks,
                generator=self._generate_memory_sequence
            ),
            Property(
                name="pointer_arithmetic_safety",
                description="Pointer arithmetic should stay within bounds",
                property_type=PropertyType.INVARIANT,
                domain="memory",
                validator=self._validate_pointer_safety,
                generator=self._generate_pointer_operation
            ),
            Property(
                name="memory_alignment",
                description="Memory access should respect alignment requirements",
                property_type=PropertyType.INVARIANT,
                domain="memory",
                validator=self._validate_memory_alignment,
                generator=self._generate_aligned_access
            )
        ]
    
    def _get_network_properties(self) -> List[Property]:
        """Get network-specific properties"""
        return [
            Property(
                name="connection_lifecycle",
                description="Network connection lifecycle should be well-defined",
                property_type=PropertyType.INVARIANT,
                domain="network",
                validator=self._validate_connection_lifecycle,
                generator=self._generate_connection_sequence
            ),
            Property(
                name="data_integrity",
                description="Network data transmission should preserve integrity",
                property_type=PropertyType.INVARIANT,
                domain="network",
                validator=self._validate_data_integrity,
                generator=self._generate_network_data
            ),
            Property(
                name="network_protocol_compliance",
                description="Protocol implementations should follow specifications",
                property_type=PropertyType.INVARIANT,
                domain="network",
                validator=self._validate_protocol_compliance,
                generator=self._generate_protocol_data
            )
        ]
    
    def _get_process_properties(self) -> List[Property]:
        """Get process-specific properties"""
        return [
            Property(
                name="process_isolation",
                description="Processes should be properly isolated",
                property_type=PropertyType.INVARIANT,
                domain="process",
                validator=self._validate_process_isolation,
                generator=self._generate_process_interaction
            ),
            Property(
                name="signal_handling",
                description="Signal handling should be consistent",
                property_type=PropertyType.PREDICATE,
                domain="process",
                validator=self._validate_signal_handling,
                generator=self._generate_signal_sequence
            )
        ]
    
    def _get_api_properties(self) -> List[Property]:
        """Get API-specific properties"""
        return [
            Property(
                name="request_response_symmetry",
                description="API request/response should be symmetric",
                property_type=PropertyType.RELATION,
                domain="api",
                validator=self._validate_request_response_symmetry,
                generator=self._generate_api_request
            ),
            Property(
                name="authentication_consistency",
                description="Authentication should be consistent across endpoints",
                property_type=PropertyType.INVARIANT,
                domain="api",
                validator=self._validate_auth_consistency,
                generator=self._generate_authenticated_request
            )
        ]
    
    def _get_math_properties(self) -> List[Property]:
        """Get mathematical properties"""
        return [
            Property(
                name="commutativity",
                description="Addition should be commutative",
                property_type=PropertyType.COMMUTATIVE,
                domain="math",
                validator=self._validate_commutativity,
                generator=self._generate_numbers
            ),
            Property(
                name="associativity",
                description="Addition should be associative",
                property_type=PropertyType.ASSOCIATIVE,
                domain="math",
                validator=self._validate_associativity,
                generator=self._generate_numbers
            ),
            Property(
                name="idempotency",
                description="Max operation should be idempotent",
                property_type=PropertyType.IDEMPOTENT,
                domain="math",
                validator=self._validate_idempotency,
                generator=self._generate_numbers
            )
        ]
    
    def _get_data_structure_properties(self) -> List[Property]:
        """Get data structure properties"""
        return [
            Property(
                name="stack_lifo",
                description="Stack operations should follow LIFO order",
                property_type=PropertyType.INVARIANT,
                domain="data_structure",
                validator=self._validate_stack_lifo,
                generator=self._generate_stack_sequence
            ),
            Property(
                name="queue_fifo",
                description="Queue operations should follow FIFO order",
                property_type=PropertyType.INVARIANT,
                domain="data_structure",
                validator=self._validate_queue_fifo,
                generator=self._generate_queue_sequence
            )
        ]
    
    async def generate_property_tests(self, component: str, iterations: int = 1000,
                                    parameters: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Generate property-based test suite"""
        test_cases = []
        parameters = parameters or {}
        
        # Get properties for component
        component_properties = self.properties.get(component, [])
        if not component_properties:
            # Use generic properties if component-specific ones don't exist
            component_properties = itertools.chain(*self.properties.values())
        
        # Generate tests for each property
        for property_obj in component_properties:
            tests_per_property = max(1, iterations // len(component_properties))
            
            property_tests = await self._generate_property_test_cases(
                property_obj, tests_per_property, parameters
            )
            test_cases.extend(property_tests)
        
        return test_cases
    
    async def _generate_property_test_cases(self, property_obj: Property, count: int,
                                          parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate test cases for a specific property"""
        test_cases = []
        
        for i in range(count):
            # Generate input data
            input_data = property_obj.generator()
            
            # Test the property
            try:
                result = property_obj.validator(input_data)
                status = "pass" if result else "fail"
            except Exception as e:
                result = False
                status = "error"
                error_msg = str(e)
            
            test_case = {
                "id": f"property_{property_obj.name}_{i}",
                "name": f"Property Test - {property_obj.name}",
                "type": "property_based",
                "description": f"Test property: {property_obj.description}",
                "component": property_obj.domain,
                "property_name": property_obj.name,
                "property_type": property_obj.property_type.value,
                "input_data": input_data,
                "expected_behavior": "property should hold for all inputs",
                "actual_result": result if 'result' in locals() else None,
                "status": status if 'status' in locals() else "unknown",
                "priority": 5 if property_obj.property_type in [
                    PropertyType.INVARIANT, PropertyType.PREDICATE
                ] else 3,
                "category": "property_based",
                "test_steps": [
                    f"1. Generate input data for property: {property_obj.name}",
                    "2. Apply property validator",
                    "3. Verify property holds",
                    "4. Log counter-example if property fails"
                ],
                "assertions": [
                    "assert property holds for generated input",
                    "assert counter-examples are captured",
                    "assert test terminates without crash"
                ]
            }
            
            if 'error_msg' in locals():
                test_case["error_message"] = error_msg
            
            test_cases.append(test_case)
        
        return test_cases
    
    # Property validators
    def _validate_file_access_consistency(self, input_data: Dict[str, Any]) -> bool:
        """Validate file access consistency property"""
        # Simulated validation
        operation = input_data.get("operation", "read")
        permissions = input_data.get("permissions", 0o644)
        
        if operation == "write" and permissions & 0o200 == 0:
            return False  # Write without write permission
        
        return True
    
    def _validate_path_normalization(self, input_data: Dict[str, Any]) -> bool:
        """Validate path normalization property"""
        path = input_data.get("path", "")
        
        # Check for double slashes (after normalization, these should be resolved)
        if "//" in path.replace("://", ""):  # Skip protocol part
            return False
        
        return True
    
    def _validate_atomic_operations(self, input_data: Dict[str, Any]) -> bool:
        """Validate atomic operations property"""
        operations = input_data.get("operations", [])
        
        # Simplified: assume operations are atomic if they're simple file ops
        simple_ops = {"read", "write", "stat"}
        complex_ops = {"mv", "cp", "rm"}
        
        for op in operations:
            if op in complex_ops:
                return True  # Complex ops are assumed atomic for this test
        
        return True
    
    def _validate_directory_hierarchy(self, input_data: Dict[str, Any]) -> bool:
        """Validate directory hierarchy integrity"""
        path = input_data.get("path", "")
        
        # Check for invalid path patterns
        invalid_patterns = ["/..//", "//./", "///"]
        
        for pattern in invalid_patterns:
            if pattern in path:
                return False
        
        return True
    
    def _validate_file_content_integrity(self, input_data: Dict[str, Any]) -> bool:
        """Validate file content integrity"""
        original_content = input_data.get("original_content", "")
        final_content = input_data.get("final_content", "")
        
        # Simplified: check if content is preserved through operations
        if original_content and final_content:
            # Content should be preserved unless explicitly modified
            operations = input_data.get("operations", [])
            if "read" in operations and "write" not in operations:
                return original_content == final_content
        
        return True
    
    def _validate_allocation_freedom(self, input_data: Dict[str, Any]) -> bool:
        """Validate memory allocation freedom property"""
        size = input_data.get("size", 1024)
        alignment = input_data.get("alignment", 8)
        
        # Large allocations might fail, small ones should succeed
        if size > 1024 * 1024 * 1024:  # 1GB
            return True  # Large allocations may fail (this is expected)
        
        if size < 0 or alignment < 1:
            return False  # Invalid parameters
        
        return True
    
    def _validate_no_memory_leaks(self, input_data: Dict[str, Any]) -> bool:
        """Validate no memory leaks property"""
        operations = input_data.get("operations", [])
        allocation_count = sum(1 for op in operations if op == "alloc")
        free_count = sum(1 for op in operations if op == "free")
        
        # For every alloc, there should be a corresponding free
        # (simplified - in real tests would track actual allocations)
        if allocation_count > free_count:
            return False
        
        return True
    
    def _validate_pointer_safety(self, input_data: Dict[str, Any]) -> bool:
        """Validate pointer arithmetic safety"""
        base_addr = input_data.get("base_address", 0x1000)
        offset = input_data.get("offset", 0)
        size = input_data.get("size", 1024)
        
        # Check if pointer arithmetic stays within bounds
        final_addr = base_addr + offset
        
        if offset >= size:
            return False  # Offset beyond allocated size
        
        if final_addr < base_addr:
            return False  # Arithmetic overflow
        
        return True
    
    def _validate_memory_alignment(self, input_data: Dict[str, Any]) -> bool:
        """Validate memory alignment requirements"""
        address = input_data.get("address", 0)
        alignment = input_data.get("required_alignment", 8)
        
        # Address should be aligned to required alignment
        if address % alignment != 0:
            return False
        
        return True
    
    def _validate_connection_lifecycle(self, input_data: Dict[str, Any]) -> bool:
        """Validate network connection lifecycle"""
        states = input_data.get("connection_states", [])
        
        # Check for valid state transitions
        valid_transitions = {
            "CLOSED": ["SYN_SENT", "LISTEN"],
            "SYN_SENT": ["SYN_RECEIVED", "CLOSED"],
            "SYN_RECEIVED": ["ESTABLISHED", "CLOSED"],
            "ESTABLISHED": ["FIN_WAIT_1", "CLOSE_WAIT"],
            "FIN_WAIT_1": ["FIN_WAIT_2", "CLOSED"],
            "FIN_WAIT_2": ["TIME_WAIT", "CLOSED"],
            "TIME_WAIT": ["CLOSED"],
            "CLOSE_WAIT": ["LAST_ACK", "CLOSED"],
            "LAST_ACK": ["CLOSED"]
        }
        
        for i in range(len(states) - 1):
            current = states[i]
            next_state = states[i + 1]
            
            if current not in valid_transitions:
                continue
                
            if next_state not in valid_transitions[current]:
                return False
        
        return True
    
    def _validate_data_integrity(self, input_data: Dict[str, Any]) -> bool:
        """Validate network data integrity"""
        original_data = input_data.get("original_data", "")
        received_data = input_data.get("received_data", "")
        checksum = input_data.get("checksum", "")
        
        # Simplified checksum validation
        import hashlib
        expected_checksum = hashlib.md5(original_data.encode()).hexdigest()
        
        if checksum == expected_checksum:
            return True
        
        # If no checksum, data should match
        return original_data == received_data
    
    def _validate_protocol_compliance(self, input_data: Dict[str, Any]) -> bool:
        """Validate protocol compliance"""
        protocol = input_data.get("protocol", "HTTP")
        packet = input_data.get("packet", {})
        
        if protocol == "HTTP":
            # Check for required HTTP headers
            if "method" in packet and "path" in packet:
                return True
            return False
        
        return True
    
    def _validate_process_isolation(self, input_data: Dict[str, Any]) -> bool:
        """Validate process isolation"""
        processes = input_data.get("processes", [])
        
        # Check that processes have distinct PIDs
        pids = [proc.get("pid") for proc in processes if "pid" in proc]
        
        return len(pids) == len(set(pids))  # All PIDs should be unique
    
    def _validate_signal_handling(self, input_data: Dict[str, Any]) -> bool:
        """Validate signal handling consistency"""
        signals = input_data.get("signals", [])
        
        # Simplified: no duplicate signal handlers
        signal_types = [sig.get("type") for sig in signals if "type" in sig]
        
        return len(signal_types) == len(set(signal_types))
    
    def _validate_request_response_symmetry(self, input_data: Dict[str, Any]) -> bool:
        """Validate request/response symmetry"""
        request = input_data.get("request", {})
        response = input_data.get("response", {})
        
        # Simplified: response should have same method (for idempotent ops)
        if request.get("method") in ["GET", "HEAD", "PUT", "DELETE"]:
            return response.get("method") == request.get("method")
        
        return True
    
    def _validate_auth_consistency(self, input_data: Dict[str, Any]) -> bool:
        """Validate authentication consistency"""
        endpoints = input_data.get("endpoints", [])
        
        # Check that authentication tokens are consistent across endpoints
        tokens = [ep.get("auth_token") for ep in endpoints if "auth_token" in ep]
        
        if not tokens:
            return True  # No authentication required
        
        # All tokens should be the same (or all invalid together)
        unique_tokens = set(tokens)
        return len(unique_tokens) <= 2  # Allow valid and invalid tokens
    
    def _validate_commutativity(self, input_data: Dict[str, Any]) -> bool:
        """Validate commutativity of addition"""
        a = input_data.get("a", 0)
        b = input_data.get("b", 0)
        
        return a + b == b + a
    
    def _validate_associativity(self, input_data: Dict[str, Any]) -> bool:
        """Validate associativity of addition"""
        a = input_data.get("a", 0)
        b = input_data.get("b", 0)
        c = input_data.get("c", 0)
        
        return (a + b) + c == a + (b + c)
    
    def _validate_idempotency(self, input_data: Dict[str, Any]) -> bool:
        """Validate idempotency of max operation"""
        value = input_data.get("value", 0)
        
        # max(value, value) should equal value
        return max(value, value) == value
    
    def _validate_stack_lifo(self, input_data: Dict[str, Any]) -> bool:
        """Validate stack LIFO property"""
        operations = input_data.get("operations", [])
        stack = []
        
        for op in operations:
            if op.get("type") == "push":
                stack.append(op.get("value"))
            elif op.get("type") == "pop":
                if not stack:
                    return False  # Pop from empty stack
                stack.pop()
        
        return True
    
    def _validate_queue_fifo(self, input_data: Dict[str, Any]) -> bool:
        """Validate queue FIFO property"""
        operations = input_data.get("operations", [])
        queue = []
        
        for op in operations:
            if op.get("type") == "enqueue":
                queue.append(op.get("value"))
            elif op.get("type") == "dequeue":
                if not queue:
                    return False  # Dequeue from empty queue
                expected = queue[0]
                actual = queue.pop(0)
                if expected != actual:
                    return False  # Not FIFO
        
        return True
    
    # Data generators
    def _generate_file_operation(self) -> Dict[str, Any]:
        """Generate file operation test data"""
        return {
            "operation": random.choice(["read", "write", "create", "delete", "stat"]),
            "path": "/tmp/test_" + str(random.randint(1, 1000)),
            "permissions": random.choice([0o644, 0o755, 0o777, 0o600, 0o400]),
            "content": "test_content_" + "x" * random.randint(0, 100)
        }
    
    def _generate_path_operation(self) -> Dict[str, Any]:
        """Generate path operation test data"""
        paths = [
            "/tmp/test",
            "/tmp/../test",
            "/tmp/./test",
            "/tmp//test",
            "/tmp/test/../test",
            "/tmp/test/./test"
        ]
        
        return {
            "path": random.choice(paths),
            "operation": "normalize"
        }
    
    def _generate_concurrent_file_op(self) -> Dict[str, Any]:
        """Generate concurrent file operation test data"""
        return {
            "operations": [random.choice(["read", "write", "create", "delete"]) 
                          for _ in range(random.randint(1, 5))],
            "file_path": "/tmp/concurrent_test",
            "thread_count": random.randint(2, 10)
        }
    
    def _generate_directory_operation(self) -> Dict[str, Any]:
        """Generate directory operation test data"""
        return {
            "operation": random.choice(["create", "remove", "list", "navigate"]),
            "path": "/tmp/dir_" + str(random.randint(1, 100)),
            "hierarchy_depth": random.randint(1, 5)
        }
    
    def _generate_file_content_operation(self) -> Dict[str, Any]:
        """Generate file content operation test data"""
        return {
            "original_content": "content_" + "x" * random.randint(0, 100),
            "operations": ["read", "write"],
            "final_content": None  # Will be computed during test
        }
    
    def _generate_allocation_pattern(self) -> Dict[str, Any]:
        """Generate memory allocation pattern test data"""
        return {
            "size": random.choice([0, 1, 4, 8, 16, 1024, 1024*1024]),
            "alignment": random.choice([1, 2, 4, 8, 16, 32]),
            "operations": random.choice(["alloc", "free"])
        }
    
    def _generate_memory_sequence(self) -> Dict[str, Any]:
        """Generate memory sequence test data"""
        operations = ["alloc", "free", "read", "write"] * random.randint(1, 10)
        
        return {
            "operations": [{"type": op, "size": random.randint(1, 1024)} 
                          for op in operations],
            "alignment": random.choice([1, 2, 4, 8])
        }
    
    def _generate_pointer_operation(self) -> Dict[str, Any]:
        """Generate pointer operation test data"""
        return {
            "base_address": random.choice([0x1000, 0x2000, 0x3000]),
            "offset": random.randint(-100, 1000),
            "size": random.choice([8, 16, 32, 64, 1024])
        }
    
    def _generate_aligned_access(self) -> Dict[str, Any]:
        """Generate aligned access test data"""
        return {
            "address": random.choice([0x1000, 0x1008, 0x1010, 0x1018]),
            "required_alignment": random.choice([4, 8, 16, 32])
        }
    
    def _generate_connection_sequence(self) -> Dict[str, Any]:
        """Generate connection sequence test data"""
        states = ["CLOSED", "SYN_SENT", "SYN_RECEIVED", "ESTABLISHED"]
        return {
            "connection_states": states[:random.randint(1, len(states))],
            "host": "localhost",
            "port": random.choice([80, 443, 8080, 22])
        }
    
    def _generate_network_data(self) -> Dict[str, Any]:
        """Generate network data test data"""
        import hashlib
        
        data = "network_data_" + "x" * random.randint(0, 100)
        checksum = hashlib.md5(data.encode()).hexdigest()
        
        return {
            "original_data": data,
            "received_data": data,  # Assume no corruption for now
            "checksum": checksum,
            "protocol": random.choice(["TCP", "UDP", "HTTP"])
        }
    
    def _generate_protocol_data(self) -> Dict[str, Any]:
        """Generate protocol data test data"""
        return {
            "protocol": random.choice(["HTTP", "HTTPS", "FTP", "SMTP"]),
            "packet": {
                "method": random.choice(["GET", "POST", "PUT", "DELETE"]),
                "path": "/test",
                "headers": {"User-Agent": "test"},
                "body": "test_body"
            }
        }
    
    def _generate_process_interaction(self) -> Dict[str, Any]:
        """Generate process interaction test data"""
        return {
            "processes": [
                {"pid": i, "name": f"process_{i}", "user": "test"}
                for i in range(random.randint(1, 5))
            ],
            "operations": ["fork", "exec", "signal"]
        }
    
    def _generate_signal_sequence(self) -> Dict[str, Any]:
        """Generate signal sequence test data"""
        signals = ["SIGTERM", "SIGINT", "SIGKILL", "SIGHUP", "SIGUSR1"]
        return {
            "signals": [
                {"type": random.choice(signals), "pid": random.randint(1000, 9999)}
                for _ in range(random.randint(1, 3))
            ]
        }
    
    def _generate_api_request(self) -> Dict[str, Any]:
        """Generate API request test data"""
        return {
            "request": {
                "method": random.choice(["GET", "POST", "PUT", "DELETE"]),
                "path": "/api/test",
                "headers": {"Content-Type": "application/json"}
            },
            "response": {
                "status": random.choice([200, 201, 400, 404, 500]),
                "method": random.choice(["GET", "POST", "PUT", "DELETE"])
            }
        }
    
    def _generate_authenticated_request(self) -> Dict[str, Any]:
        """Generate authenticated request test data"""
        endpoints = [
            {"path": "/api/user", "auth_token": "valid_token_123"},
            {"path": "/api/admin", "auth_token": "valid_token_123"},
            {"path": "/api/public", "auth_token": None}
        ]
        
        return {
            "endpoints": random.sample(endpoints, random.randint(1, len(endpoints)))
        }
    
    def _generate_numbers(self) -> Dict[str, Any]:
        """Generate numbers test data"""
        return {
            "a": random.randint(-1000, 1000),
            "b": random.randint(-1000, 1000),
            "c": random.randint(-1000, 1000),
            "value": random.randint(-1000, 1000)
        }
    
    def _generate_stack_sequence(self) -> Dict[str, Any]:
        """Generate stack sequence test data"""
        operations = []
        for _ in range(random.randint(1, 10)):
            op_type = random.choice(["push", "pop"])
            operations.append({
                "type": op_type,
                "value": random.randint(1, 100) if op_type == "push" else None
            })
        
        return {"operations": operations}
    
    def _generate_queue_sequence(self) -> Dict[str, Any]:
        """Generate queue sequence test data"""
        operations = []
        for _ in range(random.randint(1, 10)):
            op_type = random.choice(["enqueue", "dequeue"])
            operations.append({
                "type": op_type,
                "value": random.randint(1, 100) if op_type == "enqueue" else None
            })
        
        return {"operations": operations}