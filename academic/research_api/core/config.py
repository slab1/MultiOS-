"""
Research Configuration Management

Handles configuration for all research framework components.
Supports YAML, JSON, and environment variable configurations.
"""

import os
import yaml
import json
from typing import Dict, Any, List, Optional, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime
import logging


@dataclass
class LoggingConfig:
    """Logging configuration settings."""
    level: str = "INFO"
    format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    file_path: str = "logs/research.log"
    max_size_mb: int = 100
    backup_count: int = 5
    console_output: bool = True


@dataclass
class PerformanceConfig:
    """Performance benchmarking configuration."""
    enabled_metrics: List[str] = None
    measurement_interval: float = 1.0
    warmup_duration: int = 5
    measurement_duration: int = 60
    iterations: int = 3
    confidence_level: float = 0.95
    
    def __post_init__(self):
        if self.enabled_metrics is None:
            self.enabled_metrics = [
                "cpu_usage", "memory_usage", "disk_io", "network_io",
                "process_count", "thread_count", "context_switches"
            ]


@dataclass
class AnalysisConfig:
    """System analysis configuration."""
    detection_algorithms: List[str] = None
    anomaly_threshold: float = 2.0
    pattern_window: int = 100
    baseline_duration: int = 300
    update_frequency: int = 60
    
    def __post_init__(self):
        if self.detection_algorithms is None:
            self.detection_algorithms = [
                "statistical_outlier", "pattern_matching", "trend_analysis"
            ]


@dataclass
class InstrumentationConfig:
    """OS instrumentation configuration."""
    monitoring_enabled: bool = True
    hooks_enabled: bool = True
    event_tracking: bool = True
    kernel_modifications: bool = False
    security_level: str = "medium"
    performance_overhead_limit: float = 0.05  # 5% maximum overhead
    
    def __post_init__(self):
        self.allowed_modifications = [
            "syscall_tracing", "memory_tracking", "process_monitoring",
            "network_tracing", "file_system_monitoring"
        ]


@dataclass
class TestingConfig:
    """Automated testing configuration."""
    test_suites: List[str] = None
    parallel_execution: bool = True
    max_workers: int = 4
    timeout_seconds: int = 300
    retry_attempts: int = 3
    coverage_threshold: float = 0.8
    
    def __post_init__(self):
        if self.test_suites is None:
            self.test_suites = [
                "functionality", "performance", "stress", "compatibility",
                "security", "reliability", "integration"
            ]


@dataclass
class DataCollectionConfig:
    """Data collection configuration."""
    storage_backend: str = "sqlite"
    compression_enabled: bool = True
    batch_size: int = 1000
    retention_days: int = 90
    auto_cleanup: bool = True
    metrics_sample_rate: float = 1.0
    
    def __post_init__(self):
        self.supported_backends = ["sqlite", "postgresql", "mongodb", "file"]


@dataclass
class ReportingConfig:
    """Reporting and visualization configuration."""
    default_format: str = "html"
    chart_types: List[str] = None
    color_scheme: str = "default"
    template_path: str = "templates/"
    export_formats: List[str] = None
    watermark: bool = False
    
    def __post_init__(self):
        if self.chart_types is None:
            self.chart_types = ["line", "bar", "scatter", "heatmap", "box"]
        if self.export_formats is None:
            self.export_formats = ["html", "pdf", "latex", "png", "svg"]


