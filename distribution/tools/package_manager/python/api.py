# MultiOS Package Manager Python API
# 
# This module provides a Python interface to the MultiOS Package Manager,
# allowing Python applications to interact with the package system.

import asyncio
import json
import logging
from pathlib import Path
from typing import Dict, List, Optional, Union, Any
from dataclasses import dataclass, asdict
from enum import Enum
import subprocess
import sys


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class PackageStatus(Enum):
    """Package installation status"""
    INSTALLED = "installed"
    AVAILABLE = "available"
    UPDATE_AVAILABLE = "update_available"
    NOT_INSTALLED = "not_installed"


class VerificationStatus(Enum):
    """Package verification status"""
    PASSED = "passed"
    FAILED = "failed"
    WARNING = "warning"


class UpdateType(Enum):
    """Update type classification"""
    MINOR = "minor"
    MAJOR = "major"
    SECURITY = "security"
    BUGFIX = "bugfix"
    FEATURE = "feature"


@dataclass
class Version:
    """Package version representation"""
    major: int
    minor: int
    patch: int
    pre_release: Optional[str] = None
    build_metadata: Optional[str] = None

    def __str__(self) -> str:
        version_str = f"{self.major}.{self.minor}.{self.patch}"
        if self.pre_release:
            version_str += f"-{self.pre_release}"
        if self.build_metadata:
            version_str += f"+{self.build_metadata}"
        return version_str

    @classmethod
    def parse(cls, version_str: str) -> 'Version':
        """Parse version from string"""
        parts = version_str.split('-')
        version_parts = parts[0].split('.')
        
        if len(version_parts) < 3:
            raise ValueError(f"Invalid version format: {version_str}")
        
        major = int(version_parts[0])
        minor = int(version_parts[1])
        patch = int(version_parts[2])
        
        pre_release = None
        build_metadata = None
        
        if len(parts) > 1:
            # Handle pre-release and build metadata
            rest = '-'.join(parts[1:])
            if '+' in rest:
                pre_release, build_metadata = rest.split('+', 1)
            else:
                pre_release = rest
        
        return cls(major, minor, patch, pre_release, build_metadata)


@dataclass
class Package:
    """Package information"""
    name: str
    version: Version
    description: str
    architecture: str
    size: int
    install_date: Optional[str] = None
    repository: Optional[str] = None
    status: PackageStatus = PackageStatus.NOT_INSTALLED
    dependencies: List[str] = None
    tags: List[str] = None
    security_info: Optional[Dict[str, Any]] = None

    def __post_init__(self):
        if self.dependencies is None:
            self.dependencies = []
        if self.tags is None:
            self.tags = []

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Package':
        """Create Package from dictionary"""
        if isinstance(data['version'], str):
            version = Version.parse(data['version'])
        else:
            version = Version(**data['version'])
        
        status = PackageStatus(data.get('status', 'not_installed'))
        
        return cls(
            name=data['name'],
            version=version,
            description=data.get('description', ''),
            architecture=data.get('architecture', 'universal'),
            size=data.get('size', 0),
            install_date=data.get('install_date'),
            repository=data.get('repository'),
            status=status,
            dependencies=data.get('dependencies', []),
            tags=data.get('tags', []),
            security_info=data.get('security_info')
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            'name': self.name,
            'version': str(self.version),
            'description': self.description,
            'architecture': self.architecture,
            'size': self.size,
            'install_date': self.install_date,
            'repository': self.repository,
            'status': self.status.value,
            'dependencies': self.dependencies,
            'tags': self.tags,
            'security_info': self.security_info
        }


@dataclass
class UpdateInfo:
    """Update information"""
    package_name: str
    current_version: Version
    available_version: Version
    update_type: UpdateType
    security_update: bool
    delta_available: bool
    repository: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'UpdateInfo':
        """Create UpdateInfo from dictionary"""
        return cls(
            package_name=data['package_name'],
            current_version=Version.parse(data['current_version']),
            available_version=Version.parse(data['available_version']),
            update_type=UpdateType(data.get('update_type', 'minor')),
            security_update=data.get('security_update', False),
            delta_available=data.get('delta_available', False),
            repository=data.get('repository', 'unknown')
        )


@dataclass
class VerificationResult:
    """Package verification result"""
    package_name: str
    version: Version
    status: VerificationStatus
    issues: List[str]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'VerificationResult':
        """Create VerificationResult from dictionary"""
        return cls(
            package_name=data['package_name'],
            version=Version.parse(data['version']),
            status=VerificationStatus(data['status']),
            issues=data.get('issues', [])
        )


