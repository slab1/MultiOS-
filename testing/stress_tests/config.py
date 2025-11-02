#!/usr/bin/env python3
"""
Configuration file for comprehensive stress testing suite
"""

import json
from dataclasses import asdict
from pathlib import Path

# Create default configuration
default_config = {
    # Test configuration
    "test_duration": 300,  # 5 minutes
    "test_dir": "/tmp/stress_test",
    "output_dir": "./stress_test_results",
    "parallel_threads": 4,
    "verbose": False,
    
    # Memory test configuration
    "max_memory_allocation_mb": 512,
    "min_available_memory_mb": 1024,
    "memory_leak_iterations": 1000,
    "fragmentation_test_size_mb": 256,
    
    # File system test configuration
    "min_available_disk_gb": 10,
    "file_io_test_size_mb": 100,
    "concurrent_file_access_threads": 10,
    "max_file_handles": 1024,
    
    # CPU test configuration
    "cpu_stress_duration": 60,
    "cpu_threads_per_core": 2,
    "thermal_test_duration": 30,
    
    # Resource exhaustion configuration
    "max_processes": 100,
    "max_network_connections": 1000,
    
    # System resource limits
    "cpu_usage_warning_threshold": 90.0,
    "memory_usage_warning_threshold": 85.0,
    "disk_usage_warning_threshold": 90.0
}

def create_config_file(config_file: str = "stress_test_config.json"):
    """Create a configuration file with default settings"""
    config_path = Path(config_file)
    
    with open(config_path, 'w') as f:
        json.dump(default_config, f, indent=2)
    
    print(f"Created configuration file: {config_path}")
    print(f"You can edit this file to customize test parameters.")
    
    return config_path

def load_config(config_file: str) -> dict:
    """Load configuration from file"""
    config_path = Path(config_file)
    
    if not config_path.exists():
        print(f"Config file {config_file} not found. Creating default...")
        create_config_file(config_file)
        return default_config
    
    with open(config_path, 'r') as f:
        config = json.load(f)
    
    return config

if __name__ == "__main__":
    create_config_file()