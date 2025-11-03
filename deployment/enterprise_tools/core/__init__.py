"""
MultiOS Enterprise Deployment Tools
===================================

Enterprise-grade deployment tools for large-scale educational institution deployments.
Supports 1000+ systems with centralized management, monitoring, and automation.

Author: MultiOS Development Team
License: MIT
Version: 1.0.0
"""

from .manager import DeploymentManager
from .models import *
from .utils import *

__version__ = "1.0.0"
__author__ = "MultiOS Development Team"

__all__ = [
    "DeploymentManager",
    "SystemInfo",
    "SiteConfig",
    "DeploymentProfile",
    "UserAccount",
    "LicenseInfo",
    "SystemStatus",
    "HealthCheck",
    "InventoryItem",
    "UpdatePackage",
    "LabTemplate",
    "ResourceSchedule"
]