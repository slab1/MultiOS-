#!/usr/bin/env python3
"""
Performance Dashboard Utilities
Helper functions and utilities for the monitoring system
"""

import os
import sys
import json
import time
import hashlib
import sqlite3
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
import logging

class DashboardUtils:
    """Utility functions for the dashboard"""
    
    @staticmethod
    def get_system_info() -> Dict[str, Any]:
        """Get detailed system information"""
        import platform
        import socket
        import psutil
        
        try:
            # Basic system info
            info = {
                'hostname': socket.gethostname(),
                'platform': platform.platform(),
                'architecture': platform.architecture(),
                'processor': platform.processor(),
                'python_version': platform.python_version(),
                'boot_time': psutil.boot_time(),
                'boot_time_formatted': datetime.fromtimestamp(psutil.boot_time()).isoformat(),
                'uptime_seconds': time.time() - psutil.boot_time(),
                'uptime_formatted': DashboardUtils.format_uptime(time.time() - psutil.boot_time()),
                'cpu_count': psutil.cpu_count(),
                'cpu_count_logical': psutil.cpu_count(logical=True),
                'memory_total': psutil.virtual_memory().total,
                'memory_total_gb': psutil.virtual_memory().total / (1024**3),
                'disk_count': len(psutil.disk_partitions()),
                'network_interfaces': len(psutil.net_if_stats())
            }
            
            return info
            
        except Exception as e:
            logging.error(f"Error getting system info: {e}")
            return {}
    
    @staticmethod
    def format_uptime(seconds: float) -> str:
        """Format uptime in human readable format"""
        days = int(seconds // 86400)
        hours = int((seconds % 86400) // 3600)
        minutes = int((seconds % 3600) // 60)
        seconds = int(seconds % 60)
        
        parts = []
        if days > 0:
            parts.append(f"{days}d")
        if hours > 0:
            parts.append(f"{hours}h")
        if minutes > 0:
            parts.append(f"{minutes}m")
        if seconds > 0 or not parts:
            parts.append(f"{seconds}s")
        
        return " ".join(parts)
    
    @staticmethod
    def format_bytes(bytes_value: int, decimals: int = 2) -> str:
        """Format bytes into human readable format"""
        for unit in ['B', 'KB', 'MB', 'GB', 'TB', 'PB']:
            if bytes_value < 1024.0:
                return f"{bytes_value:.{decimals}f} {unit}"
            bytes_value /= 1024.0
        return f"{bytes_value:.{decimals}f} EB"
    
    @staticmethod
    def format_rate(bytes_per_second: float, decimals: int = 2) -> str:
        """Format network/disk rates"""
        return f"{DashboardUtils.format_bytes(bytes_per_second, decimals)}/s"
    
    @staticmethod
    def get_database_info(db_path: str) -> Dict[str, Any]:
        """Get database statistics and information"""
        try:
            if not os.path.exists(db_path):
                return {'exists': False}
            
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            
            # Get table information
            cursor.execute("""
                SELECT name FROM sqlite_master 
                WHERE type='table' ORDER BY name
            """)
            tables = [row[0] for row in cursor.fetchall()]
            
            # Get record counts for each table
            table_counts = {}
            for table in tables:
                try:
                    cursor.execute(f"SELECT COUNT(*) FROM {table}")
                    table_counts[table] = cursor.fetchone()[0]
                except sqlite3.Error:
                    table_counts[table] = 0
            
            # Get database file size
            db_size = os.path.getsize(db_path)
            
            # Get database timestamps
            created_time = os.path.getctime(db_path)
            modified_time = os.path.getmtime(db_path)
            
            conn.close()
            
            return {
                'exists': True,
                'size_bytes': db_size,
                'size_formatted': DashboardUtils.format_bytes(db_size),
                'tables': tables,
                'table_counts': table_counts,
                'created': datetime.fromtimestamp(created_time).isoformat(),
                'modified': datetime.fromtimestamp(modified_time).isoformat(),
                'record_count': sum(table_counts.values())
            }
            
        except Exception as e:
            logging.error(f"Error getting database info: {e}")
            return {'exists': False, 'error': str(e)}
    
    @staticmethod
    def cleanup_old_data(db_path: str, days: int = 30) -> Dict[str, int]:
        """Clean up old data from database"""
        cleanup_stats = {}
        
        try:
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            
            # Calculate cutoff date
            cutoff_date = datetime.now() - timedelta(days=days)
            cutoff_str = cutoff_date.isoformat()
            
            # Clean up system_metrics
            cursor.execute("""
                DELETE FROM system_metrics 
                WHERE timestamp < ?
            """, (cutoff_str,))
            cleanup_stats['system_metrics'] = cursor.rowcount
            
            # Clean up process_metrics
            cursor.execute("""
                DELETE FROM process_metrics 
                WHERE timestamp < ?
            """, (cutoff_str,))
            cleanup_stats['process_metrics'] = cursor.rowcount
            
            # Clean up custom_metrics
            cursor.execute("""
                DELETE FROM custom_metrics 
                WHERE timestamp < ?
            """, (cutoff_str,))
            cleanup_stats['custom_metrics'] = cursor.rowcount
            
            # Clean up acknowledged alerts older than 7 days
            alert_cutoff = datetime.now() - timedelta(days=7)
            cursor.execute("""
                DELETE FROM alerts 
                WHERE timestamp < ? AND (acknowledged = TRUE OR resolved = TRUE)
            """, (alert_cutoff.isoformat(),))
            cleanup_stats['alerts'] = cursor.rowcount
            
            conn.commit()
            conn.close()
            
            total_cleaned = sum(cleanup_stats.values())
            cleanup_stats['total'] = total_cleaned
            
            logging.info(f"Cleaned up {total_cleaned} old records")
            
        except Exception as e:
            logging.error(f"Error cleaning up old data: {e}")
            cleanup_stats['error'] = str(e)
        
        return cleanup_stats
    
    @staticmethod
    def backup_database(db_path: str, backup_dir: str = 'backups') -> Optional[str]:
        """Create database backup"""
        try:
            if not os.path.exists(db_path):
                return None
            
            os.makedirs(backup_dir, exist_ok=True)
            
            # Generate backup filename with timestamp
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            backup_filename = f"monitor_db_backup_{timestamp}.db"
            backup_path = os.path.join(backup_dir, backup_filename)
            
            # Copy database file
            import shutil
            shutil.copy2(db_path, backup_path)
            
            logging.info(f"Database backup created: {backup_path}")
            return backup_path
            
        except Exception as e:
            logging.error(f"Error creating database backup: {e}")
            return None
    
    @staticmethod
    def validate_config(config_path: str) -> Dict[str, Any]:
        """Validate configuration file"""
        import yaml
        
        validation_result = {
            'valid': True,
            'errors': [],
            'warnings': []
        }
        
        try:
            with open(config_path, 'r') as f:
                config = yaml.safe_load(f)
            
            # Check required sections
            required_sections = ['database', 'monitoring', 'thresholds', 'web_dashboard']
            for section in required_sections:
                if section not in config:
                    validation_result['valid'] = False
                    validation_result['errors'].append(f"Missing required section: {section}")
            
            # Validate thresholds
            if 'thresholds' in config:
                for category, values in config['thresholds'].items():
                    if not isinstance(values, dict):
                        validation_result['errors'].append(f"Invalid threshold format for {category}")
                        continue
                    
                    for level in ['warning', 'critical']:
                        if level not in values:
                            validation_result['warnings'].append(f"Missing {level} threshold for {category}")
                        elif not isinstance(values[level], (int, float)):
                            validation_result['errors'].append(f"Invalid {level} threshold for {category}")
            
            # Validate paths
            if 'database' in config:
                db_path = config['database'].get('path', 'data/monitor.db')
                db_dir = os.path.dirname(db_path)
                if db_dir and not os.path.exists(db_dir):
                    validation_result['warnings'].append(f"Database directory will be created: {db_dir}")
            
            if 'logging' in config:
                log_file = config['logging'].get('file', 'logs/monitor.log')
                log_dir = os.path.dirname(log_file)
                if log_dir and not os.path.exists(log_dir):
                    validation_result['warnings'].append(f"Log directory will be created: {log_dir}")
            
        except FileNotFoundError:
            validation_result['valid'] = False
            validation_result['errors'].append(f"Configuration file not found: {config_path}")
        except yaml.YAMLError as e:
            validation_result['valid'] = False
            validation_result['errors'].append(f"Invalid YAML syntax: {e}")
        except Exception as e:
            validation_result['valid'] = False
            validation_result['errors'].append(f"Validation error: {e}")
        
        return validation_result
    
    @staticmethod
    def get_performance_summary(hours: int = 24, db_path: str = 'data/monitor.db') -> Dict[str, Any]:
        """Get performance summary for the last N hours"""
        try:
            if not os.path.exists(db_path):
                return {'error': 'Database not found'}
            
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            start_str = start_time.isoformat()
            
            summary = {
                'period_hours': hours,
                'start_time': start_str,
                'generated_at': datetime.now().isoformat()
            }
            
            # Get system metrics summary
            cursor.execute("""
                SELECT AVG(cpu_percent), MAX(cpu_percent), MIN(cpu_percent)
                FROM system_metrics WHERE timestamp > ?
            """, (start_str,))
            cpu_result = cursor.fetchone()
            
            cursor.execute("""
                SELECT AVG(CAST(json_extract(memory_percent, '$.percent') AS REAL)), 
                       MAX(CAST(json_extract(memory_percent, '$.percent') AS REAL)),
                       MIN(CAST(json_extract(memory_percent, '$.percent') AS REAL))
                FROM system_metrics 
                WHERE timestamp > ? AND memory_percent IS NOT NULL
            """, (start_str,))
            memory_result = cursor.fetchone()
            
            # Get alert summary
            cursor.execute("""
                SELECT COUNT(*), 
                       SUM(CASE WHEN severity = 'critical' THEN 1 ELSE 0 END),
                       SUM(CASE WHEN severity = 'warning' THEN 1 ELSE 0 END)
                FROM alerts WHERE timestamp > ?
            """, (start_str,))
            alert_result = cursor.fetchone()
            
            # Get process summary
            cursor.execute("""
                SELECT COUNT(DISTINCT name), MAX(cpu_percent)
                FROM process_metrics WHERE timestamp > ?
            """, (start_str,))
            process_result = cursor.fetchone()
            
            # Compile summary
            if cpu_result and cpu_result[0]:
                summary['cpu'] = {
                    'average': round(cpu_result[0], 1),
                    'maximum': round(cpu_result[1], 1),
                    'minimum': round(cpu_result[2], 1)
                }
            
            if memory_result and memory_result[0]:
                summary['memory'] = {
                    'average': round(memory_result[0], 1),
                    'maximum': round(memory_result[1], 1),
                    'minimum': round(memory_result[2], 1)
                }
            
            if alert_result:
                summary['alerts'] = {
                    'total': alert_result[0] or 0,
                    'critical': alert_result[1] or 0,
                    'warning': alert_result[2] or 0
                }
            
            if process_result:
                summary['processes'] = {
                    'unique_processes': process_result[0] or 0,
                    'max_cpu_percent': round(process_result[1] or 0, 1)
                }
            
            conn.close()
            return summary
            
        except Exception as e:
            logging.error(f"Error generating performance summary: {e}")
            return {'error': str(e)}
    
    @staticmethod
    def create_status_file(status_data: Dict[str, Any], status_file: str = 'dashboard.status'):
        """Create status file for external monitoring"""
        try:
            with open(status_file, 'w') as f:
                json.dump(status_data, f, indent=2, default=str)
        except Exception as e:
            logging.error(f"Error creating status file: {e}")
    
    @staticmethod
    def get_system_load_profile(db_path: str = 'data/monitor.db', hours: int = 24) -> Dict[str, Any]:
        """Analyze system load patterns"""
        try:
            if not os.path.exists(db_path):
                return {'error': 'Database not found'}
            
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            start_str = start_time.isoformat()
            
            # Get hourly CPU averages
            cursor.execute("""
                SELECT 
                    strftime('%H', timestamp) as hour,
                    AVG(cpu_percent) as avg_cpu,
                    MAX(cpu_percent) as max_cpu,
                    COUNT(*) as samples
                FROM system_metrics 
                WHERE timestamp > ?
                GROUP BY strftime('%H', timestamp)
                ORDER BY hour
            """, (start_str,))
            
            hourly_data = {}
            for row in cursor.fetchall():
                hour = row[0]
                hourly_data[hour] = {
                    'average_cpu': round(row[1] or 0, 1),
                    'max_cpu': round(row[2] or 0, 1),
                    'sample_count': row[3] or 0
                }
            
            # Find peak hours
            if hourly_data:
                peak_hour = max(hourly_data.items(), key=lambda x: x[1]['average_cpu'])
                low_hour = min(hourly_data.items(), key=lambda x: x[1]['average_cpu'])
                
                load_profile = {
                    'peak_hour': {
                        'hour': peak_hour[0],
                        'average_cpu': peak_hour[1]['average_cpu']
                    },
                    'low_hour': {
                        'hour': low_hour[0],
                        'average_cpu': low_hour[1]['average_cpu']
                    },
                    'hourly_distribution': hourly_data,
                    'analysis_period_hours': hours
                }
            else:
                load_profile = {'error': 'No data available'}
            
            conn.close()
            return load_profile
            
        except Exception as e:
            logging.error(f"Error analyzing load profile: {e}")
            return {'error': str(e)}

if __name__ == "__main__":
    # Example usage
    utils = DashboardUtils()
    
    print("System Information:")
    print(json.dumps(utils.get_system_info(), indent=2, default=str))
    
    print("\nDatabase Information:")
    print(json.dumps(utils.get_database_info('data/monitor.db'), indent=2, default=str))
    
    print("\nPerformance Summary:")
    print(json.dumps(utils.get_performance_summary(24, 'data/monitor.db'), indent=2, default=str))