@dataclass
class Repository:
    """Repository information"""
    name: str
    url: str
    description: str = ""
    enabled: bool = True
    priority: int = 0
    last_sync: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Repository':
        """Create Repository from dictionary"""
        return cls(
            name=data['name'],
            url=data['url'],
            description=data.get('description', ''),
            enabled=data.get('enabled', True),
            priority=data.get('priority', 0),
            last_sync=data.get('last_sync')
        )


class MultiOSPackageManager:
    """Python interface to MultiOS Package Manager"""
    
    def __init__(self, data_dir: str = "/var/lib/multios-package-manager"):
        self.data_dir = Path(data_dir)
        self.binary_path = "/usr/local/bin/multios-pm"
        self._ensure_binary()
    
    def _ensure_binary(self):
        """Ensure the multios-pm binary is available"""
        if not Path(self.binary_path).exists():
            raise FileNotFoundError(
                f"multios-pm binary not found at {self.binary_path}. "
                "Please install the package manager first."
            )
    
    async def _run_command(self, command: List[str]) -> Dict[str, Any]:
        """Run a multios-pm command and return the result"""
        try:
            process = await asyncio.create_subprocess_exec(
                *command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                env={'RUST_LOG': 'info'}
            )
            
            stdout, stderr = await process.communicate()
            
            if process.returncode != 0:
                error_msg = stderr.decode('utf-8').strip() or "Unknown error"
                raise subprocess.CalledProcessError(process.returncode, command, error_msg)
            
            # Try to parse JSON output, fall back to plain text
            try:
                return json.loads(stdout.decode('utf-8'))
            except json.JSONDecodeError:
                return {"output": stdout.decode('utf-8').strip()}
        
        except Exception as e:
            logger.error(f"Command failed: {' '.join(command)}")
            logger.error(f"Error: {e}")
            raise
    
    async def install_packages(self, packages: List[str], versions: Optional[List[str]] = None) -> bool:
        """Install packages"""
        command = [self.binary_path, "install"] + packages
        
        if versions:
            for version in versions:
                command.extend(["--version", version])
        
        try:
            await self._run_command(command)
            logger.info(f"Successfully installed packages: {', '.join(packages)}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to install packages: {e}")
            return False
    
    async def uninstall_packages(self, packages: List[str], purge: bool = False) -> bool:
        """Uninstall packages"""
        command = [self.binary_path, "uninstall"] + packages
        
        if purge:
            command.append("--purge")
        
        try:
            await self._run_command(command)
            logger.info(f"Successfully uninstalled packages: {', '.join(packages)}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to uninstall packages: {e}")
            return False
    
    async def update_packages(self, packages: Optional[List[str]] = None) -> List[Dict[str, Any]]:
        """Update packages"""
        command = [self.binary_path, "update"]
        
        if packages:
            command.extend(packages)
        
        try:
            result = await self._run_command(command)
            logger.info("Package update completed")
            return result.get('updates', [])
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to update packages: {e}")
            return []
    
    async def search_packages(self, query: str, limit: int = 50) -> List[Package]:
        """Search for packages"""
        command = [self.binary_path, "search", query, "--limit", str(limit)]
        
        try:
            result = await self._run_command(command)
            packages = []
            
            if isinstance(result, dict) and 'packages' in result:
                for pkg_data in result['packages']:
                    packages.append(Package.from_dict(pkg_data))
            elif isinstance(result, list):
                for pkg_data in result:
                    packages.append(Package.from_dict(pkg_data))
            
            logger.info(f"Found {len(packages)} packages matching '{query}'")
            return packages
        except subprocess.CalledProcessError as e:
            logger.error(f"Package search failed: {e}")
            return []
    
    async def get_installed_packages(self) -> List[Package]:
        """Get list of installed packages"""
        command = [self.binary_path, "list", "--json"]
        
        try:
            result = await self._run_command(command)
            packages = []
            
            if isinstance(result, list):
                for pkg_data in result:
                    packages.append(Package.from_dict(pkg_data))
            elif isinstance(result, dict) and 'packages' in result:
                for pkg_data in result['packages']:
                    packages.append(Package.from_dict(pkg_data))
            
            logger.info(f"Found {len(packages)} installed packages")
            return packages
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to get installed packages: {e}")
            return []
    
    async def get_package_info(self, package_name: str) -> Optional[Package]:
        """Get package information"""
        command = [self.binary_path, "info", package_name, "--json"]
        
        try:
            result = await self._run_command(command)
            
            if isinstance(result, dict) and 'package' in result:
                return Package.from_dict(result['package'])
            else:
                return Package.from_dict(result) if isinstance(result, dict) else None
        except subprocess.CalledProcessError as e:
            if "Package not found" in str(e):
                logger.info(f"Package not found: {package_name}")
            else:
                logger.error(f"Failed to get package info: {e}")
            return None
    
    async def verify_packages(self, packages: Optional[List[str]] = None) -> List[VerificationResult]:
        """Verify installed packages"""
        command = [self.binary_path, "verify", "--json"]
        
        if packages:
            command.extend(packages)
        
        try:
            result = await self._run_command(command)
            verification_results = []
            
            if isinstance(result, list):
                for res_data in result:
                    verification_results.append(VerificationResult.from_dict(res_data))
            elif isinstance(result, dict) and 'results' in result:
                for res_data in result['results']:
                    verification_results.append(VerificationResult.from_dict(res_data))
            
            logger.info(f"Verified {len(verification_results)} packages")
            return verification_results
        except subprocess.CalledProcessError as e:
            logger.error(f"Package verification failed: {e}")
            return []
    
    async def check_for_updates(self) -> List[UpdateInfo]:
        """Check for available updates"""
        command = [self.binary_path, "check-updates", "--json"]
        
        try:
            result = await self._run_command(command)
            updates = []
            
            if isinstance(result, list):
                for update_data in result:
                    updates.append(UpdateInfo.from_dict(update_data))
            elif isinstance(result, dict) and 'updates' in result:
                for update_data in result['updates']:
                    updates.append(UpdateInfo.from_dict(update_data))
            
            logger.info(f"Found {len(updates)} available updates")
            return updates
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to check for updates: {e}")
            return []
    
    async def sync_repositories(self, force: bool = False) -> bool:
        """Synchronize repositories"""
        command = [self.binary_path, "sync"]
        
        if force:
            command.append("--force")
        
        try:
            await self._run_command(command)
            logger.info("Repositories synchronized successfully")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to sync repositories: {e}")
            return False
    
    async def rollback_package(self, package_name: str, version: str, backup: bool = True) -> bool:
        """Rollback package to specific version"""
        command = [self.binary_path, "rollback", package_name, version]
        
        if backup:
            command.append("--backup")
        
        try:
            await self._run_command(command)
            logger.info(f"Rolled back {package_name} to version {version}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to rollback package: {e}")
            return False
    
    async def get_status(self) -> Dict[str, Any]:
        """Get package manager status"""
        command = [self.binary_path, "status", "--json"]
        
        try:
            result = await self._run_command(command)
            return result if isinstance(result, dict) else {}
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to get status: {e}")
            return {}
    
    async def cleanup(self, old_versions: bool = True, clear_cache: bool = False, 
                     orphaned: bool = True, dry_run: bool = False) -> Dict[str, Any]:
        """Clean up old packages and cache"""
        command = [self.binary_path, "cleanup"]
        
        if old_versions:
            command.append("--old-versions")
        if clear_cache:
            command.append("--clear-cache")
        if orphaned:
            command.append("--orphaned")
        if dry_run:
            command.append("--dry-run")
        
        try:
            result = await self._run_command(command)
            logger.info("Cleanup completed")
            return result if isinstance(result, dict) else {}
        except subprocess.CalledProcessError as e:
            logger.error(f"Cleanup failed: {e}")
            return {}
    
    async def get_repositories(self) -> List[Repository]:
        """Get list of repositories"""
        command = [self.binary_path, "repository", "list", "--json"]
        
        try:
            result = await self._run_command(command)
            repositories = []
            
            if isinstance(result, list):
                for repo_data in result:
                    repositories.append(Repository.from_dict(repo_data))
            elif isinstance(result, dict) and 'repositories' in result:
                for repo_data in result['repositories']:
                    repositories.append(Repository.from_dict(repo_data))
            
            logger.info(f"Found {len(repositories)} repositories")
            return repositories
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to get repositories: {e}")
            return []
    
    async def add_repository(self, name: str, url: str, description: str = "", 
                           priority: int = 0) -> bool:
        """Add a new repository"""
        command = [self.binary_path, "repository", "add", name, url]
        
        if description:
            command.extend(["--description", description])
        if priority != 0:
            command.extend(["--priority", str(priority)])
        
        try:
            await self._run_command(command)
            logger.info(f"Added repository: {name}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to add repository: {e}")
            return False
    
    async def remove_repository(self, name: str) -> bool:
        """Remove a repository"""
        command = [self.binary_path, "repository", "remove", name]
        
        try:
            await self._run_command(command)
            logger.info(f"Removed repository: {name}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to remove repository: {e}")
            return False


