"""
Template-based Configuration Management System
"""

import os
import json
import yaml
import jinja2
import logging
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
import tempfile
import shutil

from ..core.models import SystemInfo, DeploymentProfile, SiteConfig
from ..core.utils import load_config, save_config, execute_command

class TemplateManager:
    """Manager for template-based system configurations"""
    
    def __init__(self, template_dir: str = "/etc/multios-enterprise/templates"):
        self.template_dir = Path(template_dir)
        self.logger = logging.getLogger(__name__)
        self.templates = {}
        self.system_configs = {}
        
        self._setup_directories()
        self._load_templates()
    
    def _setup_directories(self) -> None:
        """Create template management directories"""
        directories = [
            self.template_dir,
            self.template_dir / "system",
            self.template_dir / "network",
            self.template_dir / "services",
            self.template_dir / "security",
            self.template_dir / "applications",
            self.template_dir / "labs"
        ]
        
        for directory in directories:
            directory.mkdir(parents=True, exist_ok=True)
        
        self.logger.info(f"Created template directories at {self.template_dir}")
    
    def _load_templates(self) -> None:
        """Load all available templates"""
        template_types = [
            "system", "network", "services", "security", "applications", "labs"
        ]
        
        for template_type in template_types:
            type_dir = self.template_dir / template_type
            if type_dir.exists():
                for template_file in type_dir.glob("*.j2"):
                    template_name = f"{template_type}/{template_file.stem}"
                    self.templates[template_name] = str(template_file)
        
        self.logger.info(f"Loaded {len(self.templates)} templates")
    
    def create_template(self, template_name: str, template_content: str, 
                       template_type: str = "system") -> bool:
        """Create a new template"""
        try:
            template_path = self.template_dir / template_type / f"{template_name}.j2"
            template_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(template_path, 'w') as f:
                f.write(template_content)
            
            full_name = f"{template_type}/{template_name}"
            self.templates[full_name] = str(template_path)
            
            self.logger.info(f"Created template {full_name}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to create template {template_name}: {e}")
            return False
    
    def apply_template(self, system: SystemInfo, profile: DeploymentProfile) -> bool:
        """Apply deployment profile template to a system"""
        try:
            # Generate configuration files using Jinja2
            config_context = self._build_config_context(system, profile)
            
            # Apply each template in the profile configuration
            for config_key, config_value in profile.configuration.items():
                if isinstance(config_value, dict) and 'template' in config_value:
                    template_name = config_value['template']
                    template_vars = config_value.get('variables', {})
                    
                    if template_name in self.templates:
                        self._apply_single_template(system, template_name, template_vars, config_context)
            
            # Apply network configuration if provided
            if profile.network_config:
                self._apply_network_config(system, profile.network_config)
            
            # Apply security settings
            if profile.security_settings:
                self._apply_security_config(system, profile.security_settings)
            
            self.logger.info(f"Applied configuration template to {system.hostname}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to apply template to {system.hostname}: {e}")
            return False
    
    def _build_config_context(self, system: SystemInfo, profile: DeploymentProfile) -> Dict[str, Any]:
        """Build context dictionary for template rendering"""
        return {
            'system': {
                'hostname': system.hostname,
                'ip_address': system.ip_address,
                'mac_address': system.mac_address,
                'system_type': system.system_type.value,
                'cpu_model': system.cpu_model,
                'memory_gb': system.memory_gb,
                'storage_gb': system.storage_gb,
                'location': system.location
            },
            'profile': {
                'name': profile.name,
                'system_type': profile.system_type.value,
                'base_os_version': profile.base_os_version,
                'required_packages': profile.required_packages,
                'user_groups': profile.user_groups,
                'security_settings': profile.security_settings,
                'resource_limits': profile.resource_limits
            },
            'site': {
                'site_id': system.site_id,
                'timezone': 'UTC',  # Would be loaded from site config
                'dns_servers': ['8.8.8.8', '8.8.4.4'],
                'ntp_servers': ['pool.ntp.org']
            },
            'deployment': {
                'timestamp': datetime.now().isoformat(),
                'deployment_id': system.system_id
            }
        }
    
    def _apply_single_template(self, system: SystemInfo, template_name: str, 
                             template_vars: Dict[str, Any], context: Dict[str, Any]) -> bool:
        """Apply a single template to a system"""
        try:
            template_path = Path(self.templates[template_name])
            
            # Create Jinja2 environment
            env = jinja2.Environment(
                loader=jinja2.FileSystemLoader(str(template_path.parent)),
                trim_blocks=True,
                lstrip_blocks=True
            )
            
            # Load and render template
            template = env.get_template(template_path.name)
            rendered_content = template.render(**context, **template_vars)
            
            # Determine target file path based on template type
            target_path = self._get_target_path(system, template_name, template_vars)
            
            # Upload rendered configuration to target system
            self._upload_config_file(system, target_path, rendered_content)
            
            return True
        except Exception as e:
            self.logger.error(f"Failed to apply template {template_name}: {e}")
            return False
    
    def _get_target_path(self, system: SystemInfo, template_name: str, 
                        template_vars: Dict[str, Any]) -> str:
        """Determine target path for configuration file"""
        # Default paths based on template type
        default_paths = {
            'system/hostname': '/etc/hostname',
            'network/interfaces': '/etc/network/interfaces',
            'network/hosts': '/etc/hosts',
            'services/ssh': '/etc/ssh/sshd_config',
            'services/nfs': '/etc/exports',
            'security/firewall': '/etc/iptables/rules.v4',
            'security/sudoers': '/etc/sudoers',
            'applications/package': '/etc/apt/sources.list.d/custom.list'
        }
        
        # Check for custom target path in template variables
        if 'target_path' in template_vars:
            return template_vars['target_path']
        
        # Use default path based on template name
        return default_paths.get(template_name, f'/etc/multios-enterprise/{template_name.replace("/", "_")}')
    
    def _upload_config_file(self, system: SystemInfo, target_path: str, 
                          content: str) -> bool:
        """Upload configuration file to target system"""
        try:
            # Create temporary file
            with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.conf') as tmp_file:
                tmp_file.write(content)
                tmp_file_path = tmp_file.name
            
            try:
                # Transfer file via SCP (in production, this would be more sophisticated)
                scp_cmd = [
                    'scp', '-o', 'StrictHostKeyChecking=no',
                    tmp_file_path,
                    f'root@{system.ip_address}:{target_path}'
                ]
                
                result, stdout, stderr = execute_command(scp_cmd, timeout=30)
                if result == 0:
                    self.logger.info(f"Uploaded {target_path} to {system.hostname}")
                    return True
                else:
                    self.logger.error(f"Failed to upload {target_path}: {stderr}")
                    return False
            finally:
                os.unlink(tmp_file_path)
                
        except Exception as e:
            self.logger.error(f"Failed to upload config file: {e}")
            return False
    
    def _apply_network_config(self, system: SystemInfo, network_config: Dict[str, str]) -> bool:
        """Apply network configuration to system"""
        try:
            network_template = """# Network configuration for {{ system.hostname }}
# Generated by MultiOS Enterprise Deployment System

# Primary network interface
auto eth0
iface eth0 inet static
address {{ system.ip_address }}
netmask 255.255.255.0
gateway 192.168.1.1
dns-nameservers {{ ' '.join(site.dns_servers) }}

# Additional interfaces can be configured here
"""
            
            # Render network template
            env = jinja2.Environment(loader=jinja2.DictLoader({'network': network_template}))
            template = env.get_template('network')
            rendered = template.render(
                system={'hostname': system.hostname, 'ip_address': system.ip_address},
                site={'dns_servers': ['8.8.8.8', '8.8.4.4']}
            )
            
            # Upload network configuration
            return self._upload_config_file(system, '/etc/network/interfaces', rendered)
        except Exception as e:
            self.logger.error(f"Failed to apply network config: {e}")
            return False
    
    def _apply_security_config(self, system: SystemInfo, security_settings: Dict[str, bool]) -> bool:
        """Apply security configuration to system"""
        try:
            ssh_config = """# SSH Configuration for {{ system.hostname }}
# Generated by MultiOS Enterprise Deployment System

Port {{ '2222' if security.ssh_non_standard_port else '22' }}
PermitRootLogin {{ 'yes' if security.root_login else 'no' }}
PasswordAuthentication {{ 'yes' if security.password_auth else 'no' }}
PubkeyAuthentication yes
PermitEmptyPasswords no
ChallengeResponseAuthentication no
UsePAM yes
X11Forwarding {{ 'yes' if security.x11_forwarding else 'no' }}
PrintMotd no
AcceptEnv LANG LC_*
Subsystem sftp /usr/lib/openssh/sftp-server
"""
            
            env = jinja2.Environment(loader=jinja2.DictLoader({'ssh_config': ssh_config}))
            template = env.get_template('ssh_config')
            rendered = template.render(
                system={'hostname': system.hostname},
                security=type('obj', (object,), security_settings)()
            )
            
            return self._upload_config_file(system, '/etc/ssh/sshd_config', rendered)
        except Exception as e:
            self.logger.error(f"Failed to apply security config: {e}")
            return False
    
    def create_predefined_templates(self) -> None:
        """Create predefined templates for common configurations"""
        
        # Hostname template
        self.create_template("hostname", """{{ system.hostname }}""", "system")
        
        # Network interfaces template
        self.create_template("interfaces", """# Network interfaces for {{ system.hostname }}
auto eth0
iface eth0 inet static
address {{ system.ip_address }}
netmask 255.255.255.0
gateway 192.168.1.1
dns-nameservers {{ ' '.join(site.dns_servers) }}""", "network")
        
        # SSH configuration template
        self.create_template("sshd_config", """# SSH Configuration for {{ system.hostname }}
Port {{ profile.security_settings.get('ssh_port', 22) }}
PermitRootLogin {{ 'yes' if profile.security_settings.get('root_login', False) else 'no' }}
PasswordAuthentication {{ 'yes' if profile.security_settings.get('password_auth', True) else 'no' }}
PubkeyAuthentication yes
PermitEmptyPasswords no
X11Forwarding {{ 'yes' if profile.security_settings.get('x11_forwarding', False) else 'no' }}
PrintMotd no
Subsystem sftp /usr/lib/openssh/sftp-server""", "services")
        
        # Package repositories template
        self.create_template("repositories", """# MultiOS Package Repositories
deb http://archive.multios.org {{ profile.base_os_version }} main restricted universe multiverse
deb http://archive.multios.org {{ profile.base_os_version }}-updates main restricted universe multiverse
deb http://archive.multios.org {{ profile.base_os_version }}-security main restricted universe multiverse""", "applications")
        
        # Lab environment template
        self.create_template("lab_desktop", """# Lab Desktop Configuration for {{ system.hostname }}
# Target: {{ profile.name }}

# Desktop environment settings
DESKTOP_SESSION=gnome
GTK_THEME=Adwaita:dark
ICONS=Adwaita
CURSOR_THEME=Adwaita

# Educational software
EDUCATIONAL_PACKAGES="{{ ', '.join(profile.required_packages) }}"

# Lab-specific settings
LAB_MODE=enabled
AUTO_LOGIN={{ 'yes' if profile.configuration.get('auto_login', False) else 'no' }}
CLASSROOM_MODE={{ 'enabled' if profile.configuration.get('classroom_mode', False) else 'disabled' }}

# Network settings
NETWORK_PROFILE={{ profile.network_config.get('profile', 'default') }}

# Resource limits
MAX_MEMORY_MB={{ profile.resource_limits.get('memory_mb', 4096) }}
MAX_CPU_PERCENT={{ profile.resource_limits.get('cpu_percent', 80) }}""", "labs")
        
        self.logger.info("Created predefined templates")
    
    def list_templates(self) -> List[str]:
        """List all available templates"""
        return sorted(list(self.templates.keys()))
    
    def get_template_content(self, template_name: str) -> Optional[str]:
        """Get content of a specific template"""
        try:
            template_path = self.templates.get(template_name)
            if template_path and Path(template_path).exists():
                with open(template_path, 'r') as f:
                    return f.read()
            return None
        except Exception as e:
            self.logger.error(f"Failed to get template content: {e}")
            return None
    
    def update_template(self, template_name: str, new_content: str) -> bool:
        """Update an existing template"""
        try:
            if template_name in self.templates:
                template_path = self.templates[template_name]
                with open(template_path, 'w') as f:
                    f.write(new_content)
                self.logger.info(f"Updated template {template_name}")
                return True
            else:
                # Create new template if it doesn't exist
                return self.create_template(template_name, new_content)
        except Exception as e:
            self.logger.error(f"Failed to update template {template_name}: {e}")
            return False
    
    def delete_template(self, template_name: str) -> bool:
        """Delete a template"""
        try:
            if template_name in self.templates:
                template_path = self.templates[template_name]
                Path(template_path).unlink()
                del self.templates[template_name]
                self.logger.info(f"Deleted template {template_name}")
                return True
            return False
        except Exception as e:
            self.logger.error(f"Failed to delete template {template_name}: {e}")
            return False
    
    def export_templates(self, export_path: str) -> bool:
        """Export all templates to a directory"""
        try:
            export_dir = Path(export_path)
            export_dir.mkdir(parents=True, exist_ok=True)
            
            for template_name, template_path in self.templates.items():
                template_content = self.get_template_content(template_name)
                if template_content:
                    # Maintain directory structure in export
                    target_path = export_dir / template_name.replace('/', '_') + '.j2'
                    target_path.parent.mkdir(parents=True, exist_ok=True)
                    
                    with open(target_path, 'w') as f:
                        f.write(template_content)
            
            self.logger.info(f"Exported {len(self.templates)} templates to {export_path}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to export templates: {e}")
            return False
    
    def import_templates(self, import_path: str) -> int:
        """Import templates from a directory"""
        imported = 0
        try:
            import_dir = Path(import_path)
            
            for template_file in import_dir.glob("*.j2"):
                # Extract template name from filename
                template_name = template_file.stem
                
                with open(template_file, 'r') as f:
                    template_content = f.read()
                
                if self.update_template(template_name, template_content):
                    imported += 1
            
            self.logger.info(f"Imported {imported} templates from {import_path}")
            return imported
        except Exception as e:
            self.logger.error(f"Failed to import templates: {e}")
            return 0
    
    def validate_template(self, template_name: str) -> Dict[str, Any]:
        """Validate a template for syntax and structure"""
        try:
            template_path = self.templates.get(template_name)
            if not template_path:
                return {'valid': False, 'error': 'Template not found'}
            
            # Test template rendering
            env = jinja2.Environment(loader=jinja2.FileSystemLoader(str(Path(template_path).parent)))
            template = env.get_template(Path(template_path).name)
            
            # Try rendering with minimal context
            test_context = {
                'system': {'hostname': 'test', 'ip_address': '192.168.1.1'},
                'profile': {'name': 'test'},
                'site': {'dns_servers': ['8.8.8.8']}
            }
            
            rendered = template.render(**test_context)
            
            return {
                'valid': True,
                'rendered_length': len(rendered),
                'error': None
            }
        except jinja2.TemplateError as e:
            return {'valid': False, 'error': f'Jinja2 error: {str(e)}'}
        except Exception as e:
            return {'valid': False, 'error': f'Validation error: {str(e)}'}
