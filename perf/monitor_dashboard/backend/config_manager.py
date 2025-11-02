#!/usr/bin/env python3
"""
Configuration Manager - Load and manage dashboard configuration
"""

import yaml
import os
from typing import Dict, Any
import logging

class ConfigManager:
    def __init__(self, config_path: str = "config/config.yaml"):
        self.config_path = config_path
        self.config = {}
        self.load_config()
    
    def load_config(self):
        """Load configuration from YAML file"""
        try:
            if os.path.exists(self.config_path):
                with open(self.config_path, 'r') as f:
                    self.config = yaml.safe_load(f)
                logging.info(f"Configuration loaded from {self.config_path}")
            else:
                logging.warning(f"Configuration file not found: {self.config_path}")
                self.config = self.get_default_config()
        except Exception as e:
            logging.error(f"Error loading configuration: {e}")
            self.config = self.get_default_config()
    
    def get_default_config(self) -> Dict[str, Any]:
        """Return default configuration"""
        return {
            'database': {
                'path': 'data/monitor.db',
                'backup_enabled': True,
                'backup_interval': 3600,
                'cleanup_interval': 86400,
                'max_age_days': 30
            },
            'monitoring': {
                'interval': 5,
                'history_size': 1000,
                'enable_custom_metrics': True
            },
            'thresholds': {
                'cpu': {'warning': 80, 'critical': 95},
                'memory': {'warning': 85, 'critical': 95},
                'disk': {'warning': 85, 'critical': 95},
                'load_average': {'warning': 2.0, 'critical': 4.0}
            },
            'web_dashboard': {
                'host': '0.0.0.0',
                'port': 5000,
                'debug': False
            },
            'logging': {
                'level': 'INFO',
                'file': 'logs/monitor.log',
                'max_size_mb': 50,
                'backup_count': 5
            }
        }
    
    def get(self, key: str, default=None):
        """Get configuration value by key (supports dot notation)"""
        keys = key.split('.')
        value = self.config
        
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        
        return value
    
    def set(self, key: str, value: Any):
        """Set configuration value by key (supports dot notation)"""
        keys = key.split('.')
        config = self.config
        
        # Navigate to the parent dictionary
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        
        # Set the value
        config[keys[-1]] = value
    
    def save_config(self):
        """Save current configuration to file"""
        try:
            os.makedirs(os.path.dirname(self.config_path), exist_ok=True)
            with open(self.config_path, 'w') as f:
                yaml.dump(self.config, f, default_flow_style=False, indent=2)
            logging.info(f"Configuration saved to {self.config_path}")
        except Exception as e:
            logging.error(f"Error saving configuration: {e}")
    
    def get_monitoring_config(self) -> Dict[str, Any]:
        """Get monitoring configuration"""
        return self.get('monitoring', {})
    
    def get_thresholds(self) -> Dict[str, Any]:
        """Get alert thresholds"""
        return self.get('thresholds', {})
    
    def get_database_config(self) -> Dict[str, Any]:
        """Get database configuration"""
        return self.get('database', {})
    
    def get_web_config(self) -> Dict[str, Any]:
        """Get web dashboard configuration"""
        return self.get('web_dashboard', {})
    
    def get_logging_config(self) -> Dict[str, Any]:
        """Get logging configuration"""
        return self.get('logging', {})
    
    def get_notification_config(self) -> Dict[str, Any]:
        """Get notification configuration"""
        return self.get('notifications', {})
    
    def update_thresholds(self, new_thresholds: Dict[str, Any]):
        """Update alert thresholds"""
        self.set('thresholds', {**self.get('thresholds', {}), **new_thresholds})
        self.save_config()
    
    def get_custom_metrics(self) -> Dict[str, Any]:
        """Get custom metrics configuration"""
        return self.get('custom_metrics', {})
    
    def validate_config(self) -> bool:
        """Validate current configuration"""
        required_sections = ['database', 'monitoring', 'thresholds', 'web_dashboard']
        
        for section in required_sections:
            if section not in self.config:
                logging.error(f"Missing required configuration section: {section}")
                return False
        
        # Validate thresholds
        thresholds = self.get('thresholds', {})
        for category, values in thresholds.items():
            if not isinstance(values, dict):
                logging.error(f"Invalid threshold format for {category}")
                return False
            
            for level in ['warning', 'critical']:
                if level not in values:
                    logging.warning(f"Missing {level} threshold for {category}")
        
        logging.info("Configuration validation passed")
        return True

# Global configuration instance
config_manager = None

def get_config() -> ConfigManager:
    """Get global configuration manager instance"""
    global config_manager
    if config_manager is None:
        config_manager = ConfigManager()
    return config_manager

if __name__ == "__main__":
    # Test configuration loading
    config = get_config()
    print("Configuration loaded successfully")
    print(f"Database path: {config.get('database.path')}")
    print(f"Monitoring interval: {config.get('monitoring.interval')}")
    print(f"CPU warning threshold: {config.get('thresholds.cpu.warning')}")