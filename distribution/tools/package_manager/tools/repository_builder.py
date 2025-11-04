#!/usr/bin/env python3
"""
MultiOS Repository Builder - Build and manage package repositories

This tool helps repository maintainers create and manage package repositories
for the MultiOS ecosystem.
"""

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
import hashlib
import subprocess
import gzip


class RepositoryBuilder:
    """MultiOS Repository Builder"""
    
    def __init__(self, repo_name: str, repo_path: str):
        self.repo_name = repo_name
        self.repo_path = Path(repo_path)
        self.packages_path = self.repo_path / "packages"
        self.metadata_path = self.repo_path / "metadata"
        
        # Create directories
        self.repo_path.mkdir(parents=True, exist_ok=True)
        self.packages_path.mkdir(exist_ok=True)
        self.metadata_path.mkdir(exist_ok=True)
        
        self.packages_index = {}
        self.repository_info = {
            "name": repo_name,
            "version": "1.0",
            "description": f"Repository: {repo_name}",
            "maintainer": "",
            "created": datetime.now().isoformat(),
            "last_updated": datetime.now().isoformat(),
            "architectures": ["x86_64", "arm64", "riscv64", "universal"],
            "categories": []
        }
    
    def add_package(self, package_path: str, metadata: Optional[Dict[str, Any]] = None) -> bool:
        """Add a package to the repository"""
        package_file = Path(package_path)
        
        if not package_file.exists():
            print(f"Error: Package file not found: {package_path}")
            return False
        
        print(f"Adding package: {package_file.name}")
        
        try:
            # Extract metadata from package
            if metadata is None:
                metadata = self._extract_package_metadata(package_file)
            
            # Validate metadata
            if not self._validate_package_metadata(metadata):
                print("Error: Invalid package metadata")
                return False
            
            # Calculate checksums
            checksums = self._calculate_checksums(package_file)
            
            # Create package entry
            package_entry = {
                "name": metadata["name"],
                "version": metadata["version"],
                "architecture": metadata["architecture"],
                "description": metadata["description"],
                "maintainer": metadata.get("maintainer", ""),
                "license": metadata.get("license", ""),
                "dependencies": metadata.get("dependencies", []),
                "provides": metadata.get("provides", []),
                "conflicts": metadata.get("conflicts", []),
                "categories": metadata.get("categories", []),
                "tags": metadata.get("tags", []),
                "size": package_file.stat().st_size,
                "checksums": checksums,
                "filename": package_file.name,
                "added_date": datetime.now().isoformat(),
                "repository": self.repo_name
            }
            
            # Check for conflicts with existing packages
            if self._package_conflicts(package_entry):
                print(f"Error: Package conflicts with existing package")
                return False
            
            # Copy package to repository
            target_path = self.packages_path / package_file.name
            shutil.copy2(package_file, target_path)
            
            # Add to index
            package_key = f"{package_entry['name']}-{package_entry['version']}-{package_entry['architecture']}"
            self.packages_index[package_key] = package_entry
            
            print(f"  Package added: {package_entry['name']} v{package_entry['version']}")
            return True
        
        except Exception as e:
            print(f"Error adding package: {e}")
            return False
    
    def remove_package(self, package_name: str, version: Optional[str] = None, 
                      architecture: Optional[str] = None) -> bool:
        """Remove a package from the repository"""
        
        # Find packages to remove
        packages_to_remove = []
        for package_key, package_info in self.packages_index.items():
            if package_info["name"] == package_name:
                if version and package_info["version"] != version:
                    continue
                if architecture and package_info["architecture"] != architecture:
                    continue
                packages_to_remove.append((package_key, package_info))
        
        if not packages_to_remove:
            print(f"No packages found matching criteria")
            return False
        
        print(f"Removing {len(packages_to_remove)} packages:")
        
        for package_key, package_info in packages_to_remove:
            # Remove package file
            package_file = self.packages_path / package_info["filename"]
            if package_file.exists():
                package_file.unlink()
                print(f"  Removed: {package_info['filename']}")
            
            # Remove from index
            del self.packages_index[package_key]
            
            # Remove checksums file
            checksums_file = self.packages_path / f"{package_info['filename']}.sha256"
            if checksums_file.exists():
                checksums_file.unlink()
        
        return True
    
    def build_repository_index(self, compress: bool = True) -> bool:
        """Build repository package index"""
        print("Building repository index...")
        
        try:
            # Build package index by category
            index_by_category = {}
            index_by_name = {}
            index_all = []
            
            for package_key, package_info in self.packages_index.items():
                # Add to category index
                for category in package_info.get("categories", ["uncategorized"]):
                    if category not in index_by_category:
                        index_by_category[category] = []
                    index_by_category[category].append(package_info)
                
                # Add to name index
                if package_info["name"] not in index_by_name:
                    index_by_name[package_info["name"]] = []
                index_by_name[package_info["name"]].append(package_info)
                
                # Add to all index
                index_all.append(package_info)
            
            # Write main index
            index_data = {
                "repository": self.repository_info,
                "packages": {
                    "all": index_all,
                    "by_category": index_by_category,
                    "by_name": index_by_name
                },
                "statistics": {
                    "total_packages": len(index_all),
                    "categories": len(index_by_category),
                    "architectures": list(set(p["architecture"] for p in index_all)),
                    "last_updated": datetime.now().isoformat()
                }
            }
            
            index_file = self.metadata_path / "index.json"
            with open(index_file, 'w') as f:
                json.dump(index_data, f, indent=2)
            
            # Create compressed version
            if compress:
                with open(index_file, 'rb') as f_in:
                    with gzip.open(f"{index_file}.gz", 'wb') as f_out:
                        shutil.copyfileobj(f_in, f_out)
            
            # Write individual category indexes
            for category, packages in index_by_category.items():
                category_file = self.metadata_path / f"category-{category}.json"
                category_data = {
                    "category": category,
                    "packages": packages,
                    "count": len(packages),
                    "last_updated": datetime.now().isoformat()
                }
                
                with open(category_file, 'w') as f:
                    json.dump(category_data, f, indent=2)
                
                if compress:
                    with open(category_file, 'rb') as f_in:
                        with gzip.open(f"{category_file}.gz", 'wb') as f_out:
                            shutil.copyfileobj(f_in, f_out)
            
            # Write package list
            package_list_file = self.metadata_path / "packages.txt"
            with open(package_list_file, 'w') as f:
                for package_info in sorted(index_all, key=lambda p: p["name"]):
                    f.write(f"{package_info['name']:<30} {package_info['version']:<15} "
                           f"{package_info['architecture']:<10} {package_info['description']}\n")
            
            print(f"  Index built with {len(index_all)} packages")
            return True
        
        except Exception as e:
            print(f"Error building index: {e}")
            return False
    
    def update_repository_info(self, **kwargs) -> None:
        """Update repository information"""
        self.repository_info.update(kwargs)
        self.repository_info["last_updated"] = datetime.now().isoformat()
    
    def generate_repository_metadata(self) -> str:
        """Generate repository metadata for package managers"""
        metadata = {
            "name": self.repo_name,
            "description": self.repository_info["description"],
            "maintainer": self.repository_info["maintainer"],
            "version": self.repository_info["version"],
            "architectures": self.repository_info["architectures"],
            "categories": self.repository_info["categories"],
            "packages_count": len(self.packages_index),
            "last_updated": self.repository_info["last_updated"],
            "index_url": f"{self.repo_path}/metadata/index.json",
            "packages_url": f"{self.repo_path}/packages/"
        }
        
        return json.dumps(metadata, indent=2)
    
    def verify_repository(self) -> bool:
        """Verify repository integrity"""
        print(f"Verifying repository: {self.repo_name}")
        
        try:
            issues = []
            
            # Check if all indexed packages exist
            for package_key, package_info in self.packages_index.items():
                package_file = self.packages_path / package_info["filename"]
                if not package_file.exists():
                    issues.append(f"Missing package file: {package_info['filename']}")
                
                # Verify checksums
                if package_file.exists():
                    checksums = self._calculate_checksums(package_file)
                    if checksums["sha256"] != package_info["checksums"]["sha256"]:
                        issues.append(f"Checksum mismatch: {package_info['filename']}")
            
            # Check for duplicate packages
            seen_packages = set()
            for package_key, package_info in self.packages_index.items():
                package_id = f"{package_info['name']}-{package_info['version']}-{package_info['architecture']}"
                if package_id in seen_packages:
                    issues.append(f"Duplicate package: {package_info['name']}")
                seen_packages.add(package_id)
            
            if issues:
                print("Repository verification failed:")
                for issue in issues:
                    print(f"  - {issue}")
                return False
            
            print("Repository verification passed")
            return True
        
        except Exception as e:
            print(f"Error during verification: {e}")
            return False
    
    def sync_from_directory(self, directory: str, pattern: str = "*.tar.xz") -> int:
        """Sync packages from a directory"""
        directory_path = Path(directory)
        
        if not directory_path.exists():
            print(f"Error: Directory not found: {directory}")
            return 0
        
        # Find package files
        package_files = list(directory_path.glob(pattern))
        
        if not package_files:
            print(f"No packages found matching pattern: {pattern}")
            return 0
        
        print(f"Found {len(package_files)} packages in {directory}")
        
        added_count = 0
        for package_file in package_files:
            if self.add_package(str(package_file)):
                added_count += 1
        
        print(f"Added {added_count} packages to repository")
        return added_count
    
    def export_repository_manifest(self, output_file: str) -> bool:
        """Export repository manifest"""
        try:
            manifest = {
                "repository": self.repository_info,
                "packages": list(self.packages_index.values()),
                "generated": datetime.now().isoformat(),
                "signature": "# Repository manifest signature would go here"
            }
            
            with open(output_file, 'w') as f:
                json.dump(manifest, f, indent=2)
            
            print(f"Repository manifest exported to: {output_file}")
            return True
        
        except Exception as e:
            print(f"Error exporting manifest: {e}")
            return False
    
    def _extract_package_metadata(self, package_file: Path) -> Dict[str, Any]:
        """Extract metadata from package file"""
        # This would extract metadata from the package tarball
        # For now, return minimal metadata
        return {
            "name": package_file.stem.split('-')[0],
            "version": "1.0.0",
            "architecture": "universal",
            "description": f"Package from {package_file.name}",
            "maintainer": "Unknown",
            "license": "Unknown"
        }
    
    def _validate_package_metadata(self, metadata: Dict[str, Any]) -> bool:
        """Validate package metadata"""
        required_fields = ["name", "version", "architecture", "description"]
        
        for field in required_fields:
            if field not in metadata:
                print(f"Missing required field: {field}")
                return False
        
        return True
    
    def _calculate_checksums(self, file_path: Path) -> Dict[str, str]:
        """Calculate file checksums"""
        checksums = {}
        
        with open(file_path, 'rb') as f:
            data = f.read()
            
            # SHA256
            checksums["sha256"] = hashlib.sha256(data).hexdigest()
            
            # MD5 (for compatibility)
            checksums["md5"] = hashlib.md5(data).hexdigest()
        
        return checksums
    
    def _package_conflicts(self, new_package: Dict[str, Any]) -> bool:
        """Check if package conflicts with existing packages"""
        for existing_package in self.packages_index.values():
            # Check for exact conflicts
            if (new_package["name"] == existing_package["name"] and
                new_package["version"] == existing_package["version"] and
                new_package["architecture"] == existing_package["architecture"]):
                return True
            
            # Check for conflicts
            if new_package["name"] in existing_package.get("conflicts", []):
                return True
            
            if existing_package["name"] in new_package.get("conflicts", []):
                return True
        
        return False
    
    def get_repository_statistics(self) -> Dict[str, Any]:
        """Get repository statistics"""
        packages = list(self.packages_index.values())
        
        return {
            "total_packages": len(packages),
            "by_architecture": {},
            "by_category": {},
            "total_size": sum(p["size"] for p in packages),
            "unique_names": len(set(p["name"] for p in packages)),
            "latest_update": max(p["added_date"] for p in packages) if packages else None
        }


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(description="MultiOS Repository Builder")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Create command
    create_parser = subparsers.add_parser("create", help="Create new repository")
    create_parser.add_argument("name", help="Repository name")
    create_parser.add_argument("path", help="Repository path")
    
    # Add command
    add_parser = subparsers.add_parser("add", help="Add package to repository")
    add_parser.add_argument("package", help="Package file path")
    add_parser.add_argument("--repo-name", required=True, help="Repository name")
    add_parser.add_argument("--repo-path", required=True, help="Repository path")
    
    # Remove command
    remove_parser = subparsers.add_parser("remove", help="Remove package from repository")
    remove_parser.add_argument("name", help="Package name")
    remove_parser.add_argument("--repo-name", required=True, help="Repository name")
    remove_parser.add_argument("--repo-path", required=True, help="Repository path")
    remove_parser.add_argument("--version", help="Package version")
    remove_parser.add_argument("--architecture", help="Package architecture")
    
    # Build command
    build_parser = subparsers.add_parser("build", help="Build repository index")
    build_parser.add_argument("--repo-name", required=True, help="Repository name")
    build_parser.add_argument("--repo-path", required=True, help="Repository path")
    build_parser.add_argument("--no-compress", action="store_true", help="Don't compress index")
    
    # Sync command
    sync_parser = subparsers.add_parser("sync", help="Sync packages from directory")
    sync_parser.add_argument("directory", help="Source directory")
    sync_parser.add_argument("--repo-name", required=True, help="Repository name")
    sync_parser.add_argument("--repo-path", required=True, help="Repository path")
    sync_parser.add_argument("--pattern", default="*.tar.xz", help="File pattern")
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify repository")
    verify_parser.add_argument("--repo-name", required=True, help="Repository name")
    verify_parser.add_argument("--repo-path", required=True, help="Repository path")
    
    # Stats command
    stats_parser = subparsers.add_parser("stats", help="Show repository statistics")
    stats_parser.add_argument("--repo-name", required=True, help="Repository name")
    stats_parser.add_argument("--repo-path", required=True, help="Repository path")
    
    # Export command
    export_parser = subparsers.add_parser("export", help="Export repository manifest")
    export_parser.add_argument("output", help="Output file path")
    export_parser.add_argument("--repo-name", required=True, help="Repository name")
    export_parser.add_argument("--repo-path", required=True, help="Repository path")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        if args.command == "create":
            builder = RepositoryBuilder(args.name, args.path)
            print(f"Created repository: {args.name} at {args.path}")
        
        elif args.command == "add":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            builder.add_package(args.package)
        
        elif args.command == "remove":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            builder.remove_package(args.name, args.version, args.architecture)
        
        elif args.command == "build":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            builder.build_repository_index(not args.no_compress)
        
        elif args.command == "sync":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            builder.sync_from_directory(args.directory, args.pattern)
            builder.build_repository_index()
        
        elif args.command == "verify":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            success = builder.verify_repository()
            return 0 if success else 1
        
        elif args.command == "stats":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            stats = builder.get_repository_statistics()
            print("Repository Statistics:")
            print(f"  Total packages: {stats['total_packages']}")
            print(f"  Unique names: {stats['unique_names']}")
            print(f"  Total size: {stats['total_size']} bytes")
            if stats['latest_update']:
                print(f"  Latest update: {stats['latest_update']}")
        
        elif args.command == "export":
            builder = RepositoryBuilder(args.repo_name, args.repo_path)
            builder.export_repository_manifest(args.output)
        
        else:
            print(f"Unknown command: {args.command}")
            return 1
    
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())