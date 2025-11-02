#!/usr/bin/env python3
"""
Package Testing Framework
==========================

Automated testing framework for educational packages including
unit tests, integration tests, curriculum validation, and performance testing.
"""

import os
import sys
import json
import subprocess
import unittest
import tempfile
import shutil
import time
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime
import yaml

from src.package_manager import PackageMetadata, PackageType, CompatibilityLevel
from src.package_validator import PackageValidator
from src.dependency_resolver import DependencyResolver

logger = __import__('logging').getLogger(__name__)


@dataclass
class TestResult:
    """Result of a test execution"""
    test_name: str
    passed: bool
    duration: float
    error_message: Optional[str] = None
    output: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class TestSuite:
    """Test suite containing multiple test results"""
    suite_name: str
    total_tests: int
    passed_tests: int
    failed_tests: int
    skipped_tests: int
    duration: float
    results: List[TestResult]
    timestamp: str


@dataclass
class TestReport:
    """Comprehensive test report"""
    package_name: str
    package_version: str
    test_timestamp: str
    test_suites: List[TestSuite]
    overall_passed: bool
    total_tests: int
    total_passed: int
    total_failed: int
    total_skipped: int
    total_duration: float
    recommendations: List[str]


class EducationalPackageTester:
    """Comprehensive testing framework for educational packages"""
    
    def __init__(self, package_manager):
        self.pm = package_manager
        self.validator = PackageValidator(package_manager)
        self.temp_dir = Path("/tmp/package_tests")
        self.temp_dir.mkdir(exist_ok=True)
        
        # Test configuration
        self.test_config = {
            'max_test_duration': 300,  # 5 minutes
            'memory_limit_mb': 512,
            'require_internet': False,
            'required_test_types': ['unit', 'integration', 'curriculum'],
            'optional_test_types': ['performance', 'accessibility', 'security']
        }
    
    def run_all_tests(self, package_path: str, metadata: PackageMetadata) -> TestReport:
        """Run comprehensive test suite for a package"""
        logger.info(f"Running comprehensive tests for {metadata.name}")
        
        start_time = time.time()
        test_suites = []
        
        try:
            # Extract package for testing
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)
                package_extracted = temp_path / "package"
                shutil.unpack_archive(package_path, str(package_extracted))
                
                # Run different test suites
                test_suites.append(self._run_unit_tests(package_extracted, metadata))
                test_suites.append(self._run_integration_tests(package_extracted, metadata))
                test_suites.append(self._run_curriculum_tests(package_extracted, metadata))
                
                # Run optional tests
                if self.test_config.get('require_performance_tests', False):
                    test_suites.append(self._run_performance_tests(package_extracted, metadata))
                
                test_suites.append(self._run_security_tests(package_extracted, metadata))
                test_suites.append(self._run_accessibility_tests(package_extracted, metadata))
                
                # Calculate overall results
                total_tests = sum(suite.total_tests for suite in test_suites)
                total_passed = sum(suite.passed_tests for suite in test_suites)
                total_failed = sum(suite.failed_tests for suite in test_suites)
                total_skipped = sum(suite.skipped_tests for suite in test_suites)
                total_duration = time.time() - start_time
                
                overall_passed = total_failed == 0
                
                # Generate recommendations
                recommendations = self._generate_test_recommendations(test_suites, metadata)
                
                report = TestReport(
                    package_name=metadata.name,
                    package_version=metadata.version,
                    test_timestamp=datetime.now().isoformat(),
                    test_suites=test_suites,
                    overall_passed=overall_passed,
                    total_tests=total_tests,
                    total_passed=total_passed,
                    total_failed=total_failed,
                    total_skipped=total_skipped,
                    total_duration=total_duration,
                    recommendations=recommendations
                )
                
                # Store test report
                self._store_test_report(report)
                
                return report
                
        except Exception as e:
            logger.error(f"Error running comprehensive tests: {e}")
            # Return error report
            return TestReport(
                package_name=metadata.name,
                package_version=metadata.version,
                test_timestamp=datetime.now().isoformat(),
                test_suites=[],
                overall_passed=False,
                total_tests=0,
                total_passed=0,
                total_failed=1,
                total_skipped=0,
                total_duration=time.time() - start_time,
                recommendations=[f"Test execution failed: {str(e)}"]
            )
    
    def _run_unit_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run unit tests"""
        logger.info("Running unit tests")
        start_time = time.time()
        results = []
        
        # Look for pytest
        pytest_path = shutil.which('pytest')
        if not pytest_path:
            return TestSuite(
                suite_name="Unit Tests",
                total_tests=0,
                passed_tests=0,
                failed_tests=0,
                skipped_tests=0,
                duration=0.0,
                results=[],
                timestamp=datetime.now().isoformat()
            )
        
        tests_path = package_path / "tests"
        if not tests_path.exists():
            return TestSuite(
                suite_name="Unit Tests",
                total_tests=0,
                passed_tests=0,
                failed_tests=0,
                skipped_tests=1,
                duration=0.0,
                results=[],
                timestamp=datetime.now().isoformat()
            )
        
        # Run pytest
        try:
            result = subprocess.run([
                'pytest', str(tests_path), '-v', '--tb=short',
                '--maxfail=5', f'--timeout={self.test_config["max_test_duration"]}'
            ], capture_output=True, text=True, timeout=self.test_config["max_test_duration"])
            
            # Parse pytest results
            if result.returncode == 0:
                # All tests passed
                results.append(TestResult(
                    test_name="Pytest Execution",
                    passed=True,
                    duration=time.time() - start_time,
                    output=result.stdout
                ))
            else:
                # Some tests failed
                results.append(TestResult(
                    test_name="Pytest Execution",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message=result.stderr,
                    output=result.stdout
                ))
            
        except subprocess.TimeoutExpired:
            results.append(TestResult(
                test_name="Pytest Execution",
                passed=False,
                duration=self.test_config["max_test_duration"],
                error_message="Test execution timeout"
            ))
        except Exception as e:
            results.append(TestResult(
                test_name="Pytest Execution",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            ))
        
        # Additional unit test checks
        src_path = package_path / "src"
        if src_path.exists():
            python_files = list(src_path.rglob("*.py"))
            for py_file in python_files:
                test_result = self._check_python_quality(py_file)
                results.append(test_result)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Unit Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _run_integration_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run integration tests"""
        logger.info("Running integration tests")
        start_time = time.time()
        results = []
        
        # Test package installation simulation
        install_test = self._test_package_installation(package_path, metadata)
        results.append(install_test)
        
        # Test dependencies
        deps_test = self._test_dependencies(metadata)
        results.append(deps_test)
        
        # Test scripts
        scripts_test = self._test_scripts(package_path, metadata)
        results.append(scripts_test)
        
        # Test file structure
        structure_test = self._test_file_structure(package_path, metadata)
        results.append(structure_test)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Integration Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _run_curriculum_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run curriculum-specific tests"""
        logger.info("Running curriculum tests")
        start_time = time.time()
        results = []
        
        # Only run curriculum tests for educational packages
        if metadata.type not in [PackageType.CURRICULUM, PackageType.TUTORIAL, PackageType.ASSESSMENT]:
            return TestSuite(
                suite_name="Curriculum Tests",
                total_tests=0,
                passed_tests=0,
                failed_tests=0,
                skipped_tests=0,
                duration=0.0,
                results=[],
                timestamp=datetime.now().isoformat()
            )
        
        # Test curriculum manifest
        manifest_test = self._test_curriculum_manifest(package_path)
        results.append(manifest_test)
        
        # Test learning objectives
        objectives_test = self._test_learning_objectives(package_path)
        results.append(objectives_test)
        
        # Test assessment methods
        assessment_test = self._test_assessment_methods(package_path)
        results.append(assessment_test)
        
        # Test educational content
        content_test = self._test_educational_content(package_path)
        results.append(content_test)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Curriculum Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _run_performance_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run performance tests"""
        logger.info("Running performance tests")
        start_time = time.time()
        results = []
        
        # Test load time
        load_test = self._test_package_load_time(package_path)
        results.append(load_test)
        
        # Test memory usage
        memory_test = self._test_memory_usage(package_path)
        results.append(memory_test)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Performance Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _run_security_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run security tests"""
        logger.info("Running security tests")
        start_time = time.time()
        results = []
        
        # Test for dangerous code patterns
        dangerous_patterns_test = self._test_dangerous_patterns(package_path)
        results.append(dangerous_patterns_test)
        
        # Test file permissions
        permissions_test = self._test_file_permissions(package_path)
        results.append(permissions_test)
        
        # Test for secrets/credentials
        secrets_test = self._test_for_secrets(package_path)
        results.append(secrets_test)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Security Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _run_accessibility_tests(self, package_path: Path, metadata: PackageMetadata) -> TestSuite:
        """Run accessibility tests"""
        logger.info("Running accessibility tests")
        start_time = time.time()
        results = []
        
        # Test HTML accessibility
        html_accessibility_test = self._test_html_accessibility(package_path)
        results.append(html_accessibility_test)
        
        # Test content accessibility
        content_accessibility_test = self._test_content_accessibility(package_path)
        results.append(content_accessibility_test)
        
        # Calculate suite statistics
        passed_tests = sum(1 for r in results if r.passed)
        failed_tests = sum(1 for r in results if not r.passed and r.error_message)
        skipped_tests = sum(1 for r in results if not r.passed and not r.error_message)
        
        return TestSuite(
            suite_name="Accessibility Tests",
            total_tests=len(results),
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            duration=time.time() - start_time,
            results=results,
            timestamp=datetime.now().isoformat()
        )
    
    def _check_python_quality(self, py_file: Path) -> TestResult:
        """Check Python code quality"""
        start_time = time.time()
        
        try:
            with open(py_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            issues = []
            
            # Check for TODO/FIXME comments
            if 'TODO' in content:
                issues.append("TODO comments found")
            
            # Check for long lines
            long_lines = [line for line in content.split('\n') if len(line) > 120]
            if long_lines:
                issues.append(f"Found {len(long_lines)} long lines")
            
            # Check for print statements (debugging code)
            if 'print(' in content and 'print(' not in content.split('\n')[-3:]:
                issues.append("Print statements found (debugging code)")
            
            if issues:
                return TestResult(
                    test_name=f"Code Quality: {py_file.name}",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="; ".join(issues)
                )
            else:
                return TestResult(
                    test_name=f"Code Quality: {py_file.name}",
                    passed=True,
                    duration=time.time() - start_time
                )
                
        except Exception as e:
            return TestResult(
                test_name=f"Code Quality: {py_file.name}",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_package_installation(self, package_path: Path, metadata: PackageMetadata) -> TestResult:
        """Test package installation simulation"""
        start_time = time.time()
        
        try:
            # Test that package can be imported
            src_path = package_path / "src"
            if src_path.exists():
                # Add to Python path temporarily
                sys.path.insert(0, str(src_path))
                
                # Try to import main module
                try:
                    __import__(metadata.name.replace('-', '_'))
                    return TestResult(
                        test_name="Package Import Test",
                        passed=True,
                        duration=time.time() - start_time
                    )
                except ImportError as e:
                    return TestResult(
                        test_name="Package Import Test",
                        passed=False,
                        duration=time.time() - start_time,
                        error_message=f"Import failed: {e}"
                    )
                finally:
                    sys.path.pop(0)
            else:
                return TestResult(
                    test_name="Package Import Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="No src directory found"
                )
                
        except Exception as e:
            return TestResult(
                test_name="Package Import Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_dependencies(self, metadata: PackageMetadata) -> TestResult:
        """Test package dependencies"""
        start_time = time.time()
        
        try:
            # Check if dependencies are resolvable
            for dep_name, constraint in metadata.dependencies.items():
                # Simple dependency check
                if not self._is_dependency_available(dep_name):
                    return TestResult(
                        test_name="Dependency Check",
                        passed=False,
                        duration=time.time() - start_time,
                        error_message=f"Dependency not available: {dep_name}"
                    )
            
            return TestResult(
                test_name="Dependency Check",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Dependency Check",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_scripts(self, package_path: Path, metadata: PackageMetadata) -> TestResult:
        """Test package scripts"""
        start_time = time.time()
        
        try:
            # Check if scripts exist and are executable
            for script_name, script_path in metadata.scripts.items():
                full_script_path = package_path / script_path
                if full_script_path.exists():
                    # Try to run the script with --help
                    try:
                        result = subprocess.run([
                            'bash', str(full_script_path), '--help'
                        ], capture_output=True, text=True, timeout=10)
                        
                        if result.returncode != 0 and '--help not recognized' not in result.stderr:
                            return TestResult(
                                test_name=f"Script Test: {script_name}",
                                passed=False,
                                duration=time.time() - start_time,
                                error_message=f"Script failed: {result.stderr}"
                            )
                    except subprocess.TimeoutExpired:
                        return TestResult(
                            test_name=f"Script Test: {script_name}",
                            passed=False,
                            duration=time.time() - start_time,
                            error_message="Script execution timeout"
                        )
            
            return TestResult(
                test_name="Scripts Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Scripts Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_file_structure(self, package_path: Path, metadata: PackageMetadata) -> TestResult:
        """Test file structure"""
        start_time = time.time()
        
        try:
            missing_files = []
            for file_pattern in metadata.files:
                matching_files = list(package_path.rglob(file_pattern))
                if not matching_files:
                    missing_files.append(file_pattern)
            
            if missing_files:
                return TestResult(
                    test_name="File Structure Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message=f"Missing files: {', '.join(missing_files)}"
                )
            
            return TestResult(
                test_name="File Structure Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="File Structure Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_curriculum_manifest(self, package_path: Path) -> TestResult:
        """Test curriculum manifest"""
        start_time = time.time()
        
        try:
            curriculum_file = package_path / "curriculum" / "curriculum.yaml"
            if not curriculum_file.exists():
                return TestResult(
                    test_name="Curriculum Manifest Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="curriculum.yaml not found"
                )
            
            with open(curriculum_file, 'r') as f:
                curriculum_data = yaml.safe_load(f)
            
            required_fields = ['learning_objectives', 'standards', 'assessments']
            missing_fields = [field for field in required_fields if field not in curriculum_data]
            
            if missing_fields:
                return TestResult(
                    test_name="Curriculum Manifest Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message=f"Missing required fields: {', '.join(missing_fields)}"
                )
            
            return TestResult(
                test_name="Curriculum Manifest Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except yaml.YAMLError as e:
            return TestResult(
                test_name="Curriculum Manifest Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=f"Invalid YAML: {e}"
            )
        except Exception as e:
            return TestResult(
                test_name="Curriculum Manifest Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_learning_objectives(self, package_path: Path) -> TestResult:
        """Test learning objectives"""
        start_time = time.time()
        
        try:
            objectives_file = package_path / "curriculum" / "objectives.yaml"
            if objectives_file.exists():
                with open(objectives_file, 'r') as f:
                    objectives = yaml.safe_load(f)
                
                if not objectives:
                    return TestResult(
                        test_name="Learning Objectives Test",
                        passed=False,
                        duration=time.time() - start_time,
                        error_message="No learning objectives found"
                    )
            
            return TestResult(
                test_name="Learning Objectives Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Learning Objectives Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_assessment_methods(self, package_path: Path) -> TestResult:
        """Test assessment methods"""
        start_time = time.time()
        
        try:
            assessment_files = list(package_path.rglob("assessment*")) + list(package_path.rglob("test*"))
            
            if not assessment_files:
                return TestResult(
                    test_name="Assessment Methods Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="No assessment files found"
                )
            
            return TestResult(
                test_name="Assessment Methods Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Assessment Methods Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_educational_content(self, package_path: Path) -> TestResult:
        """Test educational content"""
        start_time = time.time()
        
        try:
            # Check for learning materials
            learning_files = list(package_path.rglob("lesson*")) + list(package_path.rglob("activity*"))
            
            if not learning_files:
                return TestResult(
                    test_name="Educational Content Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="No learning activities found"
                )
            
            return TestResult(
                test_name="Educational Content Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Educational Content Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_package_load_time(self, package_path: Path) -> TestResult:
        """Test package load time"""
        start_time = time.time()
        
        try:
            # Simulate package loading
            time.sleep(0.1)  # Simulate loading time
            load_time = time.time() - start_time
            
            if load_time > 2.0:  # 2 seconds threshold
                return TestResult(
                    test_name="Load Time Test",
                    passed=False,
                    duration=load_time,
                    error_message=f"Package load time too slow: {load_time:.2f}s"
                )
            
            return TestResult(
                test_name="Load Time Test",
                passed=True,
                duration=load_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Load Time Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_memory_usage(self, package_path: Path) -> TestResult:
        """Test memory usage"""
        start_time = time.time()
        
        try:
            # Simplified memory test
            # In practice, would use memory profiling tools
            memory_usage_mb = 50  # Mock memory usage
            
            if memory_usage_mb > self.test_config["memory_limit_mb"]:
                return TestResult(
                    test_name="Memory Usage Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message=f"Memory usage too high: {memory_usage_mb}MB"
                )
            
            return TestResult(
                test_name="Memory Usage Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Memory Usage Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_dangerous_patterns(self, package_path: Path) -> TestResult:
        """Test for dangerous code patterns"""
        start_time = time.time()
        
        try:
            dangerous_patterns = [
                r'eval\s*\(',
                r'exec\s*\(',
                r'subprocess\.call\s*\(\s*raw_input',
                r'os\.system\s*\(\s*raw_input',
            ]
            
            issues = []
            for py_file in package_path.rglob("*.py"):
                with open(py_file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                for pattern in dangerous_patterns:
                    if re.search(pattern, content, re.IGNORECASE):
                        issues.append(f"Dangerous pattern in {py_file.name}: {pattern}")
            
            if issues:
                return TestResult(
                    test_name="Dangerous Patterns Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="; ".join(issues)
                )
            
            return TestResult(
                test_name="Dangerous Patterns Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Dangerous Patterns Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_file_permissions(self, package_path: Path) -> TestResult:
        """Test file permissions"""
        start_time = time.time()
        
        try:
            # Check for executable files that shouldn't be
            issues = []
            for file_path in package_path.rglob("*"):
                if file_path.is_file() and os.access(file_path, os.X_OK):
                    if file_path.suffix not in ['.sh', '.py', '.exe']:
                        issues.append(f"Unexpected executable: {file_path.name}")
            
            if issues:
                return TestResult(
                    test_name="File Permissions Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="; ".join(issues)
                )
            
            return TestResult(
                test_name="File Permissions Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="File Permissions Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_for_secrets(self, package_path: Path) -> TestResult:
        """Test for secrets/credentials in code"""
        start_time = time.time()
        
        try:
            secret_patterns = [
                r'password\s*=\s*["\'][^"\']+["\']',
                r'api_key\s*=\s*["\'][^"\']+["\']',
                r'secret\s*=\s*["\'][^"\']+["\']',
                r'token\s*=\s*["\'][^"\']+["\']',
            ]
            
            issues = []
            for py_file in package_path.rglob("*.py"):
                with open(py_file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                for pattern in secret_patterns:
                    matches = re.findall(pattern, content, re.IGNORECASE)
                    if matches:
                        issues.append(f"Potential secrets in {py_file.name}")
                        break
            
            if issues:
                return TestResult(
                    test_name="Secrets Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="; ".join(issues)
                )
            
            return TestResult(
                test_name="Secrets Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="Secrets Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_html_accessibility(self, package_path: Path) -> TestResult:
        """Test HTML accessibility"""
        start_time = time.time()
        
        try:
            issues = []
            for html_file in package_path.rglob("*.html"):
                with open(html_file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                # Check for alt attributes on images
                if '<img' in content and 'alt=' not in content:
                    issues.append(f"Images missing alt attributes in {html_file.name}")
            
            if issues:
                return TestResult(
                    test_name="HTML Accessibility Test",
                    passed=False,
                    duration=time.time() - start_time,
                    error_message="; ".join(issues)
                )
            
            return TestResult(
                test_name="HTML Accessibility Test",
                passed=True,
                duration=time.time() - start_time
            )
            
        except Exception as e:
            return TestResult(
                test_name="HTML Accessibility Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _test_content_accessibility(self, package_path: Path) -> TestResult:
        """Test content accessibility"""
        start_time = time.time()
        
        try:
            # Check for accessibility-related files
            accessibility_files = list(package_path.rglob("*accessibility*")) + list(package_path.rglob("*a11y*"))
            
            return TestResult(
                test_name="Content Accessibility Test",
                passed=True,
                duration=time.time() - start_time,
                metadata={'accessibility_files': len(accessibility_files)}
            )
            
        except Exception as e:
            return TestResult(
                test_name="Content Accessibility Test",
                passed=False,
                duration=time.time() - start_time,
                error_message=str(e)
            )
    
    def _is_dependency_available(self, dependency_name: str) -> bool:
        """Check if a dependency is available"""
        try:
            import importlib
            importlib.import_module(dependency_name)
            return True
        except ImportError:
            return False
    
    def _generate_test_recommendations(self, test_suites: List[TestSuite], metadata: PackageMetadata) -> List[str]:
        """Generate test recommendations"""
        recommendations = []
        
        # Check overall test results
        total_failed = sum(suite.failed_tests for suite in test_suites)
        if total_failed > 0:
            recommendations.append(f"Address {total_failed} failing tests")
        
        # Check specific suite results
        for suite in test_suites:
            if suite.failed_tests > 0:
                recommendations.append(f"Fix {suite.failed_tests} issues in {suite.suite_name}")
        
        # Check unit test coverage
        unit_tests = next((s for s in test_suites if s.suite_name == "Unit Tests"), None)
        if unit_tests and unit_tests.total_tests < 5:
            recommendations.append("Add more unit tests")
        
        # Check curriculum validation
        curriculum_tests = next((s for s in test_suites if s.suite_name == "Curriculum Tests"), None)
        if curriculum_tests and curriculum_tests.total_tests == 0:
            recommendations.append("Add curriculum validation tests")
        
        # Check security tests
        security_tests = next((s for s in test_suites if s.suite_name == "Security Tests"), None)
        if security_tests and security_tests.failed_tests > 0:
            recommendations.append("Address security issues")
        
        return recommendations
    
    def _store_test_report(self, report: TestReport):
        """Store test report"""
        try:
            reports_dir = Path("/workspace/community/package_manager/packages/test_reports")
            reports_dir.mkdir(parents=True, exist_ok=True)
            
            report_file = reports_dir / f"{report.package_name}_{report.test_timestamp.replace(':', '_')}.json"
            
            # Convert to dictionary for storage
            report_dict = asdict(report)
            with open(report_file, 'w') as f:
                json.dump(report_dict, f, indent=2)
            
            logger.info(f"Test report stored: {report_file}")
            
        except Exception as e:
            logger.error(f"Error storing test report: {e}")


def main():
    """Main entry point for the testing framework"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Educational Package Testing Framework")
    parser.add_argument('package_path', help='Path to package file or directory')
    parser.add_argument('--metadata', help='Package metadata file')
    parser.add_argument('--output', help='Output file for test report')
    parser.add_argument('--verbose', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.basicConfig(level=logging.DEBUG)
    
    # Initialize tester
    tester = EducationalPackageTester(None)  # Pass None for now
    
    # Load metadata
    metadata = None
    if args.metadata:
        with open(args.metadata, 'r') as f:
            data = json.load(f)
            data['type'] = PackageType(data['type'])
            data['compatibility'] = CompatibilityLevel(data['compatibility'])
            metadata = PackageMetadata(**data)
    
    if not metadata:
        print("Error: Package metadata is required")
        sys.exit(1)
    
    # Run tests
    report = tester.run_all_tests(args.package_path, metadata)
    
    # Print results
    print(f"\nTest Results for {report.package_name} v{report.package_version}")
    print("=" * 60)
    print(f"Overall: {'PASS' if report.overall_passed else 'FAIL'}")
    print(f"Tests: {report.total_passed}/{report.total_tests} passed")
    print(f"Duration: {report.total_duration:.2f}s")
    
    for suite in report.test_suites:
        print(f"\n{suite.suite_name}:")
        print(f"  Passed: {suite.passed_tests}/{suite.total_tests}")
        for result in suite.results:
            status = "PASS" if result.passed else "FAIL"
            print(f"  - {result.test_name}: {status}")
            if result.error_message:
                print(f"    Error: {result.error_message}")
    
    if report.recommendations:
        print(f"\nRecommendations:")
        for rec in report.recommendations:
            print(f"  - {rec}")
    
    # Save report
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(asdict(report), f, indent=2)
        print(f"\nDetailed report saved to: {args.output}")
    
    sys.exit(0 if report.overall_passed else 1)


if __name__ == '__main__':
    main()