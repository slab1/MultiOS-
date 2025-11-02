"""
Code Quality Analysis and Educational Suggestions System
Provides automated code review and educational feedback
"""

import re
import ast
import json
from typing import Dict, List, Optional, Tuple, Set
from datetime import datetime
from dataclasses import dataclass
from enum import Enum
import math


class QualityLevel(Enum):
    EXCELLENT = "excellent"
    GOOD = "good"
    NEEDS_IMPROVEMENT = "needs_improvement"
    POOR = "poor"


@dataclass
class QualityIssue:
    """Represents a code quality issue"""
    type: str  # complexity, style, bug_risk, security, etc.
    severity: QualityLevel
    message: str
    suggestion: str
    line_number: Optional[int] = None
    file_path: str = ""
    educational_note: str = ""
    
    def to_dict(self) -> Dict:
        return {
            'type': self.type,
            'severity': self.severity.value,
            'message': self.message,
            'suggestion': self.suggestion,
            'line_number': self.line_number,
            'file_path': self.file_path,
            'educational_note': self.educational_note
        }


@dataclass
class QualityReport:
    """Complete quality analysis report"""
    file_path: str
    overall_score: float
    issues: List[QualityIssue]
    strengths: List[str]
    suggestions: List[str]
    educational_feedback: Dict
    analyzed_at: datetime
    
    def to_dict(self) -> Dict:
        return {
            'file_path': self.file_path,
            'overall_score': self.overall_score,
            'issues': [issue.to_dict() for issue in self.issues],
            'strengths': self.strengths,
            'suggestions': self.suggestions,
            'educational_feedback': self.educational_feedback,
            'analyzed_at': self.analyzed_at.isoformat()
        }


