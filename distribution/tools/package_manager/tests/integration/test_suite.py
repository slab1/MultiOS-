#!/usr/bin/env python3
"""
MultiOS Package Manager Test Suite

Comprehensive testing framework for the MultiOS Package Manager system.
Tests all major components including package operations, repository management,
security features, and CLI interfaces.
"""

import asyncio
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch, AsyncMock
from typing import List, Dict, Any

# Add project paths
sys.path.insert(0, str(Path(__file__).parent.parent / "python"))
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

try:
    import multios_pm
    from multios_pm import Package, Version, PackageManagerCLI
except ImportError as e:
    print(f"Warning: Could not import multios_pm: {e}")
    multios_pm = None


class PackageManagerTestCase(unittest.TestCase):
    """Base test case for package manager tests"""
    
    def setUp(self):
        """Set up test environment"""
        self.test_dir = Path(tempfile.mkdtemp(prefix="multios-pm-test-"))
        self.data_dir = self.test_dir / "data"
        self.data_dir.mkdir()
        
        # Create mock binary
        self.mock_binary = self.test_dir / "multios-pm"
        self.mock_binary.touch()
        self.mock_binary.chmod(0o755)
        
        # Set up test configuration
        self.test_config = {
            "name": "test-package",
            "version": "1.0.0",
            "description": "Test package for unit testing",
            "architecture": "universal",
            "maintainer": "Test <test@example.com>",
            "license": "MIT",
            "dependencies": [],
            "categories": ["testing"],
            "tags": ["test", "unit"]
        }
    
    def tearDown(self):
        """Clean up test environment"""
        import shutil
        shutil.rmtree(self.test_dir)
    
    def create_test_package(self, name: str = "test-pkg", version: str = "1.0.0") -> Dict[str, Any]:
        """Create a test package configuration"""
        package_config = self.test_config.copy()
        package_config["name"] = name
        package_config["version"] = version
        return package_config
    
    def mock_subprocess_run(self, command: List[str], returncode: int = 0, 
                          stdout: str = "", stderr: str = ""):
        """Mock subprocess.run for testing"""
        mock_process = Mock()
        mock_process.returncode = returncode
        mock_process.stdout = stdout.encode('utf-8') if stdout else b''
        mock_process.stderr = stderr.encode('utf-8') if stderr else b''
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_subprocess.return_value = mock_process
            return mock_subprocess


