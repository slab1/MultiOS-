#!/usr/bin/env python3
"""
Comprehensive Stress Testing Suite

Advanced stress testing system for memory management and file systems under extreme conditions.

This package provides:
- Memory pressure testing and leak detection
- File system stress testing
- CPU stress and thermal testing
- Concurrent process and thread testing
- Resource exhaustion testing
- Performance monitoring and reporting

Usage:
    from stress_tests import ComprehensiveStressTester
    tester = ComprehensiveStressTester()
    results = tester.run_all_tests()
"""

__version__ = "1.0.0"
__author__ = "Stress Testing Suite"
__description__ = "Advanced stress testing system for system resources"

# Import main components
from .main_stress_test import ComprehensiveStressTester, TestResult
from .utils.system_utils import StressTestConfig, SystemMonitor, ResourceManager

# Export main classes
__all__ = [
    'ComprehensiveStressTester',
    'TestResult',
    'StressTestConfig',
    'SystemMonitor',
    'ResourceManager'
]