"""
MultiOS Test Generation Framework

Intelligent test case generation system for MultiOS edge cases and boundary conditions.

Provides comprehensive test generation capabilities including:
- Systematic edge case generation
- Fuzz testing integration
- Property-based testing
- State space exploration
- Memory safety testing
- API compliance testing
- Performance testing
- Coverage analysis
"""

from .core.testgen_framework import (
    TestGenFramework,
    TestConfig,
    TestResult,
    TestType
)

from .generators.edge_case_generator import EdgeCaseGenerator, EdgeCaseType
from .generators.fuzz_test_generator import FuzzTestGenerator, FuzzType
from .generators.property_based_generator import PropertyBasedGenerator, PropertyType
from .generators.state_space_generator import StateSpaceGenerator, ConcurrencyPattern
from .generators.memory_safety_generator import MemorySafetyGenerator, MemoryViolationType
from .generators.api_compliance_generator import APIComplianceGenerator, ComplianceCheck
from .generators.performance_generator import PerformanceGenerator, PerformanceIssue, LoadPattern

from .analyzers.coverage_analyzer import CoverageAnalyzer, CoverageType, CoverageMetric

from .utils.test_utils import (
    TestDataGenerator,
    TestDataValidator,
    TestMetrics,
    TestReporter,
    FileUtils,
    StringUtils
)

__version__ = "1.0.0"
__author__ = "MultiOS Team"
__email__ = "testgen@multios.dev"

__all__ = [
    # Core framework
    "TestGenFramework",
    "TestConfig", 
    "TestResult",
    "TestType",
    
    # Generators
    "EdgeCaseGenerator",
    "FuzzTestGenerator", 
    "PropertyBasedGenerator",
    "StateSpaceGenerator",
    "MemorySafetyGenerator",
    "APIComplianceGenerator",
    "PerformanceGenerator",
    
    # Enums
    "EdgeCaseType",
    "FuzzType",
    "PropertyType", 
    "ConcurrencyPattern",
    "MemoryViolationType",
    "ComplianceCheck",
    "PerformanceIssue",
    "LoadPattern",
    
    # Analyzers
    "CoverageAnalyzer",
    "CoverageType",
    "CoverageMetric",
    
    # Utils
    "TestDataGenerator",
    "TestDataValidator",
    "TestMetrics", 
    "TestReporter",
    "FileUtils",
    "StringUtils"
]

# Default configuration
DEFAULT_CONFIG = {
    "iterations": 1000,
    "timeout": 3600,
    "output_dir": "testgen_output",
    "components": [
        "filesystem",
        "memory", 
        "network",
        "process",
        "api"
    ],
    "test_types": [
        "edge_case",
        "fuzz_test",
        "property_based",
        "state_space",
        "memory_safety",
        "api_compliance", 
        "performance",
        "comprehensive"
    ]
}

def create_framework(workspace_path: str = "/workspace") -> TestGenFramework:
    """Create and configure test generation framework instance"""
    return TestGenFramework(workspace_path)

def generate_test_suite(component: str, test_type: str = "comprehensive", 
                       iterations: int = 1000, **kwargs) -> TestResult:
    """Quick function to generate test suite"""
    framework = create_framework()
    config = TestConfig(
        test_type=TestType(test_type),
        component=component,
        iterations=iterations,
        **kwargs
    )
    
    import asyncio
    return asyncio.run(framework.generate_test_cases(config))

def get_available_components() -> List[str]:
    """Get list of supported components"""
    return DEFAULT_CONFIG["components"]

def get_available_test_types() -> List[str]:
    """Get list of available test types"""
    return DEFAULT_CONFIG["test_types"]

def validate_component(component: str) -> bool:
    """Validate if component is supported"""
    return component in DEFAULT_CONFIG["components"]

def validate_test_type(test_type: str) -> bool:
    """Validate if test type is supported"""
    return test_type in DEFAULT_CONFIG["test_types"] or hasattr(TestType, test_type.upper())

# Import typing for type hints
from typing import List