class CodeQualityAnalyzer:
    """Educational-focused code quality analyzer"""
    
    def __init__(self):
        self.quality_rules = self._initialize_quality_rules()
        self.educational_hints = self._initialize_educational_hints()
    
    def _initialize_quality_rules(self) -> Dict:
        """Initialize code quality rules and checks"""
        return {
            'complexity': {
                'max_function_length': 50,  # lines
                'max_class_length': 300,    # lines
                'max_function_complexity': 10,  # cyclomatic complexity
                'max_nesting_level': 4
            },
            'style': {
                'max_line_length': 80,
                'naming_conventions': {
                    'function': r'^[a-z_][a-z0-9_]*$',
                    'class': r'^[A-Z][a-zA-Z0-9]*$',
                    'variable': r'^[a-z_][a-z0-9_]*$',
                    'constant': r'^[A-Z_][A-Z0-9_]*$'
                }
            },
            'documentation': {
                'min_docstring_coverage': 0.8,
                'required_sections': ['description', 'parameters', 'returns']
            },
            'security': {
                'dangerous_functions': ['eval', 'exec', 'input'],
                'sql_injection_risk': True,
                'hardcoded_secrets': True
            }
        }
    
    def _initialize_educational_hints(self) -> Dict:
        """Initialize educational hints for different quality issues"""
        return {
            'complexity': {
                'high_complexity': "Complex functions are harder to understand and maintain. Consider breaking them into smaller, focused functions.",
                'long_function': "Long functions are difficult to test and debug. Try to extract logical chunks into separate functions.",
                'deep_nesting': "Deep nesting makes code hard to follow. Consider using early returns or extracting nested logic."
            },
            'style': {
                'long_line': "Long lines reduce readability. Break long statements into multiple lines.",
                'poor_naming': "Clear variable names make code self-documenting. Use descriptive names that explain the purpose."
            },
            'documentation': {
                'missing_docstring': "Docstrings help others (and future you) understand what your code does.",
                'incomplete_docstring': "Complete docstrings with parameter types and return values are more useful."
            },
            'security': {
                'dangerous_function': "Using eval() or exec() can be dangerous. Find alternative approaches.",
                'hardcoded_secret': "Never hardcode passwords or API keys. Use environment variables or config files."
            }
        }
    
    def analyze_python_code(self, code: str, file_path: str) -> QualityReport:
        """Analyze Python code for quality issues"""
        issues = []
        strengths = []
        suggestions = []
        
        try:
            # Parse the code
            tree = ast.parse(code)
            lines = code.splitlines()
            
            # Run various analyses
            issues.extend(self._analyze_complexity(tree, lines, file_path))
            issues.extend(self._analyze_style(lines, file_path))
            issues.extend(self._analyze_documentation(tree, file_path))
            issues.extend(self._analyze_security(tree, lines, file_path))
            issues.extend(self._analyze_best_practices(tree, file_path))
            
            # Collect strengths
            strengths = self._identify_strengths(tree, lines)
            
            # Generate suggestions
            suggestions = self._generate_suggestions(issues)
            
        except SyntaxError as e:
            issues.append(QualityIssue(
                type='syntax',
                severity=QualityLevel.POOR,
                message=f"Syntax error: {str(e)}",
                suggestion="Fix the syntax error before analyzing code quality",
                line_number=e.lineno,
                file_path=file_path,
                educational_note="Syntax errors prevent Python from understanding your code."
            ))
        except Exception as e:
            issues.append(QualityIssue(
                type='analysis_error',
                severity=QualityLevel.POOR,
                message=f"Analysis error: {str(e)}",
                suggestion="Please check your code for syntax errors",
                file_path=file_path,
                educational_note="The analyzer encountered an unexpected issue."
            ))
        
        # Calculate overall score
        overall_score = self._calculate_quality_score(issues)
        
        # Generate educational feedback
        educational_feedback = self._generate_educational_feedback(issues, strengths)
        
        return QualityReport(
            file_path=file_path,
            overall_score=overall_score,
            issues=issues,
            strengths=strengths,
            suggestions=suggestions,
            educational_feedback=educational_feedback,
            analyzed_at=datetime.now()
        )
    
    def _analyze_complexity(self, tree: ast.AST, lines: List[str], file_path: str) -> List[QualityIssue]:
        """Analyze code complexity"""
        issues = []
        
        class ComplexityAnalyzer(ast.NodeVisitor):
            def visit_FunctionDef(self, node):
                # Check function length
                if node.end_lineno and node.lineno:
                    function_length = node.end_lineno - node.lineno + 1
                    if function_length > self.max_function_length:
                        issues.append(QualityIssue(
                            type='complexity',
                            severity=QualityLevel.NEEDS_IMPROVEMENT,
                            message=f"Function '{node.name}' is too long ({function_length} lines)",
                            suggestion=f"Consider breaking this function into smaller functions (max {self.max_function_length} lines)",
                            line_number=node.lineno,
                            file_path=file_path,
                            educational_note=self.educational_hints['complexity']['long_function']
                        ))
                
                # Check cyclomatic complexity
                complexity = self._calculate_cyclomatic_complexity(node)
                if complexity > self.max_function_complexity:
                    issues.append(QualityIssue(
                        type='complexity',
                        severity=QualityLevel.NEEDS_IMPROVEMENT,
                        message=f"Function '{node.name}' has high complexity ({complexity})",
                        suggestion=f"Simplify the function logic (max complexity: {self.max_function_complexity})",
                        line_number=node.lineno,
                        file_path=file_path,
                        educational_note=self.educational_hints['complexity']['high_complexity']
                    ))
                
                self.generic_visit(node)
            
            def visit_ClassDef(self, node):
                # Check class length
                if node.end_lineno and node.lineno:
                    class_length = node.end_lineno - node.lineno + 1
                    if class_length > self.max_class_length:
                        issues.append(QualityIssue(
                            type='complexity',
                            severity=QualityLevel.NEEDS_IMPROVEMENT,
                            message=f"Class '{node.name}' is very long ({class_length} lines)",
                            suggestion=f"Consider splitting this class into smaller, focused classes (max {self.max_class_length} lines)",
                            line_number=node.lineno,
                            file_path=file_path,
                            educational_note="Large classes are hard to understand and maintain. Consider the Single Responsibility Principle."
                        ))
                
                self.generic_visit(node)
            
            def visit_If(self, node):
                # Check nesting level
                nesting_level = self._get_nesting_level(node)
                if nesting_level > self.max_nesting_level:
                    issues.append(QualityIssue(
                        type='complexity',
                        severity=QualityLevel.NEEDS_IMPROVEMENT,
                        message=f"Deep nesting detected (level {nesting_level})",
                        suggestion="Reduce nesting by using early returns or extracting logic",
                        line_number=node.lineno,
                        file_path=file_path,
                        educational_note=self.educational_hints['complexity']['deep_nesting']
                    ))
                
                self.generic_visit(node)
            
            def _calculate_cyclomatic_complexity(self, node):
                """Calculate cyclomatic complexity of a function"""
                complexity = 1  # Base complexity
                for child in ast.walk(node):
                    if isinstance(child, (ast.If, ast.While, ast.For, ast.With, ast.ExceptHandler)):
                        complexity += 1
                    elif isinstance(child, ast.BoolOp):
                        complexity += len(child.values) - 1
                return complexity
            
            def _get_nesting_level(self, node, level=0):
                """Get nesting level of a node"""
                max_level = level
                for child in ast.iter_child_nodes(node):
                    if isinstance(child, (ast.If, ast.While, ast.For, ast.With)):
                        max_level = max(max_level, self._get_nesting_level(child, level + 1))
                return max_level
        
        analyzer = ComplexityAnalyzer()
        analyzer.max_function_length = self.quality_rules['complexity']['max_function_length']
        analyzer.max_class_length = self.quality_rules['complexity']['max_class_length']
        analyzer.max_function_complexity = self.quality_rules['complexity']['max_function_complexity']
        analyzer.max_nesting_level = self.quality_rules['complexity']['max_nesting_level']
        
        analyzer.visit(tree)
        return analyzer.issues if hasattr(analyzer, 'issues') else []
    
    def _analyze_style(self, lines: List[str], file_path: str) -> List[QualityIssue]:
        """Analyze code style"""
        issues = []
        naming_conventions = self.quality_rules['style']['naming_conventions']
        
        for i, line in enumerate(lines, 1):
            # Check line length
            if len(line) > self.quality_rules['style']['max_line_length']:
                issues.append(QualityIssue(
                    type='style',
                    severity=QualityLevel.NEEDS_IMPROVEMENT,
                    message=f"Line {i} is too long ({len(line)} characters)",
                    suggestion=f"Break line into multiple lines (max {self.quality_rules['style']['max_line_length']} characters)",
                    line_number=i,
                    file_path=file_path,
                    educational_note=self.educational_hints['style']['long_line']
                ))
        
        return issues
    
    def _analyze_documentation(self, tree: ast.AST, file_path: str) -> List[QualityIssue]:
        """Analyze documentation coverage"""
        issues = []
        
        class DocumentationAnalyzer(ast.NodeVisitor):
            def visit_FunctionDef(self, node):
                if not ast.get_docstring(node):
                    issues.append(QualityIssue(
                        type='documentation',
                        severity=QualityLevel.NEEDS_IMPROVEMENT,
                        message=f"Function '{node.name}' missing docstring",
                        suggestion="Add a docstring explaining what the function does",
                        line_number=node.lineno,
                        file_path=file_path,
                        educational_note=self.educational_hints['documentation']['missing_docstring']
                    ))
                
                self.generic_visit(node)
            
            def visit_ClassDef(self, node):
                if not ast.get_docstring(node):
                    issues.append(QualityIssue(
                        type='documentation',
                        severity=QualityLevel.NEEDS_IMPROVEMENT,
                        message=f"Class '{node.name}' missing docstring",
                        suggestion="Add a docstring explaining what the class represents",
                        line_number=node.lineno,
                        file_path=file_path,
                        educational_note=self.educational_hints['documentation']['missing_docstring']
                    ))
                
                self.generic_visit(node)
        
        analyzer = DocumentationAnalyzer()
        analyzer.visit(tree)
        return analyzer.issues if hasattr(analyzer, 'issues') else []
    
    def _analyze_security(self, tree: ast.AST, lines: List[str], file_path: str) -> List[QualityIssue]:
        """Analyze for security issues"""
        issues = []
        dangerous_functions = self.quality_rules['security']['dangerous_functions']
        
        for i, line in enumerate(lines, 1):
            # Check for dangerous functions
            for func in dangerous_functions:
                if re.search(rf'\b{func}\s*\(', line):
                    issues.append(QualityIssue(
                        type='security',
                        severity=QualityLevel.NEEDS_IMPROVEMENT,
                        message=f"Potentially dangerous function '{func}' used on line {i}",
                        suggestion=f"Avoid using {func}() for security reasons",
                        line_number=i,
                        file_path=file_path,
                        educational_note=self.educational_hints['security']['dangerous_function']
                    ))
            
            # Check for hardcoded secrets
            if re.search(r'(password|api_key|secret|token)\s*=\s*["\'][^"\']+["\']', line, re.IGNORECASE):
                issues.append(QualityIssue(
                    type='security',
                    severity=QualityLevel.POOR,
                    message=f"Hardcoded secret detected on line {i}",
                    suggestion="Use environment variables or configuration files for secrets",
                    line_number=i,
                    file_path=file_path,
                    educational_note=self.educational_hints['security']['hardcoded_secret']
                ))
        
        return issues
    
    def _analyze_best_practices(self, tree: ast.AST, file_path: str) -> List[QualityIssue]:
        """Analyze for general best practices"""
        issues = []
        
        # Look for common anti-patterns
        for node in ast.walk(tree):
            if isinstance(node, ast.For):
                # Check for 'for i in range(len(...))' pattern
                if isinstance(node.target, ast.Name) and node.target.id == 'i':
                    if isinstance(node.iter, ast.Call) and isinstance(node.iter.func, ast.Name):
                        if node.iter.func.id == 'len':
                            issues.append(QualityIssue(
                                type='best_practice',
                                severity=QualityLevel.NEEDS_IMPROVEMENT,
                                message="Avoid 'for i in range(len(...))' pattern",
                                suggestion="Use enumerate() or direct iteration instead",
                                line_number=node.lineno,
                                file_path=file_path,
                                educational_note="Using enumerate() makes your code more readable and Pythonic."
                            ))
        
        return issues
    
    def _identify_strengths(self, tree: ast.AST, lines: List[str]) -> List[str]:
        """Identify code strengths"""
        strengths = []
        
        # Check for good documentation
        docstring_count = 0
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
                if ast.get_docstring(node):
                    docstring_count += 1
        
        total_functions_classes = sum(1 for node in ast.walk(tree) 
                                    if isinstance(node, (ast.FunctionDef, ast.ClassDef)))
        
        if total_functions_classes > 0 and docstring_count / total_functions_classes > 0.7:
            strengths.append("Good documentation coverage")
        
        # Check for type hints
        type_hint_count = 0
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                if node.returns or any(arg.annotation for arg in node.args.args):
                    type_hint_count += 1
        
        if type_hint_count > 0:
            strengths.append("Uses type hints for better code clarity")
        
        # Check for reasonable function lengths
        short_functions = 0
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                if node.end_lineno and node.lineno:
                    length = node.end_lineno - node.lineno + 1
                    if length < 20:
                        short_functions += 1
        
        if short_functions > 0:
            strengths.append("Uses appropriately sized functions")
        
        return strengths
    
    def _generate_suggestions(self, issues: List[QualityIssue]) -> List[str]:
        """Generate improvement suggestions based on issues"""
        suggestions = []
        
        issue_counts = {}
        for issue in issues:
            issue_counts[issue.type] = issue_counts.get(issue.type, 0) + 1
        
        # Prioritize suggestions based on severity
        if 'complexity' in issue_counts:
            suggestions.append("Consider refactoring complex functions into smaller, focused ones")
        
        if 'documentation' in issue_counts:
            suggestions.append("Add docstrings to improve code documentation")
        
        if 'security' in issue_counts:
            suggestions.append("Review security concerns and fix potential vulnerabilities")
        
        if 'style' in issue_counts:
            suggestions.append("Improve code style for better readability")
        
        return suggestions
    
    def _calculate_quality_score(self, issues: List[QualityIssue]) -> float:
        """Calculate overall quality score (0-100)"""
        if not issues:
            return 100.0
        
        # Weight issues by severity
        severity_weights = {
            QualityLevel.POOR: 10,
            QualityLevel.NEEDS_IMPROVEMENT: 5,
            QualityLevel.GOOD: 2,
            QualityLevel.EXCELLENT: 0
        }
        
        total_penalty = sum(severity_weights.get(issue.severity, 0) for issue in issues)
        score = max(0, 100 - total_penalty)
        
        return score
    
    def _generate_educational_feedback(self, issues: List[QualityIssue], 
                                     strengths: List[str]) -> Dict:
        """Generate educational feedback for learning"""
        feedback = {
            'learning_objectives': [],
            'priority_fixes': [],
            'concepts_to_review': [],
            'progress_highlights': []
        }
        
        # Identify learning objectives based on issues
        issue_types = set(issue.type for issue in issues)
        
        if 'complexity' in issue_types:
            feedback['learning_objectives'].append('Software Design Principles')
            feedback['concepts_to_review'].append('Function decomposition and code organization')
        
        if 'documentation' in issue_types:
            feedback['learning_objectives'].append('Technical Communication')
            feedback['concepts_to_review'].append('Writing effective docstrings and comments')
        
        if 'security' in issue_types:
            feedback['learning_objectives'].append('Secure Coding Practices')
            feedback['concepts_to_review'].append('Common security vulnerabilities and prevention')
        
        # Priority fixes
        severe_issues = [issue for issue in issues 
                        if issue.severity in [QualityLevel.POOR, QualityLevel.NEEDS_IMPROVEMENT]]
        feedback['priority_fixes'] = [issue.message for issue in severe_issues[:3]]
        
        # Progress highlights
        feedback['progress_highlights'] = strengths
        
        return feedback
    
    def get_recommended_resources(self, issues: List[QualityIssue]) -> Dict[str, List[str]]:
        """Get recommended learning resources based on issues"""
        resources = {
            'books': [],
            'articles': [],
            'courses': [],
            'practice_exercises': []
        }
        
        issue_types = set(issue.type for issue in issues)
        
        if 'complexity' in issue_types:
            resources['books'].append("Clean Code by Robert Martin")
            resources['articles'].append("https://refactoring.guru/design-patterns")
            resources['courses'].append("Software Design and Architecture")
        
        if 'documentation' in issue_types:
            resources['articles'].append("https://realpython.com/documenting-python-code/")
            resources['practice_exercises'].append("Write docstrings for existing code")
        
        if 'security' in issue_types:
            resources['courses'].append("Secure Coding Practices")
            resources['articles'].append("https://owasp.org/www-project-top-ten/")
        
        return resources
