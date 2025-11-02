#!/usr/bin/env python3
"""
Educational Package Manager for MultiOS
=======================================

A comprehensive package manager designed specifically for educational software
and curriculum packages with dependency resolution, security scanning,
and community sharing capabilities.

Author: MultiOS Development Team
Version: 1.0.0
"""

import os
import sys
import json
import hashlib
import subprocess
import logging
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple, Any
from dataclasses import dataclass, asdict
from enum import Enum
import tempfile
import shutil
import re

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class PackageType(Enum):
    """Package types supported by the educational package manager"""
    CURRICULUM = "curriculum"
    TUTORIAL = "tutorial"
    SIMULATION = "simulation"
    INTERACTIVE = "interactive"
    ASSESSMENT = "assessment"
    LIBRARY = "library"
    TOOL = "tool"
    DATA = "data"


class CompatibilityLevel(Enum):
    """Compatibility levels for packages"""
    BEGINNER = "beginner"
    INTERMEDIATE = "intermediate"
    ADVANCED = "advanced"
    EXPERT = "expert"


@dataclass
class PackageMetadata:
    """Package metadata structure"""
    name: str
    version: str
    description: str
    author: str
    email: str
    type: PackageType
    compatibility: CompatibilityLevel
    subjects: List[str]
    grade_levels: List[str]
    prerequisites: List[str]
    dependencies: Dict[str, str]
    size: int
    checksum: str
    created_at: str
    updated_at: str
    license: str
    homepage: Optional[str] = None
    repository: Optional[str] = None
    tags: List[str] = None
    files: List[str] = None
    scripts: Dict[str, str] = None
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = []
        if self.files is None:
            self.files = []
        if self.scripts is None:
            self.scripts = {}


