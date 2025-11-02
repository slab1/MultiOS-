"""
Dependency Resolution Module
============================

Handles complex dependency resolution for educational packages with
conflict detection, version constraints, and compatibility checking.
"""

import json
import logging
from typing import Dict, List, Set, Optional, Tuple
from pathlib import Path
from dataclasses import dataclass
from enum import Enum
import re

from package_manager import PackageMetadata, PackageType, CompatibilityLevel

logger = logging.getLogger(__name__)


@dataclass
class DependencyNode:
    """Represents a package dependency in the resolution graph"""
    name: str
    version: str
    constraints: str
    is_dev: bool = False
    optional: bool = False
    conflicts: List[str] = None
    
    def __post_init__(self):
        if self.conflicts is None:
            self.conflicts = []


@dataclass
class DependencyConflict:
    """Represents a dependency conflict"""
    package1: str
    version1: str
    package2: str
    version2: str
    reason: str


class DependencyResolver:
    """Resolves package dependencies with conflict detection"""
    
    def __init__(self, package_manager):
        self.pm = package_manager
        self.resolved_dependencies = {}
        self.conflict_cache = {}
        
    def resolve_dependencies(self, metadata: PackageMetadata) -> List[str]:
        """Resolve all dependencies for a package"""
        logger.info(f"Resolving dependencies for {metadata.name}")
        
        try:
            # Build dependency graph
            dependency_graph = self._build_dependency_graph(metadata)
            
            # Check for conflicts
            conflicts = self._find_conflicts(dependency_graph)
            if conflicts:
                logger.error(f"Dependency conflicts detected: {conflicts}")
                return []
            
            # Topological sort to get installation order
            install_order = self._topological_sort(dependency_graph)
            
            # Filter out already installed packages
            to_install = []
            for dep_name in install_order:
                if not self._is_package_installed(dep_name):
                    to_install.append(dep_name)
            
            logger.info(f"Dependencies to install: {to_install}")
            return to_install
            
        except Exception as e:
            logger.error(f"Error resolving dependencies: {e}")
            return []
    
    def find_dependents(self, package_name: str) -> List[str]:
        """Find packages that depend on the given package"""
        dependents = []
        
        # Check all installed packages
        installed_packages = self._get_installed_packages()
        for pkg_metadata in installed_packages.values():
            for dep_name, _ in pkg_metadata.dependencies.items():
                if dep_name == package_name:
                    dependents.append(pkg_metadata.name)
                    break
        
        return dependents
    
    def check_compatibility(self, packages: List[PackageMetadata]) -> List[DependencyConflict]:
        """Check compatibility between a set of packages"""
        conflicts = []
        
        for i, pkg1 in enumerate(packages):
            for pkg2 in packages[i+1:]:
                conflict = self._check_pair_compatibility(pkg1, pkg2)
                if conflict:
                    conflicts.append(conflict)
        
        return conflicts
    
    def resolve_version_constraints(self, package_name: str, 
                                  constraints: str, 
                                  available_versions: List[str]) -> Optional[str]:
        """Resolve version constraints to find the best matching version"""
        # Parse version constraints
        constraint_patterns = self._parse_version_constraints(constraints)
        
        # Find matching versions
        matching_versions = []
        for version in available_versions:
            if self._version_matches_constraints(version, constraint_patterns):
                matching_versions.append(version)
        
        if not matching_versions:
            return None
        
        # Return the highest matching version
        matching_versions.sort(reverse=True)
        return matching_versions[0]
    
    def suggest_conflict_resolution(self, conflict: DependencyConflict) -> List[str]:
        """Suggest ways to resolve a dependency conflict"""
        suggestions = []
        
        # Suggest updating packages
        suggestions.append(f"Try updating {conflict.package1} to a compatible version")
        suggestions.append(f"Try updating {conflict.package2} to a compatible version")
        
        # Suggest using alternative packages
        suggestions.append(f"Consider using an alternative to {conflict.package1}")
        suggestions.append(f"Consider using an alternative to {conflict.package2}")
        
        # Suggest version pinning
        suggestions.append(f"Pin {conflict.package1} to version {conflict.version1} if compatible")
        suggestions.append(f"Pin {conflict.package2} to version {conflict.version2} if compatible")
        
        return suggestions
    
    def _build_dependency_graph(self, metadata: PackageMetadata) -> Dict[str, List[str]]:
        """Build dependency graph from package metadata"""
        graph = {}
        
        # Add main package
        graph[metadata.name] = []
        
        # Process dependencies
        for dep_name, constraint in metadata.dependencies.items():
            if dep_name not in graph:
                graph[dep_name] = []
            graph[metadata.name].append(dep_name)
            
            # Recursively build dependency graph
            dep_metadata = self._find_package_metadata(dep_name)
            if dep_metadata:
                dep_graph = self._build_dependency_graph(dep_metadata)
                self._merge_graphs(graph, dep_graph)
        
        return graph
    
    def _merge_graphs(self, target: Dict[str, List[str]], source: Dict[str, List[str]]):
        """Merge two dependency graphs"""
        for node, dependencies in source.items():
            if node not in target:
                target[node] = []
            for dep in dependencies:
                if dep not in target[node]:
                    target[node].append(dep)
    
    def _find_conflicts(self, graph: Dict[str, List[str]]) -> List[DependencyConflict]:
        """Find conflicts in dependency graph"""
        conflicts = []
        
        # Check for circular dependencies
        circular = self._detect_circular_dependencies(graph)
        if circular:
            conflicts.append(DependencyConflict(
                package1=circular[0],
                version1="",
                package2=circular[1],
                version2="",
                reason=f"Circular dependency: {' -> '.join(circular)}"
            ))
        
        # Check for version conflicts
        conflicts.extend(self._check_version_conflicts(graph))
        
        return conflicts
    
    def _detect_circular_dependencies(self, graph: Dict[str, List[str]]) -> Optional[List[str]]:
        """Detect circular dependencies using DFS"""
        visited = set()
        rec_stack = set()
        
        def dfs(node, path):
            if node in rec_stack:
                cycle_start = path.index(node)
                return path[cycle_start:] + [node]
            
            if node in visited:
                return None
            
            visited.add(node)
            rec_stack.add(node)
            
            for neighbor in graph.get(node, []):
                result = dfs(neighbor, path + [node])
                if result:
                    return result
            
            rec_stack.remove(node)
            return None
        
        for node in graph:
            if node not in visited:
                result = dfs(node, [])
                if result:
                    return result
        
        return None
    
    def _check_version_conflicts(self, graph: Dict[str, List[str]]) -> List[DependencyConflict]:
        """Check for version conflicts between dependencies"""
        conflicts = []
        
        # Get all package versions in the graph
        package_versions = {}
        for package_name in graph:
            metadata = self._find_package_metadata(package_name)
            if metadata:
                package_versions[package_name] = metadata.version
        
        # Check for version conflicts
        for package_name, version in package_versions.items():
            for dep_name, _ in graph.get(package_name, []):
                dep_metadata = self._find_package_metadata(dep_name)
                if dep_metadata:
                    # Check if different versions of the same package are required
                    for other_name, other_version in package_versions.items():
                        if other_name == dep_name and other_version != dep_metadata.version:
                            conflicts.append(DependencyConflict(
                                package1=package_name,
                                version1=version,
                                package2=other_name,
                                version2=other_version,
                                reason=f"Different versions of {dep_name} required"
                            ))
        
        return conflicts
    
    def _topological_sort(self, graph: Dict[str, List[str]]) -> List[str]:
        """Perform topological sort to determine installation order"""
        in_degree = {node: 0 for node in graph}
        
        # Calculate in-degrees
        for node, dependencies in graph.items():
            for dep in dependencies:
                if dep in in_degree:
                    in_degree[dep] += 1
        
        # Initialize queue with nodes that have no dependencies
        queue = [node for node, degree in in_degree.items() if degree == 0]
        result = []
        
        while queue:
            node = queue.pop(0)
            result.append(node)
            
            # Reduce in-degree for dependent nodes
            for dep in graph.get(node, []):
                if dep in in_degree:
                    in_degree[dep] -= 1
                    if in_degree[dep] == 0:
                        queue.append(dep)
        
        return result
    
    def _is_package_installed(self, package_name: str) -> bool:
        """Check if a package is already installed"""
        return self._find_installed_package(package_name) is not None
    
    def _find_package_metadata(self, package_name: str) -> Optional[PackageMetadata]:
        """Find package metadata"""
        # Check local packages
        local_packages = self.pm._load_local_packages()
        for metadata in local_packages:
            if metadata.name == package_name:
                return metadata
        
        return None
    
    def _find_installed_package(self, package_name: str) -> Optional[PackageMetadata]:
        """Find installed package"""
        installed_file = self.pm.metadata_dir / "installed.json"
        if installed_file.exists():
            with open(installed_file, 'r') as f:
                installed = json.load(f)
                
                if package_name in installed:
                    data = installed[package_name]
                    data['type'] = PackageType(data['type'])
                    data['compatibility'] = CompatibilityLevel(data['compatibility'])
                    return PackageMetadata(**data)
        
        return None
    
    def _get_installed_packages(self) -> Dict[str, PackageMetadata]:
        """Get all installed packages"""
        installed = {}
        installed_file = self.pm.metadata_dir / "installed.json"
        
        if installed_file.exists():
            with open(installed_file, 'r') as f:
                data = json.load(f)
                for package_name, package_data in data.items():
                    package_data['type'] = PackageType(package_data['type'])
                    package_data['compatibility'] = CompatibilityLevel(package_data['compatibility'])
                    installed[package_name] = PackageMetadata(**package_data)
        
        return installed
    
    def _check_pair_compatibility(self, pkg1: PackageMetadata, pkg2: PackageMetadata) -> Optional[DependencyConflict]:
        """Check compatibility between two specific packages"""
        # Check for conflicting dependencies
        for dep_name, version in pkg1.dependencies.items():
            if dep_name == pkg2.name:
                # Check version compatibility
                if not self._versions_compatible(pkg2.version, version):
                    return DependencyConflict(
                        package1=pkg1.name,
                        version1=pkg1.version,
                        package2=pkg2.name,
                        version2=pkg2.version,
                        reason=f"Version {pkg2.version} of {pkg2.name} incompatible with {pkg1.name}'s requirement: {version}"
                    )
        
        for dep_name, version in pkg2.dependencies.items():
            if dep_name == pkg1.name:
                if not self._versions_compatible(pkg1.version, version):
                    return DependencyConflict(
                        package1=pkg1.name,
                        version1=pkg1.version,
                        package2=pkg2.name,
                        version2=pkg2.version,
                        reason=f"Version {pkg1.version} of {pkg1.name} incompatible with {pkg2.name}'s requirement: {version}"
                    )
        
        return None
    
    def _versions_compatible(self, version: str, constraint: str) -> bool:
        """Check if a version satisfies a constraint"""
        if not constraint:
            return True
        
        # Simple version matching (can be enhanced with semver parsing)
        if constraint.startswith('>='):
            required_version = constraint[2:]
            return self._compare_versions(version, required_version) >= 0
        elif constraint.startswith('<='):
            required_version = constraint[2:]
            return self._compare_versions(version, required_version) <= 0
        elif constraint.startswith('>'):
            required_version = constraint[1:]
            return self._compare_versions(version, required_version) > 0
        elif constraint.startswith('<'):
            required_version = constraint[1:]
            return self._compare_versions(version, required_version) < 0
        elif constraint.startswith('=='):
            required_version = constraint[2:]
            return version == required_version
        elif constraint.startswith('~='):
            required_version = constraint[2:]
            # Patch version compatibility
            version_parts = version.split('.')
            required_parts = required_version.split('.')
            return (len(version_parts) >= 2 and len(required_parts) >= 2 and
                   version_parts[0] == required_parts[0] and version_parts[1] == required_parts[1])
        else:
            # Exact match
            return version == constraint
    
    def _compare_versions(self, version1: str, version2: str) -> int:
        """Compare two version strings"""
        def version_tuple(v):
            parts = []
            for part in v.split('.'):
                try:
                    parts.append(int(part))
                except ValueError:
                    # Handle non-numeric parts
                    parts.append(0)
            return tuple(parts)
        
        v1 = version_tuple(version1)
        v2 = version_tuple(version2)
        
        if v1 < v2:
            return -1
        elif v1 > v2:
            return 1
        else:
            return 0
    
    def _parse_version_constraints(self, constraints: str) -> List[str]:
        """Parse version constraints string into individual constraints"""
        # Split by comma for multiple constraints
        return [c.strip() for c in constraints.split(',')]
    
    def _version_matches_constraints(self, version: str, constraints: List[str]) -> bool:
        """Check if version matches all constraints"""
        for constraint in constraints:
            if not self._versions_compatible(version, constraint):
                return False
        return True
    
    def _remove_unused_dependencies(self):
        """Remove dependencies that are no longer needed"""
        installed = self._get_installed_packages()
        
        # Find all dependencies of installed packages
        required_packages = set(installed.keys())
        for metadata in installed.values():
            required_packages.update(metadata.dependencies.keys())
        
        # Remove packages that are not required
        for package_name in list(installed.keys()):
            if package_name not in required_packages:
                dependents = self.find_dependents(package_name)
                if not dependents:
                    logger.info(f"Removing unused dependency: {package_name}")
                    self.pm.remove_package(package_name, remove_dependencies=False)