class TestPackageManagerCore(PackageManagerTestCase):
    """Test core package manager functionality"""
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    def test_version_parsing(self):
        """Test version parsing functionality"""
        # Test basic version parsing
        version = Version.parse("1.2.3")
        self.assertEqual(version.major, 1)
        self.assertEqual(version.minor, 2)
        self.assertEqual(version.patch, 3)
        self.assertIsNone(version.pre_release)
        
        # Test version with pre-release
        version = Version.parse("1.2.3-beta")
        self.assertEqual(version.major, 1)
        self.assertEqual(version.minor, 2)
        self.assertEqual(version.patch, 3)
        self.assertEqual(version.pre_release, "beta")
        
        # Test version with build metadata
        version = Version.parse("1.2.3+build.123")
        self.assertEqual(version.major, 1)
        self.assertEqual(version.minor, 2)
        self.assertEqual(version.patch, 3)
        self.assertEqual(version.build_metadata, "build.123")
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    def test_package_creation(self):
        """Test package creation and metadata handling"""
        package_data = {
            "name": "test-app",
            "version": "2.1.0",
            "description": "Test application",
            "architecture": "x86_64",
            "size": 1024000,
            "dependencies": ["libssl3", "glibc"],
            "tags": ["application", "test"]
        }
        
        package = Package.from_dict(package_data)
        
        self.assertEqual(package.name, "test-app")
        self.assertEqual(str(package.version), "2.1.0")
        self.assertEqual(package.architecture, "x86_64")
        self.assertEqual(package.size, 1024000)
        self.assertEqual(len(package.dependencies), 2)
        self.assertIn("application", package.tags)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_package_manager_initialization(self):
        """Test package manager initialization"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Test that it initializes without errors
        self.assertIsNotNone(pm)
        self.assertTrue(self.data_dir.exists())


class TestPackageOperations(PackageManagerTestCase):
    """Test package installation, removal, and updates"""
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_install_packages(self):
        """Test package installation"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock the subprocess call
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(return_value=(b"", b""))
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test installation
            result = await pm.install_packages(["test-package"])
            self.assertTrue(result)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_uninstall_packages(self):
        """Test package uninstallation"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock the subprocess call
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(return_value=(b"", b""))
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test uninstallation
            result = await pm.uninstall_packages(["test-package"])
            self.assertTrue(result)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_search_packages(self):
        """Test package search functionality"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock search results
        search_results = [
            {
                "name": "firefox",
                "version": "91.0",
                "description": "Web browser",
                "architecture": "universal",
                "status": "available",
                "dependencies": [],
                "tags": ["browser", "web"]
            },
            {
                "name": "chromium",
                "version": "94.0",
                "description": "Open source web browser",
                "architecture": "universal", 
                "status": "available",
                "dependencies": [],
                "tags": ["browser", "web"]
            }
        ]
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(
                return_value=(json.dumps({"packages": search_results}).encode('utf-8'), b"")
            )
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test search
            results = await pm.search_packages("browser")
            
            self.assertEqual(len(results), 2)
            self.assertEqual(results[0].name, "firefox")
            self.assertEqual(results[1].name, "chromium")
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_check_for_updates(self):
        """Test update checking functionality"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock update results
        update_results = [
            {
                "package_name": "system-libs",
                "current_version": "2.35",
                "available_version": "2.36",
                "update_type": "minor",
                "security_update": False,
                "delta_available": True,
                "repository": "main"
            }
        ]
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(
                return_value=(json.dumps(update_results).encode('utf-8'), b"")
            )
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test update check
            updates = await pm.check_for_updates()
            
            self.assertEqual(len(updates), 1)
            self.assertEqual(updates[0].package_name, "system-libs")
            self.assertFalse(updates[0].security_update)


class TestRepositoryManagement(PackageManagerTestCase):
    """Test repository management functionality"""
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_add_repository(self):
        """Test adding a new repository"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(return_value=(b"", b""))
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test adding repository
            result = await pm.add_repository(
                "test-repo", 
                "https://example.com/repo", 
                "Test repository"
            )
            self.assertTrue(result)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_list_repositories(self):
        """Test listing repositories"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(
                return_value=(json.dumps(["main", "extra", "community"]).encode('utf-8'), b"")
            )
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Test listing repositories
            repos = await pm.get_repositories()
            
            self.assertEqual(len(repos), 3)
            repo_names = [repo.name for repo in repos]
            self.assertIn("main", repo_names)
            self.assertIn("extra", repo_names)
            self.assertIn("community", repo_names)


class TestPackageBuilder(unittest.TestCase):
    """Test package building functionality"""
    
    def setUp(self):
        """Set up test environment for package builder"""
        self.test_dir = Path(tempfile.mkdtemp(prefix="multios-pm-builder-test-"))
        self.builder_dir = self.test_dir / "builder"
        self.builder_dir.mkdir()
        
        # Import builder module
        sys.path.insert(0, str(self.builder_dir))
    
    def tearDown(self):
        """Clean up test environment"""
        import shutil
        shutil.rmtree(self.test_dir)
    
    def create_minimal_package_config(self) -> Dict[str, Any]:
        """Create minimal package configuration for testing"""
        return {
            "name": "test-builder-package",
            "version": "1.0.0",
            "description": "Test package for builder",
            "architecture": "universal",
            "maintainer": "Test Builder <test@example.com>",
            "license": "MIT",
            "dependencies": [],
            "categories": ["testing"],
            "tags": ["test", "builder"],
            "files": [],
            "install_scripts": {}
        }
    
    def test_package_skeleton_creation(self):
        """Test creating package skeleton"""
        # This would test the skeleton creation functionality
        # Implementation depends on the builder module structure
        pass
    
    def test_package_metadata_generation(self):
        """Test package metadata generation"""
        # This would test metadata generation
        pass
    
    def test_package_building_process(self):
        """Test the complete package building process"""
        # This would test the full build pipeline
        pass


class TestRepositoryBuilder(unittest.TestCase):
    """Test repository building functionality"""
    
    def setUp(self):
        """Set up test environment for repository builder"""
        self.test_dir = Path(tempfile.mkdtemp(prefix="multios-repo-builder-test-"))
        self.repo_dir = self.test_dir / "repository"
        self.repo_dir.mkdir()
        
        # Import repository builder module
        sys.path.insert(0, str(self.test_dir))
    
    def tearDown(self):
        """Clean up test environment"""
        import shutil
        shutil.rmtree(self.test_dir)
    
    def test_repository_creation(self):
        """Test creating a new repository"""
        # This would test repository creation
        pass
    
    def test_package_addition(self):
        """Test adding packages to repository"""
        # This would test package addition
        pass
    
    def test_repository_index_building(self):
        """Test building repository index"""
        # This would test index generation
        pass


class TestIntegrationScenarios(PackageManagerTestCase):
    """Integration tests for common user scenarios"""
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_complete_installation_workflow(self):
        """Test complete package installation workflow"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock successful responses for all operations
        success_responses = [
            (b'{"repositories": ["main"]}', b''),  # Status
            (b'{"packages": []}', b''),             # Search
            (b'{"updates": []}', b''),              # Check updates
            (b'', b'')                              # Install
        ]
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            call_count = 0
            async def mock_communicate():
                nonlocal call_count
                response = success_responses[min(call_count, len(success_responses) - 1)]
                call_count += 1
                return response
            
            mock_process = Mock()
            mock_process.communicate = mock_communicate
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Simulate complete workflow
            status = await pm.get_status()
            self.assertIsInstance(status, dict)
            
            search_results = await pm.search_packages("test")
            self.assertIsInstance(search_results, list)
            
            updates = await pm.check_for_updates()
            self.assertIsInstance(updates, list)
            
            install_success = await pm.install_packages(["test-package"])
            self.assertTrue(install_success)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_update_workflow(self):
        """Test package update workflow"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        # Mock update response
        update_response = {
            "package_name": "test-app",
            "current_version": "1.0.0", 
            "available_version": "1.1.0",
            "update_type": "minor",
            "security_update": False,
            "delta_available": True,
            "repository": "main"
        }
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            call_count = 0
            async def mock_communicate():
                nonlocal call_count
                if call_count == 0:
                    call_count += 1
                    return (json.dumps([update_response]).encode('utf-8'), b'')
                else:
                    call_count += 1
                    return (b'', b'')
            
            mock_process = Mock()
            mock_process.communicate = mock_communicate
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            # Check for updates
            updates = await pm.check_for_updates()
            self.assertEqual(len(updates), 1)
            self.assertEqual(updates[0].package_name, "test-app")
            
            # Perform update
            results = await pm.update_packages(["test-app"])
            self.assertEqual(len(results), 1)
            self.assertTrue(results[0].get('success', False))


class TestErrorHandling(PackageManagerTestCase):
    """Test error handling and edge cases"""
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_invalid_package_installation(self):
        """Test handling of invalid package installation"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(return_value=(b"", b"Package not found".encode('utf-8')))
            mock_process.returncode = 1
            mock_subprocess.return_value = mock_process
            
            # Test installation of non-existent package
            result = await pm.install_packages(["nonexistent-package"])
            self.assertFalse(result)
    
    @unittest.skipIf(multios_pm is None, "multios_pm module not available")
    async def test_network_error_handling(self):
        """Test handling of network errors"""
        pm = multios_pm.MultiOSPackageManager(str(self.data_dir))
        
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = Mock()
            mock_process.communicate = AsyncMock(
                side_effect=Exception("Network connection failed")
            )
            mock_subprocess.return_value = mock_process
            
            # Test search with network error
            results = await pm.search_packages("test")
            self.assertEqual(len(results), 0)


