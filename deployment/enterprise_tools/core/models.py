"""
Core data models for MultiOS Enterprise Deployment System
"""

from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any
from datetime import datetime
from enum import Enum

class SystemType(Enum):
    DESKTOP = "desktop"
    LAPTOP = "laptop"
    TABLET = "tablet"
    SERVER = "server"
    IOT_DEVICE = "iot_device"
    THIN_CLIENT = "thin_client"

class DeploymentStatus(Enum):
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    MAINTENANCE = "maintenance"

class UserRole(Enum):
    ADMIN = "admin"
    TEACHER = "teacher"
    STUDENT = "student"
    SUPPORT = "support"
    GUEST = "guest"

class LicenseType(Enum):
    SINGLE_USER = "single_user"
    SITE_LICENSE = "site_license"
    VOLUME_LICENSE = "volume_license"
    EDUCATIONAL = "educational"

class SystemStatus(Enum):
    ONLINE = "online"
    OFFLINE = "offline"
    DEGRADED = "degraded"
    MAINTENANCE = "maintenance"

@dataclass
class SystemInfo:
    """System hardware and software information"""
    system_id: str
    hostname: str
    ip_address: str
    mac_address: str
    system_type: SystemType
    cpu_model: str
    memory_gb: int
    storage_gb: int
    network_interface: str
    site_id: str
    location: str
    purchase_date: Optional[datetime] = None
    warranty_expires: Optional[datetime] = None
    last_seen: Optional[datetime] = None

@dataclass
class SiteConfig:
    """Configuration for a deployment site"""
    site_id: str
    name: str
    address: str
    network_range: str
    dhcp_range: str
    dns_servers: List[str]
    ntp_servers: List[str]
    proxy_settings: Optional[Dict[str, str]] = None
    timezone: str = "UTC"
    backup_enabled: bool = True
    monitoring_enabled: bool = True

@dataclass
class DeploymentProfile:
    """Template for system deployment configuration"""
    profile_id: str
    name: str
    system_type: SystemType
    base_os_version: str
    required_packages: List[str]
    optional_packages: List[str]
    configuration: Dict[str, Any]
    network_config: Dict[str, str]
    user_groups: List[str]
    allowed_software: List[str]
    security_settings: Dict[str, bool]
    resource_limits: Dict[str, int]
    backup_schedule: Optional[str] = None
    maintenance_window: Optional[str] = None

@dataclass
class UserAccount:
    """User account information"""
    user_id: str
    username: str
    email: str
    full_name: str
    role: UserRole
    site_id: str
    department: str
    created_date: datetime
    last_login: Optional[datetime] = None
    assigned_systems: List[str] = field(default_factory=list)
    group_memberships: List[str] = field(default_factory=list)
    license_assignments: List[str] = field(default_factory=list)
    preferences: Dict[str, Any] = field(default_factory=dict)

@dataclass
class LicenseInfo:
    """Software license tracking"""
    license_id: str
    software_name: str
    license_type: LicenseType
    total_licenses: int
    used_licenses: int
    purchase_date: datetime
    expiry_date: Optional[datetime]
    vendor: str
    license_key: str
    assigned_systems: List[str] = field(default_factory=list)
    compliance_status: bool = True
    cost_per_license: float = 0.0

@dataclass
class SystemStatus:
    """Current system status information"""
    system_id: str
    status: SystemStatus
    uptime: int  # seconds
    cpu_usage: float  # percentage
    memory_usage: float  # percentage
    disk_usage: float  # percentage
    network_traffic: Dict[str, int] = field(default_factory=dict)
    running_services: List[str] = field(default_factory=list)
    error_count: int = 0
    last_updated: datetime = field(default_factory=datetime.now)

@dataclass
class HealthCheck:
    """System health check results"""
    system_id: str
    timestamp: datetime
    overall_status: SystemStatus
    checks: Dict[str, bool] = field(default_factory=dict)
    performance_metrics: Dict[str, float] = field(default_factory=dict)
    issues: List[str] = field(default_factory=list)
    recommendations: List[str] = field(default_factory=list)

@dataclass
class InventoryItem:
    """Hardware and software inventory item"""
    item_id: str
    system_id: str
    item_type: str  # hardware or software
    name: str
    version: str
    manufacturer: Optional[str]
    serial_number: Optional[str]
    install_date: datetime
    last_updated: datetime = field(default_factory=datetime.now)
    warranty_status: Optional[str] = None
    cost: Optional[float] = None
    specifications: Dict[str, str] = field(default_factory=dict)

@dataclass
class UpdatePackage:
    """System update package information"""
    package_id: str
    name: str
    version: str
    package_type: str  # security, feature, bugfix
    size_mb: float
    download_url: str
    checksum: str
    dependencies: List[str] = field(default_factory=list)
    affected_systems: List[str] = field(default_factory=list)
    required: bool = False
    release_date: datetime = field(default_factory=datetime.now)
    description: str = ""

@dataclass
class LabTemplate:
    """Educational lab environment template"""
    template_id: str
    name: str
    description: str
    target_audience: str
    estimated_duration: int  # minutes
    required_systems: int
    software_requirements: List[str] = field(default_factory=list)
    hardware_requirements: Dict[str, int] = field(default_factory=dict)
    network_config: Dict[str, str] = field(default_factory=dict)
    setup_scripts: List[str] = field(default_factory=list)
    cleanup_scripts: List[str] = field(default_factory=list)
    learning_objectives: List[str] = field(default_factory=list)

@dataclass
class ResourceSchedule:
    """Resource allocation and scheduling"""
    schedule_id: str
    resource_type: str  # lab, system, software
    resource_id: str
    start_time: datetime
    end_time: datetime
    user_id: str
    purpose: str
    status: str  # scheduled, in_progress, completed, cancelled
    priority: int = 1
    reservation_notes: Optional[str] = None