class EducationalPackageManager:
    """Main package manager class for educational software"""
    
    def __init__(self, config_path: str = None):
        self.config_path = config_path or "/workspace/community/package_manager/config/config.json"
        self.packages_dir = Path("/workspace/community/package_manager/packages")
        self.cache_dir = Path("/workspace/community/package_manager/cache")
        self.metadata_dir = Path("/workspace/community/package_manager/packages/metadata")
        self.repository_url = "https://packages.multios.edu"
        
        # Ensure directories exist
        for directory in [self.packages_dir, self.cache_dir, self.metadata_dir]:
            directory.mkdir(parents=True, exist_ok=True)
        
        # Load configuration
        self.config = self._load_config()
        
        # Initialize components
        self.dependency_resolver = DependencyResolver(self)
        self.security_scanner = SecurityScanner(self)
        self.validator = PackageValidator(self)
        self.community = CommunityPortal(self)
        
    def _load_config(self) -> Dict[str, Any]:
        """Load package manager configuration"""
        config_file = Path(self.config_path)
        default_config = {
            "repositories": [
                "https://packages.multios.edu/official",
                "https://packages.multios.edu/community"
            ],
            "security": {
                "require_signature": True,
                "scan_packages": True,
                "max_file_size": 100 * 1024 * 1024,  # 100MB
                "allowed_extensions": [".zip", ".tar.gz", ".tar.xz", ".deb"]
            },
            "validation": {
                "run_tests": True,
                "check_curriculum": True,
                "validate_metadata": True
            },
            "limits": {
                "max_package_size": 500 * 1024 * 1024,  # 500MB
                "max_dependencies": 20
            }
        }
        
        if config_file.exists():
            try:
                with open(config_file, 'r') as f:
                    config = json.load(f)
                    # Merge with defaults
                    for key, value in default_config.items():
                        if key not in config:
                            config[key] = value
                return config
            except Exception as e:
                logger.error(f"Error loading config: {e}")
                return default_config
        else:
            return default_config
    
    def create_package(self, package_dir: str, metadata: PackageMetadata, 
                      output_path: str = None) -> bool:
        """Create a new educational package"""
        logger.info(f"Creating package: {metadata.name} v{metadata.version}")
        
        try:
            # Validate package structure
            if not self.validator.validate_package_structure(package_dir, metadata):
                logger.error("Package structure validation failed")
                return False
            
            # Create package archive
            if output_path is None:
                output_path = f"{self.packages_dir}/{metadata.name}-{metadata.version}.edu"
            
            self._create_package_archive(package_dir, metadata, output_path)
            
            # Security scan
            if self.config["security"]["scan_packages"]:
                if not self.security_scanner.scan_package(output_path, metadata):
                    logger.warning("Security scan completed with warnings")
            
            # Store metadata
            self._store_metadata(metadata)
            
            # Add to local index
            self._add_to_index(metadata)
            
            logger.info(f"Package created successfully: {output_path}")
            return True
            
        except Exception as e:
            logger.error(f"Error creating package: {e}")
            return False
    
    def install_package(self, package_name: str, version: str = None, 
                       target_dir: str = None) -> bool:
        """Install an educational package"""
        logger.info(f"Installing package: {package_name}")
        
        try:
            # Find package
            package_path = self._find_package(package_name, version)
            if not package_path:
                logger.error(f"Package not found: {package_name}")
                return False
            
            # Load metadata
            metadata = self._load_package_metadata(package_path)
            if not metadata:
                return False
            
            # Check dependencies
            dependencies = self.dependency_resolver.resolve_dependencies(metadata)
            if dependencies:
                logger.info(f"Installing dependencies: {dependencies}")
                for dep_name in dependencies:
                    if not self.install_package(dep_name):
                        logger.error(f"Failed to install dependency: {dep_name}")
                        return False
            
            # Verify package
            if not self.security_scanner.verify_package(package_path, metadata):
                logger.error("Package verification failed")
                return False
            
            # Extract and install
            if target_dir is None:
                target_dir = f"/opt/education/{metadata.name}"
            
            self._extract_package(package_path, target_dir)
            
            # Run post-install scripts
            self._run_scripts(metadata.scripts.get("post_install"), target_dir)
            
            # Update index
            self._add_installed_package(metadata)
            
            logger.info(f"Package installed successfully: {metadata.name}")
            return True
            
        except Exception as e:
            logger.error(f"Error installing package: {e}")
            return False
    
    def list_packages(self, category: str = None, subject: str = None) -> List[PackageMetadata]:
        """List available packages"""
        packages = []
        
        # Load from local index
        local_packages = self._load_local_packages()
        
        # Filter by category/subject
        for metadata in local_packages:
            if category and metadata.type.value != category:
                continue
            if subject and subject not in metadata.subjects:
                continue
            packages.append(metadata)
        
        return packages
    
    def search_packages(self, query: str) -> List[PackageMetadata]:
        """Search for packages by name, description, or tags"""
        results = []
        packages = self._load_local_packages()
        
        query_lower = query.lower()
        
        for metadata in packages:
            # Search in name, description, tags, and subjects
            searchable_text = " ".join([
                metadata.name.lower(),
                metadata.description.lower(),
                " ".join([tag.lower() for tag in metadata.tags]),
                " ".join([subject.lower() for subject in metadata.subjects])
            ])
            
            if query_lower in searchable_text:
                results.append(metadata)
        
        return results
    
    def remove_package(self, package_name: str, remove_dependencies: bool = False) -> bool:
        """Remove an installed package"""
        logger.info(f"Removing package: {package_name}")
        
        try:
            metadata = self._find_installed_package(package_name)
            if not metadata:
                logger.error(f"Package not installed: {package_name}")
                return False
            
            # Check for dependents
            dependents = self.dependency_resolver.find_dependents(package_name)
            if dependents and not remove_dependencies:
                logger.error(f"Cannot remove {package_name}: other packages depend on it: {dependents}")
                return False
            
            # Run pre-uninstall scripts
            self._run_scripts(metadata.scripts.get("pre_uninstall"), "/")
            
            # Remove files
            install_path = f"/opt/education/{package_name}"
            if os.path.exists(install_path):
                shutil.rmtree(install_path)
            
            # Remove from index
            self._remove_installed_package(package_name)
            
            # Optionally remove unused dependencies
            if remove_dependencies:
                self._remove_unused_dependencies()
            
            logger.info(f"Package removed successfully: {package_name}")
            return True
            
        except Exception as e:
            logger.error(f"Error removing package: {e}")
            return False
    
    def update_package(self, package_name: str) -> bool:
        """Update an installed package to latest version"""
        logger.info(f"Updating package: {package_name}")
        
        try:
            # Get current version
            current_metadata = self._find_installed_package(package_name)
            if not current_metadata:
                logger.error(f"Package not installed: {package_name}")
                return False
            
            # Check for updates
            latest_metadata = self._find_package(package_name)
            if not latest_metadata or latest_metadata.version == current_metadata.version:
                logger.info(f"No updates available for {package_name}")
                return True
            
            # Install new version
            if not self.install_package(package_name, latest_metadata.version):
                return False
            
            # Clean up old version
            old_install_path = f"/opt/education/{package_name}-old"
            if os.path.exists(old_install_path):
                shutil.move(f"/opt/education/{package_name}", old_install_path)
                shutil.move(old_install_path, f"/opt/education/{package_name}")
            
            logger.info(f"Package updated successfully: {package_name}")
            return True
            
        except Exception as e:
            logger.error(f"Error updating package: {e}")
            return False
    
    def _create_package_archive(self, source_dir: str, metadata: PackageMetadata, output_path: str):
        """Create compressed package archive"""
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            
            # Copy source files
            source_path = Path(source_dir)
            for item in source_path.rglob("*"):
                if item.is_file():
                    rel_path = item.relative_to(source_path)
                    dest_file = temp_path / rel_path
                    dest_file.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(item, dest_file)
            
            # Create metadata file
            metadata_file = temp_path / "metadata.json"
            with open(metadata_file, 'w') as f:
                json.dump(asdict(metadata), f, indent=2)
            
            # Calculate checksum
            metadata.checksum = self._calculate_checksum(str(metadata_file))
            
            # Update metadata file with checksum
            with open(metadata_file, 'w') as f:
                json.dump(asdict(metadata), f, indent=2)
            
            # Create archive
            shutil.make_archive(output_path.replace('.edu', ''), 'gztar', temp_path)
            
            # Rename to .edu extension
            os.rename(f"{output_path.replace('.edu', '')}.tar.gz", output_path)
    
    def _calculate_checksum(self, file_path: str) -> str:
        """Calculate SHA256 checksum of file"""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def _store_metadata(self, metadata: PackageMetadata):
        """Store package metadata"""
        metadata_file = self.metadata_dir / f"{metadata.name}.json"
        with open(metadata_file, 'w') as f:
            json.dump(asdict(metadata), f, indent=2)
    
    def _load_package_metadata(self, package_path: str) -> Optional[PackageMetadata]:
        """Load package metadata from archive"""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                shutil.unpack_archive(package_path, temp_dir)
                metadata_file = Path(temp_dir) / "metadata.json"
                
                if metadata_file.exists():
                    with open(metadata_file, 'r') as f:
                        data = json.load(f)
                        # Convert back to enum types
                        data['type'] = PackageType(data['type'])
                        data['compatibility'] = CompatibilityLevel(data['compatibility'])
                        return PackageMetadata(**data)
        except Exception as e:
            logger.error(f"Error loading metadata: {e}")
        
        return None
    
    def _find_package(self, package_name: str, version: str = None) -> Optional[str]:
        """Find package file path"""
        # Search in local packages
        for package_file in self.packages_dir.glob("*.edu"):
            metadata = self._load_package_metadata(str(package_file))
            if metadata and metadata.name == package_name:
                if version is None or metadata.version == version:
                    return str(package_file)
        
        # TODO: Search remote repositories
        return None
    
    def _load_local_packages(self) -> List[PackageMetadata]:
        """Load all local package metadata"""
        packages = []
        for metadata_file in self.metadata_dir.glob("*.json"):
            try:
                with open(metadata_file, 'r') as f:
                    data = json.load(f)
                    data['type'] = PackageType(data['type'])
                    data['compatibility'] = CompatibilityLevel(data['compatibility'])
                    packages.append(PackageMetadata(**data))
            except Exception as e:
                logger.error(f"Error loading metadata from {metadata_file}: {e}")
        
        return packages
    
    def _add_to_index(self, metadata: PackageMetadata):
        """Add package to local index"""
        index_file = self.metadata_dir / "index.json"
        index = {}
        
        if index_file.exists():
            with open(index_file, 'r') as f:
                index = json.load(f)
        
        index[metadata.name] = asdict(metadata)
        
        with open(index_file, 'w') as f:
            json.dump(index, f, indent=2)
    
    def _run_scripts(self, script_path: Optional[str], working_dir: str):
        """Run package scripts"""
        if script_path and os.path.exists(script_path):
            try:
                os.chdir(working_dir)
                subprocess.run([script_path], check=True)
                logger.info(f"Script executed: {script_path}")
            except subprocess.CalledProcessError as e:
                logger.error(f"Script failed: {script_path}: {e}")
    
    def _add_installed_package(self, metadata: PackageMetadata):
        """Add to installed packages list"""
        installed_file = self.metadata_dir / "installed.json"
        installed = {}
        
        if installed_file.exists():
            with open(installed_file, 'r') as f:
                installed = json.load(f)
        
        installed[metadata.name] = asdict(metadata)
        
        with open(installed_file, 'w') as f:
            json.dump(installed, f, indent=2)
    
    def _remove_installed_package(self, package_name: str):
        """Remove from installed packages list"""
        installed_file = self.metadata_dir / "installed.json"
        if installed_file.exists():
            with open(installed_file, 'r') as f:
                installed = json.load(f)
            
            if package_name in installed:
                del installed[package_name]
                
                with open(installed_file, 'w') as f:
                    json.dump(installed, f, indent=2)
    
    def _find_installed_package(self, package_name: str) -> Optional[PackageMetadata]:
        """Find installed package metadata"""
        installed_file = self.metadata_dir / "installed.json"
        if installed_file.exists():
            with open(installed_file, 'r') as f:
                installed = json.load(f)
                
                if package_name in installed:
                    data = installed[package_name]
                    data['type'] = PackageType(data['type'])
                    data['compatibility'] = CompatibilityLevel(data['compatibility'])
                    return PackageMetadata(**data)
        
        return None


