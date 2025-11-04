"""
Automated Educational Software Package Deployment System
"""

import os
import json
import logging
import subprocess
import hashlib
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
import tempfile
import requests

from ..core.models import SystemInfo
from ..core.utils import execute_command, calculate_md5, download_file

class PackageManager:
    """Manager for automated educational software deployment"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/packages.yaml"):
        self.config_path = config_path
        self.packages = {}
        self.educational_catalog = {}
        self.deployment_queue = {}
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
        self._setup_directories()
        self._load_package_catalog()
    
    def _load_configuration(self) -> None:
        """Load package manager configuration"""
        self.config = {
            'repository': {
                'base_url': 'https://packages.multios.org',
                'local_mirror': '/var/lib/multios-enterprise/packages',
                'enable_local_mirror': True
            },
            'deployment': {
                'concurrent_installations': 10,
                'installation_timeout': 1800,  # 30 minutes
                'retry_attempts': 3,
                'backup_before_install': True
            },
            'educational_packages': {
                'programming_languages': [
                    'gcc', 'g++', 'python3', 'python3-dev', 'nodejs', 'npm',
                    'rustc', 'cargo', 'golang', 'openjdk-11-jdk'
                ],
                'ide_editors': [
                    'vim', 'nano', 'code', 'eclipse', 'intellij-idea-community',
                    'geany', 'atom', 'sublime-text'
                ],
                'educational_software': [
                    'sage', 'wolfram-engine', 'octave', 'scilab', 'r-base',
                    'sagemath', 'geogebra', 'fritzing', 'arduino', 'processing'
                ],
                'lab_software': [
                    'docker-ce', 'virtualbox', 'vmware-workstation',
                    'nginx', 'apache2', 'mysql-server', 'postgresql',
                    'redis', 'mongodb'
                ],
                'multimedia': [
                    'gimp', 'inkscape', 'blender', 'audacity', 'vlc',
                    'shotcut', 'obs-studio', 'freecad', 'libreoffice'
                ]
            },
            'security': {
                'verify_signatures': True,
                'scan_for_malware': True,
                'quarantine_suspicious': True
            }
        }
    
    def _setup_directories(self) -> None:
        """Create package management directories"""
        directories = [
            self.config['repository']['local_mirror'],
            f"{self.config['repository']['local_mirror']}/cache",
            f"{self.config['repository']['local_mirror']}/quarantine",
            "/var/lib/multios-enterprise/deployment_queue",
            "/var/lib/multios-enterprise/package_logs"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _load_package_catalog(self) -> None:
        """Load educational software package catalog"""
        # Create educational package catalog
        self.educational_catalog = {
            'programming_languages': {
                'python3': {
                    'name': 'Python 3',
                    'version': '3.9+',
                    'description': 'Python programming language interpreter',
                    'category': 'programming',
                    'dependencies': ['python3-pip', 'python3-dev'],
                    'size_mb': 50,
                    'license': 'PSF',
                    'educational_use': 'Programming courses, data science',
                    'install_commands': ['apt install -y python3 python3-pip python3-dev'],
                    'required_systems': ['desktop', 'laptop', 'server'],
                    'lab_templates': ['programming_basics', 'web_development', 'data_science']
                },
                'gcc': {
                    'name': 'GCC Compiler',
                    'version': '9.0+',
                    'description': 'GNU Compiler Collection',
                    'category': 'programming',
                    'dependencies': ['make', 'build-essential'],
                    'size_mb': 150,
                    'license': 'GPL',
                    'educational_use': 'Computer science, system programming',
                    'install_commands': ['apt install -y gcc g++ make build-essential'],
                    'required_systems': ['desktop', 'laptop', 'server'],
                    'lab_templates': ['c_programming', 'systems_programming']
                }
            },
            'ide_editors': {
                'code': {
                    'name': 'Visual Studio Code',
                    'version': '1.70+',
                    'description': 'Lightweight code editor with extensive plugin support',
                    'category': 'development',
                    'dependencies': [],
                    'size_mb': 80,
                    'license': 'MIT',
                    'educational_use': 'General programming courses',
                    'install_commands': ['snap install --classic code'],
                    'required_systems': ['desktop', 'laptop'],
                    'lab_templates': ['general_programming', 'web_development']
                }
            },
            'educational_software': {
                'sage': {
                    'name': 'SageMath',
                    'version': '9.0+',
                    'description': 'Mathematical software system',
                    'category': 'mathematics',
                    'dependencies': ['python3', 'latexmk'],
                    'size_mb': 500,
                    'license': 'GPL',
                    'educational_use': 'Mathematics courses, research',
                    'install_commands': ['apt install -y sagemath'],
                    'required_systems': ['desktop', 'laptop', 'server'],
                    'lab_templates': ['calculus', 'linear_algebra', 'number_theory']
                },
                'octave': {
                    'name': 'GNU Octave',
                    'version': '6.0+',
                    'description': 'Numerical computation software',
                    'category': 'mathematics',
                    'dependencies': ['gnuplot'],
                    'size_mb': 100,
                    'license': 'GPL',
                    'educational_use': 'Engineering, signal processing',
                    'install_commands': ['apt install -y octave'],
                    'required_systems': ['desktop', 'laptop', 'server'],
                    'lab_templates': ['signal_processing', 'numerical_methods']
                }
            },
            'lab_software': {
                'docker-ce': {
                    'name': 'Docker CE',
                    'version': '20.0+',
                    'description': 'Container platform for development',
                    'category': 'infrastructure',
                    'dependencies': ['apt-transport-https', 'ca-certificates'],
                    'size_mb': 200,
                    'license': 'Apache 2.0',
                    'educational_use': 'DevOps, cloud computing courses',
                    'install_commands': ['apt install -y docker.io'],
                    'required_systems': ['desktop', 'laptop', 'server'],
                    'lab_templates': ['containerization', 'devops', 'microservices']
                }
            }
        }
        
        self.logger.info(f"Loaded educational package catalog with {sum(len(cat) for cat in self.educational_catalog.values())} packages")
    
    def add_package(self, package_info: Dict[str, Any]) -> bool:
        """Add a package to the deployment system"""
        try:
            package_id = package_info.get('id') or self._generate_package_id(package_info['name'])
            
            # Validate package information
            required_fields = ['name', 'version', 'description', 'install_commands']
            for field in required_fields:
                if field not in package_info:
                    self.logger.error(f"Missing required field: {field}")
                    return False
            
            # Store package information
            package_data = {
                'id': package_id,
                'created': datetime.now().isoformat(),
                'source': 'custom',
                **package_info
            }
            
            self.packages[package_id] = package_data
            
            # Save to storage
            self._save_package(package_id, package_data)
            
            self.logger.info(f"Added package: {package_info['name']} ({package_id})")
            return True
        except Exception as e:
            self.logger.error(f"Failed to add package: {e}")
            return False
    
    def install_package(self, system_id: str, package_name: str, version: Optional[str] = None) -> bool:
        """Install a package on a specific system"""
        try:
            # Find package in catalog
            package_info = self._find_package(package_name, version)
            if not package_info:
                self.logger.error(f"Package {package_name} not found in catalog")
                return False
            
            # Create installation task
            task_id = self._generate_task_id()
            task = {
                'task_id': task_id,
                'system_id': system_id,
                'package_name': package_name,
                'package_info': package_info,
                'status': 'queued',
                'created': datetime.now().isoformat(),
                'attempts': 0
            }
            
            self.deployment_queue[task_id] = task
            
            # Process installation task
            success = self._process_installation_task(task)
            
            return success
        except Exception as e:
            self.logger.error(f"Failed to install package {package_name} on {system_id}: {e}")
            return False
    
    def bulk_install_packages(self, system_id: str, package_list: List[str]) -> Dict[str, Any]:
        """Install multiple packages on a system"""
        results = {
            'system_id': system_id,
            'total_packages': len(package_list),
            'successful': 0,
            'failed': 0,
            'results': []
        }
        
        for package_name in package_list:
            success = self.install_package(system_id, package_name)
            
            result = {
                'package_name': package_name,
                'success': success
            }
            
            results['results'].append(result)
            
            if success:
                results['successful'] += 1
            else:
                results['failed'] += 1
        
        self.logger.info(f"Bulk installation on {system_id}: {results['successful']}/{results['total_packages']} successful")
        return results
    
    def deploy_lab_environment(self, system_id: str, lab_template: str, 
                              custom_packages: Optional[List[str]] = None) -> bool:
        """Deploy complete lab environment with educational software"""
        try:
            # Define lab environments
            lab_environments = {
                'programming_lab': [
                    'python3', 'gcc', 'code', 'vim', 'git'
                ],
                'web_development_lab': [
                    'nodejs', 'npm', 'code', 'nginx', 'docker-ce'
                ],
                'data_science_lab': [
                    'python3', 'octave', 'r-base', 'jupyter', 'spyder'
                ],
                'multimedia_lab': [
                    'gimp', 'inkscape', 'blender', 'audacity', 'vlc'
                ],
                'systems_lab': [
                    'gcc', 'docker-ce', 'virtualbox', 'qemu-system-x86', 'git'
                ],
                'security_lab': [
                    'wireshark', 'nmap', 'metasploit', 'burpsuite', 'john'
                ],
                'networking_lab': [
                    'wireshark', 'tcpdump', 'iperf3', 'bind9', 'isc-dhcp-server'
                ]
            }
            
            # Get package list for lab template
            if lab_template not in lab_environments:
                self.logger.error(f"Unknown lab template: {lab_template}")
                return False
            
            package_list = lab_environments[lab_template]
            
            # Add custom packages if specified
            if custom_packages:
                package_list.extend(custom_packages)
            
            # Install all packages
            results = self.bulk_install_packages(system_id, package_list)
            
            self.logger.info(f"Deployed lab environment '{lab_template}' on {system_id}")
            return results['failed'] == 0
            
        except Exception as e:
            self.logger.error(f"Failed to deploy lab environment {lab_template}: {e}")
            return False
    
    def _find_package(self, package_name: str, version: Optional[str] = None) -> Optional[Dict[str, Any]]:
        """Find package in educational catalog"""
        # Search in all categories
        for category_packages in self.educational_catalog.values():
            for pkg_name, pkg_info in category_packages.items():
                if pkg_name.lower() == package_name.lower():
                    if version and pkg_info.get('version') != version:
                        continue
                    return pkg_info
        
        # Search in custom packages
        for pkg_id, pkg_info in self.packages.items():
            if pkg_info['name'].lower() == package_name.lower():
                if version and pkg_info.get('version') != version:
                    continue
                return pkg_info
        
        return None
    
    def _process_installation_task(self, task: Dict[str, Any]) -> bool:
        """Process a package installation task"""
        system_id = task['system_id']
        package_info = task['package_info']
        max_attempts = self.config['deployment']['retry_attempts']
        
        for attempt in range(max_attempts):
            try:
                task['attempts'] = attempt + 1
                task['status'] = 'installing'
                
                self.logger.info(f"Installing {package_info['name']} on {system_id} (attempt {attempt + 1})")
                
                # Execute installation commands
                for command in package_info['install_commands']:
                    result, stdout, stderr = execute_command(command.split(), 
                                                           timeout=self.config['deployment']['installation_timeout'])
                    if result != 0:
                        raise Exception(f"Command failed: {command} - {stderr}")
                
                # Verify installation
                if self._verify_installation(system_id, package_info):
                    task['status'] = 'completed'
                    self.logger.info(f"Successfully installed {package_info['name']} on {system_id}")
                    return True
                else:
                    raise Exception("Installation verification failed")
                    
            except Exception as e:
                self.logger.warning(f"Installation attempt {attempt + 1} failed: {e}")
                if attempt < max_attempts - 1:
                    time.sleep(5)  # Wait before retry
                else:
                    task['status'] = 'failed'
                    self.logger.error(f"Failed to install {package_info['name']} on {system_id} after {max_attempts} attempts")
                    return False
        
        return False
    
    def _verify_installation(self, system_id: str, package_info: Dict[str, Any]) -> bool:
        """Verify package installation on system"""
        try:
            # Check if package binary/command is available
            if 'verify_command' in package_info:
                result, stdout, stderr = execute_command(package_info['verify_command'].split())
                return result == 0
            
            # Default verification based on package type
            package_name = package_info['name']
            
            # Common verification methods
            if package_name == 'python3':
                result, stdout, stderr = execute_command(['python3', '--version'])
                return result == 0
            elif package_name == 'gcc':
                result, stdout, stderr = execute_command(['gcc', '--version'])
                return result == 0
            elif package_name == 'code':
                result, stdout, stderr = execute_command(['code', '--version'])
                return result == 0
            
            # For system packages, assume success
            return True
            
        except Exception as e:
            self.logger.error(f"Installation verification failed: {e}")
            return False
    
    def _generate_package_id(self, package_name: str) -> str:
        """Generate unique package ID"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        hash_obj = hashlib.md5(f"{package_name}_{timestamp}".encode())
        return f"PKG-{hash_obj.hexdigest()[:8].upper()}"
    
    def _generate_task_id(self) -> str:
        """Generate unique task ID"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        import uuid
        return f"TASK-{timestamp}-{str(uuid.uuid4())[:8]}"
    
    def _save_package(self, package_id: str, package_data: Dict[str, Any]) -> None:
        """Save package data to storage"""
        try:
            package_file = Path(self.config['repository']['local_mirror']) / f"{package_id}.json"
            
            with open(package_file, 'w') as f:
                json.dump(package_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save package {package_id}: {e}")
    
    def list_educational_packages(self, category: Optional[str] = None) -> Dict[str, Any]:
        """List available educational packages"""
        if category:
            return {category: self.educational_catalog.get(category, {})}
        return self.educational_catalog
    
    def get_package_info(self, package_name: str) -> Optional[Dict[str, Any]]:
        """Get information about a specific package"""
        return self._find_package(package_name)
    
    def get_deployment_status(self, task_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a deployment task"""
        return self.deployment_queue.get(task_id)
    
    def list_deployment_queue(self) -> List[Dict[str, Any]]:
        """List all pending deployment tasks"""
        return [task for task in self.deployment_queue.values() if task['status'] in ['queued', 'installing']]
    
    def remove_package(self, package_id: str) -> bool:
        """Remove a package from the catalog"""
        try:
            if package_id in self.packages:
                del self.packages[package_id]
                
                # Remove package file
                package_file = Path(self.config['repository']['local_mirror']) / f"{package_id}.json"
                if package_file.exists():
                    package_file.unlink()
                
                self.logger.info(f"Removed package {package_id}")
                return True
            return False
        except Exception as e:
            self.logger.error(f"Failed to remove package {package_id}: {e}")
            return False
    
    def create_package_snapshot(self, system_id: str) -> Optional[str]:
        """Create a snapshot of installed packages on a system"""
        try:
            # Get list of installed packages
            result, stdout, stderr = execute_command(['dpkg', '--get-selections'])
            if result != 0:
                self.logger.error(f"Failed to get package list for {system_id}")
                return None
            
            # Parse installed packages
            installed_packages = []
            for line in stdout.strip().split('\n'):
                if line and line.split()[1] == 'install':
                    installed_packages.append(line.split()[0])
            
            # Create snapshot
            snapshot = {
                'system_id': system_id,
                'timestamp': datetime.now().isoformat(),
                'packages': installed_packages,
                'package_count': len(installed_packages)
            }
            
            # Save snapshot
            snapshot_file = Path("/var/lib/multios-enterprise/package_logs") / f"snapshot_{system_id}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
            
            with open(snapshot_file, 'w') as f:
                json.dump(snapshot, f, indent=2)
            
            self.logger.info(f"Created package snapshot for {system_id}")
            return str(snapshot_file)
            
        except Exception as e:
            self.logger.error(f"Failed to create package snapshot for {system_id}: {e}")
            return None