class PackageManagerCLI:
    """Command-line interface wrapper for package manager"""
    
    def __init__(self):
        self.pm = MultiOSPackageManager()
    
    async def interactive_search(self):
        """Interactive package search"""
        print("MultiOS Package Manager - Interactive Search")
        print("=" * 50)
        
        while True:
            query = input("\nSearch for packages (or 'quit' to exit): ").strip()
            
            if query.lower() in ['quit', 'exit', 'q']:
                break
            
            if not query:
                continue
            
            packages = await self.pm.search_packages(query)
            
            if not packages:
                print(f"No packages found for '{query}'")
                continue
            
            print(f"\nFound {len(packages)} packages:")
            print("-" * 50)
            
            for i, pkg in enumerate(packages, 1):
                status_icon = {
                    PackageStatus.INSTALLED: "✓",
                    PackageStatus.UPDATE_AVAILABLE: "↻",
                    PackageStatus.AVAILABLE: "○",
                    PackageStatus.NOT_INSTALLED: "○"
                }.get(pkg.status, "○")
                
                print(f"{i:2d}. {status_icon} {pkg.name:<20} {pkg.version}")
                print(f"    {pkg.description[:60]}")
                if pkg.tags:
                    print(f"    Tags: {', '.join(pkg.tags[:5])}")
                print()
            
            choice = input("Install package by number, or Enter to continue: ").strip()
            if choice.isdigit():
                idx = int(choice) - 1
                if 0 <= idx < len(packages):
                    pkg = packages[idx]
                    if await self.pm.install_packages([pkg.name]):
                        print(f"✓ Installed {pkg.name}")
                    else:
                        print(f"✗ Failed to install {pkg.name}")
    
    async def interactive_update(self):
        """Interactive package updates"""
        print("MultiOS Package Manager - Interactive Update")
        print("=" * 50)
        
        # Check for updates
        updates = await self.pm.check_for_updates()
        
        if not updates:
            print("No updates available.")
            return
        
        print(f"\nFound {len(updates)} updates:")
        print("-" * 50)
        
        for i, update in enumerate(updates, 1):
            security_indicator = " [SECURITY]" if update.security_update else ""
            print(f"{i:2d}. {update.package_name:<20} {update.current_version} → {update.available_version}{security_indicator}")
        
        print("\nOptions:")
        print("1. Install all updates")
        print("2. Install security updates only")
        print("3. Select specific packages")
        print("4. Cancel")
        
        choice = input("\nChoose option (1-4): ").strip()
        
        if choice == "1":
            # Install all updates
            packages = [u.package_name for u in updates]
            success = await self.pm.install_packages(packages)
            if success:
                print("✓ All updates installed successfully")
            else:
                print("✗ Some updates failed")
        
        elif choice == "2":
            # Install security updates only
            security_packages = [u.package_name for u in updates if u.security_update]
            if security_packages:
                success = await self.pm.install_packages(security_packages)
                if success:
                    print("✓ Security updates installed successfully")
                else:
                    print("✗ Some security updates failed")
            else:
                print("No security updates found")
        
        elif choice == "3":
            # Select specific packages
            selected = []
            print("Enter package numbers to install (comma-separated):")
            choice = input("Numbers: ").strip()
            
            if choice:
                for num in choice.split(','):
                    if num.strip().isdigit():
                        idx = int(num.strip()) - 1
                        if 0 <= idx < len(updates):
                            selected.append(updates[idx].package_name)
                
                if selected:
                    success = await self.pm.install_packages(selected)
                    if success:
                        print("✓ Selected updates installed successfully")
                    else:
                        print("✗ Some updates failed")
        
        elif choice == "4":
            print("Update cancelled")
    
    async def main_menu(self):
        """Main interactive menu"""
        while True:
            print("\nMultiOS Package Manager")
            print("=" * 30)
            print("1. Search and install packages")
            print("2. Update packages")
            print("3. Remove packages")
            print("4. List installed packages")
            print("5. Check for updates")
            print("6. Package information")
            print("7. Repository management")
            print("8. System maintenance")
            print("9. Status and statistics")
            print("0. Exit")
            
            choice = input("\nSelect option (0-9): ").strip()
            
            try:
                if choice == "1":
                    await self.interactive_search()
                elif choice == "2":
                    await self.interactive_update()
                elif choice == "3":
                    packages = await self.pm.get_installed_packages()
                    if packages:
                        print("\nInstalled packages:")
                        for i, pkg in enumerate(packages, 1):
                            print(f"{i:2d}. {pkg.name:<20} {pkg.version}")
                        
                        choice = input("\nEnter numbers to remove (comma-separated): ").strip()
                        if choice:
                            selected = []
                            for num in choice.split(','):
                                if num.strip().isdigit():
                                    idx = int(num.strip()) - 1
                                    if 0 <= idx < len(packages):
                                        selected.append(packages[idx].name)
                            
                            if selected:
                                if await self.pm.uninstall_packages(selected):
                                    print("✓ Packages removed successfully")
                                else:
                                    print("✗ Some packages could not be removed")
                elif choice == "4":
                    packages = await self.pm.get_installed_packages()
                    print(f"\nInstalled packages ({len(packages)} total):")
                    print("-" * 50)
                    for pkg in packages:
                        print(f"{pkg.name:<25} {pkg.version}")
                elif choice == "5":
                    updates = await self.pm.check_for_updates()
                    if updates:
                        print(f"\nAvailable updates ({len(updates)} total):")
                        print("-" * 50)
                        for update in updates:
                            security_indicator = " [SECURITY]" if update.security_update else ""
                            print(f"{update.package_name:<25} {update.current_version} → {update.available_version}{security_indicator}")
                    else:
                        print("\nNo updates available")
                elif choice == "6":
                    package_name = input("Package name: ").strip()
                    if package_name:
                        pkg_info = await self.pm.get_package_info(package_name)
                        if pkg_info:
                            print(f"\nPackage: {pkg_info.name}")
                            print(f"Version: {pkg_info.version}")
                            print(f"Description: {pkg_info.description}")
                            print(f"Architecture: {pkg_info.architecture}")
                            print(f"Status: {pkg_info.status.value}")
                            print(f"Size: {pkg_info.size} bytes")
                            if pkg_info.dependencies:
                                print(f"Dependencies: {', '.join(pkg_info.dependencies)}")
                        else:
                            print("Package not found")
                elif choice == "7":
                    repos = await self.pm.get_repositories()
                    print(f"\nRepositories ({len(repos)} total):")
                    print("-" * 50)
                    for repo in repos:
                        status = "enabled" if repo.enabled else "disabled"
                        print(f"{repo.name:<20} {status}")
                elif choice == "8":
                    print("\nMaintenance options:")
                    print("1. Check for updates")
                    print("2. Verify installed packages")
                    print("3. Clean up old packages")
                    print("4. Synchronize repositories")
                    
                    choice = input("Select option (1-4): ").strip()
                    
                    if choice == "1":
                        await self.interactive_update()
                    elif choice == "2":
                        results = await self.pm.verify_packages()
                        if results:
                            print(f"\nVerification results ({len(results)} packages):")
                            print("-" * 50)
                            for result in results:
                                icon = "✓" if result.status == VerificationStatus.PASSED else "✗"
                                print(f"{icon} {result.package_name:<25} {result.status.value}")
                    elif choice == "3":
                        result = await self.pm.cleanup()
                        print(f"\nCleanup completed:")
                        print(f"Packages removed: {result.get('packages_removed', 0)}")
                        print(f"Cache size freed: {result.get('cache_size_freed', 0)} bytes")
                    elif choice == "4":
                        if await self.pm.sync_repositories():
                            print("Repositories synchronized successfully")
                        else:
                            print("Failed to synchronize repositories")
                elif choice == "9":
                    status = await self.pm.get_status()
                    print(f"\nPackage Manager Status:")
                    print("-" * 30)
                    for key, value in status.items():
                        print(f"{key.replace('_', ' ').title()}: {value}")
                elif choice == "0":
                    print("Goodbye!")
                    break
                else:
                    print("Invalid option. Please try again.")
            
            except KeyboardInterrupt:
                print("\nOperation cancelled")
            except Exception as e:
                print(f"Error: {e}")
                logger.error(f"Unexpected error: {e}", exc_info=True)


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description="MultiOS Package Manager Python API")
    parser.add_argument("--interactive", action="store_true", help="Run in interactive mode")
    parser.add_argument("--data-dir", default="/var/lib/multios-package-manager", 
                       help="Package manager data directory")
    
    args = parser.parse_args()
    
    if args.interactive:
        cli = PackageManagerCLI()
        asyncio.run(cli.main_menu())
    else:
        print("MultiOS Package Manager Python API")
        print("Usage examples:")
        print("  python3 -c \"from multios_pm import *; pm = MultiOSPackageManager(); print('Ready')\"")
        print("  multios-pm --interactive")


if __name__ == "__main__":
    main()