def main():
    """Main entry point for the package manager"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Educational Package Manager for MultiOS")
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Create package command
    create_parser = subparsers.add_parser('create', help='Create a new package')
    create_parser.add_argument('package_dir', help='Package source directory')
    create_parser.add_argument('--metadata', help='Metadata JSON file')
    create_parser.add_argument('--output', help='Output package file')
    
    # Install package command
    install_parser = subparsers.add_parser('install', help='Install a package')
    install_parser.add_argument('package', help='Package name or file')
    install_parser.add_argument('--version', help='Package version')
    install_parser.add_argument('--target', help='Installation directory')
    
    # List packages command
    list_parser = subparsers.add_parser('list', help='List available packages')
    list_parser.add_argument('--category', help='Filter by package type')
    list_parser.add_argument('--subject', help='Filter by subject')
    
    # Search packages command
    search_parser = subparsers.add_parser('search', help='Search for packages')
    search_parser.add_argument('query', help='Search query')
    
    # Remove package command
    remove_parser = subparsers.add_parser('remove', help='Remove a package')
    remove_parser.add_argument('package', help='Package name')
    remove_parser.add_argument('--remove-deps', action='store_true', 
                              help='Remove unused dependencies')
    
    # Update package command
    update_parser = subparsers.add_parser('update', help='Update a package')
    update_parser.add_argument('package', help='Package name')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    # Initialize package manager
    pm = EducationalPackageManager()
    
    try:
        if args.command == 'create':
            if not args.metadata:
                print("Error: --metadata required for create command")
                return
            
            with open(args.metadata, 'r') as f:
                data = json.load(f)
                data['type'] = PackageType(data['type'])
                data['compatibility'] = CompatibilityLevel(data['compatibility'])
                metadata = PackageMetadata(**data)
            
            success = pm.create_package(args.package_dir, metadata, args.output)
            print(f"Package creation {'successful' if success else 'failed'}")
            
        elif args.command == 'install':
            success = pm.install_package(args.package, args.version, args.target)
            print(f"Package installation {'successful' if success else 'failed'}")
            
        elif args.command == 'list':
            packages = pm.list_packages(args.category, args.subject)
            for pkg in packages:
                print(f"{pkg.name} v{pkg.version} - {pkg.description}")
                
        elif args.command == 'search':
            results = pm.search_packages(args.query)
            for pkg in results:
                print(f"{pkg.name} v{pkg.version} - {pkg.description}")
                
        elif args.command == 'remove':
            success = pm.remove_package(args.package, args.remove_deps)
            print(f"Package removal {'successful' if success else 'failed'}")
            
        elif args.command == 'update':
            success = pm.update_package(args.package)
            print(f"Package update {'successful' if success else 'failed'}")
            
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        print(f"Error: {e}")


if __name__ == '__main__':
    main()