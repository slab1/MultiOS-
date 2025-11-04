"""
Main deployment manager for MultiOS Enterprise Deployment System
"""

import logging
import asyncio
from typing import Dict, List, Optional, Any
from datetime import datetime
from pathlib import Path
import threading
from concurrent.futures import ThreadPoolExecutor

try:
    from .models import *
    from .utils import *
    from ..pxe_installer.pxe_server import PXEServer
    from ..config_management.template_manager import TemplateManager
    from ..user_management.user_manager import UserManager
    from ..license_tracking.license_manager import LicenseManager
    from ..system_monitoring.monitor import SystemMonitor
    from ..software_deployment.package_manager import PackageManager
    from ..update_distribution.update_server import UpdateServer
    from ..inventory_management.inventory import InventoryManager
    from ..ldap_integration.directory_integration import DirectoryIntegration
    from ..automation.deployment_automation import DeploymentAutomation
    from ..lab_templates.lab_manager import LabManager
    from ..resource_scheduling.scheduler import ResourceScheduler
    from ..analytics.analytics_engine import AnalyticsEngine
except ImportError:
    # Fallback for direct script execution
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).parent.parent))
    
    from core.models import *
    from core.utils import *
    from pxe_installer.pxe_server import PXEServer
    from config_management.template_manager import TemplateManager
    from user_management.user_manager import UserManager
    from license_tracking.license_manager import LicenseManager
    from system_monitoring.monitor import SystemMonitor
    from software_deployment.package_manager import PackageManager
    from update_distribution.update_server import UpdateServer
    from inventory_management.inventory import InventoryManager
    from ldap_integration.directory_integration import DirectoryIntegration
    from automation.deployment_automation import DeploymentAutomation
    from lab_templates.lab_manager import LabManager
    from resource_scheduling.scheduler import ResourceScheduler
    from analytics.analytics_engine import AnalyticsEngine