class ResearchConfig:
    """
    Main configuration manager for the research framework.
    
    Supports multiple configuration sources:
    - Configuration files (YAML, JSON)
    - Environment variables
    - Command line arguments
    - Runtime modifications
    """
    
    def __init__(self, config_path: Optional[str] = None):
        """
        Initialize configuration manager.
        
        Args:
            config_path: Path to configuration file
        """
        self.config_path = Path(config_path) if config_path else None
        
        # Initialize configuration sections
        self.logging = LoggingConfig()
        self.performance = PerformanceConfig()
        self.analysis = AnalysisConfig()
        self.instrumentation = InstrumentationConfig()
        self.testing = TestingConfig()
        self.data_collection = DataCollectionConfig()
        self.reporting = ReportingConfig()
        
        # Framework settings
        self.framework = {
            'version': '1.0.0',
            'debug': False,
            'parallel_processing': True,
            'cache_enabled': True,
            'temp_directory': '/tmp/research_api',
            'backup_enabled': True
        }
        
        # Environment settings
        self.environment = {
            'name': 'default',
            'type': 'simulation',  # simulation, virtualization, physical
            'resource_limits': {},
            'isolation_level': 'medium'
        }
        
        # Load configuration
        self._load_config()
        
        # Setup logging
        self._setup_logging()
    
    def _setup_logging(self):
        """Setup configuration-specific logging."""
        self.logger = logging.getLogger(__name__)
        
        # Configure log file path
        log_dir = Path(self.logging.file_path).parent
        log_dir.mkdir(parents=True, exist_ok=True)
        
        # Configure level
        level = getattr(logging, self.logging.level.upper(), logging.INFO)
        logging.getLogger().setLevel(level)
    
    def _load_config(self):
        """Load configuration from various sources."""
        # Load from file if specified
        if self.config_path and self.config_path.exists():
            self._load_from_file(self.config_path)
        else:
            # Load from default locations
            self._load_from_default_locations()
        
        # Load from environment variables
        self._load_from_environment()
        
        # Validate configuration
        self._validate_config()
    
    def _load_from_file(self, config_path: Path):
        """Load configuration from file."""
        try:
            with open(config_path, 'r') as f:
                if config_path.suffix.lower() in ['.yaml', '.yml']:
                    config_data = yaml.safe_load(f)
                elif config_path.suffix.lower() == '.json':
                    config_data = json.load(f)
                else:
                    raise ValueError(f"Unsupported configuration format: {config_path.suffix}")
            
            self._apply_config_data(config_data)
            self.logger.info(f"Loaded configuration from: {config_path}")
            
        except Exception as e:
            self.logger.error(f"Failed to load configuration from {config_path}: {e}")
            raise
    
    def _load_from_default_locations(self):
        """Load configuration from default locations."""
        default_paths = [
            Path.cwd() / "research_config.yaml",
            Path.cwd() / "research_config.yml", 
            Path.cwd() / "research_config.json",
            Path.home() / ".research_api" / "config.yaml",
            Path.home() / ".research_api" / "config.json"
        ]
        
        for path in default_paths:
            if path.exists():
                self._load_from_file(path)
                return
        
        self.logger.info("No configuration file found, using defaults")
    
    def _load_from_environment(self):
        """Load configuration from environment variables."""
        env_mappings = {
            'RESEARCH_LOG_LEVEL': ('logging', 'level'),
            'RESEARCH_PERF_METRICS': ('performance', 'enabled_metrics'),
            'RESEARCH_ANALYSIS_THRESHOLD': ('analysis', 'anomaly_threshold'),
            'RESEARCH_INSTRUMENTATION_ENABLED': ('instrumentation', 'monitoring_enabled'),
            'RESEARCH_TESTING_SUITES': ('testing', 'test_suites'),
            'RESEARCH_DATA_BACKEND': ('data_collection', 'storage_backend'),
            'RESEARCH_REPORT_FORMAT': ('reporting', 'default_format'),
            'RESEARCH_DEBUG': ('framework', 'debug'),
            'RESEARCH_ENV_NAME': ('environment', 'name'),
            'RESEARCH_ENV_TYPE': ('environment', 'type')
        }
        
        for env_var, (section, key) in env_mappings.items():
            value = os.getenv(env_var)
            if value:
                try:
                    # Parse value based on type
                    if key == 'enabled_metrics' or key == 'test_suites':
                        # Handle list values
                        parsed_value = [item.strip() for item in value.split(',')]
                    elif key in ['anomaly_threshold', 'performance_overhead_limit']:
                        # Handle float values
                        parsed_value = float(value)
                    elif key in ['debug', 'monitoring_enabled', 'hooks_enabled', 
                                'event_tracking', 'compression_enabled', 'parallel_processing']:
                        # Handle boolean values
                        parsed_value = value.lower() in ['true', '1', 'yes', 'on']
                    else:
                        parsed_value = value
                    
                    # Set the value
                    if section == 'framework':
                        self.framework[key] = parsed_value
                    elif section == 'environment':
                        self.environment[key] = parsed_value
                    else:
                        getattr(self, section)[key] = parsed_value
                        
                except ValueError as e:
                    self.logger.warning(f"Failed to parse environment variable {env_var}: {e}")
    
    def _apply_config_data(self, config_data: Dict[str, Any]):
        """Apply configuration data to configuration objects."""
        for section_name, section_data in config_data.items():
            if section_name == 'logging' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.logging, key):
                        setattr(self.logging, key, value)
            
            elif section_name == 'performance' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.performance, key):
                        setattr(self.performance, key, value)
            
            elif section_name == 'analysis' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.analysis, key):
                        setattr(self.analysis, key, value)
            
            elif section_name == 'instrumentation' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.instrumentation, key):
                        setattr(self.instrumentation, key, value)
            
            elif section_name == 'testing' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.testing, key):
                        setattr(self.testing, key, value)
            
            elif section_name == 'data_collection' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.data_collection, key):
                        setattr(self.data_collection, key, value)
            
            elif section_name == 'reporting' and isinstance(section_data, dict):
                for key, value in section_data.items():
                    if hasattr(self.reporting, key):
                        setattr(self.reporting, key, value)
            
            elif section_name == 'framework' and isinstance(section_data, dict):
                self.framework.update(section_data)
            
            elif section_name == 'environment' and isinstance(section_data, dict):
                self.environment.update(section_data)
    
    def _validate_config(self):
        """Validate configuration values."""
        validation_errors = []
        
        # Validate logging configuration
        if not hasattr(logging, self.logging.level.upper()):
            validation_errors.append(f"Invalid logging level: {self.logging.level}")
        
        # Validate performance configuration
        if self.performance.measurement_interval <= 0:
            validation_errors.append("Performance measurement interval must be positive")
        
        if not 0 < self.performance.confidence_level <= 1:
            validation_errors.append("Confidence level must be between 0 and 1")
        
        # Validate analysis configuration
        if self.analysis.anomaly_threshold <= 0:
            validation_errors.append("Anomaly threshold must be positive")
        
        # Validate instrumentation configuration
        if self.instrumentation.performance_overhead_limit <= 0:
            validation_errors.append("Performance overhead limit must be positive")
        
        # Validate testing configuration
        if self.testing.max_workers <= 0:
            validation_errors.append("Max workers must be positive")
        
        if not 0 < self.testing.coverage_threshold <= 1:
            validation_errors.append("Coverage threshold must be between 0 and 1")
        
        # Validate data collection configuration
        if self.data_collection.batch_size <= 0:
            validation_errors.append("Batch size must be positive")
        
        # Validate framework configuration
        if not Path(self.framework['temp_directory']).is_absolute():
            validation_errors.append("Temp directory must be an absolute path")
        
        if validation_errors:
            error_msg = "Configuration validation errors:\n" + "\n".join(validation_errors)
            raise ValueError(error_msg)
    
    def get_config_dict(self) -> Dict[str, Any]:
        """Get complete configuration as dictionary."""
        return {
            'logging': asdict(self.logging),
            'performance': asdict(self.performance),
            'analysis': asdict(self.analysis),
            'instrumentation': asdict(self.instrumentation),
            'testing': asdict(self.testing),
            'data_collection': asdict(self.data_collection),
            'reporting': asdict(self.reporting),
            'framework': self.framework,
            'environment': self.environment
        }
    
    def save_config(self, config_path: Optional[str] = None):
        """Save current configuration to file."""
        save_path = Path(config_path) if config_path else self.config_path
        if not save_path:
            # Default to current directory
            save_path = Path.cwd() / "research_config.yaml"
        
        # Ensure directory exists
        save_path.parent.mkdir(parents=True, exist_ok=True)
        
        config_dict = self.get_config_dict()
        
        try:
            with open(save_path, 'w') as f:
                yaml.dump(config_dict, f, default_flow_style=False, indent=2)
            self.logger.info(f"Configuration saved to: {save_path}")
        except Exception as e:
            self.logger.error(f"Failed to save configuration: {e}")
            raise
    
    def update_config(self, section: str, key: str, value: Any):
        """
        Update a configuration value.
        
        Args:
            section: Configuration section name
            key: Configuration key
            value: New value
        """
        if section == 'framework':
            if key in self.framework:
                self.framework[key] = value
            else:
                raise ValueError(f"Unknown framework configuration key: {key}")
        
        elif section == 'environment':
            if key in self.environment:
                self.environment[key] = value
            else:
                raise ValueError(f"Unknown environment configuration key: {key}")
        
        else:
            config_obj = getattr(self, section, None)
            if config_obj and hasattr(config_obj, key):
                setattr(config_obj, key, value)
            else:
                raise ValueError(f"Unknown configuration section/key: {section}.{key}")
        
        self.logger.info(f"Updated configuration: {section}.{key} = {value}")
    
    def get_section(self, section: str):
        """
        Get a configuration section.
        
        Args:
            section: Section name
            
        Returns:
            Configuration section object
        """
        if hasattr(self, section):
            return getattr(self, section)
        elif section in ['framework', 'environment']:
            return getattr(self, section)
        else:
            raise ValueError(f"Unknown configuration section: {section}")
    
    def reset_to_defaults(self):
        """Reset configuration to default values."""
        self.__init__(self.config_path)
        self.logger.info("Configuration reset to defaults")
    
    def export_config(self, export_path: str, format: str = 'yaml'):
        """
        Export configuration in specified format.
        
        Args:
            export_path: Path to export file
            format: Export format ('yaml', 'json')
        """
        config_dict = self.get_config_dict()
        export_file = Path(export_path)
        
        try:
            if format.lower() == 'yaml':
                with open(export_file, 'w') as f:
                    yaml.dump(config_dict, f, default_flow_style=False, indent=2)
            elif format.lower() == 'json':
                with open(export_file, 'w') as f:
                    json.dump(config_dict, f, indent=2)
            else:
                raise ValueError(f"Unsupported export format: {format}")
            
            self.logger.info(f"Configuration exported to: {export_file}")
            
        except Exception as e:
            self.logger.error(f"Failed to export configuration: {e}")
            raise
    
    def get_template(self, template_type: str) -> Dict[str, Any]:
        """
        Get configuration template.
        
        Args:
            template_type: Type of template
            
        Returns:
            Configuration template
        """
        templates = {
            'basic': {
                'framework': {
                    'debug': False,
                    'parallel_processing': True
                },
                'logging': {
                    'level': 'INFO',
                    'console_output': True
                },
                'performance': {
                    'enabled_metrics': ['cpu_usage', 'memory_usage'],
                    'measurement_duration': 60
                }
            },
            'research': {
                'framework': {
                    'debug': False,
                    'parallel_processing': True,
                    'backup_enabled': True
                },
                'performance': {
                    'enabled_metrics': ['cpu_usage', 'memory_usage', 'disk_io', 'network_io'],
                    'measurement_duration': 300,
                    'iterations': 5
                },
                'analysis': {
                    'detection_algorithms': ['statistical_outlier', 'pattern_matching'],
                    'anomaly_threshold': 2.0
                },
                'data_collection': {
                    'storage_backend': 'sqlite',
                    'compression_enabled': True,
                    'retention_days': 90
                }
            },
            'production': {
                'framework': {
                    'debug': False,
                    'parallel_processing': True,
                    'cache_enabled': True
                },
                'logging': {
                    'level': 'WARNING',
                    'file_path': 'logs/production.log'
                },
                'performance': {
                    'measurement_interval': 5.0,
                    'measurement_duration': 300
                },
                'data_collection': {
                    'storage_backend': 'postgresql',
                    'compression_enabled': True,
                    'retention_days': 365
                }
            }
        }
        
        if template_type not in templates:
            raise ValueError(f"Unknown template type: {template_type}")
        
        return templates[template_type]