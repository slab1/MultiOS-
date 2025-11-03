"""
Environment Management for Research Framework

Manages research environments including virtual machines, containers, 
and simulation environments for OS experimentation.
"""

import os
import json
import time
import asyncio
import subprocess
# Optional Docker dependency - only import if available
try:
    import docker
    DOCKER_AVAILABLE = True
except ImportError:
    docker = None
    DOCKER_AVAILABLE = False
import psutil
import yaml
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime
import logging
import tempfile
import shutil
import uuid

from .config import ResearchConfig


@dataclass
class EnvironmentSpec:
    """Specification for a research environment."""
    name: str
    type: str  # 'vm', 'container', 'simulation', 'physical'
    os_type: str  # 'linux', 'windows', 'macos', 'multios'
    resources: Dict[str, Any]  # CPU, memory, disk, network
    network_config: Dict[str, Any]
    software_requirements: List[str]
    instrumentation_enabled: bool = False
    isolation_level: str = 'medium'  # 'low', 'medium', 'high'
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


@dataclass
class EnvironmentStatus:
    """Status information for a research environment."""
    environment_id: str
    name: str
    state: str  # 'creating', 'running', 'stopped', 'error', 'terminated'
    created_at: datetime
    started_at: Optional[datetime] = None
    stopped_at: Optional[datetime] = None
    resource_usage: Dict[str, Any] = None
    network_info: Dict[str, Any] = None
    error_message: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert status to dictionary."""
        data = asdict(self)
        data['created_at'] = self.created_at.isoformat()
        if self.started_at:
            data['started_at'] = self.started_at.isoformat()
        if self.stopped_at:
            data['stopped_at'] = self.stopped_at.isoformat()
        return data


class EnvironmentManager:
    """
    Manager for research environments.
    
    Supports creating and managing:
    - Virtual machines
    - Docker containers
    - Simulation environments
    - Physical systems
    """
    
    def __init__(self, workspace_dir: Path, config: ResearchConfig):
        """
        Initialize environment manager.
        
        Args:
            workspace_dir: Workspace directory for environment files
            config: Research configuration
        """
        self.workspace_dir = workspace_dir
        self.config = config
        self.environments_dir = workspace_dir / "environments"
        self.environments_dir.mkdir(exist_ok=True)
        
        # Environment registry
        self._environments: Dict[str, EnvironmentSpec] = {}
        self._instances: Dict[str, str] = {}  # environment_name -> instance_id
        self._status: Dict[str, EnvironmentStatus] = {}
        
        # Docker client for container management
        self.docker_client = None
        if DOCKER_AVAILABLE and self._is_docker_available():
            try:
                self.docker_client = docker.from_env()
            except Exception as e:
                logging.warning(f"Failed to initialize Docker client: {e}")
                self.docker_client = None
        
        # Setup logging
        self.logger = logging.getLogger(__name__)
        
        # Load existing environments
        self._load_environments()
        
        # Cleanup old instances on startup
        self._cleanup_stale_instances()
    
    def _is_docker_available(self) -> bool:
        """Check if Docker is available."""
        try:
            subprocess.run(['docker', '--version'], 
                         capture_output=True, check=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
    
    def _load_environments(self):
        """Load environment specifications from files."""
        env_files = list(self.environments_dir.glob("*.yaml"))
        
        for env_file in env_files:
            try:
                with open(env_file, 'r') as f:
                    env_data = yaml.safe_load(f)
                
                env_spec = EnvironmentSpec(**env_data)
                self._environments[env_spec.name] = env_spec
                
                self.logger.info(f"Loaded environment specification: {env_spec.name}")
                
            except Exception as e:
                self.logger.error(f"Failed to load environment {env_file}: {e}")
    
    def _cleanup_stale_instances(self):
        """Clean up any stale environment instances."""
        # This would clean up any orphaned containers, VMs, etc.
        # Implementation depends on the virtualization platform
        pass
    
    def create_environment(self, 
                          env_spec: EnvironmentSpec) -> str:
        """
        Create a new environment specification.
        
        Args:
            env_spec: Environment specification
            
        Returns:
            Environment name
        """
        # Validate specification
        self._validate_environment_spec(env_spec)
        
        # Store specification
        self._environments[env_spec.name] = env_spec
        
        # Save to file
        env_file = self.environments_dir / f"{env_spec.name}.yaml"
        with open(env_file, 'w') as f:
            yaml.dump(asdict(env_spec), f, default_flow_style=False, indent=2)
        
        self.logger.info(f"Created environment specification: {env_spec.name}")
        return env_spec.name
    
    def _validate_environment_spec(self, env_spec: EnvironmentSpec):
        """Validate environment specification."""
        # Validate resource requirements
        if 'cpu_cores' not in env_spec.resources:
            raise ValueError("CPU cores must be specified in resources")
        
        if 'memory_gb' not in env_spec.resources:
            raise ValueError("Memory must be specified in resources")
        
        if 'disk_gb' not in env_spec.resources:
            raise ValueError("Disk space must be specified in resources")
        
        # Validate network configuration
        required_network_keys = ['subnet', 'gateway']
        for key in required_network_keys:
            if key not in env_spec.network_config:
                raise ValueError(f"Network configuration missing required key: {key}")
        
        # Validate software requirements
        if not env_spec.software_requirements:
            raise ValueError("Software requirements cannot be empty")
    
    def start_environment(self, 
                         env_name: str,
                         instance_name: Optional[str] = None) -> str:
        """
        Start an environment instance.
        
        Args:
            env_name: Name of environment specification
            instance_name: Custom instance name
            
        Returns:
            Instance ID
        """
        if env_name not in self._environments:
            raise ValueError(f"Environment {env_name} not found")
        
        env_spec = self._environments[env_name]
        
        # Generate instance ID if not provided
        instance_id = instance_name or f"{env_name}_{uuid.uuid4().hex[:8]}"
        
        # Create status record
        status = EnvironmentStatus(
            environment_id=instance_id,
            name=env_name,
            state='creating',
            created_at=datetime.now(),
            resource_usage={},
            network_info={}
        )
        self._status[instance_id] = status
        
        try:
            # Start environment based on type
            if env_spec.type == 'container':
                self._start_container_environment(env_spec, instance_id, status)
            elif env_spec.type == 'vm':
                self._start_vm_environment(env_spec, instance_id, status)
            elif env_spec.type == 'simulation':
                self._start_simulation_environment(env_spec, instance_id, status)
            elif env_spec.type == 'physical':
                self._start_physical_environment(env_spec, instance_id, status)
            else:
                raise ValueError(f"Unsupported environment type: {env_spec.type}")
            
            # Update status
            status.state = 'running'
            status.started_at = datetime.now()
            
            # Register instance
            self._instances[env_name] = instance_id
            
            self.logger.info(f"Started environment instance: {instance_id}")
            return instance_id
            
        except Exception as e:
            status.state = 'error'
            status.error_message = str(e)
            self.logger.error(f"Failed to start environment {instance_id}: {e}")
            raise
    
    def _start_container_environment(self, 
                                   env_spec: EnvironmentSpec, 
                                   instance_id: str, 
                                   status: EnvironmentStatus):
        """Start a container-based environment."""
        if not self.docker_client:
            raise RuntimeError("Docker client not available")
        
        # Create container configuration
        container_config = {
            'image': self._get_container_image(env_spec),
            'name': instance_id,
            'detach': True,
            'remove': False,
            'mem_limit': f"{env_spec.resources['memory_gb']}g",
            'cpus': env_spec.resources['cpu_cores'],
            'network': env_spec.network_config.get('network_name', 'bridge')
        }
        
        # Add volume mounts for workspace
        if 'mounts' in env_spec.network_config:
            container_config['volumes'] = env_spec.network_config['mounts']
        
        # Create and start container
        container = self.docker_client.containers.run(**container_config)
        
        # Wait for container to be ready
        self._wait_for_container_ready(container, env_spec)
        
        # Collect network information
        container.reload()
        status.network_info = {
            'container_id': container.id,
            'ip_address': container.attrs['NetworkSettings']['IPAddress'],
            'ports': container.attrs['NetworkSettings']['Ports']
        }
    
    def _get_container_image(self, env_spec: EnvironmentSpec) -> str:
        """Get appropriate container image for environment specification."""
        # Map OS types to base images
        image_mapping = {
            'linux': 'ubuntu:20.04',
            'ubuntu': 'ubuntu:20.04',
            'debian': 'debian:11',
            'centos': 'centos:8',
            'alpine': 'alpine:3.15',
            'windows': 'mcr.microsoft.com/windows/servercore:ltsc2022'
        }
        
        base_image = image_mapping.get(env_spec.os_type.lower(), 'ubuntu:20.04')
        
        # Apply instrumentation configuration
        if env_spec.instrumentation_enabled:
            # Add instrumentation tools to base image
            # This would typically involve building a custom image
            base_image = f"{env_spec.name}_instrumented"
        
        return base_image
    
    def _wait_for_container_ready(self, container, env_spec: EnvironmentSpec, timeout: int = 300):
        """Wait for container to be ready."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                container.reload()
                if container.status == 'running':
                    # Check if container is accepting connections
                    # This would involve executing a health check command
                    return True
                time.sleep(1)
            except Exception as e:
                self.logger.warning(f"Container health check failed: {e}")
                time.sleep(1)
        
        raise TimeoutError(f"Container {container.name} did not become ready within {timeout} seconds")
    
    def _start_vm_environment(self, 
                            env_spec: EnvironmentSpec, 
                            instance_id: str, 
                            status: EnvironmentStatus):
        """Start a VM-based environment."""
        # VM creation would require integration with virtualization platforms
        # like libvirt, VMware, VirtualBox, or cloud providers
        
        # For now, create a placeholder implementation
        self.logger.info(f"VM environment creation not fully implemented for: {env_spec.os_type}")
        
        # Create VM configuration file
        vm_config = {
            'name': instance_id,
            'vcpu': env_spec.resources['cpu_cores'],
            'memory': env_spec.resources['memory_gb'] * 1024,  # KB
            'disk': env_spec.resources['disk_gb'],
            'os_type': env_spec.os_type,
            'network': env_spec.network_config
        }
        
        # Save VM configuration
        vm_config_file = self.environments_dir / f"{instance_id}_config.json"
        with open(vm_config_file, 'w') as f:
            json.dump(vm_config, f, indent=2)
        
        # Simulate VM creation
        time.sleep(2)  # Simulate VM creation time
        
        status.network_info = {'instance_id': instance_id}
    
    def _start_simulation_environment(self, 
                                    env_spec: EnvironmentSpec, 
                                    instance_id: str, 
                                    status: EnvironmentStatus):
        """Start a simulation environment."""
        # Create simulation workspace
        sim_workspace = self.environments_dir / "simulations" / instance_id
        sim_workspace.mkdir(parents=True, exist_ok=True)
        
        # Create simulation configuration
        sim_config = {
            'environment_id': instance_id,
            'resources': env_spec.resources,
            'network': env_spec.network_config,
            'simulation_time': 0,
            'real_time_factor': 1.0
        }
        
        # Save simulation configuration
        sim_config_file = sim_workspace / "config.json"
        with open(sim_config_file, 'w') as f:
            json.dump(sim_config, f, indent=2)
        
        # Initialize simulation state
        sim_state = {
            'running': True,
            'started_at': datetime.now().isoformat(),
            'current_time': 0,
            'events': []
        }
        
        state_file = sim_workspace / "state.json"
        with open(state_file, 'w') as f:
            json.dump(sim_state, f, indent=2)
        
        status.network_info = {'workspace': str(sim_workspace)}
    
    def _start_physical_environment(self, 
                                  env_spec: EnvironmentSpec, 
                                  instance_id: str, 
                                  status: EnvironmentStatus):
        """Start a physical environment (host system)."""
        # Physical environment uses the host system
        
        # Validate system resources
        available_cpu = psutil.cpu_count()
        available_memory = psutil.virtual_memory().total / (1024**3)  # GB
        available_disk = psutil.disk_usage('/').free / (1024**3)  # GB
        
        if env_spec.resources['cpu_cores'] > available_cpu:
            raise ValueError(f"Insufficient CPU cores: requested {env_spec.resources['cpu_cores']}, available {available_cpu}")
        
        if env_spec.resources['memory_gb'] > available_memory:
            raise ValueError(f"Insufficient memory: requested {env_spec.resources['memory_gb']}GB, available {available_memory}GB")
        
        if env_spec.resources['disk_gb'] > available_disk:
            raise ValueError(f"Insufficient disk space: requested {env_spec.resources['disk_gb']}GB, available {available_disk}GB")
        
        # Create working directory for physical environment
        env_workspace = self.environments_dir / "physical" / instance_id
        env_workspace.mkdir(parents=True, exist_ok=True)
        
        status.network_info = {
            'workspace': str(env_workspace),
            'hostname': os.uname().nodename
        }
    
    def stop_environment(self, instance_id: str, force: bool = False):
        """
        Stop an environment instance.
        
        Args:
            instance_id: Instance ID to stop
            force: Force stop (kill processes)
        """
        if instance_id not in self._status:
            raise ValueError(f"Environment instance {instance_id} not found")
        
        status = self._status[instance_id]
        
        if status.state != 'running':
            self.logger.warning(f"Environment {instance_id} is not running (state: {status.state})")
            return
        
        try:
            # Get environment specification
            env_spec = self._environments[status.name]
            
            # Stop environment based on type
            if env_spec.type == 'container':
                self._stop_container_environment(instance_id, force)
            elif env_spec.type == 'vm':
                self._stop_vm_environment(instance_id, force)
            elif env_spec.type == 'simulation':
                self._stop_simulation_environment(instance_id, force)
            elif env_spec.type == 'physical':
                self._stop_physical_environment(instance_id, force)
            
            # Update status
            status.state = 'stopped'
            status.stopped_at = datetime.now()
            
            # Remove from active instances
            if status.name in self._instances:
                del self._instances[status.name]
            
            self.logger.info(f"Stopped environment instance: {instance_id}")
            
        except Exception as e:
            status.state = 'error'
            status.error_message = str(e)
            self.logger.error(f"Failed to stop environment {instance_id}: {e}")
            raise
    
    def _stop_container_environment(self, instance_id: str, force: bool):
        """Stop a container environment."""
        if not self.docker_client:
            return
        
        try:
            container = self.docker_client.containers.get(instance_id)
            
            if force:
                container.kill()
            else:
                container.stop(timeout=30)
            
        except Exception as e:
            if DOCKER_AVAILABLE and 'docker' in str(type(e).__module__):
                # Docker-specific errors
                self.logger.warning(f"Container {instance_id} not found or error: {e}")
            else:
                self.logger.warning(f"Failed to stop container {instance_id}: {e}")
            # Continue execution regardless of container state
    
    def _stop_vm_environment(self, instance_id: str, force: bool):
        """Stop a VM environment."""
        # VM shutdown implementation would depend on virtualization platform
        self.logger.info(f"VM shutdown not implemented for: {instance_id}")
    
    def _stop_simulation_environment(self, instance_id: str, force: bool):
        """Stop a simulation environment."""
        sim_workspace = self.environments_dir / "simulations" / instance_id
        state_file = sim_workspace / "state.json"
        
        if state_file.exists():
            with open(state_file, 'r') as f:
                sim_state = json.load(f)
            
            sim_state['running'] = False
            sim_state['stopped_at'] = datetime.now().isoformat()
            
            with open(state_file, 'w') as f:
                json.dump(sim_state, f, indent=2)
    
    def _stop_physical_environment(self, instance_id: str, force: bool):
        """Stop a physical environment."""
        # Physical environment cleanup
        env_workspace = self.environments_dir / "physical" / instance_id
        
        if env_workspace.exists():
            # Cleanup any processes or resources
            # This would depend on what was started in the physical environment
            pass
    
    def get_environment_status(self, instance_id: str) -> Optional[EnvironmentStatus]:
        """
        Get status of an environment instance.
        
        Args:
            instance_id: Instance ID
            
        Returns:
            Environment status or None if not found
        """
        return self._status.get(instance_id)
    
    def list_environments(self) -> List[EnvironmentSpec]:
        """List all environment specifications."""
        return list(self._environments.values())
    
    def list_instances(self) -> List[EnvironmentStatus]:
        """List all running instances."""
        return [status for status in self._status.values() if status.state == 'running']
    
    def get_resource_usage(self, instance_id: str) -> Dict[str, Any]:
        """
        Get current resource usage for an environment instance.
        
        Args:
            instance_id: Instance ID
            
        Returns:
            Resource usage information
        """
        if instance_id not in self._status:
            raise ValueError(f"Environment instance {instance_id} not found")
        
        status = self._status[instance_id]
        env_spec = self._environments[status.name]
        
        # Get usage based on environment type
        if env_spec.type == 'container':
            return self._get_container_usage(instance_id)
        elif env_spec.type == 'vm':
            return self._get_vm_usage(instance_id)
        elif env_spec.type == 'simulation':
            return self._get_simulation_usage(instance_id)
        elif env_spec.type == 'physical':
            return self._get_physical_usage()
        
        return {}
    
    def _get_container_usage(self, instance_id: str) -> Dict[str, Any]:
        """Get container resource usage."""
        if not self.docker_client:
            return {}
        
        try:
            container = self.docker_client.containers.get(instance_id)
            stats = container.stats(stream=False)
            
            return {
                'cpu_percent': stats['cpu_stats']['cpu_usage']['total_usage'] / 1000000000,
                'memory_usage': stats['memory_stats']['usage'],
                'memory_limit': stats['memory_stats']['limit'],
                'network_rx': stats['networks']['eth0']['rx_bytes'],
                'network_tx': stats['networks']['eth0']['tx_bytes'],
                'block_read': stats['blkio_stats']['io_service_bytes_recursive'][0]['value'] if stats['blkio_stats']['io_service_bytes_recursive'] else 0,
                'block_write': stats['blkio_stats']['io_service_bytes_recursive'][1]['value'] if len(stats['blkio_stats']['io_service_bytes_recursive']) > 1 else 0
            }
            
        except Exception as e:
            self.logger.error(f"Failed to get container usage for {instance_id}: {e}")
            return {}
    
    def _get_vm_usage(self, instance_id: str) -> Dict[str, Any]:
        """Get VM resource usage."""
        # VM usage monitoring would depend on virtualization platform
        return {'status': 'not_implemented'}
    
    def _get_simulation_usage(self, instance_id: str) -> Dict[str, Any]:
        """Get simulation resource usage."""
        sim_workspace = self.environments_dir / "simulations" / instance_id
        state_file = sim_workspace / "state.json"
        
        if state_file.exists():
            with open(state_file, 'r') as f:
                sim_state = json.load(f)
            
            return {
                'simulation_time': sim_state.get('current_time', 0),
                'events_processed': len(sim_state.get('events', [])),
                'running': sim_state.get('running', False)
            }
        
        return {}
    
    def _get_physical_usage(self) -> Dict[str, Any]:
        """Get physical system resource usage."""
        try:
            return {
                'cpu_percent': psutil.cpu_percent(),
                'memory_percent': psutil.virtual_memory().percent,
                'disk_percent': psutil.disk_usage('/').percent,
                'network_io': psutil.net_io_counters()._asdict(),
                'process_count': len(psutil.pids()),
                'load_average': os.getloadavg() if hasattr(os, 'getloadavg') else None
            }
        except Exception as e:
            self.logger.error(f"Failed to get physical system usage: {e}")
            return {}
    
    def execute_command(self, instance_id: str, command: str, timeout: int = 300) -> Dict[str, Any]:
        """
        Execute a command in an environment instance.
        
        Args:
            instance_id: Instance ID
            command: Command to execute
            timeout: Command timeout in seconds
            
        Returns:
            Command execution results
        """
        if instance_id not in self._status:
            raise ValueError(f"Environment instance {instance_id} not found")
        
        status = self._status[instance_id]
        env_spec = self._environments[status.name]
        
        if env_spec.type == 'container':
            return self._execute_in_container(instance_id, command, timeout)
        elif env_spec.type == 'vm':
            return self._execute_in_vm(instance_id, command, timeout)
        elif env_spec.type == 'simulation':
            return self._execute_in_simulation(instance_id, command, timeout)
        elif env_spec.type == 'physical':
            return self._execute_on_physical(instance_id, command, timeout)
        
        return {}
    
    def _execute_in_container(self, instance_id: str, command: str, timeout: int) -> Dict[str, Any]:
        """Execute command in container."""
        if not self.docker_client:
            raise RuntimeError("Docker client not available")
        
        try:
            container = self.docker_client.containers.get(instance_id)
            result = container.exec_run(command, detach=False, stream=False)
            
            return {
                'exit_code': result.exit_code,
                'output': result.output.decode('utf-8') if result.output else '',
                'error': result.output.decode('utf-8') if result.output and result.exit_code != 0 else ''
            }
            
        except Exception as e:
            self.logger.error(f"Failed to execute command in container {instance_id}: {e}")
            return {'exit_code': -1, 'error': str(e)}
    
    def _execute_in_vm(self, instance_id: str, command: str, timeout: int) -> Dict[str, Any]:
        """Execute command in VM."""
        # VM command execution would depend on virtualization platform
        return {'exit_code': -1, 'error': 'VM command execution not implemented'}
    
    def _execute_in_simulation(self, instance_id: str, command: str, timeout: int) -> Dict[str, Any]:
        """Execute command in simulation."""
        # Simulation command execution
        sim_workspace = self.environments_dir / "simulations" / instance_id
        
        return {
            'exit_code': 0,
            'output': f"Command executed in simulation environment: {command}",
            'simulation_workspace': str(sim_workspace)
        }
    
    def _execute_on_physical(self, instance_id: str, command: str, timeout: int) -> Dict[str, Any]:
        """Execute command on physical system."""
        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            return {
                'exit_code': result.returncode,
                'output': result.stdout,
                'error': result.stderr
            }
            
        except subprocess.TimeoutExpired:
            return {'exit_code': -1, 'error': f'Command timed out after {timeout} seconds'}
        except Exception as e:
            return {'exit_code': -1, 'error': str(e)}
    
    def cleanup(self):
        """Cleanup all environments and instances."""
        self.logger.info("Cleaning up all environment instances")
        
        # Stop all running instances
        for instance_id in list(self._status.keys()):
            try:
                self.stop_environment(instance_id, force=True)
            except Exception as e:
                self.logger.error(f"Failed to cleanup instance {instance_id}: {e}")
        
        # Clear registries
        self._environments.clear()
        self._instances.clear()
        self._status.clear()
        
        self.logger.info("Environment cleanup complete")
    
    def initialize(self):
        """Initialize the environment manager."""
        self.logger.info("Initializing environment manager")
        
        # Check system requirements
        self._check_system_requirements()
        
        # Setup cleanup schedule
        self._setup_cleanup_schedule()
    
    def _check_system_requirements(self):
        """Check system requirements for environment management."""
        # Check available disk space
        disk_usage = psutil.disk_usage(self.workspace_dir)
        free_gb = disk_usage.free / (1024**3)
        
        if free_gb < 10:  # Less than 10GB free
            self.logger.warning(f"Low disk space: {free_gb:.1f}GB free")
        
        # Check available memory
        memory = psutil.virtual_memory()
        available_gb = memory.available / (1024**3)
        
        if available_gb < 2:  # Less than 2GB available
            self.logger.warning(f"Low available memory: {available_gb:.1f}GB available")
        
        # Check Docker availability
        if self._is_docker_available():
            self.logger.info("Docker is available for container environments")
        else:
            self.logger.warning("Docker is not available - container environments will not be supported")
    
    def _setup_cleanup_schedule(self):
        """Setup periodic cleanup of stale environments."""
        # This would typically involve a background thread or cron job
        pass