class DeploymentManager:
    """Main deployment manager coordinating all enterprise tools"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/config.yaml"):
        self.config_path = config_path
        self.config = load_config(config_path)
        self.logger = logging.getLogger(__name__)
        
        # Initialize all managers
        self.pxe_server = PXEServer()
        self.template_manager = TemplateManager()
        self.user_manager = UserManager()
        self.license_manager = LicenseManager()
        self.monitor = SystemMonitor()
        self.package_manager = PackageManager()
        self.update_server = UpdateServer()
        self.inventory = InventoryManager()
        self.directory_integration = DirectoryIntegration()
        self.automation = DeploymentAutomation()
        self.lab_manager = LabManager()
        self.scheduler = ResourceScheduler()
        self.analytics = AnalyticsEngine()
        
        # Deployment state
        self.active_deployments = {}
        self.system_registry = {}
        self.site_configs = {}
        
        self._setup_logging()
        self._load_config()
    
    def _setup_logging(self) -> None:
        """Setup logging configuration"""
        log_level = self.config.get('logging', {}).get('level', 'INFO')
        set_logging_level(log_level)
        
        log_file = self.config.get('logging', {}).get('file', '/var/log/multios-enterprise.log')
        Path(log_file).parent.mkdir(parents=True, exist_ok=True)
    
    def _load_config(self) -> None:
        """Load and validate configuration"""
        try:
            if not self.config:
                self.logger.warning(f"Configuration file not found at {self.config_path}")
                self.config = self._create_default_config()
            
            # Initialize site configurations
            for site_config in self.config.get('sites', {}).values():
                site = SiteConfig(**site_config)
                self.site_configs[site.site_id] = site
                
            self.logger.info(f"Loaded configuration for {len(self.site_configs)} sites")
            
        except Exception as e:
            self.logger.error(f"Failed to load configuration: {e}")
            raise
    
    def _create_default_config(self) -> Dict[str, Any]:
        """Create default configuration"""
        return {
            'sites': {},
            'deployment': {
                'max_concurrent_deployments': 100,
                'default_timeout': 3600,
                'retry_attempts': 3,
                'backup_enabled': True
            },
            'monitoring': {
                'interval': 300,
                'health_check_enabled': True,
                'alert_thresholds': {
                    'cpu_usage': 80,
                    'memory_usage': 85,
                    'disk_usage': 90
                }
            },
            'security': {
                'ldap_enabled': False,
                'ssl_enabled': True,
                'audit_logging': True
            },
            'logging': {
                'level': 'INFO',
                'file': '/var/log/multios-enterprise.log'
            }
        }
    
    def register_system(self, system_info: SystemInfo) -> bool:
        """Register a new system in the enterprise deployment system"""
        try:
            self.system_registry[system_info.system_id] = system_info
            self.inventory.add_item(system_info)
            self.logger.info(f"Registered system {system_info.hostname} ({system_info.system_id})")
            return True
        except Exception as e:
            self.logger.error(f"Failed to register system: {e}")
            return False
    
    def unregister_system(self, system_id: str) -> bool:
        """Remove a system from the registry"""
        try:
            if system_id in self.system_registry:
                del self.system_registry[system_id]
                self.inventory.remove_system(system_id)
                self.logger.info(f"Unregistered system {system_id}")
                return True
            return False
        except Exception as e:
            self.logger.error(f"Failed to unregister system {system_id}: {e}")
            return False
    
    def start_deployment(self, profile: DeploymentProfile, target_systems: List[str]) -> str:
        """Start deployment of a profile to target systems"""
        try:
            deployment_id = generate_system_id()
            
            deployment = {
                'deployment_id': deployment_id,
                'profile': profile,
                'target_systems': target_systems,
                'status': DeploymentStatus.PENDING,
                'start_time': datetime.now(),
                'progress': 0,
                'results': {}
            }
            
            self.active_deployments[deployment_id] = deployment
            
            # Start deployment in background thread
            thread = threading.Thread(
                target=self._execute_deployment,
                args=(deployment_id,)
            )
            thread.start()
            
            self.logger.info(f"Started deployment {deployment_id} to {len(target_systems)} systems")
            return deployment_id
            
        except Exception as e:
            self.logger.error(f"Failed to start deployment: {e}")
            raise
    
    def _execute_deployment(self, deployment_id: str) -> None:
        """Execute deployment in background thread"""
        try:
            deployment = self.active_deployments[deployment_id]
            deployment['status'] = DeploymentStatus.IN_PROGRESS
            target_systems = deployment['target_systems']
            
            max_concurrent = self.config.get('deployment', {}).get('max_concurrent_deployments', 100)
            
            # Process systems in batches to avoid overwhelming the network
            for i in range(0, len(target_systems), max_concurrent):
                batch = target_systems[i:i + max_concurrent]
                
                with ThreadPoolExecutor(max_workers=max_concurrent) as executor:
                    futures = []
                    
                    for system_id in batch:
                        future = executor.submit(self._deploy_to_system, deployment_id, system_id)
                        futures.append(future)
                    
                    # Wait for batch to complete
                    for future in futures:
                        try:
                            result = future.result(timeout=3600)  # 1 hour timeout per system
                            deployment['results'][result['system_id']] = result
                        except Exception as e:
                            self.logger.error(f"Deployment to system failed: {e}")
                
                # Update progress
                completed = len([r for r in deployment['results'].values() if r.get('success', False)])
                deployment['progress'] = (completed / len(target_systems)) * 100
            
            # Mark deployment as completed or failed
            successful = sum(1 for r in deployment['results'].values() if r.get('success', False))
            if successful == len(target_systems):
                deployment['status'] = DeploymentStatus.COMPLETED
            elif successful == 0:
                deployment['status'] = DeploymentStatus.FAILED
            else:
                deployment['status'] = DeploymentStatus.COMPLETED  # Partial success
            
            deployment['end_time'] = datetime.now()
            
            self.logger.info(f"Deployment {deployment_id} completed: {successful}/{len(target_systems)} successful")
            
        except Exception as e:
            self.logger.error(f"Deployment {deployment_id} failed: {e}")
            deployment['status'] = DeploymentStatus.FAILED
            deployment['error'] = str(e)
    
    def _deploy_to_system(self, deployment_id: str, system_id: str) -> Dict[str, Any]:
        """Deploy profile to a specific system"""
        try:
            deployment = self.active_deployments[deployment_id]
            profile = deployment['profile']
            system = self.system_registry.get(system_id)
            
            if not system:
                return {
                    'system_id': system_id,
                    'success': False,
                    'error': 'System not found in registry'
                }
            
            # Install base OS via PXE if needed
            if profile.base_os_version not in self._get_installed_versions(system_id):
                self.pxe_server.install_os(system, profile.base_os_version)
            
            # Apply configuration template
            self.template_manager.apply_template(system, profile)
            
            # Install required packages
            for package in profile.required_packages:
                self.package_manager.install_package(system_id, package)
            
            # Configure network settings
            self._configure_network(system, profile.network_config)
            
            # Setup user accounts
            for group in profile.user_groups:
                self._setup_user_group(system_id, group)
            
            # Configure monitoring
            if self.config.get('monitoring', {}).get('health_check_enabled', True):
                self.monitor.enable_monitoring(system_id)
            
            # Update inventory
            self.inventory.update_system_status(system_id, SystemStatus.ONLINE)
            
            return {
                'system_id': system_id,
                'success': True,
                'timestamp': datetime.now().isoformat()
            }
            
        except Exception as e:
            self.logger.error(f"Deployment to system {system_id} failed: {e}")
            return {
                'system_id': system_id,
                'success': False,
                'error': str(e),
                'timestamp': datetime.now().isoformat()
            }
    
    def get_deployment_status(self, deployment_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a deployment"""
        return self.active_deployments.get(deployment_id)
    
    def cancel_deployment(self, deployment_id: str) -> bool:
        """Cancel an active deployment"""
        try:
            if deployment_id in self.active_deployments:
                # Note: In production, this would need to send cancellation signals to running processes
                self.active_deployments[deployment_id]['status'] = DeploymentStatus.FAILED
                self.active_deployments[deployment_id]['cancelled'] = True
                return True
            return False
        except Exception as e:
            self.logger.error(f"Failed to cancel deployment {deployment_id}: {e}")
            return False
    
    def create_site(self, site_config: SiteConfig) -> bool:
        """Create a new deployment site"""
        try:
            self.site_configs[site_config.site_id] = site_config
            
            # Start site-specific services
            self.pxe_server.add_site(site_config)
            self.update_server.add_site(site_config)
            self.monitor.add_site(site_config)
            
            # Save to configuration
            self.config['sites'][site_config.site_id] = {
                'site_id': site_config.site_id,
                'name': site_config.name,
                'address': site_config.address,
                'network_range': site_config.network_range,
                'dhcp_range': site_config.dhcp_range,
                'dns_servers': site_config.dns_servers,
                'ntp_servers': site_config.ntp_servers,
                'timezone': site_config.timezone,
                'backup_enabled': site_config.backup_enabled,
                'monitoring_enabled': site_config.monitoring_enabled
            }
            
            save_config(self.config_path, self.config)
            self.logger.info(f"Created site {site_config.name} ({site_config.site_id})")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to create site: {e}")
            return False
    
    def get_site_systems(self, site_id: str) -> List[SystemInfo]:
        """Get all systems at a specific site"""
        return [
            system for system in self.system_registry.values()
            if system.site_id == site_id
        ]
    
    def start_monitoring(self) -> None:
        """Start system monitoring for all registered systems"""
        self.monitor.start_monitoring()
        self.logger.info("Started system monitoring")
    
    def stop_monitoring(self) -> None:
        """Stop system monitoring"""
        self.monitor.stop_monitoring()
        self.logger.info("Stopped system monitoring")
    
    def get_system_health(self, system_id: str) -> Optional[HealthCheck]:
        """Get health status of a specific system"""
        return self.monitor.get_health_check(system_id)
    
    def get_all_systems_health(self) -> Dict[str, HealthCheck]:
        """Get health status of all systems"""
        return self.monitor.get_all_health_checks()
    
    def schedule_maintenance(self, system_ids: List[str], start_time: datetime, duration: int) -> str:
        """Schedule maintenance window for systems"""
        schedule_id = generate_system_id()
        
        maintenance = {
            'schedule_id': schedule_id,
            'system_ids': system_ids,
            'start_time': start_time,
            'duration_minutes': duration,
            'status': 'scheduled',
            'created_by': 'system'
        }
        
        self.scheduler.schedule_maintenance(maintenance)
        self.logger.info(f"Scheduled maintenance {schedule_id} for {len(system_ids)} systems")
        return schedule_id
    
    def generate_analytics_report(self, report_type: str, start_date: datetime, end_date: datetime) -> Dict[str, Any]:
        """Generate analytics report"""
        return self.analytics.generate_report(report_type, start_date, end_date)
    
    def export_inventory_report(self, format_type: str = 'json') -> str:
        """Export complete inventory report"""
        return self.inventory.export_report(format_type)
    
    def sync_with_ldap(self) -> bool:
        """Synchronize user accounts with LDAP directory"""
        if not self.config.get('security', {}).get('ldap_enabled', False):
            self.logger.warning("LDAP integration not enabled")
            return False
        
        try:
            return self.directory_integration.sync_users()
        except Exception as e:
            self.logger.error(f"LDAP synchronization failed: {e}")
            return False
    
    def _get_installed_versions(self, system_id: str) -> List[str]:
        """Get list of installed OS versions on a system"""
        # This would query the actual system
        return ['1.0.0']  # Placeholder
    
    def _configure_network(self, system: SystemInfo, network_config: Dict[str, str]) -> bool:
        """Configure network settings for a system"""
        try:
            # This would apply network configuration via SSH or other remote method
            self.logger.info(f"Configured network for system {system.hostname}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to configure network for {system.hostname}: {e}")
            return False
    
    def _setup_user_group(self, system_id: str, group_name: str) -> bool:
        """Setup user group on a system"""
        try:
            # This would create groups via SSH or other remote method
            self.logger.info(f"Setup user group {group_name} on system {system_id}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to setup group {group_name} on {system_id}: {e}")
            return False
    
    def shutdown(self) -> None:
        """Shutdown deployment manager and all services"""
        try:
            self.stop_monitoring()
            
            # Stop all active deployments gracefully
            for deployment_id in list(self.active_deployments.keys()):
                if self.active_deployments[deployment_id]['status'] == DeploymentStatus.IN_PROGRESS:
                    self.cancel_deployment(deployment_id)
            
            self.logger.info("Deployment manager shutdown complete")
            
        except Exception as e:
            self.logger.error(f"Error during shutdown: {e}")
