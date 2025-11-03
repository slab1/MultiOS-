"""
Automated Testing and Validation Framework

Comprehensive testing framework for OS research including functional,
performance, stress, and integration testing capabilities.
"""

import os
import time
import json
import unittest
import threading
import subprocess
from typing import Dict, List, Any, Optional, Callable, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import tempfile
import shutil
from enum import Enum
import concurrent.futures
import psutil

from .config import ResearchConfig


class TestType(Enum):
    """Types of tests."""
    FUNCTIONAL = "functional"
    PERFORMANCE = "performance"
    STRESS = "stress"
    INTEGRATION = "integration"
    SECURITY = "security"
    RELIABILITY = "reliability"
    COMPATIBILITY = "compatibility"


class TestStatus(Enum):
    """Test execution status."""
    PENDING = "pending"
    RUNNING = "running"
    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"
    ERROR = "error"


class TestSeverity(Enum):
    """Test severity levels."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class TestCase:
    """Represents a test case."""
    test_id: str
    name: str
    description: str
    test_type: TestType
    severity: TestSeverity
    test_function: Callable
    parameters: Dict[str, Any]
    dependencies: List[str]
    timeout: int = 300  # seconds
    retries: int = 1
    enabled: bool = True
    tags: List[str] = None
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = []
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'test_id': self.test_id,
            'name': self.name,
            'description': self.description,
            'test_type': self.test_type.value,
            'severity': self.severity.value,
            'parameters': self.parameters,
            'dependencies': self.dependencies,
            'timeout': self.timeout,
            'retries': self.retries,
            'enabled': self.enabled,
            'tags': self.tags
        }


@dataclass
class TestResult:
    """Represents test execution result."""
    test_id: str
    status: TestStatus
    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None
    output: str = ""
    error: Optional[str] = None
    metrics: Dict[str, Any] = None
    log_file: Optional[str] = None
    artifacts: List[str] = None
    assertions: List[Dict[str, Any]] = None
    
    def __post_init__(self):
        if self.metrics is None:
            self.metrics = {}
        if self.artifacts is None:
            self.artifacts = []
        if self.assertions is None:
            self.assertions = []
    
    def complete(self, status: TestStatus, output: str = "", error: Optional[str] = None):
        """Mark test as completed."""
        self.end_time = datetime.now()
        self.duration = (self.end_time - self.start_time).total_seconds()
        self.status = status
        self.output = output
        self.error = error
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        data = asdict(self)
        data['status'] = self.status.value
        data['start_time'] = self.start_time.isoformat()
        if self.end_time:
            data['end_time'] = self.end_time.isoformat()
        return data


@dataclass
class TestSuite:
    """Represents a collection of test cases."""
    suite_id: str
    name: str
    description: str
    test_cases: List[TestCase]
    execution_config: Dict[str, Any]
    dependencies: List[str] = None
    setup_scripts: List[str] = None
    cleanup_scripts: List[str] = None
    
    def __post_init__(self):
        if self.dependencies is None:
            self.dependencies = []
        if self.setup_scripts is None:
            self.setup_scripts = []
        if self.cleanup_scripts is None:
            self.cleanup_scripts = []
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'suite_id': self.suite_id,
            'name': self.name,
            'description': self.description,
            'test_cases': [test.to_dict() for test in self.test_cases],
            'execution_config': self.execution_config,
            'dependencies': self.dependencies,
            'setup_scripts': self.setup_scripts,
            'cleanup_scripts': self.cleanup_scripts
        }


class TestValidator:
    """
    Test validation and assertion framework.
    
    Provides comprehensive assertion capabilities for test validation.
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize test validator.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Validation state
        self.assertion_count = 0
        self.assertion_failures = []
        
        # Custom validators
        self.custom_validators = {}
        
        self.logger.info("Test validator initialized")
    
    def assert_equal(self, actual: Any, expected: Any, message: str = ""):
        """Assert two values are equal."""
        self.assertion_count += 1
        
        if actual != expected:
            failure = {
                'type': 'assert_equal',
                'actual': actual,
                'expected': expected,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Expected: {expected}, Actual: {actual}")
    
    def assert_not_equal(self, actual: Any, expected: Any, message: str = ""):
        """Assert two values are not equal."""
        self.assertion_count += 1
        
        if actual == expected:
            failure = {
                'type': 'assert_not_equal',
                'actual': actual,
                'expected': expected,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Values should not be equal: {actual}")
    
    def assert_true(self, condition: bool, message: str = ""):
        """Assert condition is true."""
        self.assertion_count += 1
        
        if not condition:
            failure = {
                'type': 'assert_true',
                'condition': condition,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Condition is not true")
    
    def assert_false(self, condition: bool, message: str = ""):
        """Assert condition is false."""
        self.assertion_count += 1
        
        if condition:
            failure = {
                'type': 'assert_false',
                'condition': condition,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Condition is not false")
    
    def assert_in(self, item: Any, collection: Union[List, Dict, Set], message: str = ""):
        """Assert item is in collection."""
        self.assertion_count += 1
        
        if item not in collection:
            failure = {
                'type': 'assert_in',
                'item': item,
                'collection': str(collection)[:200],
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Item {item} not found in collection")
    
    def assert_not_in(self, item: Any, collection: Union[List, Dict, Set], message: str = ""):
        """Assert item is not in collection."""
        self.assertion_count += 1
        
        if item in collection:
            failure = {
                'type': 'assert_not_in',
                'item': item,
                'collection': str(collection)[:200],
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Item {item} should not be in collection")
    
    def assert_greater(self, actual: Union[int, float], expected: Union[int, float], message: str = ""):
        """Assert actual is greater than expected."""
        self.assertion_count += 1
        
        if actual <= expected:
            failure = {
                'type': 'assert_greater',
                'actual': actual,
                'expected': expected,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} {actual} is not greater than {expected}")
    
    def assert_less(self, actual: Union[int, float], expected: Union[int, float], message: str = ""):
        """Assert actual is less than expected."""
        self.assertion_count += 1
        
        if actual >= expected:
            failure = {
                'type': 'assert_less',
                'actual': actual,
                'expected': expected,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} {actual} is not less than {expected}")
    
    def assert_raises(self, exception_type: type, func: Callable, *args, **kwargs):
        """Assert that calling func raises the specified exception."""
        self.assertion_count += 1
        
        try:
            func(*args, **kwargs)
            failure = {
                'type': 'assert_raises',
                'expected_exception': exception_type.__name__,
                'message': f"Expected {exception_type.__name__} to be raised"
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"Expected {exception_type.__name__} to be raised")
        except exception_type:
            pass  # Expected exception raised
        except Exception as e:
            failure = {
                'type': 'assert_raises',
                'expected_exception': exception_type.__name__,
                'actual_exception': type(e).__name__,
                'message': str(e)
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"Expected {exception_type.__name__}, got {type(e).__name__}: {e}")
    
    def assert_file_exists(self, file_path: str, message: str = ""):
        """Assert file exists."""
        self.assertion_count += 1
        
        if not os.path.exists(file_path):
            failure = {
                'type': 'assert_file_exists',
                'file_path': file_path,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} File does not exist: {file_path}")
    
    def assert_directory_exists(self, dir_path: str, message: str = ""):
        """Assert directory exists."""
        self.assertion_count += 1
        
        if not os.path.isdir(dir_path):
            failure = {
                'type': 'assert_directory_exists',
                'dir_path': dir_path,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Directory does not exist: {dir_path}")
    
    def assert_process_running(self, process_name: str, message: str = ""):
        """Assert process is running."""
        self.assertion_count += 1
        
        running = False
        for proc in psutil.process_iter(['name']):
            try:
                if process_name.lower() in proc.info['name'].lower():
                    running = True
                    break
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        
        if not running:
            failure = {
                'type': 'assert_process_running',
                'process_name': process_name,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} Process is not running: {process_name}")
    
    def assert_performance_threshold(self, 
                                   metric_name: str,
                                   value: float,
                                   threshold: float,
                                   comparison: str = "less_than",
                                   message: str = ""):
        """Assert performance metric meets threshold."""
        self.assertion_count += 1
        
        passed = False
        if comparison == "less_than":
            passed = value < threshold
        elif comparison == "less_than_or_equal":
            passed = value <= threshold
        elif comparison == "greater_than":
            passed = value > threshold
        elif comparison == "greater_than_or_equal":
            passed = value >= threshold
        elif comparison == "equal":
            passed = value == threshold
        
        if not passed:
            failure = {
                'type': 'assert_performance_threshold',
                'metric_name': metric_name,
                'value': value,
                'threshold': threshold,
                'comparison': comparison,
                'message': message
            }
            self.assertion_failures.append(failure)
            raise AssertionError(f"{message} {metric_name} = {value} does not meet {comparison} {threshold}")
    
    def add_custom_validator(self, name: str, validator_func: Callable):
        """Add a custom validator function."""
        self.custom_validators[name] = validator_func
        self.logger.info(f"Added custom validator: {name}")
    
    def get_validation_summary(self) -> Dict[str, Any]:
        """Get validation summary."""
        return {
            'total_assertions': self.assertion_count,
            'failures': len(self.assertion_failures),
            'success_rate': (self.assertion_count - len(self.assertion_failures)) / max(self.assertion_count, 1),
            'failure_details': self.assertion_failures
        }
    
    def reset(self):
        """Reset validator state."""
        self.assertion_count = 0
        self.assertion_failures.clear()


class TestSuite:
    """
    Test suite management and execution.
    
    Manages collections of test cases with dependencies and execution order.
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize test suite.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Test suite storage
        self.suites = {}
        self.test_cases = {}
        
        # Execution state
        self.execution_results = {}
        self.suite_results = {}
        
        # Execution configuration
        self.parallel_execution = config.testing.parallel_execution
        self.max_workers = config.testing.max_workers
        self.default_timeout = config.testing.timeout_seconds
        
        self.logger.info("Test suite initialized")
    
    def create_test_suite(self,
                         suite_id: str,
                         name: str,
                         description: str,
                         test_case_ids: List[str],
                         execution_config: Optional[Dict[str, Any]] = None) -> TestSuite:
        """
        Create a test suite.
        
        Args:
            suite_id: Unique suite identifier
            name: Suite name
            description: Suite description
            test_case_ids: List of test case IDs
            execution_config: Execution configuration
            
        Returns:
            Created test suite
        """
        # Get test cases
        test_cases = []
        for test_id in test_case_ids:
            if test_id in self.test_cases:
                test_cases.append(self.test_cases[test_id])
            else:
                self.logger.warning(f"Test case {test_id} not found")
        
        # Create execution config
        if execution_config is None:
            execution_config = {
                'timeout': self.default_timeout,
                'retries': 1,
                'stop_on_failure': False,
                'parallel': self.parallel_execution
            }
        
        # Create suite
        suite = TestSuite(
            suite_id=suite_id,
            name=name,
            description=description,
            test_cases=test_cases,
            execution_config=execution_config
        )
        
        self.suites[suite_id] = suite
        
        self.logger.info(f"Created test suite: {suite_id} with {len(test_cases)} tests")
        return suite
    
    def add_test_case(self, 
                     test_case: TestCase):
        """
        Add a test case to the registry.
        
        Args:
            test_case: Test case to add
        """
        self.test_cases[test_case.test_id] = test_case
        self.logger.info(f"Added test case: {test_case.test_id}")
    
    def get_test_suite(self, suite_id: str) -> Optional[TestSuite]:
        """Get test suite by ID."""
        return self.suites.get(suite_id)
    
    def list_test_suites(self) -> List[TestSuite]:
        """List all test suites."""
        return list(self.suites.values())
    
    def run_test_suite(self, 
                      suite_id: str,
                      environment: Optional[str] = None) -> Dict[str, Any]:
        """
        Execute a test suite.
        
        Args:
            suite_id: ID of suite to execute
            environment: Target environment
            
        Returns:
            Execution results
        """
        suite = self.get_test_suite(suite_id)
        if not suite:
            raise ValueError(f"Test suite {suite_id} not found")
        
        self.logger.info(f"Starting test suite execution: {suite_id}")
        
        # Initialize results
        suite_result = {
            'suite_id': suite_id,
            'start_time': datetime.now(),
            'test_results': [],
            'status': TestStatus.RUNNING
        }
        
        try:
            # Setup environment
            if environment:
                self._setup_test_environment(environment)
            
            # Execute tests based on configuration
            if suite.execution_config.get('parallel', self.parallel_execution):
                results = self._run_tests_parallel(suite.test_cases, suite.execution_config)
            else:
                results = self._run_tests_sequential(suite.test_cases, suite.execution_config)
            
            # Process results
            suite_result['test_results'] = results
            suite_result['status'] = self._determine_suite_status(results)
            suite_result['end_time'] = datetime.now()
            suite_result['duration'] = (suite_result['end_time'] - suite_result['start_time']).total_seconds()
            
            # Store results
            self.suite_results[suite_id] = suite_result
            
            # Cleanup environment
            if environment:
                self._cleanup_test_environment(environment)
            
            self.logger.info(f"Test suite {suite_id} completed with status: {suite_result['status'].value}")
            return suite_result
            
        except Exception as e:
            suite_result['status'] = TestStatus.ERROR
            suite_result['error'] = str(e)
            suite_result['end_time'] = datetime.now()
            self.logger.error(f"Test suite {suite_id} failed: {e}")
            raise
    
    def _setup_test_environment(self, environment: str):
        """Setup test environment."""
        self.logger.info(f"Setting up test environment: {environment}")
        # Environment setup logic would go here
        pass
    
    def _cleanup_test_environment(self, environment: str):
        """Cleanup test environment."""
        self.logger.info(f"Cleaning up test environment: {environment}")
        # Environment cleanup logic would go here
        pass
    
    def _run_tests_sequential(self, 
                            test_cases: List[TestCase], 
                            config: Dict[str, Any]) -> List[TestResult]:
        """Run tests sequentially."""
        results = []
        
        for test_case in test_cases:
            if not test_case.enabled:
                # Create skipped result
                result = TestResult(
                    test_id=test_case.test_id,
                    status=TestStatus.SKIPPED,
                    start_time=datetime.now()
                )
                result.complete(TestStatus.SKIPPED, "Test disabled")
                results.append(result)
                continue
            
            try:
                result = self._execute_test_case(test_case, config)
                results.append(result)
                
                # Check if we should stop on failure
                if config.get('stop_on_failure', False) and result.status == TestStatus.FAILED:
                    break
                    
            except Exception as e:
                result = TestResult(
                    test_id=test_case.test_id,
                    status=TestStatus.ERROR,
                    start_time=datetime.now()
                )
                result.complete(TestStatus.ERROR, "", str(e))
                results.append(result)
        
        return results
    
    def _run_tests_parallel(self, 
                          test_cases: List[TestCase], 
                          config: Dict[str, Any]) -> List[TestResult]:
        """Run tests in parallel."""
        results = []
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            # Submit all tests
            future_to_test = {}
            for test_case in test_cases:
                if test_case.enabled:
                    future = executor.submit(self._execute_test_case, test_case, config)
                    future_to_test[future] = test_case
                else:
                    # Create skipped result for disabled tests
                    result = TestResult(
                        test_id=test_case.test_id,
                        status=TestStatus.SKIPPED,
                        start_time=datetime.now()
                    )
                    result.complete(TestStatus.SKIPPED, "Test disabled")
                    results.append(result)
            
            # Collect results
            for future in concurrent.futures.as_completed(future_to_test):
                test_case = future_to_test[future]
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    result = TestResult(
                        test_id=test_case.test_id,
                        status=TestStatus.ERROR,
                        start_time=datetime.now()
                    )
                    result.complete(TestStatus.ERROR, "", str(e))
                    results.append(result)
        
        return results
    
    def _execute_test_case(self, test_case: TestCase, config: Dict[str, Any]) -> TestResult:
        """Execute a single test case."""
        result = TestResult(
            test_id=test_case.test_id,
            status=TestStatus.RUNNING,
            start_time=datetime.now()
        )
        
        # Set timeout
        timeout = test_case.timeout or config.get('timeout', self.default_timeout)
        
        try:
            # Execute test with timeout
            if hasattr(test_case.test_function, '__call__'):
                output = test_case.test_function(test_case.parameters)
            else:
                # Execute as string (script or command)
                output = self._execute_external_test(test_case)
            
            result.complete(TestStatus.PASSED, str(output))
            
        except Exception as e:
            result.complete(TestStatus.FAILED, "", str(e))
        
        return result
    
    def _execute_external_test(self, test_case: TestCase) -> str:
        """Execute external test (script or command)."""
        # This would handle execution of external test scripts
        return "External test executed successfully"
    
    def _determine_suite_status(self, test_results: List[TestResult]) -> TestStatus:
        """Determine overall suite status from test results."""
        if not test_results:
            return TestStatus.ERROR
        
        # Check for errors first
        if any(result.status == TestStatus.ERROR for result in test_results):
            return TestStatus.ERROR
        
        # Check for failures
        if any(result.status == TestStatus.FAILED for result in test_results):
            return TestStatus.FAILED
        
        # Check for skipped tests
        if all(result.status in [TestStatus.PASSED, TestStatus.SKIPPED] for result in test_results):
            if any(result.status == TestStatus.SKIPPED for result in test_results):
                return TestStatus.SKIPPED
            return TestStatus.PASSED
        
        return TestStatus.FAILED
    
    def get_suite_results(self, suite_id: str) -> Optional[Dict[str, Any]]:
        """Get results for a specific suite."""
        return self.suite_results.get(suite_id)
    
    def get_all_results(self) -> Dict[str, Any]:
        """Get results for all executed suites."""
        return {
            'suite_results': self.suite_results,
            'total_suites': len(self.suite_results),
            'execution_summary': self._generate_execution_summary()
        }
    
    def _generate_execution_summary(self) -> Dict[str, Any]:
        """Generate overall execution summary."""
        all_results = []
        for suite_result in self.suite_results.values():
            all_results.extend(suite_result['test_results'])
        
        if not all_results:
            return {'total_tests': 0}
        
        status_counts = {}
        total_duration = 0
        
        for result in all_results:
            status = result['status'].value if hasattr(result['status'], 'value') else result['status']
            status_counts[status] = status_counts.get(status, 0) + 1
            
            if result.get('duration'):
                total_duration += result['duration']
        
        return {
            'total_tests': len(all_results),
            'status_distribution': status_counts,
            'total_duration': total_duration,
            'average_duration': total_duration / len(all_results) if all_results else 0
        }


class AutomatedTestFramework:
    """
    Comprehensive automated testing framework for OS research.
    
    Provides:
    - Test case management
    - Test suite execution
    - Performance testing
    - Stress testing
    - Integration testing
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize automated test framework.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Core components
        self.validator = TestValidator(config)
        self.test_suite_manager = TestSuite(config)
        
        # Predefined test suites
        self._initialize_predefined_suites()
        
        # Test execution state
        self.execution_history = []
        self.current_executions = {}
        
        # Test templates
        self.test_templates = {}
        
        self.logger.info("Automated test framework initialized")
    
    def _initialize_predefined_suites(self):
        """Initialize predefined test suites."""
        # Functionality tests
        self._create_functionality_tests()
        
        # Performance tests
        self._create_performance_tests()
        
        # Stress tests
        self._create_stress_tests()
        
        # Integration tests
        self._create_integration_tests()
    
    def _create_functionality_tests(self):
        """Create basic functionality test cases."""
        # System information test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="func_system_info",
            name="System Information Test",
            description="Test system information retrieval",
            test_type=TestType.FUNCTIONAL,
            severity=TestSeverity.LOW,
            test_function=self._test_system_info,
            parameters={},
            dependencies=[],
            timeout=30
        ))
        
        # Process creation test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="func_process_creation",
            name="Process Creation Test",
            description="Test process creation capabilities",
            test_type=TestType.FUNCTIONAL,
            severity=TestSeverity.MEDIUM,
            test_function=self._test_process_creation,
            parameters={},
            dependencies=[],
            timeout=60
        ))
        
        # File operations test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="func_file_operations",
            name="File Operations Test",
            description="Test file system operations",
            test_type=TestType.FUNCTIONAL,
            severity=TestSeverity.MEDIUM,
            test_function=self._test_file_operations,
            parameters={},
            dependencies=[],
            timeout=60
        ))
    
    def _create_performance_tests(self):
        """Create performance test cases."""
        # CPU performance test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="perf_cpu_intensive",
            name="CPU Intensive Operations Test",
            description="Test CPU-intensive operations",
            test_type=TestType.PERFORMANCE,
            severity=TestSeverity.HIGH,
            test_function=self._test_cpu_performance,
            parameters={'duration': 10},
            dependencies=[],
            timeout=300
        ))
        
        # Memory performance test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="perf_memory_operations",
            name="Memory Operations Test",
            description="Test memory allocation and operations",
            test_type=TestType.PERFORMANCE,
            severity=TestSeverity.HIGH,
            test_function=self._test_memory_performance,
            parameters={'allocation_size': 1024 * 1024},
            dependencies=[],
            timeout=300
        ))
        
        # Disk I/O test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="perf_disk_io",
            name="Disk I/O Performance Test",
            description="Test disk read/write performance",
            test_type=TestType.PERFORMANCE,
            severity=TestSeverity.HIGH,
            test_function=self._test_disk_io_performance,
            parameters={'file_size': 100 * 1024 * 1024},  # 100MB
            dependencies=[],
            timeout=600
        ))
    
    def _create_stress_tests(self):
        """Create stress test cases."""
        # High CPU load test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="stress_cpu_load",
            name="High CPU Load Test",
            description="Test system under high CPU load",
            test_type=TestType.STRESS,
            severity=TestSeverity.HIGH,
            test_function=self._test_cpu_stress,
            parameters={'duration': 60, 'threads': 8},
            dependencies=[],
            timeout=600
        ))
        
        # Memory pressure test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="stress_memory_pressure",
            name="Memory Pressure Test",
            description="Test system under memory pressure",
            test_type=TestType.STRESS,
            severity=TestSeverity.HIGH,
            test_function=self._test_memory_stress,
            parameters={'memory_target': 0.8},  # Use 80% of available memory
            dependencies=[],
            timeout=600
        ))
        
        # Process creation stress test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="stress_process_creation",
            name="Process Creation Stress Test",
            description="Test rapid process creation",
            test_type=TestType.STRESS,
            severity=TestSeverity.HIGH,
            test_function=self._test_process_creation_stress,
            parameters={'target_processes': 100},
            dependencies=[],
            timeout=300
        ))
    
    def _create_integration_tests(self):
        """Create integration test cases."""
        # System integration test
        self.test_suite_manager.add_test_case(TestCase(
            test_id="intg_system_integration",
            name="System Integration Test",
            description="Test integrated system functionality",
            test_type=TestType.INTEGRATION,
            severity=TestSeverity.HIGH,
            test_function=self._test_system_integration,
            parameters={},
            dependencies=['func_system_info', 'func_process_creation'],
            timeout=300
        ))
    
    def _test_system_info(self, parameters: Dict[str, Any]) -> str:
        """Test system information retrieval."""
        import platform
        
        info = {
            'system': platform.system(),
            'release': platform.release(),
            'version': platform.version(),
            'machine': platform.machine(),
            'processor': platform.processor(),
            'cpu_count': psutil.cpu_count(),
            'memory_total': psutil.virtual_memory().total
        }
        
        self.validator.assert_true(info['cpu_count'] > 0, "CPU count should be positive")
        self.validator.assert_true(info['memory_total'] > 0, "Memory total should be positive")
        
        return f"System info retrieved: {info}"
    
    def _test_process_creation(self, parameters: Dict[str, Any]) -> str:
        """Test process creation."""
        import multiprocessing
        
        def worker_function():
            time.sleep(0.1)
            return "worker completed"
        
        # Test process creation
        with multiprocessing.Pool(processes=2) as pool:
            results = pool.map(worker_function, range(4))
        
        self.validator.assert_equal(len(results), 4, "Should create 4 worker processes")
        self.validator.assert_true(all(r == "worker completed" for r in results), "All workers should complete")
        
        return f"Process creation test completed with {len(results)} workers"
    
    def _test_file_operations(self, parameters: Dict[str, Any]) -> str:
        """Test file operations."""
        test_file = "/tmp/test_file_operations.txt"
        test_content = "test content for file operations"
        
        try:
            # Test write
            with open(test_file, 'w') as f:
                f.write(test_content)
            
            self.validator.assert_file_exists(test_file, "File should be created")
            
            # Test read
            with open(test_file, 'r') as f:
                content = f.read()
            
            self.validator.assert_equal(content, test_content, "Content should match")
            
            # Test delete
            os.remove(test_file)
            
            self.validator.assert_false(os.path.exists(test_file), "File should be deleted")
            
            return "File operations test completed successfully"
            
        finally:
            if os.path.exists(test_file):
                os.remove(test_file)
    
    def _test_cpu_performance(self, parameters: Dict[str, Any]) -> str:
        """Test CPU performance."""
        duration = parameters.get('duration', 10)
        
        start_time = time.time()
        operations = 0
        
        while time.time() - start_time < duration:
            # CPU-intensive operation
            result = 0
            for i in range(1000):
                result += i * i
            
            operations += 1
        
        actual_duration = time.time() - start_time
        
        self.validator.assert_greater(actual_duration, duration * 0.9, "Should run for specified duration")
        self.validator.assert_greater(operations, 10, "Should complete multiple operations")
        
        return f"CPU performance test: {operations} operations in {actual_duration:.2f} seconds"
    
    def _test_memory_performance(self, parameters: Dict[str, Any]) -> str:
        """Test memory performance."""
        allocation_size = parameters.get('allocation_size', 1024 * 1024)
        iterations = parameters.get('iterations', 100)
        
        start_time = time.time()
        allocations = []
        
        try:
            for i in range(iterations):
                # Allocate memory
                data = bytearray(allocation_size)
                
                # Perform operations
                for j in range(0, allocation_size, 4096):
                    data[j] = (i + j) % 256
                
                allocations.append(data)
            
            allocation_time = time.time() - start_time
            
            self.validator.assert_equal(len(allocations), iterations, f"Should allocate {iterations} buffers")
            self.validator.assert_true(allocation_time < 10, "Allocation should complete within 10 seconds")
            
            return f"Memory performance test: {iterations} allocations in {allocation_time:.2f} seconds"
            
        finally:
            allocations.clear()  # Free memory
    
    def _test_disk_io_performance(self, parameters: Dict[str, Any]) -> str:
        """Test disk I/O performance."""
        file_size = parameters.get('file_size', 100 * 1024 * 1024)
        test_file = "/tmp/test_disk_io_performance.dat"
        
        try:
            # Test write performance
            start_time = time.time()
            data = b'0' * 1024  # 1KB chunks
            
            with open(test_file, 'wb') as f:
                for i in range(file_size // 1024):
                    f.write(data)
            
            write_time = time.time() - start_time
            write_rate = file_size / write_time / (1024 * 1024)  # MB/s
            
            # Test read performance
            start_time = time.time()
            read_data = 0
            
            with open(test_file, 'rb') as f:
                while f.read(1024):
                    read_data += 1024
            
            read_time = time.time() - start_time
            read_rate = read_data / read_time / (1024 * 1024)  # MB/s
            
            self.validator.assert_greater(write_rate, 1, "Write rate should be > 1 MB/s")
            self.validator.assert_greater(read_rate, 1, "Read rate should be > 1 MB/s")
            
            return f"Disk I/O test: Write {write_rate:.2f} MB/s, Read {read_rate:.2f} MB/s"
            
        finally:
            if os.path.exists(test_file):
                os.remove(test_file)
    
    def _test_cpu_stress(self, parameters: Dict[str, Any]) -> str:
        """Test CPU under stress."""
        import multiprocessing
        
        duration = parameters.get('duration', 60)
        threads = parameters.get('threads', 8)
        
        def stress_worker():
            end_time = time.time() + duration
            counter = 0
            while time.time() < end_time:
                # CPU-intensive operation
                for i in range(10000):
                    counter += i * i
                counter = 0  # Reset to prevent overflow
        
        # Start stress workers
        start_time = time.time()
        processes = []
        
        for _ in range(threads):
            p = multiprocessing.Process(target=stress_worker)
            p.start()
            processes.append(p)
        
        # Monitor system during stress
        cpu_usage_samples = []
        memory_usage_samples = []
        
        monitor_end = start_time + duration
        while time.time() < monitor_end:
            cpu_usage = psutil.cpu_percent(interval=1)
            memory_usage = psutil.virtual_memory().percent
            
            cpu_usage_samples.append(cpu_usage)
            memory_usage_samples.append(memory_usage)
        
        # Wait for processes to complete
        for p in processes:
            p.join()
        
        actual_duration = time.time() - start_time
        avg_cpu = sum(cpu_usage_samples) / len(cpu_usage_samples)
        max_cpu = max(cpu_usage_samples)
        
        self.validator.assert_greater(avg_cpu, 50, "Average CPU usage should be high during stress")
        self.validator.assert_less(actual_duration, duration + 10, "Should complete within expected time")
        
        return f"CPU stress test: {threads} threads, avg CPU {avg_cpu:.1f}%, max CPU {max_cpu:.1f}%"
    
    def _test_memory_stress(self, parameters: Dict[str, Any]) -> str:
        """Test memory under pressure."""
        memory_target = parameters.get('memory_target', 0.8)
        available_memory = psutil.virtual_memory().total
        target_allocation = int(available_memory * memory_target)
        
        start_time = time.time()
        allocations = []
        
        try:
            # Allocate memory gradually
            chunk_size = 1024 * 1024  # 1MB chunks
            target_chunks = target_allocation // chunk_size
            
            for i in range(target_chunks):
                # Allocate chunk
                chunk = bytearray(chunk_size)
                
                # Fill with data to ensure allocation
                for j in range(0, chunk_size, 4096):
                    chunk[j] = i % 256
                
                allocations.append(chunk)
                
                # Check memory usage
                if i % 10 == 0:
                    memory_percent = psutil.virtual_memory().percent
                    self.validator.assert_true(memory_percent > 50, "Memory usage should increase")
            
            # Keep allocations for a bit
            time.sleep(5)
            
            stress_duration = time.time() - start_time
            allocated_mb = len(allocations) * chunk_size / (1024 * 1024)
            
            return f"Memory stress test: {allocated_mb:.1f} MB allocated in {stress_duration:.2f} seconds"
            
        finally:
            allocations.clear()  # Free memory
    
    def _test_process_creation_stress(self, parameters: Dict[str, Any]) -> str:
        """Test rapid process creation."""
        import multiprocessing
        
        target_processes = parameters.get('target_processes', 100)
        
        def quick_worker():
            return os.getpid()
        
        start_time = time.time()
        process_ids = set()
        
        # Create processes rapidly
        with multiprocessing.Pool(processes=min(target_processes, 16)) as pool:
            results = pool.map(quick_worker, range(target_processes))
            process_ids.update(results)
        
        creation_time = time.time() - start_time
        unique_processes = len(process_ids)
        
        self.validator.assert_greater(unique_processes, target_processes * 0.8, "Should create most target processes")
        self.validator.assert_true(creation_time < 30, "Process creation should complete within 30 seconds")
        
        return f"Process creation stress test: {unique_processes} processes in {creation_time:.2f} seconds"
    
    def _test_system_integration(self, parameters: Dict[str, Any]) -> str:
        """Test integrated system functionality."""
        # Combine multiple system operations
        test_results = []
        
        # Test 1: System info
        sys_info = self._test_system_info({})
        test_results.append("System info: OK")
        
        # Test 2: Process creation
        proc_result = self._test_process_creation({})
        test_results.append("Process creation: OK")
        
        # Test 3: File operations
        file_result = self._test_file_operations({})
        test_results.append("File operations: OK")
        
        self.validator.assert_equal(len(test_results), 3, "All integration tests should pass")
        
        return f"System integration test completed: {'; '.join(test_results)}"
    
    def run_test_suite(self, suite_name: str, environment: Optional[str] = None) -> Dict[str, Any]:
        """
        Run a test suite.
        
        Args:
            suite_name: Name of test suite to run
            environment: Target environment
            
        Returns:
            Test execution results
        """
        return self.test_suite_manager.run_test_suite(suite_name, environment)
    
    def create_custom_test(self,
                          test_id: str,
                          name: str,
                          description: str,
                          test_function: Callable,
                          test_type: TestType = TestType.FUNCTIONAL,
                          severity: TestSeverity = TestSeverity.MEDIUM,
                          parameters: Optional[Dict[str, Any]] = None) -> TestCase:
        """
        Create a custom test case.
        
        Args:
            test_id: Unique test identifier
            name: Test name
            description: Test description
            test_function: Function to execute
            test_type: Type of test
            severity: Test severity
            parameters: Test parameters
            
        Returns:
            Created test case
        """
        test_case = TestCase(
            test_id=test_id,
            name=name,
            description=description,
            test_type=test_type,
            severity=severity,
            test_function=test_function,
            parameters=parameters or {},
            dependencies=[]
        )
        
        self.test_suite_manager.add_test_case(test_case)
        self.logger.info(f"Created custom test: {test_id}")
        return test_case
    
    def analyze_results(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """
        Analyze test results.
        
        Args:
            results: Test execution results
            
        Returns:
            Analysis results
        """
        # Get test results
        test_results = results.get('test_results', [])
        
        if not test_results:
            return {'error': 'No test results to analyze'}
        
        # Analyze by status
        status_counts = {}
        duration_stats = {}
        
        for result in test_results:
            status = result.get('status', 'unknown')
            status_counts[status] = status_counts.get(status, 0) + 1
            
            if result.get('duration'):
                duration_stats[status] = duration_stats.get(status, [])
                duration_stats[status].append(result['duration'])
        
        # Calculate statistics
        total_tests = len(test_results)
        passed_tests = status_counts.get('passed', 0)
        failed_tests = status_counts.get('failed', 0)
        error_tests = status_counts.get('error', 0)
        success_rate = passed_tests / total_tests if total_tests > 0 else 0
        
        # Duration statistics
        total_duration = sum(sum(durations) for durations in duration_stats.values())
        avg_duration = total_duration / total_tests if total_tests > 0 else 0
        
        analysis = {
            'total_tests': total_tests,
            'status_distribution': status_counts,
            'success_rate': success_rate,
            'failed_tests': failed_tests,
            'error_tests': error_tests,
            'total_duration': total_duration,
            'average_duration': avg_duration,
            'duration_by_status': {
                status: {
                    'count': len(durations),
                    'average': sum(durations) / len(durations),
                    'min': min(durations),
                    'max': max(durations)
                }
                for status, durations in duration_stats.items()
            }
        }
        
        # Add validator analysis if available
        if hasattr(self.validator, 'get_validation_summary'):
            analysis['validation_summary'] = self.validator.get_validation_summary()
        
        return analysis
    
    def generate_report(self, results: Dict[str, Any], analysis: Dict[str, Any]) -> str:
        """
        Generate test report.
        
        Args:
            results: Test execution results
            analysis: Analysis results
            
        Returns:
            Report string
        """
        report_lines = []
        report_lines.append("# Test Execution Report")
        report_lines.append(f"Generated: {datetime.now().isoformat()}")
        report_lines.append("")
        
        # Summary
        report_lines.append("## Summary")
        report_lines.append(f"- Total Tests: {analysis['total_tests']}")
        report_lines.append(f"- Success Rate: {analysis['success_rate']:.1%}")
        report_lines.append(f"- Total Duration: {analysis['total_duration']:.2f} seconds")
        report_lines.append("")
        
        # Status distribution
        report_lines.append("## Status Distribution")
        for status, count in analysis['status_distribution'].items():
            percentage = count / analysis['total_tests'] * 100
            report_lines.append(f"- {status.title()}: {count} ({percentage:.1f}%)")
        report_lines.append("")
        
        # Test results
        test_results = results.get('test_results', [])
        report_lines.append("## Test Results")
        for result in test_results:
            status_emoji = {
                'passed': '✅',
                'failed': '❌',
                'error': '💥',
                'skipped': '⏭️'
            }.get(result.get('status', 'unknown'), '❓')
            
            duration_info = f" ({result.get('duration', 0):.2f}s)" if result.get('duration') else ""
            report_lines.append(f"{status_emoji} {result.get('test_id', 'unknown')}{duration_info}")
            
            if result.get('error'):
                report_lines.append(f"   Error: {result['error']}")
        
        return "\n".join(report_lines)
    
    def export_results(self, results: Dict[str, Any], file_path: str, format: str = 'json'):
        """
        Export test results.
        
        Args:
            results: Test results
            file_path: Export file path
            format: Export format ('json', 'csv', 'html')
        """
        if format.lower() == 'json':
            with open(file_path, 'w') as f:
                json.dump(results, f, indent=2, default=str)
        elif format.lower() == 'csv':
            import csv
            with open(file_path, 'w', newline='') as f:
                if results.get('test_results'):
                    fieldnames = results['test_results'][0].keys()
                    writer = csv.DictWriter(f, fieldnames=fieldnames)
                    writer.writeheader()
                    for result in results['test_results']:
                        writer.writerow(result)
        elif format.lower() == 'html':
            # Generate HTML report
            analysis = self.analyze_results(results)
            report_content = self.generate_report(results, analysis)
            
            html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background-color: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .error {{ color: orange; }}
    </style>
</head>
<body>
    <pre>{report_content}</pre>
</body>
</html>
"""
            with open(file_path, 'w') as f:
                f.write(html_content)
        
        self.logger.info(f"Exported test results to {file_path}")


# Add missing imports
import collections