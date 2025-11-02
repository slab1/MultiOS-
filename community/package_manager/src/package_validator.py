"""
Package Validator Module
========================

Validates educational packages for structure, curriculum integration,
automated testing compliance, and quality standards.
"""

import os
import json
import logging
import subprocess
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any, Set
from dataclasses import dataclass
from datetime import datetime
import re
import yaml

from package_manager import PackageMetadata, PackageType, CompatibilityLevel

logger = logging.getLogger(__name__)


@dataclass
class ValidationIssue:
    """Represents a validation issue"""
    severity: str  # error, warning, info
    type: str      # structure, curriculum, metadata, testing, compatibility
    message: str
    file_path: Optional[str] = None
    line_number: Optional[int] = None
    suggestion: Optional[str] = None


@dataclass
class ValidationResult:
    """Results of package validation"""
    package_name: str
    package_version: str
    validation_timestamp: str
    passed: bool
    issues: List[ValidationIssue]
    warnings: List[ValidationIssue]
    info: List[ValidationIssue]
    test_results: Dict[str, Any]
    curriculum_compliance: Dict[str, Any]
    quality_score: float
    recommendations: List[str]


class PackageValidator:
    """Comprehensive validator for educational packages"""
    
    def __init__(self, package_manager):
        self.pm = package_manager
        
        # Standard curriculum frameworks
        self.curriculum_frameworks = {
            'common_core': self._load_curriculum_standards('common_core'),
            'ngss': self._load_curriculum_standards('ngss'),  # Next Generation Science Standards
            'iste': self._load_curriculum_standards('iste'),  # ISTE Standards
            'csta': self._load_curriculum_standards('csta'),  # Computer Science Teachers Association
            'math': self._load_curriculum_standards('math')
        }
        
        # Quality standards
        self.quality_standards = {
            'documentation_required': True,
            'tests_required': True,
            'examples_required': True,
            'accessibility_required': True,
            'localization_support': True
        }
        
    def validate_package(self, package_dir: str, metadata: PackageMetadata) -> ValidationResult:
        """Perform comprehensive package validation"""
        logger.info(f"Validating package: {metadata.name}")
        validation_start = datetime.now()
        
        try:
            issues = []
            warnings = []
            info_messages = []
            test_results = {}
            curriculum_compliance = {}
            
            # 1. Structure validation
            structure_issues = self._validate_package_structure(package_dir, metadata)
            issues.extend(structure_issues)
            
            # 2. Metadata validation
            metadata_issues = self._validate_metadata(metadata)
            issues.extend(metadata_issues)
            
            # 3. Content validation
            content_issues = self._validate_content(package_dir, metadata)
            issues.extend(content_issues)
            
            # 4. Curriculum integration validation
            curriculum_results = self._validate_curriculum_integration(package_dir, metadata)
            curriculum_compliance = curriculum_results
            issues.extend(curriculum_results.get('issues', []))
            warnings.extend(curriculum_results.get('warnings', []))
            
            # 5. Testing validation
            test_results = self._validate_testing_compliance(package_dir, metadata)
            issues.extend(test_results.get('issues', []))
            warnings.extend(test_results.get('warnings', []))
            
            # 6. Accessibility validation
            accessibility_issues = self._validate_accessibility(package_dir, metadata)
            issues.extend(accessibility_issues.get('errors', []))
            warnings.extend(accessibility_issues.get('warnings', []))
            
            # 7. Quality standards validation
            quality_issues = self._validate_quality_standards(package_dir, metadata)
            issues.extend(quality_issues.get('errors', []))
            warnings.extend(quality_issues.get('warnings', []))
            
            # Calculate overall quality score
            quality_score = self._calculate_quality_score(issues, warnings, test_results)
            
            # Determine if validation passed
            passed = len(issues) == 0
            
            validation_duration = (datetime.now() - validation_start).total_seconds()
            
            # Generate recommendations
            recommendations = self._generate_validation_recommendations(
                issues, warnings, curriculum_compliance, test_results
            )
            
            # Log results
            if passed:
                logger.info(f"Validation passed for {metadata.name} (quality score: {quality_score:.2f})")
            else:
                logger.warning(f"Validation failed for {metadata.name}: {len(issues)} issues, {len(warnings)} warnings")
            
            return ValidationResult(
                package_name=metadata.name,
                package_version=metadata.version,
                validation_timestamp=datetime.now().isoformat(),
                passed=passed,
                issues=issues,
                warnings=warnings,
                info=info_messages,
                test_results=test_results,
                curriculum_compliance=curriculum_compliance,
                quality_score=quality_score,
                recommendations=recommendations
            )
            
        except Exception as e:
            logger.error(f"Error during package validation: {e}")
            return ValidationResult(
                package_name=metadata.name,
                package_version=metadata.version,
                validation_timestamp=datetime.now().isoformat(),
                passed=False,
                issues=[ValidationIssue('error', 'validation_error', f'Validation failed: {e}')],
                warnings=[],
                info=[],
                test_results={},
                curriculum_compliance={},
                quality_score=0.0,
                recommendations=['Fix validation errors and retry']
            )
    
    def validate_package_structure(self, package_dir: str, metadata: PackageMetadata) -> List[ValidationIssue]:
        """Validate package directory structure"""
        issues = []
        
        package_path = Path(package_dir)
        
        # Required directories
        required_dirs = ['src', 'tests', 'docs']
        if metadata.type in [PackageType.CURRICULUM, PackageType.TUTORIAL]:
            required_dirs.extend(['curriculum', 'resources'])
        if metadata.type in [PackageType.SIMULATION, PackageType.INTERACTIVE]:
            required_dirs.extend(['assets', 'configs'])
        
        for required_dir in required_dirs:
            if not (package_path / required_dir).exists():
                issues.append(ValidationIssue(
                    'error', 'structure', f"Required directory missing: {required_dir}"
                ))
        
        # Required files
        required_files = ['README.md', 'LICENSE']
        for required_file in required_files:
            if not (package_path / required_file).exists():
                issues.append(ValidationIssue(
                    'error', 'structure', f"Required file missing: {required_file}"
                ))
        
        # Check file organization
        src_path = package_path / 'src'
        if src_path.exists():
            python_files = list(src_path.rglob("*.py"))
            if python_files and not (src_path / '__init__.py').exists():
                issues.append(ValidationIssue(
                    'warning', 'structure', "Python package missing __init__.py files"
                ))
        
        # Check for documentation files
        docs_path = package_path / 'docs'
        if docs_path.exists():
            doc_files = list(docs_path.glob("*.md")) + list(docs_path.glob("*.rst"))
            if len(doc_files) < 2:
                issues.append(ValidationIssue(
                    'warning', 'structure', "Insufficient documentation files in docs/"
                ))
        
        return issues
    
    def validate_curriculum_integration(self, package_dir: str, metadata: PackageMetadata) -> Dict[str, Any]:
        """Validate curriculum standards integration"""
        results = {
            'passed': True,
            'issues': [],
            'warnings': [],
            'compliance_score': 0.0,
            'framework_compliance': {},
            'learning_objectives': {},
            'assessment_methods': []
        }
        
        if metadata.type not in [PackageType.CURRICULUM, PackageType.TUTORIAL, PackageType.ASSESSMENT]:
            return results
        
        package_path = Path(package_dir)
        
        # Check for curriculum manifest
        curriculum_file = package_path / 'curriculum' / 'curriculum.yaml'
        if not curriculum_file.exists():
            results['issues'].append(ValidationIssue(
                'error', 'curriculum', 'Curriculum manifest (curriculum.yaml) not found'
            ))
            results['passed'] = False
            return results
        
        try:
            with open(curriculum_file, 'r') as f:
                curriculum_data = yaml.safe_load(f)
            
            # Validate curriculum structure
            required_fields = ['learning_objectives', 'standards', 'assessments']
            for field in required_fields:
                if field not in curriculum_data:
                    results['issues'].append(ValidationIssue(
                        'error', 'curriculum', f"Curriculum field missing: {field}"
                    ))
            
            # Check standards alignment
            if 'standards' in curriculum_data:
                standards_issues = self._validate_standards_alignment(curriculum_data['standards'])
                results['issues'].extend(standards_issues.get('errors', []))
                results['warnings'].extend(standards_issues.get('warnings', []))
                
                # Calculate compliance score
                total_standards = len(curriculum_data['standards'])
                aligned_standards = total_standards - len(standards_issues.get('errors', []))
                results['compliance_score'] = (aligned_standards / total_standards) if total_standards > 0 else 0.0
            
            # Validate learning objectives
            if 'learning_objectives' in curriculum_data:
                objectives_issues = self._validate_learning_objectives(curriculum_data['learning_objectives'])
                results['issues'].extend(objectives_issues.get('errors', []))
                results['warnings'].extend(objectives_issues.get('warnings', []))
            
            # Check assessment methods
            if 'assessments' in curriculum_data:
                assessment_issues = self._validate_assessments(curriculum_data['assessments'])
                results['issues'].extend(assessment_issues.get('errors', []))
                results['warnings'].extend(assessment_issues.get('warnings', []))
            
        except yaml.YAMLError as e:
            results['issues'].append(ValidationIssue(
                'error', 'curriculum', f"Invalid YAML in curriculum file: {e}"
            ))
        except Exception as e:
            results['issues'].append(ValidationIssue(
                'error', 'curriculum', f"Error reading curriculum file: {e}"
            ))
        
        if results['issues']:
            results['passed'] = False
        
        return results
    
    def validate_testing_compliance(self, package_dir: str, metadata: PackageMetadata) -> Dict[str, Any]:
        """Validate automated testing compliance"""
        results = {
            'passed': True,
            'issues': [],
            'warnings': [],
            'test_coverage': 0.0,
            'test_results': {},
            'recommendations': []
        }
        
        if not self.pm.config["validation"]["run_tests"]:
            return results
        
        package_path = Path(package_dir)
        tests_path = package_path / 'tests'
        
        # Check for test directory
        if not tests_path.exists():
            results['warnings'].append(ValidationIssue(
                'warning', 'testing', 'Tests directory not found'
            ))
            return results
        
        # Check for test files
        test_files = list(tests_path.glob("test_*.py"))
        if not test_files:
            results['warnings'].append(ValidationIssue(
                'warning', 'testing', 'No test files found (expected test_*.py pattern)'
            ))
            return results
        
        # Run tests if pytest is available
        test_executable = shutil.which('pytest')
        if test_executable:
            results['test_results'] = self._run_pytest_tests(str(tests_path))
            if not results['test_results'].get('passed', False):
                results['issues'].append(ValidationIssue(
                    'error', 'testing', f"Tests failed: {results['test_results'].get('error', 'Unknown error')}"
                ))
        
        # Check test coverage
        results['test_coverage'] = self._calculate_test_coverage(package_path, tests_path)
        
        if results['test_coverage'] < 70:
            results['warnings'].append(ValidationIssue(
                'warning', 'testing', f"Low test coverage: {results['test_coverage']:.1f}% (recommended: >= 70%)"
            ))
        
        # Quality checks for tests
        quality_issues = self._validate_test_quality(test_files)
        results['warnings'].extend(quality_issues)
        
        return results
    
    def _validate_package_structure(self, package_dir: str, metadata: PackageMetadata) -> List[ValidationIssue]:
        """Validate package structure (delegated to main method)"""
        return self.validate_package_structure(package_dir, metadata)
    
    def _validate_metadata(self, metadata: PackageMetadata) -> List[ValidationIssue]:
        """Validate package metadata"""
        issues = []
        
        # Check required fields
        required_fields = ['name', 'version', 'description', 'author', 'email', 'type', 'compatibility']
        for field in required_fields:
            if not getattr(metadata, field):
                issues.append(ValidationIssue(
                    'error', 'metadata', f"Required metadata field missing: {field}"
                ))
        
        # Validate email format
        if metadata.email and '@' not in metadata.email:
            issues.append(ValidationIssue(
                'error', 'metadata', f"Invalid email format: {metadata.email}"
            ))
        
        # Validate version format
        version_pattern = r'^\d+\.\d+(\.\d+)?([a-zA-Z]+\d*)?$'
        if not re.match(version_pattern, metadata.version):
            issues.append(ValidationIssue(
                'warning', 'metadata', f"Non-standard version format: {metadata.version}"
            ))
        
        # Check dependencies
        if len(metadata.dependencies) > self.pm.config["limits"]["max_dependencies"]:
            issues.append(ValidationIssue(
                'warning', 'metadata', f"Too many dependencies ({len(metadata.dependencies)}), consider reducing"
            ))
        
        # Validate subjects
        if not metadata.subjects:
            issues.append(ValidationIssue(
                'warning', 'metadata', "No subjects specified"
            ))
        
        # Validate grade levels
        valid_grades = [f"Grade {i}" for i in range(1, 13)] + ["College", "Adult"]
        for grade in metadata.grade_levels:
            if grade not in valid_grades:
                issues.append(ValidationIssue(
                    'warning', 'metadata', f"Non-standard grade level: {grade}"
                ))
        
        return issues
    
    def _validate_content(self, package_dir: str, metadata: PackageMetadata) -> List[ValidationIssue]:
        """Validate package content"""
        issues = []
        package_path = Path(package_dir)
        
        # Check README content
        readme_file = package_path / 'README.md'
        if readme_file.exists():
            with open(readme_file, 'r', encoding='utf-8') as f:
                readme_content = f.read()
            
            if len(readme_content) < 100:
                issues.append(ValidationIssue(
                    'warning', 'content', 'README.md content is too short (recommended: >= 100 characters)'
                ))
            
            # Check for required sections
            required_sections = ['Description', 'Installation', 'Usage']
            for section in required_sections:
                if f"## {section}" not in readme_content:
                    issues.append(ValidationIssue(
                        'warning', 'content', f"README.md missing section: {section}"
                    ))
        
        # Check code quality
        src_path = package_path / 'src'
        if src_path.exists():
            code_issues = self._analyze_code_quality(src_path)
            issues.extend(code_issues)
        
        # Check for educational content based on package type
        if metadata.type in [PackageType.CURRICULUM, PackageType.TUTORIAL]:
            content_issues = self._validate_educational_content(package_path)
            issues.extend(content_issues)
        
        return issues
    
    def _validate_accessibility(self, package_dir: str, metadata: PackageMetadata) -> Dict[str, List[ValidationIssue]]:
        """Validate accessibility compliance"""
        errors = []
        warnings = []
        package_path = Path(package_dir)
        
        # Check for accessibility features in HTML/web content
        html_files = list(package_path.rglob("*.html"))
        for html_file in html_files:
            content_issues = self._check_html_accessibility(html_file)
            errors.extend(content_issues.get('errors', []))
            warnings.extend(content_issues.get('warnings', []))
        
        # Check for alt text in images
        img_files = list(package_path.rglob("*.png")) + list(package_path.rglob("*.jpg")) + list(package_path.rglob("*.jpeg"))
        if img_files:
            warnings.append(ValidationIssue(
                'warning', 'accessibility', f"Found {len(img_files)} images without explicit accessibility descriptions"
            ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _validate_quality_standards(self, package_dir: str, metadata: PackageMetadata) -> Dict[str, List[ValidationIssue]]:
        """Validate against quality standards"""
        errors = []
        warnings = []
        package_path = Path(package_dir)
        
        # Check documentation requirements
        if self.quality_standards['documentation_required']:
            docs_path = package_path / 'docs'
            if not docs_path.exists():
                errors.append(ValidationIssue(
                    'error', 'quality', 'Documentation directory is required'
                ))
            else:
                doc_files = list(docs_path.glob("*.md")) + list(docs_path.glob("*.rst"))
                if len(doc_files) < 2:
                    errors.append(ValidationIssue(
                        'error', 'quality', 'Insufficient documentation files'
                    ))
        
        # Check test requirements
        if self.quality_standards['tests_required']:
            tests_path = package_path / 'tests'
            if not tests_path.exists():
                errors.append(ValidationIssue(
                    'error', 'quality', 'Tests directory is required'
                ))
        
        # Check examples
        if self.quality_standards['examples_required']:
            examples_found = len(list(package_path.rglob("example*"))) > 0
            if not examples_found:
                warnings.append(ValidationIssue(
                    'warning', 'quality', 'No examples found (recommended)'
                ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _load_curriculum_standards(self, framework: str) -> Dict[str, Any]:
        """Load curriculum standards for a framework"""
        # This would load actual curriculum standards
        # For demonstration, return empty standards
        return {}
    
    def _validate_standards_alignment(self, standards: List[Dict]) -> Dict[str, List[ValidationIssue]]:
        """Validate standards alignment"""
        errors = []
        warnings = []
        
        # Check for valid standard codes
        for standard in standards:
            if 'code' not in standard:
                errors.append(ValidationIssue(
                    'error', 'curriculum', 'Standard missing required code field'
                ))
            if 'description' not in standard:
                errors.append(ValidationIssue(
                    'error', 'curriculum', 'Standard missing required description field'
                ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _validate_learning_objectives(self, objectives: List[Dict]) -> Dict[str, List[ValidationIssue]]:
        """Validate learning objectives"""
        errors = []
        warnings = []
        
        for obj in objectives:
            if 'objective' not in obj:
                errors.append(ValidationIssue(
                    'error', 'curriculum', 'Learning objective missing objective field'
                ))
            if 'assessment' not in obj:
                warnings.append(ValidationIssue(
                    'warning', 'curriculum', 'Learning objective missing assessment method'
                ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _validate_assessments(self, assessments: List[Dict]) -> Dict[str, List[ValidationIssue]]:
        """Validate assessment methods"""
        errors = []
        warnings = []
        
        for assessment in assessments:
            if 'type' not in assessment:
                errors.append(ValidationIssue(
                    'error', 'curriculum', 'Assessment missing type field'
                ))
            if 'criteria' not in assessment:
                warnings.append(ValidationIssue(
                    'warning', 'curriculum', 'Assessment missing evaluation criteria'
                ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _run_pytest_tests(self, tests_dir: str) -> Dict[str, Any]:
        """Run pytest tests and return results"""
        try:
            result = subprocess.run(
                ['pytest', tests_dir, '--json-report', '--json-report-file=/tmp/test_results.json'],
                capture_output=True,
                text=True,
                timeout=300
            )
            
            # Try to read test results
            if os.path.exists('/tmp/test_results.json'):
                with open('/tmp/test_results.json', 'r') as f:
                    return json.load(f)
            else:
                return {
                    'passed': result.returncode == 0,
                    'error': result.stderr if result.returncode != 0 else None
                }
                
        except subprocess.TimeoutExpired:
            return {'passed': False, 'error': 'Test execution timeout'}
        except Exception as e:
            return {'passed': False, 'error': str(e)}
    
    def _calculate_test_coverage(self, src_path: Path, tests_path: Path) -> float:
        """Calculate test coverage percentage"""
        # Simplified coverage calculation
        # In practice, would use coverage.py or similar tools
        src_files = len(list(src_path.rglob("*.py")))
        test_files = len(list(tests_path.glob("test_*.py")))
        
        if src_files == 0:
            return 0.0
        
        # Rough estimation: each test file covers ~25% of a src file
        estimated_coverage = min(100.0, (test_files * 25) / src_files)
        return estimated_coverage
    
    def _validate_test_quality(self, test_files: List[Path]) -> List[ValidationIssue]:
        """Validate test file quality"""
        warnings = []
        
        for test_file in test_files:
            try:
                with open(test_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Check for test assertions
                if 'assert' not in content:
                    warnings.append(ValidationIssue(
                        'warning', 'testing', f"Test file {test_file.name} contains no assertions"
                    ))
                
                # Check for test organization
                if 'def test_' not in content:
                    warnings.append(ValidationIssue(
                        'warning', 'testing', f"Test file {test_file.name} contains no test functions"
                    ))
                    
            except Exception:
                warnings.append(ValidationIssue(
                    'warning', 'testing', f"Could not analyze test file: {test_file.name}"
                ))
        
        return warnings
    
    def _analyze_code_quality(self, src_path: Path) -> List[ValidationIssue]:
        """Analyze code quality"""
        issues = []
        
        for py_file in src_path.rglob("*.py"):
            try:
                with open(py_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Check for TODOs
                if 'TODO' in content:
                    issues.append(ValidationIssue(
                        'info', 'content', f"Code contains TODO comments in {py_file.name}"
                    ))
                
                # Check for long lines
                lines = content.split('\n')
                long_lines = [i+1 for i, line in enumerate(lines) if len(line) > 120]
                if long_lines:
                    issues.append(ValidationIssue(
                        'info', 'content', f"Code contains {len(long_lines)} long lines in {py_file.name}"
                    ))
                    
            except Exception:
                issues.append(ValidationIssue(
                    'warning', 'content', f"Could not analyze code file: {py_file.name}"
                ))
        
        return issues
    
    def _validate_educational_content(self, package_path: Path) -> List[ValidationIssue]:
        """Validate educational content specific to curriculum packages"""
        issues = []
        
        # Check for learning materials
        learning_files = list(package_path.rglob("lesson*")) + list(package_path.rglob("activity*"))
        if not learning_files:
            issues.append(ValidationIssue(
                'warning', 'curriculum', 'No learning activities found'
            ))
        
        # Check for multimedia content
        media_files = list(package_path.rglob("*.mp4")) + list(package_path.rglob("*.mp3")) + list(package_path.rglob("*.png"))
        if not media_files:
            issues.append(ValidationIssue(
                'info', 'content', 'No multimedia content found (recommended for engagement)'
            ))
        
        return issues
    
    def _check_html_accessibility(self, html_file: Path) -> Dict[str, List[ValidationIssue]]:
        """Check HTML file for accessibility issues"""
        errors = []
        warnings = []
        
        try:
            with open(html_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Check for alt attributes on images
            if '<img' in content and 'alt=' not in content:
                errors.append(ValidationIssue(
                    'error', 'accessibility', f"Images missing alt attributes in {html_file.name}"
                ))
            
            # Check for proper heading structure
            if '<h1>' in content and '<h2>' not in content and '<h3>' not in content:
                warnings.append(ValidationIssue(
                    'warning', 'accessibility', f"Poor heading structure in {html_file.name}"
                ))
            
            # Check for color contrast indicators
            if 'style=' in content and 'color:' in content:
                warnings.append(ValidationIssue(
                    'info', 'accessibility', f"Consider using CSS for better accessibility in {html_file.name}"
                ))
                
        except Exception:
            errors.append(ValidationIssue(
                'error', 'accessibility', f"Could not analyze HTML file: {html_file.name}"
            ))
        
        return {'errors': errors, 'warnings': warnings}
    
    def _calculate_quality_score(self, issues: List[ValidationIssue], 
                               warnings: List[ValidationIssue], 
                               test_results: Dict) -> float:
        """Calculate overall quality score (0-100)"""
        score = 100.0
        
        # Penalize errors
        error_penalty = len([i for i in issues if i.severity == 'error']) * 10
        score -= error_penalty
        
        # Penalize warnings
        warning_penalty = len([i for i in issues if i.severity == 'warning']) * 5
        score -= warning_penalty
        
        # Penalize failed tests
        if not test_results.get('passed', True):
            score -= 20
        
        # Bonus for test coverage
        coverage = test_results.get('test_coverage', 0)
        if coverage >= 80:
            score += 10
        elif coverage >= 60:
            score += 5
        
        return max(0.0, min(100.0, score))
    
    def _generate_validation_recommendations(self, issues: List[ValidationIssue], 
                                           warnings: List[ValidationIssue],
                                           curriculum_compliance: Dict,
                                           test_results: Dict) -> List[str]:
        """Generate validation recommendations"""
        recommendations = []
        
        # Error-based recommendations
        error_types = [issue.type for issue in issues if issue.severity == 'error']
        
        if 'structure' in error_types:
            recommendations.append("Review and fix package structure issues")
        
        if 'metadata' in error_types:
            recommendations.append("Complete and validate package metadata")
        
        if 'curriculum' in error_types:
            recommendations.append("Review curriculum standards alignment")
        
        if 'testing' in error_types:
            recommendations.append("Fix failing tests and improve test coverage")
        
        # Warning-based recommendations
        warning_types = [issue.type for issue in warnings if issue.severity == 'warning']
        
        if 'content' in warning_types:
            recommendations.append("Improve content documentation and examples")
        
        if 'accessibility' in warning_types:
            recommendations.append("Enhance accessibility features")
        
        # Quality score recommendations
        quality_score = self._calculate_quality_score(issues, warnings, test_results)
        if quality_score < 70:
            recommendations.append("Improve overall package quality (current score below 70)")
        
        return recommendations