def run_package_manager_tests():
    """Run all package manager tests"""
    print("Running MultiOS Package Manager Test Suite")
    print("=" * 50)
    
    # Discover and run tests
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    # Add test cases
    test_cases = [
        TestPackageManagerCore,
        TestPackageOperations,
        TestRepositoryManagement,
        TestPackageBuilder,
        TestRepositoryBuilder,
        TestIntegrationScenarios,
        TestErrorHandling
    ]
    
    for test_case in test_cases:
        suite.addTests(loader.loadTestsFromTestCase(test_case))
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    # Print summary
    print("\n" + "=" * 50)
    print("Test Summary:")
    print(f"Tests run: {result.testsRun}")
    print(f"Failures: {len(result.failures)}")
    print(f"Errors: {len(result.errors)}")
    print(f"Skipped: {len(result.skipped)}")
    
    if result.failures:
        print("\nFailures:")
        for test, traceback in result.failures:
            print(f"  {test}: {traceback.split('AssertionError:')[-1].strip()}")
    
    if result.errors:
        print("\nErrors:")
        for test, traceback in result.errors:
            print(f"  {test}: {traceback.split('Exception:')[-1].strip()}")
    
    return result.wasSuccessful()


def run_performance_tests():
    """Run performance benchmarks"""
    print("\nRunning Performance Tests")
    print("-" * 30)
    
    # This would include performance benchmarks for:
    # - Package search speed
    # - Repository sync performance
    # - Package installation time
    # - Memory usage under load
    
    print("Performance tests not yet implemented")


def main():
    """Main test runner"""
    import argparse
    
    parser = argparse.ArgumentParser(description="MultiOS Package Manager Test Suite")
    parser.add_argument("--unit-tests", action="store_true", help="Run unit tests")
    parser.add_argument("--integration-tests", action="store_true", help="Run integration tests")
    parser.add_argument("--performance-tests", action="store_true", help="Run performance tests")
    parser.add_argument("--all", action="store_true", help="Run all tests")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    if not any([args.unit_tests, args.integration_tests, args.performance_tests, args.all]):
        args.all = True
    
    success = True
    
    if args.unit_tests or args.all or args.integration_tests:
        if not run_package_manager_tests():
            success = False
    
    if args.performance_tests or args.all:
        run_performance_tests()
    
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())