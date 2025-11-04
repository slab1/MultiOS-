"""
Centralized Update Distribution Server for MultiOS Enterprise
"""

import os
import json
import logging
import hashlib
import shutil
import threading
import time
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
from concurrent.futures import ThreadPoolExecutor
import http.server
import socketserver
from urllib.parse import urlparse, parse_qs
import zipfile
import tarfile

from ..core.models import UpdatePackage, SystemInfo
from ..core.utils import execute_command, calculate_md5, download_file

class UpdateServer:
    """Centralized update distribution server"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/updates.yaml"):
        self.config_path = config_path
        self.sites = {}
        self.update_packages = {}
        self.distribution_queue = {}
        self.server_active = False
        self.http_server = None
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
        self._setup_directories()
        self._load_update_catalog()
    
    def _load_configuration(self) -> None:
        """Load update server configuration"""
        self.config = {
            'server': {
                'host': '0.0.0.0',
                'port': 8080,
                'document_root': '/var/lib/multios-enterprise/updates',
                'max_connections': 100
            },
            'updates': {
                'auto_approval': False,
                'require_approval': True,
                'download_timeout': 3600,  # 1 hour
                'concurrent_downloads': 10
            },
            'distribution': {
                'max_concurrent_distributions': 20,
                'retry_failed_distributions': True,
                'distribution_timeout': 7200  # 2 hours
            },
            'security': {
                'require_https': False,
                'ssl_cert': '',
                'ssl_key': '',
                'allowed_hosts': ['*']
            },
            'mirrors': {
                'enable_p2p_distribution': True,
                'peer_timeout': 300,
                'max_peers_per_update': 50
            }
        }
    
    def _setup_directories(self) -> None:
        """Create update distribution directories"""
        directories = [
            self.config['server']['document_root'],
            f"{self.config['server']['document_root']}/packages",
            f"{self.config['server']['document_root']}/mirrors",
            f"{self.config['server']['document_root']}/quarantine",
            "/var/lib/multios-enterprise/distribution_queue",
            "/var/lib/multios-enterprise/update_logs"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _load_update_catalog(self) -> None:
        """Load existing update packages from storage"""
        packages_dir = Path(self.config['server']['document_root']) / "packages"
        
        for package_file in packages_dir.glob("*.json"):
            try:
                with open(package_file, 'r') as f:
                    package_data = json.load(f)
                
                update_package = UpdatePackage(
                    package_id=package_data['package_id'],
                    name=package_data['name'],
                    version=package_data['version'],
                    package_type=package_data['package_type'],
                    size_mb=package_data['size_mb'],
                    download_url=package_data['download_url'],
                    checksum=package_data['checksum'],
                    dependencies=package_data.get('dependencies', []),
                    affected_systems=package_data.get('affected_systems', []),
                    required=package_data.get('required', False),
                    release_date=datetime.fromisoformat(package_data['release_date']),
                    description=package_data.get('description', '')
                )
                
                self.update_packages[update_package.package_id] = update_package
                
            except Exception as e:
                self.logger.error(f"Failed to load update package from {package_file}: {e}")
        
        self.logger.info(f"Loaded {len(self.update_packages)} update packages")
    
    def add_site(self, site_config: 'SiteConfig') -> None:
        """Add a site to update distribution"""
        self.sites[site_config.site_id] = site_config
        self.logger.info(f"Added site {site_config.site_id} to update distribution")
    
    def add_update_package(self, update_data: Dict[str, Any]) -> Optional[str]:
        """Add a new update package"""
        try:
            # Validate update data
            validation_result = self._validate_update_data(update_data)
            if not validation_result['valid']:
                self.logger.error(f"Invalid update data: {validation_result['error']}")
                return None
            
            # Generate package ID if not provided
            package_id = update_data.get('package_id') or self._generate_package_id(
                update_data['name'], update_data['version']
            )
            
            # Download update package if URL provided
            local_path = None
            if update_data.get('download_url'):
                local_path = self._download_update_package(update_data['download_url'], package_id)
                if not local_path:
                    return None
            
            # Create update package
            update_package = UpdatePackage(
                package_id=package_id,
                name=update_data['name'],
                version=update_data['version'],
                package_type=update_data['package_type'],
                size_mb=update_data.get('size_mb', 0),
                download_url=update_data['download_url'],
                checksum=update_data.get('checksum', ''),
                dependencies=update_data.get('dependencies', []),
                affected_systems=update_data.get('affected_systems', []),
                required=update_data.get('required', False),
                release_date=datetime.now(),
                description=update_data.get('description', '')
            )
            
            # Store update package
            self.update_packages[package_id] = update_package
            
            # Save package metadata
            self._save_update_package(update_package)
            
            # Start distribution to affected systems if configured
            if update_data.get('auto_distribute', False):
                self._start_distribution(update_package)
            
            self.logger.info(f"Added update package: {update_package.name} v{update_package.version}")
            return package_id
            
        except Exception as e:
            self.logger.error(f"Failed to add update package: {e}")
            return None
    
    def distribute_update(self, package_id: str, system_ids: List[str]) -> bool:
        """Distribute update to specific systems"""
        try:
            if package_id not in self.update_packages:
                self.logger.error(f"Update package {package_id} not found")
                return False
            
            update_package = self.update_packages[package_id]
            
            # Create distribution tasks for each system
            distribution_id = self._generate_distribution_id()
            
            distribution_task = {
                'distribution_id': distribution_id,
                'package_id': package_id,
                'update_package': update_package,
                'target_systems': system_ids,
                'status': 'queued',
                'created': datetime.now().isoformat(),
                'progress': 0,
                'results': {}
            }
            
            self.distribution_queue[distribution_id] = distribution_task
            
            # Start distribution in background
            thread = threading.Thread(
                target=self._process_distribution,
                args=(distribution_id,)
            )
            thread.start()
            
            self.logger.info(f"Started distribution of {update_package.name} to {len(system_ids)} systems")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to distribute update {package_id}: {e}")
            return False
    
    def distribute_update_sitewide(self, package_id: str, site_id: str) -> bool:
        """Distribute update to all systems at a site"""
        try:
            if site_id not in self.sites:
                self.logger.error(f"Site {site_id} not found")
                return False
            
            # Get all systems at the site (would integrate with system registry)
            site = self.sites[site_id]
            system_ids = self._get_site_systems(site_id)
            
            if not system_ids:
                self.logger.warning(f"No systems found at site {site_id}")
                return False
            
            return self.distribute_update(package_id, system_ids)
            
        except Exception as e:
            self.logger.error(f"Failed to distribute update sitewide: {e}")
            return False
    
    def start_update_server(self) -> bool:
        """Start the update distribution HTTP server"""
        try:
            if self.server_active:
                self.logger.warning("Update server is already running")
                return True
            
            # Create HTTP request handler
            handler = UpdateServerHTTPHandler
            handler.update_server = self
            
            # Start server
            self.http_server = socketserver.TCPServer(
                (self.config['server']['host'], self.config['server']['port']),
                handler
            )
            
            # Start server in background thread
            server_thread = threading.Thread(target=self.http_server.serve_forever, daemon=True)
            server_thread.start()
            
            self.server_active = True
            
            self.logger.info(f"Started update server on {self.config['server']['host']}:{self.config['server']['port']}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to start update server: {e}")
            return False
    
    def stop_update_server(self) -> bool:
        """Stop the update distribution HTTP server"""
        try:
            if not self.server_active:
                return True
            
            if self.http_server:
                self.http_server.shutdown()
                self.http_server = None
            
            self.server_active = False
            self.logger.info("Stopped update server")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to stop update server: {e}")
            return False
    
    def get_available_updates(self, system_id: Optional[str] = None, 
                            package_type: Optional[str] = None) -> List[UpdatePackage]:
        """Get list of available updates"""
        updates = list(self.update_packages.values())
        
        # Filter by package type
        if package_type:
            updates = [update for update in updates if update.package_type == package_type]
        
        # Filter by affected systems
        if system_id:
            updates = [update for update in updates 
                      if not update.affected_systems or system_id in update.affected_systems]
        
        return updates
    
    def get_distribution_status(self, distribution_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a distribution task"""
        return self.distribution_queue.get(distribution_id)
    
    def _validate_update_data(self, update_data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate update package data"""
        required_fields = ['name', 'version', 'package_type', 'download_url']
        
        for field in required_fields:
            if field not in update_data or not update_data[field]:
                return {'valid': False, 'error': f'Missing required field: {field}'}
        
        # Validate package type
        valid_types = ['security', 'feature', 'bugfix', 'driver', 'application']
        if update_data['package_type'] not in valid_types:
            return {'valid': False, 'error': f'Invalid package type: {update_data["package_type"]}'}
        
        return {'valid': True, 'error': None}
    
    def _generate_package_id(self, name: str, version: str) -> str:
        """Generate unique package ID"""
        combined = f"{name}_{version}_{datetime.now().isoformat()}"
        hash_obj = hashlib.md5(combined.encode())
        return f"UP{hash_obj.hexdigest()[:10].upper()}"
    
    def _download_update_package(self, download_url: str, package_id: str) -> Optional[str]:
        """Download update package from URL"""
        try:
            packages_dir = Path(self.config['server']['document_root']) / "packages"
            
            # Determine file extension from URL
            parsed_url = urlparse(download_url)
            path_parts = parsed_url.path.split('.')
            file_extension = path_parts[-1] if len(path_parts) > 1 else 'pkg'
            
            local_path = packages_dir / f"{package_id}.{file_extension}"
            
            # Download package
            if download_file(download_url, str(local_path)):
                # Calculate checksum
                checksum = calculate_md5(str(local_path))
                
                # Update package data
                if package_id in self.update_packages:
                    self.update_packages[package_id].checksum = checksum
                    self.update_packages[package_id].size_mb = local_path.stat().st_size / (1024 * 1024)
                
                self.logger.info(f"Downloaded update package: {package_id}")
                return str(local_path)
            else:
                return None
                
        except Exception as e:
            self.logger.error(f"Failed to download update package: {e}")
            return None
    
    def _process_distribution(self, distribution_id: str) -> None:
        """Process distribution task in background"""
        try:
            distribution_task = self.distribution_queue[distribution_id]
            distribution_task['status'] = 'in_progress'
            
            target_systems = distribution_task['target_systems']
            update_package = distribution_task['update_package']
            max_concurrent = self.config['distribution']['max_concurrent_distributions']
            
            # Process systems in batches
            for i in range(0, len(target_systems), max_concurrent):
                batch = target_systems[i:i + max_concurrent]
                
                with ThreadPoolExecutor(max_workers=max_concurrent) as executor:
                    futures = []
                    
                    for system_id in batch:
                        future = executor.submit(self._distribute_to_system, 
                                               distribution_id, system_id)
                        futures.append(future)
                    
                    # Wait for batch to complete
                    for future in futures:
                        try:
                            result = future.result(timeout=self.config['distribution']['distribution_timeout'])
                            system_id = result['system_id']
                            distribution_task['results'][system_id] = result
                        except Exception as e:
                            self.logger.error(f"Distribution to system failed: {e}")
                
                # Update progress
                completed = len([r for r in distribution_task['results'].values() if r.get('success', False)])
                distribution_task['progress'] = (completed / len(target_systems)) * 100
            
            # Mark distribution as completed
            successful = sum(1 for r in distribution_task['results'].values() if r.get('success', False))
            if successful == len(target_systems):
                distribution_task['status'] = 'completed'
            elif successful == 0:
                distribution_task['status'] = 'failed'
            else:
                distribution_task['status'] = 'completed'  # Partial success
            
            distribution_task['end_time'] = datetime.now().isoformat()
            
            self.logger.info(f"Distribution {distribution_id} completed: {successful}/{len(target_systems)} successful")
            
        except Exception as e:
            self.logger.error(f"Distribution {distribution_id} failed: {e}")
            distribution_task['status'] = 'failed'
            distribution_task['error'] = str(e)
    
    def _distribute_to_system(self, distribution_id: str, system_id: str) -> Dict[str, Any]:
        """Distribute update to a specific system"""
        try:
            distribution_task = self.distribution_queue[distribution_id]
            update_package = distribution_task['update_package']
            
            # Create download task for system
            download_task = {
                'system_id': system_id,
                'package_id': update_package.package_id,
                'package_url': f"http://{self._get_server_address()}:{self.config['server']['port']}/packages/{update_package.package_id}",
                'checksum': update_package.checksum,
                'size_mb': update_package.size_mb
            }
            
            # Simulate distribution (in real implementation, this would send download instructions)
            self.logger.info(f"Distributing {update_package.name} to system {system_id}")
            
            # Mark as successful (in real implementation, would wait for confirmation)
            return {
                'system_id': system_id,
                'success': True,
                'timestamp': datetime.now().isoformat(),
                'download_task': download_task
            }
            
        except Exception as e:
            self.logger.error(f"Failed to distribute to system {system_id}: {e}")
            return {
                'system_id': system_id,
                'success': False,
                'error': str(e),
                'timestamp': datetime.now().isoformat()
            }
    
    def _start_distribution(self, update_package: UpdatePackage) -> None:
        """Start automatic distribution of update package"""
        if not update_package.affected_systems:
            return
        
        self.distribute_update(update_package.package_id, update_package.affected_systems)
    
    def _get_site_systems(self, site_id: str) -> List[str]:
        """Get list of system IDs at a site (would integrate with system registry)"""
        # Placeholder implementation
        return [f"system_{i:03d}" for i in range(1, 21)]  # Mock 20 systems
    
    def _get_server_address(self) -> str:
        """Get server address for distribution"""
        try:
            import socket
            hostname = socket.gethostname()
            return socket.gethostbyname(hostname)
        except Exception:
            return "127.0.0.1"
    
    def _generate_distribution_id(self) -> str:
        """Generate unique distribution ID"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        hash_obj = hashlib.md5(f"{timestamp}_{len(self.distribution_queue)}".encode())
        return f"DIST-{hash_obj.hexdigest()[:8].upper()}"
    
    def _save_update_package(self, update_package: UpdatePackage) -> None:
        """Save update package metadata"""
        try:
            package_file = Path(self.config['server']['document_root']) / "packages" / f"{update_package.package_id}.json"
            
            package_data = {
                'package_id': update_package.package_id,
                'name': update_package.name,
                'version': update_package.version,
                'package_type': update_package.package_type,
                'size_mb': update_package.size_mb,
                'download_url': update_package.download_url,
                'checksum': update_package.checksum,
                'dependencies': update_package.dependencies,
                'affected_systems': update_package.affected_systems,
                'required': update_package.required,
                'release_date': update_package.release_date.isoformat(),
                'description': update_package.description
            }
            
            with open(package_file, 'w') as f:
                json.dump(package_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save update package {update_package.package_id}: {e}")

class UpdateServerHTTPHandler(http.server.SimpleHTTPRequestHandler):
    """HTTP request handler for update server"""
    
    update_server = None  # Will be set by UpdateServer
    
    def do_GET(self):
        """Handle GET requests"""
        try:
            parsed_path = urlparse(self.path)
            path = parsed_path.path
            
            if path == '/api/updates':
                self._handle_get_updates(parsed_path.query)
            elif path.startswith('/packages/'):
                self._handle_package_download(path)
            elif path == '/api/status':
                self._handle_server_status()
            else:
                self.send_error(404)
                
        except Exception as e:
            self.update_server.logger.error(f"HTTP request failed: {e}")
            self.send_error(500)
    
    def _handle_get_updates(self, query_string: str) -> None:
        """Handle GET /api/updates"""
        try:
            params = parse_qs(query_string)
            system_id = params.get('system_id', [None])[0]
            package_type = params.get('type', [None])[0]
            
            updates = self.update_server.get_available_updates(system_id, package_type)
            
            response = {
                'updates': [
                    {
                        'package_id': update.package_id,
                        'name': update.name,
                        'version': update.version,
                        'type': update.package_type,
                        'size_mb': update.size_mb,
                        'required': update.required,
                        'release_date': update.release_date.isoformat(),
                        'description': update.description
                    }
                    for update in updates
                ]
            }
            
            self.send_json_response(response)
            
        except Exception as e:
            self.send_error(500, str(e))
    
    def _handle_package_download(self, path: str) -> None:
        """Handle package download requests"""
        try:
            package_id = path.split('/')[-1]
            
            # Check if package exists
            package_file = Path(self.update_server.config['server']['document_root']) / "packages" / f"{package_id}.json"
            if not package_file.exists():
                self.send_error(404)
                return
            
            # Read package metadata
            with open(package_file, 'r') as f:
                package_data = json.load(f)
            
            # Check if actual package file exists
            actual_file = Path(self.update_server.config['server']['document_root']) / "packages" / f"{package_id}.{package_data.get('file_extension', 'pkg')}"
            if not actual_file.exists():
                self.send_error(404)
                return
            
            # Serve file
            self.send_response(200)
            self.send_header('Content-Type', 'application/octet-stream')
            self.send_header('Content-Length', str(actual_file.stat().st_size))
            self.send_header('Content-Disposition', f'attachment; filename="{package_data["name"]}_{package_data["version"]}.pkg"')
            self.end_headers()
            
            with open(actual_file, 'rb') as f:
                shutil.copyfileobj(f, self.wfile)
                
        except Exception as e:
            self.send_error(500, str(e))
    
    def _handle_server_status(self) -> None:
        """Handle server status requests"""
        try:
            status = {
                'server_active': self.update_server.server_active,
                'available_updates': len(self.update_server.update_packages),
                'active_distributions': len([d for d in self.update_server.distribution_queue.values() 
                                          if d['status'] == 'in_progress']),
                'total_distributions': len(self.update_server.distribution_queue)
            }
            
            self.send_json_response(status)
            
        except Exception as e:
            self.send_error(500, str(e))
    
    def send_json_response(self, data: Dict[str, Any]) -> None:
        """Send JSON response"""
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())
