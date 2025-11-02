"""
Test Coverage Analysis for MultiOS
Analyzes test coverage and identifies gaps in test scenarios
"""

import json
import logging
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass, field
from enum import Enum
import math
from collections import defaultdict, Counter

class CoverageType(Enum):
    """Types of coverage metrics"""
    LINE_COVERAGE = "line_coverage"
    BRANCH_COVERAGE = "branch_coverage"
    FUNCTION_COVERAGE = "function_coverage"
    CONDITION_COVERAGE = "condition_coverage"
    PATH_COVERAGE = "path_coverage"
    REQUIREMENT_COVERAGE = "requirement_coverage"
    API_COVERAGE = "api_coverage"
    EDGE_CASE_COVERAGE = "edge_case_coverage"
    STATE_COVERAGE = "state_coverage"
    PERFORMANCE_COVERAGE = "performance_coverage"

@dataclass
class CoverageMetric:
    """Coverage metric definition"""
    name: str
    coverage_type: CoverageType
    total_items: int
    covered_items: int
    coverage_percentage: float
    uncovered_items: List[str] = field(default_factory=list)
    coverage_gaps: List[str] = field(default_factory=list)

@dataclass
class CoverageReport:
    """Comprehensive coverage report"""
    test_suite: str
    timestamp: str
    overall_coverage: float
    metrics: List[CoverageMetric]
    gap_analysis: Dict[str, List[str]]
    recommendations: List[str]
    test_effectiveness: Dict[str, float]

