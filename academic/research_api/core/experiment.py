"""
Experiment Management for Research Framework

Handles the creation, execution, and management of research experiments.
Supports both synchronous and asynchronous execution with comprehensive tracking.
"""

import os
import json
import time
import asyncio
import uuid
import threading
from typing import Dict, List, Any, Optional, Callable, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import traceback
import hashlib
import pickle

from .config import ResearchConfig
from .environment import EnvironmentManager


@dataclass
class ExperimentParameters:
    """Parameters for experiment configuration."""
    name: str
    description: str
    version: str = "1.0.0"
    author: str = "Unknown"
    tags: List[str] = None
    dependencies: List[str] = None
    input_data: Dict[str, Any] = None
    configuration: Dict[str, Any] = None
    metrics_to_collect: List[str] = None
    expected_duration: Optional[int] = None  # seconds
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = []
        if self.dependencies is None:
            self.dependencies = []
        if self.input_data is None:
            self.input_data = {}
        if self.configuration is None:
            self.configuration = {}
        if self.metrics_to_collect is None:
            self.metrics_to_collect = []


@dataclass
class ExperimentEnvironment:
    """Environment configuration for experiment."""
    environment_type: str  # 'container', 'vm', 'simulation', 'physical'
    resource_requirements: Dict[str, Any]
    network_configuration: Dict[str, Any]
    software_setup: List[str]
    instrumentation_enabled: bool = False
    isolation_level: str = 'medium'
    custom_scripts: List[str] = None
    
    def __post_init__(self):
        if self.custom_scripts is None:
            self.custom_scripts = []


