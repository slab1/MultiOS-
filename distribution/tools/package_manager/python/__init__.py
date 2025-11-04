"""
MultiOS Package Manager Python API

A comprehensive Python interface to the MultiOS Package Manager system.
Provides both programmatic API and command-line tools for package management.

Example usage:
    from multios_pm import MultiOSPackageManager
    
    pm = MultiOSPackageManager()
    
    # Install packages
    await pm.install_packages(['firefox', 'git'])
    
    # Search for packages
    packages = await pm.search_packages('web browser')
    
    # Check for updates
    updates = await pm.check_for_updates()
"""

from .api import (
    MultiOSPackageManager,
    PackageManagerCLI,
    Package,
    PackageStatus,
    Version,
    UpdateInfo,
    UpdateType,
    VerificationResult,
    VerificationStatus,
    Repository
)

__version__ = "0.1.0"
__author__ = "MultiOS Team"
__email__ = "team@multios.org"

__all__ = [
    "MultiOSPackageManager",
    "PackageManagerCLI", 
    "Package",
    "PackageStatus",
    "Version",
    "UpdateInfo",
    "UpdateType", 
    "VerificationResult",
    "VerificationStatus",
    "Repository"
]