#!/usr/bin/env python3
"""
MultiOS Package Manager Python CLI
Command-line interface wrapper for the MultiOS Package Manager
"""

import asyncio
import argparse
import sys
import json
from pathlib import Path
from typing import List, Optional

# Add the API module to the path
sys.path.insert(0, str(Path(__file__).parent))

from multios_pm import MultiOSPackageManager, PackageManagerCLI


class PackageManagerCLIWrapper:
    """Enhanced CLI wrapper for package manager operations"""
    
    def __init__(self, data_dir: str = "/var/lib/multios-package-manager"):
        self.pm = MultiOSPackageManager(data_dir)
    
    async def install_command(self, packages: List[str], versions: Optional[List[str]] = None, 
                             force: bool = False, repository: Optional[str] = None):
        """Handle install command"""
        print(f"Installing packages: {', '.join(packages)}")
        
        if versions and len(versions) != len(packages):
            print("Error: Number of versions must match number of packages")
            return 1
        
        success = await self.pm.install_packages(packages, versions)
        return 0 if success else 1
    
    async def search_command(self, query: str, description: bool = False, tags: bool = False, 
                           limit: int = 50):
        """Handle search command"""
        packages = await self.pm.search_packages(query, limit)
        
        if not packages:
            print(f"No packages found for '{query}'")
            return 0
        
        print(f"Found {len(packages)} packages for '{query}':")
        print("-" * 70)
        
        for pkg in packages:
            status_icon = "✓" if pkg.status.value == "installed" else "○"
            print(f"{status_icon} {pkg.name:<25} {pkg.version}")
            print(f"    {pkg.description}")
            if pkg.tags and tags:
                print(f"    Tags: {', '.join(pkg.tags)}")
            print()
        
        return 0
    
    async def update_command(self, packages: Optional[List[str]] = None, development: bool = False,
                           security_only: bool = False):
        """Handle update command"""
        if packages:
            print(f"Updating packages: {', '.join(packages)}")
        else:
            print("Updating all packages")
        
        updates = await self.pm.check_for_updates()
        
        if not updates:
            print("No updates available")
            return 0
        
        # Filter updates based on options
        if security_only:
            updates = [u for u in updates if u.security_update]
        
        if packages:
            updates = [u for u in updates if u.package_name in packages]
        
        if not updates:
            print("No matching updates found")
            return 0
        
        print(f"Found {len(updates)} updates:")
        for update in updates:
            security_indicator = " [SECURITY]" if update.security_update else ""
            print(f"  {update.package_name}: {update.current_version} → {update.available_version}{security_indicator}")
        
        # Confirm before updating
        if not packages:  # Only ask for confirmation if updating all packages
            response = input("\nProceed with updates? (y/N): ").strip().lower()
            if response not in ['y', 'yes']:
                print("Update cancelled")
                return 0
        
        # Perform updates
        success = await self.pm.update_packages(packages)
        
        if success:
            print("Updates completed successfully")
        else:
            print("Some updates failed")
        
        return 0 if success else 1
    
    async def list_command(self, architecture: Optional[str] = None, detailed: bool = False,
                          sort_by_name: bool = False):
        """Handle list command"""
        packages = await self.pm.get_installed_packages()
        
        # Filter by architecture
        if architecture:
            packages = [p for p in packages if p.architecture == architecture]
        
        # Sort
        if sort_by_name:
            packages.sort(key=lambda p: p.name)
        
        if not packages:
            print("No packages installed")
            return 0
        
        if detailed:
            print(f"Installed packages ({len(packages)} total):")
            print("-" * 70)
            for pkg in packages:
                print(f"{pkg.name:<25} {pkg.version}")
                print(f"  Description: {pkg.description}")
                print(f"  Architecture: {pkg.architecture}")
                print(f"  Size: {pkg.size} bytes")
                if pkg.install_date:
                    print(f"  Installed: {pkg.install_date}")
                print()
        else:
            print(f"Installed packages ({len(packages)} total):")
            print("-" * 50)
            for pkg in packages:
                print(f"{pkg.name:<25} {pkg.version}")
        
        return 0
    
    async def info_command(self, package: str, dependencies: bool = False, files: bool = False,
                         security: bool = False):
        """Handle info command"""
        pkg_info = await self.pm.get_package_info(package)
        
        if not pkg_info:
            print(f"Package '{package}' not found")
            return 1
        
        print(f"Package: {pkg_info.name}")
        print(f"Version: {pkg_info.version}")
        print(f"Description: {pkg_info.description}")
        print(f"Architecture: {pkg_info.architecture}")
        print(f"Status: {pkg_info.status.value}")
        print(f"Size: {pkg_info.size} bytes")
        
        if pkg_info.install_date:
            print(f"Install Date: {pkg_info.install_date}")
        
        if pkg_info.repository:
            print(f"Repository: {pkg_info.repository}")
        
        if dependencies and pkg_info.dependencies:
            print(f"\nDependencies:")
            for dep in pkg_info.dependencies:
                print(f"  - {dep}")
        
        if security and pkg_info.security_info:
            print(f"\nSecurity Information:")
            for key, value in pkg_info.security_info.items():
                print(f"  {key}: {value}")
        
        return 0
    
    async def verify_command(self, packages: Optional[List[str]] = None, fix: bool = False):
        """Handle verify command"""
        results = await self.pm.verify_packages(packages)
        
        if not results:
            print("No packages to verify")
            return 0
        
        print(f"Verification results ({len(results)} packages):")
        print("-" * 50)
        
        passed = 0
        failed = 0
        warnings = 0
        
        for result in results:
            if result.status.value == "passed":
                print(f"✓ {result.package_name:<25} {result.version} - PASS")
                passed += 1
            elif result.status.value == "failed":
                print(f"✗ {result.package_name:<25} {result.version} - FAIL")
                failed += 1
                for issue in result.issues:
                    print(f"    {issue}")
            elif result.status.value == "warning":
                print(f"⚠ {result.package_name:<25} {result.version} - WARNING")
                warnings += 1
        
        print(f"\nSummary:")
        print(f"  Passed: {passed}")
        print(f"  Failed: {failed}")
        print(f"  Warnings: {warnings}")
        
        if failed > 0:
            print("\nRun with --fix to attempt automatic fixes")
            return 1
        
        return 0
    
    async def check_updates_command(self, development: bool = False, security_only: bool = False,
                                  json_output: bool = False):
        """Handle check-updates command"""
        updates = await self.pm.check_for_updates()
        
        # Filter updates
        if security_only:
            updates = [u for u in updates if u.security_update]
        
        if json_output:
            result = {
                'updates': [
                    {
                        'package_name': u.package_name,
                        'current_version': str(u.current_version),
                        'available_version': str(u.available_version),
                        'update_type': u.update_type.value,
                        'security_update': u.security_update,
                        'delta_available': u.delta_available,
                        'repository': u.repository
                    }
                    for u in updates
                ]
            }
            print(json.dumps(result, indent=2))
        else:
            if not updates:
                print("No updates available")
            else:
                print(f"Available updates ({len(updates)}):")
                for update in updates:
                    security_indicator = " [SECURITY]" if update.security_update else ""
                    print(f"  {update.package_name}: {update.current_version} → {update.available_version}{security_indicator}")
        
        return 0
    
    async def sync_command(self, force: bool = False, repository: Optional[str] = None):
        """Handle sync command"""
        if repository:
            print(f"Synchronizing repository: {repository}")
        else:
            print("Synchronizing all repositories")
        
        success = await self.pm.sync_repositories(force)
        
        if success:
            print("Repositories synchronized successfully")
        else:
            print("Failed to synchronize repositories")
        
        return 0 if success else 1
    
    async def rollback_command(self, package: str, version: str, backup: bool = True):
        """Handle rollback command"""
        print(f"Rolling back {package} to version {version}")
        
        if backup:
            print("Creating backup before rollback...")
        
        success = await self.pm.rollback_package(package, version, backup)
        
        if success:
            print(f"Successfully rolled back {package} to version {version}")
        else:
            print(f"Failed to roll back {package}")
        
        return 0 if success else 1
    
    async def status_command(self, detailed: bool = False, json_output: bool = False):
        """Handle status command"""
        status = await self.pm.get_status()
        
        if json_output:
            print(json.dumps(status, indent=2))
        else:
            print("Package Manager Status:")
            print("-" * 30)
            for key, value in status.items():
                key_formatted = key.replace('_', ' ').title()
                print(f"{key_formatted:<20}: {value}")
        
        return 0
    
    async def cleanup_command(self, old_versions: bool = True, clear_cache: bool = False,
                            orphaned: bool = True, dry_run: bool = False):
        """Handle cleanup command"""
        print("Cleaning up package system...")
        
        result = await self.pm.cleanup(old_versions, clear_cache, orphaned, dry_run)
        
        if result:
            print("Cleanup completed:")
            print(f"  Packages removed: {result.get('packages_removed', 0)}")
            print(f"  Cache size freed: {result.get('cache_size_freed', 0)} bytes")
            print(f"  Temporary files removed: {result.get('temporary_files_removed', 0)}")
        else:
            print("Cleanup failed or no cleanup needed")
        
        return 0
    
    async def repository_command(self, subcommand: str, **kwargs):
        """Handle repository subcommands"""
        if subcommand == "list":
            repos = await self.pm.get_repositories()
            print(f"Repositories ({len(repos)} total):")
            print("-" * 50)
            for repo in repos:
                status = "enabled" if repo.enabled else "disabled"
                print(f"{repo.name:<20} {status:<8} {repo.url}")
                if repo.description:
                    print(f"  {repo.description}")
        
        elif subcommand == "add":
            name = kwargs.get('name')
            url = kwargs.get('url')
            description = kwargs.get('description', '')
            priority = kwargs.get('priority', 0)
            
            if not name or not url:
                print("Error: name and url are required")
                return 1
            
            success = await self.pm.add_repository(name, url, description, priority)
            
            if success:
                print(f"Repository '{name}' added successfully")
            else:
                print(f"Failed to add repository '{name}'")
            
            return 0 if success else 1
        
        elif subcommand == "remove":
            name = kwargs.get('name')
            
            if not name:
                print("Error: repository name is required")
                return 1
            
            success = await self.pm.remove_repository(name)
            
            if success:
                print(f"Repository '{name}' removed successfully")
            else:
                print(f"Failed to remove repository '{name}'")
            
            return 0 if success else 1
        
        else:
            print(f"Unknown repository subcommand: {subcommand}")
            return 1
    
    async def configure_command(self, auto_check: bool = None, auto_install: bool = None,
                              check_interval: Optional[int] = None,
                              maintenance_day: Optional[str] = None,
                              maintenance_time: Optional[str] = None,
                              show: bool = False):
        """Handle configure command"""
        if show:
            print("Configuration display not yet implemented")
            return 0
        
        if all(v is None for v in [auto_check, auto_install, check_interval, 
                                  maintenance_day, maintenance_time]):
            print("No configuration changes specified")
            return 0
        
        # This would require implementation of configuration management
        print("Configuration update not yet implemented")
        return 0
    
    async def sign_command(self, subcommand: str, **kwargs):
        """Handle sign subcommands"""
        print(f"Package signing operations not yet implemented")
        print(f"Subcommand: {subcommand}")
        return 0
    
    async def export_command(self, format_type: str, include_files: bool = False,
                           output: Optional[str] = None):
        """Handle export command"""
        print("Package export not yet implemented")
        print(f"Format: {format_type}")
        print(f"Include files: {include_files}")
        print(f"Output: {output}")
        return 0