class CoverageAnalyzer:
    """Analyzes test coverage and identifies gaps"""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        self.component_profiles = self._initialize_component_profiles()
        
    def _initialize_component_profiles(self) -> Dict[str, Dict[str, Any]]:
        """Initialize component test profiles"""
        return {
            "filesystem": {
                "components": [
                    "file_operations", "directory_operations", "path_handling",
                    "permission_management", "file_locking", "metadata_handling"
                ],
                "edge_cases": [
                    "empty_file", "very_large_file", "special_characters_in_path",
                    "permission_denied", "disk_full", "concurrent_access",
                    "symbolic_links", "hard_links", "file_not_found"
                ],
                "api_endpoints": [
                    "/files/create", "/files/read", "/files/write", "/files/delete",
                    "/directories/create", "/directories/list", "/directories/remove"
                ],
                "performance_scenarios": [
                    "small_files", "large_files", "many_small_files",
                    "concurrent_access", "slow_storage"
                ]
            },
            "memory": {
                "components": [
                    "allocation", "deallocation", "bounds_checking",
                    "garbage_collection", "memory_leak_detection", "heap_management"
                ],
                "edge_cases": [
                    "null_pointer", "buffer_overflow", "buffer_underflow",
                    "memory_leak", "double_free", "use_after_free",
                    "heap_fragmentation", "stack_overflow"
                ],
                "api_endpoints": [
                    "/memory/allocate", "/memory/free", "/memory/realloc",
                    "/memory/info", "/memory/defrag"
                ],
                "performance_scenarios": [
                    "small_allocation", "large_allocation", "many_allocations",
                    "fragmented_allocation", "concurrent_allocation"
                ]
            },
            "network": {
                "components": [
                    "connection_management", "data_transmission", "protocol_handling",
                    "error_handling", "timeout_management", "flow_control"
                ],
                "edge_cases": [
                    "connection_timeout", "connection_refused", "network_unreachable",
                    "packet_loss", "corrupted_data", "slow_network",
                    "partial_read", "partial_write"
                ],
                "api_endpoints": [
                    "/network/connect", "/network/send", "/network/receive",
                    "/network/disconnect", "/network/status"
                ],
                "performance_scenarios": [
                    "low_latency", "high_latency", "low_bandwidth",
                    "high_bandwidth", "packet_loss", "jitter"
                ]
            },
            "process": {
                "components": [
                    "process_creation", "process_termination", "signal_handling",
                    "process_communication", "resource_management", "child_management"
                ],
                "edge_cases": [
                    "process_crash", "signal_delivery", "zombie_process",
                    "resource_exhaustion", "process_hang", "permission_denied"
                ],
                "api_endpoints": [
                    "/process/create", "/process/terminate", "/process/signal",
                    "/process/status", "/process/wait"
                ],
                "performance_scenarios": [
                    "many_processes", "process_startup_time",
                    "process_communication", "signal_delivery"
                ]
            },
            "api": {
                "components": [
                    "request_parsing", "response_building", "authentication",
                    "authorization", "rate_limiting", "error_handling"
                ],
                "edge_cases": [
                    "malformed_request", "unauthorized_access", "rate_limit_exceeded",
                    "missing_parameters", "invalid_parameters", "timeout",
                    "service_unavailable", "payload_too_large"
                ],
                "api_endpoints": [
                    "/api/GET/*", "/api/POST/*", "/api/PUT/*", "/api/DELETE/*"
                ],
                "performance_scenarios": [
                    "low_load", "high_load", "spike_load",
                    "sustained_load", "concurrent_requests"
                ]
            }
        }
    
    async def analyze_coverage(self, test_cases: List[Dict[str, Any]], 
                             component: str) -> Dict[str, Any]:
        """Analyze coverage for a set of test cases"""
        profile = self.component_profiles.get(component, {})
        
        # Initialize coverage metrics
        coverage_analysis = {
            "component": component,
            "test_case_count": len(test_cases),
            "coverage_metrics": self._calculate_coverage_metrics(test_cases, profile),
            "gap_analysis": self._analyze_coverage_gaps(test_cases, profile),
            "effectiveness_score": self._calculate_effectiveness_score(test_cases),
            "recommendations": self._generate_coverage_recommendations(test_cases, profile)
        }
        
        return coverage_analysis
    
    async def generate_comprehensive_report(self, test_registry: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive coverage report across all tests"""
        all_components = set()
        total_tests = 0
        component_reports = {}
        
        # Collect all components and tests
        for test_result in test_registry.values():
            all_components.add(test_result.component)
            total_tests += test_result.generated_count
            
            # Analyze each component's tests
            if test_result.component not in component_reports:
                component_reports[test_result.component] = []
            component_reports[test_result.component].extend(test_result.test_cases)
        
        # Generate component-specific reports
        component_analysis = {}
        for component, test_cases in component_reports.items():
            component_analysis[component] = await self.analyze_coverage(test_cases, component)
        
        # Generate overall report
        comprehensive_report = CoverageReport(
            test_suite="MultiOS Test Suite",
            timestamp=str(pd.Timestamp.now()),
            overall_coverage=self._calculate_overall_coverage(component_analysis),
            metrics=self._aggregate_coverage_metrics(component_analysis),
            gap_analysis=self._analyze_cross_component_gaps(component_analysis),
            recommendations=self._generate_system_wide_recommendations(component_analysis),
            test_effectiveness=self._calculate_system_effectiveness(component_analysis)
        )
        
        return {
            "summary": {
                "total_test_cases": total_tests,
                "components_tested": len(all_components),
                "overall_coverage": comprehensive_report.overall_coverage,
                "coverage_trend": "stable"  # Could be calculated from historical data
            },
            "component_analysis": component_analysis,
            "system_wide_gaps": comprehensive_report.gap_analysis,
            "recommendations": comprehensive_report.recommendations,
            "coverage_by_type": self._analyze_coverage_by_type(component_analysis)
        }
    
    def _calculate_coverage_metrics(self, test_cases: List[Dict[str, Any]], 
                                  profile: Dict[str, Any]) -> Dict[str, CoverageMetric]:
        """Calculate various coverage metrics"""
        metrics = {}
        
        # Component coverage
        components = profile.get("components", [])
        tested_components = set()
        
        for test_case in test_cases:
            comp = test_case.get("component", "")
            if comp in components:
                tested_components.add(comp)
        
        metrics["component_coverage"] = CoverageMetric(
            name="Component Coverage",
            coverage_type=CoverageType.REQUIREMENT_COVERAGE,
            total_items=len(components),
            covered_items=len(tested_components),
            coverage_percentage=(len(tested_components) / len(components) * 100) if components else 0,
            uncovered_items=list(set(components) - tested_components)
        )
        
        # Edge case coverage
        edge_cases = profile.get("edge_cases", [])
        tested_edge_cases = set()
        
        for test_case in test_cases:
            test_type = test_case.get("type", "")
            test_name = test_case.get("name", "")
            
            # Map test types to edge cases
            if test_type in ["edge_case", "fuzz_test"] or "boundary" in test_name.lower():
                for edge_case in edge_cases:
                    if edge_case.lower().replace(" ", "_") in test_name.lower():
                        tested_edge_cases.add(edge_case)
        
        metrics["edge_case_coverage"] = CoverageMetric(
            name="Edge Case Coverage",
            coverage_type=CoverageType.EDGE_CASE_COVERAGE,
            total_items=len(edge_cases),
            covered_items=len(tested_edge_cases),
            coverage_percentage=(len(tested_edge_cases) / len(edge_cases) * 100) if edge_cases else 0,
            uncovered_items=list(set(edge_cases) - tested_edge_cases)
        )
        
        # API coverage
        api_endpoints = profile.get("api_endpoints", [])
        tested_endpoints = set()
        
        for test_case in test_cases:
            # Extract API endpoints from test cases
            if "api" in test_case.get("component", "").lower():
                # For now, assume each API test covers one endpoint
                tested_endpoints.add(f"endpoint_{test_case.get('id', '')}")
        
        metrics["api_coverage"] = CoverageMetric(
            name="API Coverage",
            coverage_type=CoverageType.API_COVERAGE,
            total_items=len(api_endpoints),
            covered_items=min(len(tested_endpoints), len(api_endpoints)),
            coverage_percentage=min(len(tested_endpoints), len(api_endpoints)) / len(api_endpoints) * 100 if api_endpoints else 0,
            uncovered_items=list(set(api_endpoints) - tested_endpoints) if len(tested_endpoints) < len(api_endpoints) else []
        )
        
        # Performance coverage
        performance_scenarios = profile.get("performance_scenarios", [])
        tested_performance = set()
        
        for test_case in test_cases:
            if test_case.get("type") == "performance":
                test_name = test_case.get("name", "").lower()
                for scenario in performance_scenarios:
                    if scenario.lower().replace(" ", "_") in test_name:
                        tested_performance.add(scenario)
        
        metrics["performance_coverage"] = CoverageMetric(
            name="Performance Coverage",
            coverage_type=CoverageType.PERFORMANCE_COVERAGE,
            total_items=len(performance_scenarios),
            covered_items=len(tested_performance),
            coverage_percentage=(len(tested_performance) / len(performance_scenarios) * 100) if performance_scenarios else 0,
            uncovered_items=list(set(performance_scenarios) - tested_performance)
        )
        
        return metrics
    
    def _analyze_coverage_gaps(self, test_cases: List[Dict[str, Any]], 
                             profile: Dict[str, Any]) -> Dict[str, List[str]]:
        """Analyze coverage gaps"""
        gaps = {
            "missing_components": [],
            "missing_edge_cases": [],
            "missing_api_endpoints": [],
            "missing_performance_scenarios": [],
            "low_coverage_areas": []
        }
        
        # Identify missing components
        components = profile.get("components", [])
        tested_components = set()
        
        for test_case in test_cases:
            comp = test_case.get("component", "")
            if comp in components:
                tested_components.add(comp)
        
        gaps["missing_components"] = list(set(components) - tested_components)
        
        # Identify missing edge cases
        edge_cases = profile.get("edge_cases", [])
        tested_edge_cases = set()
        
        for test_case in test_cases:
            test_type = test_case.get("type", "")
            if test_type in ["edge_case", "fuzz_test", "property_based"]:
                tested_edge_cases.add(test_case.get("id", ""))
        
        gaps["missing_edge_cases"] = [ec for ec in edge_cases 
                                     if not any(ec.lower().replace(" ", "_") in tc.get("name", "").lower() 
                                               for tc in test_cases)]
        
        # Identify low coverage areas
        coverage_metrics = self._calculate_coverage_metrics(test_cases, profile)
        for metric_name, metric in coverage_metrics.items():
            if metric.coverage_percentage < 70:  # Threshold for low coverage
                gaps["low_coverage_areas"].append(f"{metric_name}: {metric.coverage_percentage:.1f}%")
        
        return gaps
    
    def _calculate_effectiveness_score(self, test_cases: List[Dict[str, Any]]) -> float:
        """Calculate test effectiveness score"""
        if not test_cases:
            return 0.0
        
        # Factors contributing to effectiveness
        diversity_score = self._calculate_test_diversity(test_cases)
        quality_score = self._calculate_test_quality(test_cases)
        coverage_score = self._calculate_coverage_adequacy(test_cases)
        
        # Weighted average
        effectiveness = (
            diversity_score * 0.3 +
            quality_score * 0.4 +
            coverage_score * 0.3
        )
        
        return min(100.0, effectiveness)
    
    def _calculate_test_diversity(self, test_cases: List[Dict[str, Any]]) -> float:
        """Calculate test case diversity score"""
        if not test_cases:
            return 0.0
        
        # Count different test types
        test_types = set(tc.get("type", "") for tc in test_cases)
        type_diversity = len(test_types) / 8 * 100  # Assume max 8 types
        
        # Count different components
        components = set(tc.get("component", "") for tc in test_cases)
        component_diversity = len(components) / 5 * 100  # Assume max 5 components
        
        # Count priority distribution
        priorities = [tc.get("priority", 3) for tc in test_cases]
        priority_distribution = 1 - (max(priorities) - min(priorities)) / 5 if priorities else 0
        priority_distribution *= 100
        
        return (type_diversity + component_diversity + priority_distribution) / 3
    
    def _calculate_test_quality(self, test_cases: List[Dict[str, Any]]) -> float:
        """Calculate test case quality score"""
        if not test_cases:
            return 0.0
        
        quality_factors = []
        
        for test_case in test_cases:
            factors = []
            
            # Has description
            if test_case.get("description"):
                factors.append(1)
            else:
                factors.append(0)
            
            # Has assertions
            assertions = test_case.get("assertions", [])
            factors.append(min(1, len(assertions) / 3))  # Normalize to max 3 assertions
            
            # Has test steps
            steps = test_case.get("test_steps", [])
            factors.append(min(1, len(steps) / 3))  # Normalize to max 3 steps
            
            # Has priority
            priority = test_case.get("priority", 0)
            factors.append(1 if priority >= 3 else 0.5)  # Prioritized tests are better
            
            quality_factors.append(sum(factors) / len(factors))
        
        return sum(quality_factors) / len(quality_factors) * 100
    
    def _calculate_coverage_adequacy(self, test_cases: List[Dict[str, Any]]) -> float:
        """Calculate coverage adequacy score"""
        # Count high-priority tests (better coverage)
        high_priority_tests = sum(1 for tc in test_cases if tc.get("priority", 0) >= 4)
        priority_score = (high_priority_tests / len(test_cases)) * 100 if test_cases else 0
        
        # Count test with thorough assertions
        thorough_tests = sum(1 for tc in test_cases if len(tc.get("assertions", [])) >= 3)
        thoroughness_score = (thorough_tests / len(test_cases)) * 100 if test_cases else 0
        
        return (priority_score + thoroughness_score) / 2
    
    def _generate_coverage_recommendations(self, test_cases: List[Dict[str, Any]], 
                                         profile: Dict[str, Any]) -> List[str]:
        """Generate coverage improvement recommendations"""
        recommendations = []
        
        # Analyze coverage gaps
        gaps = self._analyze_coverage_gaps(test_cases, profile)
        
        if gaps["missing_components"]:
            recommendations.append(
                f"Add tests for missing components: {', '.join(gaps['missing_components'])}"
            )
        
        if gaps["missing_edge_cases"]:
            recommendations.append(
                f"Cover missing edge cases: {', '.join(gaps['missing_edge_cases'][:5])}"
            )
        
        if gaps["low_coverage_areas"]:
            recommendations.append(
                f"Improve coverage in low-coverage areas: {', '.join(gaps['low_coverage_areas'])}"
            )
        
        # Check test diversity
        test_types = set(tc.get("type", "") for tc in test_cases)
        if len(test_types) < 5:
            recommendations.append(
                f"Increase test type diversity (currently: {', '.join(test_types)})"
            )
        
        # Check priority distribution
        priorities = [tc.get("priority", 3) for tc in test_cases]
        if max(priorities) - min(priorities) < 2:
            recommendations.append("Add tests with different priority levels")
        
        return recommendations
    
    def _calculate_overall_coverage(self, component_analysis: Dict[str, Any]) -> float:
        """Calculate overall coverage across components"""
        if not component_analysis:
            return 0.0
        
        total_coverage = 0
        component_count = len(component_analysis)
        
        for component, analysis in component_analysis.items():
            metrics = analysis.get("coverage_metrics", {})
            if metrics:
                # Use component coverage as primary metric
                component_coverage = metrics.get("component_coverage", {})
                if isinstance(component_coverage, dict):
                    coverage = component_coverage.get("coverage_percentage", 0)
                else:
                    coverage = component_coverage.coverage_percentage
                total_coverage += coverage
        
        return total_coverage / component_count if component_count > 0 else 0
    
    def _aggregate_coverage_metrics(self, component_analysis: Dict[str, Any]) -> List[CoverageMetric]:
        """Aggregate coverage metrics across all components"""
        # This would aggregate metrics from all components
        # For now, return empty list - implementation would be more complex
        return []
    
    def _analyze_cross_component_gaps(self, component_analysis: Dict[str, Any]) -> Dict[str, List[str]]:
        """Analyze gaps across components"""
        gaps = {
            "integration_gaps": [],
            "missing_test_types": [],
            "performance_gaps": [],
            "security_gaps": []
        }
        
        # Check for missing integration tests
        components = list(component_analysis.keys())
        if len(components) > 1:
            gaps["integration_gaps"].append(
                f"Missing integration tests between: {', '.join(components)}"
            )
        
        # Check for missing test types across system
        all_test_types = set()
        for analysis in component_analysis.values():
            # This would need to track test types per component
            pass
        
        return gaps
    
    def _generate_system_wide_recommendations(self, component_analysis: Dict[str, Any]) -> List[str]:
        """Generate system-wide recommendations"""
        recommendations = []
        
        # Analyze overall coverage
        overall_coverage = self._calculate_overall_coverage(component_analysis)
        
        if overall_coverage < 70:
            recommendations.append(f"Overall coverage is {overall_coverage:.1f}%, aim for 80%+")
        
        # Check for consistent coverage across components
        coverages = []
        for analysis in component_analysis.values():
            metrics = analysis.get("coverage_metrics", {})
            if metrics:
                component_coverage = metrics.get("component_coverage", {})
                if isinstance(component_coverage, dict):
                    coverage = component_coverage.get("coverage_percentage", 0)
                else:
                    coverage = component_coverage.coverage_percentage
                coverages.append(coverage)
        
        if coverages and max(coverages) - min(coverages) > 30:
            recommendations.append(
                "High variance in coverage across components - standardize testing"
            )
        
        return recommendations
    
    def _calculate_system_effectiveness(self, component_analysis: Dict[str, Any]) -> Dict[str, float]:
        """Calculate system-wide effectiveness scores"""
        effectiveness = {}
        
        for component, analysis in component_analysis.items():
            effectiveness[component] = analysis.get("effectiveness_score", 0)
        
        # Overall effectiveness
        if effectiveness:
            effectiveness["overall"] = sum(effectiveness.values()) / len(effectiveness)
        
        return effectiveness
    
    def _analyze_coverage_by_type(self, component_analysis: Dict[str, Any]) -> Dict[str, float]:
        """Analyze coverage broken down by test type"""
        coverage_by_type = {
            "edge_case": 0,
            "fuzz_test": 0,
            "property_based": 0,
            "performance": 0,
            "api_compliance": 0,
            "memory_safety": 0
        }
        
        # This would analyze test types across all components
        # For now, return default values
        return coverage_by_type

# Import pandas for timestamp functionality
try:
    import pandas as pd
    HAS_PANDAS = True
except ImportError:
    HAS_PANDAS = False
    # Fallback for timestamp
    from datetime import datetime
    pd = type('pd', (), {'Timestamp': datetime})()