@dataclass
class ExperimentResult:
    """Results from experiment execution."""
    experiment_id: str
    status: str  # 'running', 'completed', 'failed', 'cancelled'
    started_at: datetime
    completed_at: Optional[datetime] = None
    duration: Optional[float] = None
    exit_code: Optional[int] = None
    output_data: Dict[str, Any] = None
    error_message: Optional[str] = None
    metrics_collected: Dict[str, Any] = None
    artifacts: List[str] = None
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.output_data is None:
            self.output_data = {}
        if self.metrics_collected is None:
            self.metrics_collected = {}
        if self.artifacts is None:
            self.artifacts = []
        if self.metadata is None:
            self.metadata = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert result to dictionary."""
        data = asdict(self)
        data['started_at'] = self.started_at.isoformat()
        if self.completed_at:
            data['completed_at'] = self.completed_at.isoformat()
        return data


class Experiment:
    """
    Represents a research experiment.
    
    Contains experiment parameters, environment configuration,
    and handles execution lifecycle.
    """
    
    def __init__(self,
                 name: str,
                 description: str,
                 parameters: Dict[str, Any],
                 environment: Dict[str, Any],
                 framework):
        """
        Initialize experiment.
        
        Args:
            name: Experiment name
            description: Experiment description
            parameters: Experiment parameters
            environment: Environment configuration
            framework: Parent framework instance
        """
        self.name = name
        self.description = description
        self.framework = framework
        self.workspace_dir = framework.workspace_dir
        self.config = framework.config
        
        # Generate unique experiment ID
        self.experiment_id = str(uuid.uuid4())
        
        # Create experiment directory
        self.experiment_dir = self.workspace_dir / "experiments" / self.experiment_id
        self.experiment_dir.mkdir(parents=True, exist_ok=True)
        
        # Setup logging
        self.logger = logging.getLogger(f"{__name__}.{name}")
        
        # Initialize experiment components
        self.parameters = ExperimentParameters(**parameters)
        self.environment = ExperimentEnvironment(**environment) if environment else None
        
        # Experiment state
        self.status = 'created'
        self.current_run = None
        self.runs_history = []
        self.execution_threads = {}
        
        # Initialize experiment configuration
        self._initialize_experiment()
        
        self.logger.info(f"Created experiment: {name} ({self.experiment_id})")
    
    def _initialize_experiment(self):
        """Initialize experiment configuration and setup."""
        # Create experiment configuration file
        exp_config = {
            'experiment_id': self.experiment_id,
            'name': self.name,
            'description': self.description,
            'parameters': asdict(self.parameters),
            'environment': asdict(self.environment) if self.environment else None,
            'created_at': datetime.now().isoformat(),
            'framework_version': self.config.framework.get('version', 'unknown')
        }
        
        config_file = self.experiment_dir / "experiment_config.json"
        with open(config_file, 'w') as f:
            json.dump(exp_config, f, indent=2)
        
        # Create subdirectories
        (self.experiment_dir / "runs").mkdir(exist_ok=True)
        (self.experiment_dir / "results").mkdir(exist_ok=True)
        (self.experiment_dir / "artifacts").mkdir(exist_ok=True)
        (self.experiment_dir / "logs").mkdir(exist_ok=True)
        
        # Setup environment if specified
        if self.environment:
            self._setup_environment()
    
    def _setup_environment(self):
        """Setup experiment environment."""
        try:
            # Create environment specification
            env_spec = {
                'name': f"{self.name}_{self.experiment_id[:8]}",
                'type': self.environment.environment_type,
                'os_type': self.parameters.configuration.get('os_type', 'linux'),
                'resources': self.environment.resource_requirements,
                'network_config': self.environment.network_configuration,
                'software_requirements': self.environment.software_setup,
                'instrumentation_enabled': self.environment.instrumentation_enabled,
                'isolation_level': self.environment.isolation_level
            }
            
            # Store environment specification
            env_file = self.experiment_dir / "environment_spec.json"
            with open(env_file, 'w') as f:
                json.dump(env_spec, f, indent=2)
            
            self.logger.info("Environment specification created")
            
        except Exception as e:
            self.logger.error(f"Failed to setup environment: {e}")
            raise
    
    def add_step(self, 
                step_name: str,
                step_type: str,
                step_config: Dict[str, Any],
                step_function: Optional[Callable] = None):
        """
        Add a step to the experiment.
        
        Args:
            step_name: Name of the step
            step_type: Type of step ('setup', 'execution', 'measurement', 'cleanup')
            step_config: Step configuration
            step_function: Optional function to execute for this step
        """
        steps_file = self.experiment_dir / "steps.json"
        
        # Load existing steps or create new
        if steps_file.exists():
            with open(steps_file, 'r') as f:
                steps = json.load(f)
        else:
            steps = {}
        
        # Add new step
        steps[step_name] = {
            'type': step_type,
            'config': step_config,
            'created_at': datetime.now().isoformat(),
            'enabled': True
        }
        
        # Add function reference if provided
        if step_function:
            # Store function reference
            func_ref_file = self.experiment_dir / f"function_{step_name}.pkl"
            with open(func_ref_file, 'wb') as f:
                pickle.dump(step_function, f)
            steps[step_name]['function_file'] = str(func_ref_file)
        
        # Save steps
        with open(steps_file, 'w') as f:
            json.dump(steps, f, indent=2)
        
        self.logger.info(f"Added step: {step_name} ({step_type})")
    
    def get_steps(self) -> Dict[str, Any]:
        """Get all experiment steps."""
        steps_file = self.experiment_dir / "steps.json"
        
        if steps_file.exists():
            with open(steps_file, 'r') as f:
                return json.load(f)
        
        return {}
    
    def run(self, 
           run_name: Optional[str] = None,
           parameters_override: Optional[Dict[str, Any]] = None,
           environment_override: Optional[Dict[str, Any]] = None) -> str:
        """
        Execute experiment synchronously.
        
        Args:
            run_name: Name for this run
            parameters_override: Override experiment parameters
            environment_override: Override environment configuration
            
        Returns:
            Run ID
        """
        runner = ExperimentRunner(self, self.config)
        run_id = runner.run_sync(run_name, parameters_override, environment_override)
        
        self.runs_history.append(run_id)
        return run_id
    
    async def run_async(self,
                       run_name: Optional[str] = None,
                       parameters_override: Optional[Dict[str, Any]] = None,
                       environment_override: Optional[Dict[str, Any]] = None) -> str:
        """
        Execute experiment asynchronously.
        
        Args:
            run_name: Name for this run
            parameters_override: Override experiment parameters
            environment_override: Override environment configuration
            
        Returns:
            Run ID
        """
        runner = ExperimentRunner(self, self.config)
        run_id = await runner.run_async(run_name, parameters_override, environment_override)
        
        self.runs_history.append(run_id)
        return run_id
    
    def get_run_result(self, run_id: str) -> Optional[ExperimentResult]:
        """
        Get result of a specific run.
        
        Args:
            run_id: Run ID
            
        Returns:
            Experiment result or None if not found
        """
        result_file = self.experiment_dir / "runs" / f"{run_id}_result.json"
        
        if result_file.exists():
            with open(result_file, 'r') as f:
                result_data = json.load(f)
            
            # Convert datetime strings back to datetime objects
            if 'started_at' in result_data:
                result_data['started_at'] = datetime.fromisoformat(result_data['started_at'])
            if 'completed_at' in result_data:
                result_data['completed_at'] = datetime.fromisoformat(result_data['completed_at'])
            
            return ExperimentResult(**result_data)
        
        return None
    
    def list_runs(self) -> List[str]:
        """List all run IDs for this experiment."""
        runs_dir = self.experiment_dir / "runs"
        if runs_dir.exists():
            return [f.stem.replace('_result', '') for f in runs_dir.glob("*_result.json")]
        return []
    
    def get_experiment_summary(self) -> Dict[str, Any]:
        """Get experiment summary information."""
        runs = self.list_runs()
        
        summary = {
            'experiment_id': self.experiment_id,
            'name': self.name,
            'description': self.description,
            'status': self.status,
            'total_runs': len(runs),
            'created_at': datetime.fromtimestamp(self.experiment_dir.stat().st_ctime).isoformat(),
            'runs': runs
        }
        
        # Analyze runs
        completed_runs = 0
        failed_runs = 0
        total_duration = 0
        
        for run_id in runs:
            result = self.get_run_result(run_id)
            if result:
                if result.status == 'completed':
                    completed_runs += 1
                elif result.status == 'failed':
                    failed_runs += 1
                
                if result.duration:
                    total_duration += result.duration
        
        summary.update({
            'completed_runs': completed_runs,
            'failed_runs': failed_runs,
            'total_execution_time': total_duration
        })
        
        return summary
    
    def export_experiment(self, export_path: str):
        """
        Export complete experiment including all runs and results.
        
        Args:
            export_path: Path to export archive
        """
        import tarfile
        
        with tarfile.open(export_path, 'w:gz') as tar:
            tar.add(self.experiment_dir, arcname=f"{self.name}_{self.experiment_id}")
        
        self.logger.info(f"Exported experiment to: {export_path}")
    
    def cleanup(self):
        """Clean up experiment resources."""
        try:
            # Stop any running executions
            for thread in self.execution_threads.values():
                if thread.is_alive():
                    # This would need proper thread management
                    pass
            
            # Clean up experiment directory if configured to do so
            if self.config.framework.get('cleanup_experiments', False):
                shutil.rmtree(self.experiment_dir)
                self.logger.info(f"Cleaned up experiment directory: {self.experiment_dir}")
            
        except Exception as e:
            self.logger.error(f"Failed to cleanup experiment {self.experiment_id}: {e}")


class ExperimentRunner:
    """
    Runner for executing experiments.
    
    Handles both synchronous and asynchronous execution,
    environment setup, step execution, and result collection.
    """
    
    def __init__(self, experiment: Experiment, config: ResearchConfig):
        """
        Initialize experiment runner.
        
        Args:
            experiment: Experiment to run
            config: Research configuration
        """
        self.experiment = experiment
        self.config = config
        self.workspace_dir = experiment.workspace_dir
        
        # Setup logging
        self.logger = logging.getLogger(f"{__name__}.{experiment.name}")
        
        # Environment management
        self.env_manager = experiment.framework.environment_manager
        self.current_instance_id = None
        
        # Execution tracking
        self.current_run_id = None
        self.current_result = None
        self.step_results = {}
        
        # Data collection
        self.metrics_collector = None
        if hasattr(experiment.framework, 'data_collector'):
            self.metrics_collector = experiment.framework.data_collector
    
    def run_sync(self,
                run_name: Optional[str] = None,
                parameters_override: Optional[Dict[str, Any]] = None,
                environment_override: Optional[Dict[str, Any]] = None) -> str:
        """
        Execute experiment synchronously.
        
        Args:
            run_name: Name for this run
            parameters_override: Override experiment parameters
            environment_override: Override environment configuration
            
        Returns:
            Run ID
        """
        return asyncio.run(self.run_async(run_name, parameters_override, environment_override))
    
    async def run_async(self,
                       run_name: Optional[str] = None,
                       parameters_override: Optional[Dict[str, Any]] = None,
                       environment_override: Optional[Dict[str, Any]] = None) -> str:
        """
        Execute experiment asynchronously.
        
        Args:
            run_name: Name for this run
            parameters_override: Override experiment parameters
            environment_override: Override environment configuration
            
        Returns:
            Run ID
        """
        # Generate run ID
        self.current_run_id = run_name or f"run_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        
        # Create result object
        self.current_result = ExperimentResult(
            experiment_id=self.experiment.experiment_id,
            status='running',
            started_at=datetime.now()
        )
        
        try:
            self.logger.info(f"Starting experiment run: {self.current_run_id}")
            
            # Merge overrides
            final_parameters = self._merge_parameters(parameters_override)
            final_environment = self._merge_environment(environment_override)
            
            # Execute experiment steps
            success = await self._execute_experiment_steps(final_parameters, final_environment)
            
            # Complete result
            if success:
                self.current_result.status = 'completed'
                self.current_result.exit_code = 0
            else:
                self.current_result.status = 'failed'
                self.current_result.exit_code = 1
            
        except Exception as e:
            self.logger.error(f"Experiment run failed: {e}")
            self.logger.error(traceback.format_exc())
            
            self.current_result.status = 'failed'
            self.current_result.exit_code = -1
            self.current_result.error_message = str(e)
        
        finally:
            # Finalize result
            self.current_result.completed_at = datetime.now()
            self.current_result.duration = (self.current_result.completed_at - 
                                          self.current_result.started_at).total_seconds()
            
            # Save result
            await self._save_run_result()
            
            # Cleanup environment
            await self._cleanup_environment()
        
        self.logger.info(f"Experiment run completed: {self.current_run_id}")
        return self.current_run_id
    
    def _merge_parameters(self, override: Optional[Dict[str, Any]]) -> ExperimentParameters:
        """Merge parameter overrides."""
        params = self.experiment.parameters
        
        if override:
            # Create new parameters with overrides
            params_dict = asdict(params)
            params_dict.update(override)
            return ExperimentParameters(**params_dict)
        
        return params
    
    def _merge_environment(self, override: Optional[Dict[str, Any]]) -> Optional[ExperimentEnvironment]:
        """Merge environment overrides."""
        env = self.experiment.environment
        
        if not env:
            return None
        
        if override:
            # Create new environment with overrides
            env_dict = asdict(env)
            env_dict.update(override)
            return ExperimentEnvironment(**env_dict)
        
        return env
    
    async def _execute_experiment_steps(self, 
                                      parameters: ExperimentParameters,
                                      environment: Optional[ExperimentEnvironment]) -> bool:
        """Execute all experiment steps."""
        success = True
        
        try:
            # Setup phase
            if not await self._execute_setup_steps(parameters, environment):
                success = False
            
            # Execution phase
            if success and not await self._execute_main_steps(parameters, environment):
                success = False
            
            # Measurement phase
            if success and not await self._execute_measurement_steps(parameters, environment):
                success = False
            
            # Cleanup phase
            await self._execute_cleanup_steps(parameters, environment)
            
        except Exception as e:
            self.logger.error(f"Error executing experiment steps: {e}")
            success = False
        
        return success
    
    async def _execute_setup_steps(self, 
                                 parameters: ExperimentParameters,
                                 environment: Optional[ExperimentEnvironment]) -> bool:
        """Execute setup steps."""
        self.logger.info("Executing setup steps")
        
        steps = self.experiment.get_steps()
        setup_steps = [name for name, step in steps.items() 
                      if step['type'] == 'setup' and step.get('enabled', True)]
        
        for step_name in setup_steps:
            if not await self._execute_step(step_name, steps[step_name]):
                return False
        
        return True
    
    async def _execute_main_steps(self, 
                                parameters: ExperimentParameters,
                                environment: Optional[ExperimentEnvironment]) -> bool:
        """Execute main experiment steps."""
        self.logger.info("Executing main steps")
        
        steps = self.experiment.get_steps()
        main_steps = [name for name, step in steps.items() 
                     if step['type'] == 'execution' and step.get('enabled', True)]
        
        for step_name in main_steps:
            if not await self._execute_step(step_name, steps[step_name]):
                return False
        
        return True
    
    async def _execute_measurement_steps(self, 
                                       parameters: ExperimentParameters,
                                       environment: Optional[ExperimentEnvironment]) -> bool:
        """Execute measurement and data collection steps."""
        self.logger.info("Executing measurement steps")
        
        steps = self.experiment.get_steps()
        measurement_steps = [name for name, step in steps.items() 
                           if step['type'] == 'measurement' and step.get('enabled', True)]
        
        for step_name in measurement_steps:
            if not await self._execute_step(step_name, steps[step_name]):
                return False
        
        return True
    
    async def _execute_cleanup_steps(self, 
                                   parameters: ExperimentParameters,
                                   environment: Optional[ExperimentEnvironment]):
        """Execute cleanup steps."""
        self.logger.info("Executing cleanup steps")
        
        steps = self.experiment.get_steps()
        cleanup_steps = [name for name, step in steps.items() 
                        if step['type'] == 'cleanup' and step.get('enabled', True)]
        
        for step_name in cleanup_steps:
            try:
                await self._execute_step(step_name, steps[step_name])
            except Exception as e:
                self.logger.warning(f"Cleanup step {step_name} failed: {e}")
    
    async def _execute_step(self, step_name: str, step_config: Dict[str, Any]) -> bool:
        """Execute a single step."""
        self.logger.info(f"Executing step: {step_name}")
        
        step_start_time = time.time()
        
        try:
            # Check if step has a function
            if 'function_file' in step_config:
                return await self._execute_function_step(step_name, step_config)
            else:
                return await self._execute_configured_step(step_name, step_config)
        
        except Exception as e:
            self.logger.error(f"Step {step_name} failed: {e}")
            self.step_results[step_name] = {
                'success': False,
                'error': str(e),
                'duration': time.time() - step_start_time
            }
            return False
    
    async def _execute_function_step(self, step_name: str, step_config: Dict[str, Any]) -> bool:
        """Execute a step with custom function."""
        func_file = step_config['function_file']
        
        if not os.path.exists(func_file):
            self.logger.error(f"Function file not found: {func_file}")
            return False
        
        try:
            with open(func_file, 'rb') as f:
                step_function = pickle.load(f)
            
            # Execute function
            if asyncio.iscoroutinefunction(step_function):
                result = await step_function(self.experiment, step_config['config'])
            else:
                result = step_function(self.experiment, step_config['config'])
            
            self.step_results[step_name] = {
                'success': result,
                'duration': 0  # Will be updated later
            }
            
            return result
            
        except Exception as e:
            self.logger.error(f"Failed to execute function step {step_name}: {e}")
            return False
    
    async def _execute_configured_step(self, step_name: str, step_config: Dict[str, Any]) -> bool:
        """Execute a configured step."""
        step_type = step_config['type']
        config = step_config['config']
        
        if step_type == 'execution':
            return await self._execute_configured_execution_step(step_name, config)
        elif step_type == 'measurement':
            return await self._execute_configured_measurement_step(step_name, config)
        else:
            # Generic step execution
            return await self._execute_generic_step(step_name, config)
    
    async def _execute_configured_execution_step(self, step_name: str, config: Dict[str, Any]) -> bool:
        """Execute a configured execution step."""
        # This would handle specific execution step types
        # like running commands, scripts, etc.
        
        if 'command' in config:
            return await self._run_command_step(config['command'])
        
        return True
    
    async def _execute_configured_measurement_step(self, step_name: str, config: Dict[str, Any]) -> bool:
        """Execute a configured measurement step."""
        if self.metrics_collector and 'metrics' in config:
            # Collect specified metrics
            duration = config.get('duration', 60)
            metrics = config['metrics']
            
            # This would start data collection
            collection_id = self.metrics_collector.start_collection(duration)
            
            self.step_results[step_name] = {
                'success': True,
                'collection_id': collection_id,
                'metrics': metrics
            }
            
            return True
        
        return True
    
    async def _execute_generic_step(self, step_name: str, config: Dict[str, Any]) -> bool:
        """Execute a generic step."""
        # Generic step handling
        self.logger.info(f"Executing generic step: {step_name}")
        
        self.step_results[step_name] = {
            'success': True,
            'config': config
        }
        
        return True
    
    async def _run_command_step(self, command: str) -> bool:
        """Run a command step."""
        try:
            process = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await process.communicate()
            
            if process.returncode == 0:
                self.logger.info(f"Command completed successfully: {command}")
                return True
            else:
                self.logger.error(f"Command failed: {command}")
                self.logger.error(f"Error output: {stderr.decode()}")
                return False
                
        except Exception as e:
            self.logger.error(f"Failed to run command {command}: {e}")
            return False
    
    async def _setup_environment(self, environment: ExperimentEnvironment) -> bool:
        """Setup experiment environment."""
        try:
            # Create environment specification
            env_spec = {
                'name': f"{self.experiment.name}_{self.current_run_id}",
                'type': environment.environment_type,
                'os_type': self.experiment.parameters.configuration.get('os_type', 'linux'),
                'resources': environment.resource_requirements,
                'network_config': environment.network_configuration,
                'software_requirements': environment.software_setup,
                'instrumentation_enabled': environment.instrumentation_enabled,
                'isolation_level': environment.isolation_level
            }
            
            # Start environment
            self.current_instance_id = self.env_manager.start_environment(
                self.experiment.name, 
                f"{self.current_run_id}_env"
            )
            
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to setup environment: {e}")
            return False
    
    async def _cleanup_environment(self):
        """Clean up experiment environment."""
        if self.current_instance_id:
            try:
                self.env_manager.stop_environment(self.current_instance_id)
            except Exception as e:
                self.logger.warning(f"Failed to cleanup environment {self.current_instance_id}: {e}")
    
    async def _save_run_result(self):
        """Save run result to file."""
        runs_dir = self.experiment_dir / "runs"
        result_file = runs_dir / f"{self.current_run_id}_result.json"
        
        # Add step results to current result
        self.current_result.output_data['step_results'] = self.step_results
        
        # Convert result to dict and save
        result_dict = self.current_result.to_dict()
        
        with open(result_file, 'w') as f:
            json.dump(result_dict, f, indent=2)
        
        self.logger.info(f"Saved run result: {result_file}")