async def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(
        description="MultiOS Package Manager Python CLI",
        prog="multios-pm-python"
    )
    
    # Global options
    parser.add_argument("--data-dir", default="/var/lib/multios-package-manager",
                       help="Package manager data directory")
    parser.add_argument("--interactive", action="store_true",
                       help="Run in interactive mode")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose output")
    
    # Subcommands
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Install command
    install_parser = subparsers.add_parser("install", help="Install packages")
    install_parser.add_argument("packages", nargs="+", help="Package names to install")
    install_parser.add_argument("--version", action="append", help="Package versions")
    install_parser.add_argument("--force", action="store_true", help="Force installation")
    install_parser.add_argument("--repository", help="Specific repository")
    
    # Search command
    search_parser = subparsers.add_parser("search", help="Search for packages")
    search_parser.add_argument("query", help="Search query")
    search_parser.add_argument("--description", action="store_true", 
                              help="Search in descriptions")
    search_parser.add_argument("--tags", action="store_true", help="Search in tags")
    search_parser.add_argument("--limit", type=int, default=50, help="Result limit")
    
    # Update command
    update_parser = subparsers.add_parser("update", help="Update packages")
    update_parser.add_argument("packages", nargs="*", help="Package names to update")
    update_parser.add_argument("--development", action="store_true",
                              help="Include development packages")
    update_parser.add_argument("--security-only", action="store_true",
                              help="Update only security fixes")
    
    # List command
    list_parser = subparsers.add_parser("list", help="List installed packages")
    list_parser.add_argument("--architecture", help="Filter by architecture")
    list_parser.add_argument("--detailed", "-d", action="store_true", help="Detailed output")
    list_parser.add_argument("--sort", action="store_true", help="Sort by name")
    
    # Info command
    info_parser = subparsers.add_parser("info", help="Show package information")
    info_parser.add_argument("package", help="Package name")
    info_parser.add_argument("--dependencies", action="store_true", help="Show dependencies")
    info_parser.add_argument("--files", action="store_true", help="Show files")
    info_parser.add_argument("--security", action="store_true", help="Show security info")
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify installed packages")
    verify_parser.add_argument("packages", nargs="*", help="Packages to verify")
    verify_parser.add_argument("--fix", action="store_true", help="Fix detected issues")
    
    # Check-updates command
    check_parser = subparsers.add_parser("check-updates", help="Check for updates")
    check_parser.add_argument("--development", action="store_true",
                             help="Include development packages")
    check_parser.add_argument("--security-only", action="store_true",
                             help="Show only security updates")
    check_parser.add_argument("--json", action="store_true", help="JSON output")
    
    # Sync command
    sync_parser = subparsers.add_parser("sync", help="Synchronize repositories")
    sync_parser.add_argument("--force", action="store_true", help="Force refresh")
    sync_parser.add_argument("--repository", help="Specific repository")
    
    # Rollback command
    rollback_parser = subparsers.add_parser("rollback", help="Rollback package")
    rollback_parser.add_argument("package", help="Package name")
    rollback_parser.add_argument("version", help="Target version")
    rollback_parser.add_argument("--no-backup", action="store_true", help="Skip backup")
    
    # Status command
    status_parser = subparsers.add_parser("status", help="Show status")
    status_parser.add_argument("--detailed", "-d", action="store_true", help="Detailed output")
    status_parser.add_argument("--json", action="store_true", help="JSON output")
    
    # Cleanup command
    cleanup_parser = subparsers.add_parser("cleanup", help="Clean up system")
    cleanup_parser.add_argument("--old-versions", action="store_true", help="Remove old versions")
    cleanup_parser.add_argument("--clear-cache", action="store_true", help="Clear cache")
    cleanup_parser.add_argument("--orphaned", action="store_true", help="Remove orphaned packages")
    cleanup_parser.add_argument("--dry-run", action="store_true", help="Preview changes")
    
    # Repository subcommands
    repo_parser = subparsers.add_parser("repository", help="Manage repositories")
    repo_subparsers = repo_parser.add_subparsers(dest="subcommand")
    
    repo_add = repo_subparsers.add_parser("add", help="Add repository")
    repo_add.add_argument("name", help="Repository name")
    repo_add.add_argument("url", help="Repository URL")
    repo_add.add_argument("--description", help="Description")
    repo_add.add_argument("--priority", type=int, default=0, help="Priority")
    
    repo_remove = repo_subparsers.add_parser("remove", help="Remove repository")
    repo_remove.add_argument("name", help="Repository name")
    
    repo_list = repo_subparsers.add_parser("list", help="List repositories")
    
    # Configure command
    config_parser = subparsers.add_parser("configure", help="Configure settings")
    config_parser.add_argument("--auto-check", action="store_true", help="Enable auto-check")
    config_parser.add_argument("--auto-install", action="store_true", help="Enable auto-install")
    config_parser.add_argument("--check-interval", type=int, help="Check interval (hours)")
    config_parser.add_argument("--maintenance-day", help="Maintenance day")
    config_parser.add_argument("--maintenance-time", help="Maintenance time")
    config_parser.add_argument("--show", action="store_true", help="Show current config")
    
    # Sign subcommands
    sign_parser = subparsers.add_parser("sign", help="Package signing")
    sign_subparsers = sign_parser.add_subparsers(dest="subcommand")
    
    sign_genkey = sign_subparsers.add_parser("generate-key", help="Generate key")
    sign_genkey.add_argument("key_id", help="Key ID")
    
    sign_sign = sign_subparsers.add_parser("sign", help="Sign package")
    sign_sign.add_argument("package", help="Package file")
    sign_sign.add_argument("key_id", help="Key ID")
    
    sign_verify = sign_subparsers.add_parser("verify", help="Verify signature")
    sign_verify.add_argument("package", help="Package file")
    sign_verify.add_argument("key_id", help="Key ID")
    
    sign_trust = sign_subparsers.add_parser("trust", help="Trust key")
    sign_trust.add_argument("key_id", help="Key ID")
    
    sign_revoke = sign_subparsers.add_parser("revoke", help="Revoke key")
    sign_revoke.add_argument("key_id", help="Key ID")
    
    # Export command
    export_parser = subparsers.add_parser("export", help="Export packages")
    export_parser.add_argument("--format", choices=["json", "csv", "xml"], 
                              default="json", help="Output format")
    export_parser.add_argument("--include-files", action="store_true", help="Include files")
    export_parser.add_argument("--output", help="Output file")
    
    args = parser.parse_args()
    
    if args.interactive:
        cli = PackageManagerCLI()
        await cli.main_menu()
        return 0
    
    if not args.command:
        parser.print_help()
        return 1
    
    cli_wrapper = PackageManagerCLIWrapper(args.data_dir)
    
    try:
        if args.command == "install":
            return await cli_wrapper.install_command(
                args.packages, args.version, args.force, args.repository
            )
        
        elif args.command == "search":
            return await cli_wrapper.search_command(
                args.query, args.description, args.tags, args.limit
            )
        
        elif args.command == "update":
            return await cli_wrapper.update_command(
                args.packages, args.development, args.security_only
            )
        
        elif args.command == "list":
            return await cli_wrapper.list_command(
                args.architecture, args.detailed, args.sort
            )
        
        elif args.command == "info":
            return await cli_wrapper.info_command(
                args.package, args.dependencies, args.files, args.security
            )
        
        elif args.command == "verify":
            return await cli_wrapper.verify_command(args.packages, args.fix)
        
        elif args.command == "check-updates":
            return await cli_wrapper.check_updates_command(
                args.development, args.security_only, args.json
            )
        
        elif args.command == "sync":
            return await cli_wrapper.sync_command(args.force, args.repository)
        
        elif args.command == "rollback":
            return await cli_wrapper.rollback_command(
                args.package, args.version, not args.no_backup
            )
        
        elif args.command == "status":
            return await cli_wrapper.status_command(args.detailed, args.json)
        
        elif args.command == "cleanup":
            return await cli_wrapper.cleanup_command(
                args.old_versions, args.clear_cache, args.orphaned, args.dry_run
            )
        
        elif args.command == "repository":
            return await cli_wrapper.repository_command(
                args.subcommand, **vars(args)
            )
        
        elif args.command == "configure":
            return await cli_wrapper.configure_command(
                args.auto_check, args.auto_install, args.check_interval,
                args.maintenance_day, args.maintenance_time, args.show
            )
        
        elif args.command == "sign":
            return await cli_wrapper.sign_command(args.subcommand, **vars(args))
        
        elif args.command == "export":
            return await cli_wrapper.export_command(
                args.format, args.include_files, args.output
            )
        
        else:
            print(f"Unknown command: {args.command}")
            return 1
    
    except KeyboardInterrupt:
        print("\nOperation cancelled")
        return 1
    except Exception as e:
        print(f"Error: {e}")
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)