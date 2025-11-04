"""
Deployment Automation with Scripting Capabilities
"""

import os
import json
import logging
import subprocess
import threading
import time
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

from ..core.models import SystemInfo, DeploymentProfile
from ..core.utils import execute_command, generate_system_id

class DeploymentAutomation:
    """Manager for deployment automation and scripting"""
    
    def __init__(self):
        self.scripts = {}
        self.automation_jobs = {}
        self.logger = logging.getLogger(__name__)
        
        self._setup_directories()
        self._load_predefined_scripts()
    
    def _setup_directories(self) -> None:
        """Create automation directories"""
        directories = [
            "/var/lib/multios-enterprise/automation",
            "/var/lib/multios-enterprise/automation/scripts",
            "/var/lib/multios-enterprise/automation/jobs"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _load_predefined_scripts(self) -> None:
        """Load predefined automation scripts"""
        # System hardening script
        self.create_script("system_hardening", """#!/bin/bash
# MultiOS System Hardening Script

echo "Applying security hardening to MultiOS system..."

# Disable unnecessary services
systemctl disable avahi-daemon
systemctl disable cups

# Configure firewall
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X

# Set secure permissions
chmod 600 /etc/shadow
chmod 644 /etc/passwd
chmod 644 /etc/group

# Configure SSH security
sed -i 's/#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config

systemctl restart sshd

echo "System hardening completed"
""")
        
        # Lab setup script
        self.create_script("setup_lab_environment", """#!/bin/bash
# MultiOS Lab Environment Setup Script

echo "Setting up MultiOS lab environment..."

# Create lab directories
mkdir -p /home/lab/{students,teachers,shared}
chmod 755 /home/lab
chmod 770 /home/lab/students
chmod 770 /home/lab/teachers
chmod 777 /home/lab/shared

# Configure shared access
echo "/home/lab/shared *(rw,sync,no_subtree_check)" >> /etc/exports
exportfs -a

# Setup lab monitoring
echo "LAB_MODE=enabled" > /etc/lab.conf
echo "CLASSROOM_MODE=enabled" >> /etc/lab.conf

# Install common lab software
apt update
apt install -y htop tree curl wget vim

echo "Lab environment setup completed"
""")
        
        # Network configuration script
        self.create_script("configure_network", """#!/bin/bash
# MultiOS Network Configuration Script

echo "Configuring network settings..."

# Get system information
HOSTNAME=$(hostname)
IP_ADDRESS=$(hostname -I | awk '{print $1}')

# Backup current network configuration
cp /etc/network/interfaces /etc/network/interfaces.backup.$(date +%Y%m%d)

# Configure network interface
cat > /etc/network/interfaces << EOF
# Network configuration for MultiOS
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
address $IP_ADDRESS
netmask 255.255.255.0
gateway 192.168.1.1
dns-nameservers 8.8.8.8 8.8.4.4
EOF

# Restart networking
systemctl restart networking

echo "Network configuration completed for $HOSTNAME ($IP_ADDRESS)"
""")
    
    def create_script(self, script_name: str, script_content: str, 
                     script_type: str = 'bash') -> bool:
        """Create a new automation script"""
        try:
            script_id = generate_system_id()
            
            script_path = Path("/var/lib/multios-enterprise/automation/scripts") / f"{script_name}.{script_type}"
            
            with open(script_path, 'w') as f:
                f.write(script_content)
            
            # Make script executable
            os.chmod(script_path, 0o755)
            
            self.scripts[script_name] = {
                'script_id': script_id,
                'name': script_name,
                'type': script_type,
                'path': str(script_path),
                'created': datetime.now().isoformat()
            }
            
            self.logger.info(f"Created automation script: {script_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to create script {script_name}: {e}")
            return False
    
    def execute_script(self, script_name: str, target_systems: List[str], 
                      parameters: Optional[Dict[str, str]] = None) -> str:
        """Execute script on target systems"""
        try:
            if script_name not in self.scripts:
                raise ValueError(f"Script {script_name} not found")
            
            job_id = generate_system_id()
            
            job = {
                'job_id': job_id,
                'script_name': script_name,
                'script_info': self.scripts[script_name],
                'target_systems': target_systems,
                'parameters': parameters or {},
                'status': 'queued',
                'created': datetime.now().isoformat(),
                'progress': 0,
                'results': {}
            }
            
            self.automation_jobs[job_id] = job
            
            # Execute job in background
            thread = threading.Thread(
                target=self._execute_automation_job,
                args=(job_id,)
            )
            thread.start()
            
            self.logger.info(f"Started automation job {job_id} for script {script_name}")
            return job_id
            
        except Exception as e:
            self.logger.error(f"Failed to execute script {script_name}: {e}")
            raise
    
    def _execute_automation_job(self, job_id: str) -> None:
        """Execute automation job in background"""
        try:
            job = self.automation_jobs[job_id]
            job['status'] = 'in_progress'
            
            script_info = job['script_info']
            target_systems = job['target_systems']
            parameters = job['parameters']
            
            with ThreadPoolExecutor(max_workers=10) as executor:
                futures = []
                
                for system_id in target_systems:
                    future = executor.submit(self._execute_script_on_system, 
                                           job_id, system_id, script_info, parameters)
                    futures.append(future)
                
                # Wait for all executions to complete
                for future in futures:
                    try:
                        result = future.result(timeout=3600)
                        system_id = result['system_id']
                        job['results'][system_id] = result
                    except Exception as e:
                        self.logger.error(f"Script execution failed: {e}")
            
            # Update progress
            completed = len([r for r in job['results'].values() if r.get('success', False)])
            job['progress'] = (completed / len(target_systems)) * 100
            
            # Mark job as completed
            successful = sum(1 for r in job['results'].values() if r.get('success', False))
            if successful == len(target_systems):
                job['status'] = 'completed'
            elif successful == 0:
                job['status'] = 'failed'
            else:
                job['status'] = 'completed'  # Partial success
            
            job['completed'] = datetime.now().isoformat()
            
            self.logger.info(f"Automation job {job_id} completed: {successful}/{len(target_systems)} successful")
            
        except Exception as e:
            self.logger.error(f"Automation job {job_id} failed: {e}")
            job['status'] = 'failed'
            job['error'] = str(e)
    
    def _execute_script_on_system(self, job_id: str, system_id: str, 
                                script_info: Dict[str, Any], 
                                parameters: Dict[str, str]) -> Dict[str, Any]:
        """Execute script on a specific system"""
        try:
            # Get system information (simplified)
            system_info = self._get_system_info(system_id)
            if not system_info:
                return {
                    'system_id': system_id,
                    'success': False,
                    'error': 'System not found'
                }
            
            # Prepare script execution command
            script_path = script_info['path']
            
            # Add parameters to script execution
            cmd = [script_path]
            for key, value in parameters.items():
                cmd.append(f"{key}={value}")
            
            # Execute script via SSH (simplified)
            ssh_cmd = ['ssh', '-o', 'StrictHostKeyChecking=no', 
                      f'root@{system_info.ip_address}'] + cmd
            
            start_time = time.time()
            result, stdout, stderr = execute_command(ssh_cmd, timeout=3600)
            end_time = time.time()
            
            execution_time = end_time - start_time
            
            return {
                'system_id': system_id,
                'success': result == 0,
                'stdout': stdout,
                'stderr': stderr,
                'execution_time': execution_time,
                'timestamp': datetime.now().isoformat()
            }
            
        except Exception as e:
            return {
                'system_id': system_id,
                'success': False,
                'error': str(e),
                'timestamp': datetime.now().isoformat()
            }
    
    def _get_system_info(self, system_id: str) -> Optional[SystemInfo]:
        """Get system information (placeholder implementation)"""
        # This would integrate with the main system registry
        return SystemInfo(
            system_id=system_id,
            hostname=f"system-{system_id[:8]}",
            ip_address=f"192.168.1.{int(system_id[:2], 16) % 254 + 1}",
            mac_address="00:00:00:00:00:00",
            system_type='desktop',
            cpu_model='Unknown',
            memory_gb=8,
            storage_gb=500,
            network_interface='eth0',
            site_id='default',
            location='Unknown'
        )
    
    def get_job_status(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Get status of an automation job"""
        return self.automation_jobs.get(job_id)
    
    def list_scripts(self) -> List[Dict[str, Any]]:
        """List all available automation scripts"""
        return list(self.scripts.values())
    
    def delete_script(self, script_name: str) -> bool:
        """Delete an automation script"""
        try:
            if script_name in self.scripts:
                script_info = self.scripts[script_name]
                script_path = Path(script_info['path'])
                
                if script_path.exists():
                    script_path.unlink()
                
                del self.scripts[script_name]
                
                self.logger.info(f"Deleted script: {script_name}")
                return True
            return False
            
        except Exception as e:
            self.logger.error(f"Failed to delete script {script_name}: {e}")
            return False
