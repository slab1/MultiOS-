#!/usr/bin/env python3
"""
MultiOS Logging Infrastructure
Centralized log aggregation, parsing, analysis, and management
"""

import asyncio
import json
import logging
import time
import re
import sqlite3
import threading
import os
import sys
import gzip
import shutil
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Callable, Generator
from dataclasses import dataclass, asdict
from pathlib import Path
from collections import defaultdict, deque
import hashlib
import yaml

@dataclass
class LogEntry:
    """Structured log entry"""
    timestamp: float
    level: str  # DEBUG, INFO, WARNING, ERROR, CRITICAL
    source: str  # application, service, component name
    message: str
    details: Dict[str, Any] = None
    tags: List[str] = None
    user_id: str = None
    session_id: str = None
    trace_id: str = None
    correlation_id: str = None

@dataclass
class LogFilter:
    """Log filtering criteria"""
    levels: List[str] = None
    sources: List[str] = None
    start_time: float = None
    end_time: float = None
    search_text: str = None
    regex_pattern: str = None
    tags: List[str] = None
    user_id: str = None

class LogParser:
    """Parse various log formats"""
    
    def __init__(self):
        self.parsers = {
            'syslog': self._parse_syslog,
            'nginx': self._parse_nginx,
            'apache': self._parse_apache,
            'systemd': self._parse_systemd,
            'application': self._parse_application,
            'multios': self._parse_multios
        }
        
        # Common regex patterns
        self.patterns = {
            'timestamp': r'(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)',
            'level': r'(DEBUG|INFO|WARNING|ERROR|CRITICAL|TRACE)',
            'source': r'([\w\-]+(?:\.[\w\-]+)*)',
            'message': r'(.+)'
        }
    
    def register_parser(self, format_name: str, parser_func: Callable):
        """Register a custom parser"""
        self.parsers[format_name] = parser_func
    
    def parse_line(self, line: str, format_type: str = 'auto') -> Optional[LogEntry]:
        """Parse a single log line"""
        try:
            if format_type == 'auto':
                format_type = self._detect_format(line)
            
            if format_type in self.parsers:
                return self.parsers[format_type](line)
            else:
                return self._parse_generic(line)
                
        except Exception as e:
            logging.error(f"Error parsing log line: {e}")
            return None
    
    def _detect_format(self, line: str) -> str:
        """Auto-detect log format"""
        # Check for common patterns
        if '[' in line and ']' in line and ' - ' in line:
            return 'syslog'
        elif re.search(r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}', line):
            return 'nginx'
        elif 'GET' in line or 'POST' in line:
            return 'apache'
        elif line.startswith('[') and ']' in line:
            return 'systemd'
        elif line.startswith('{') or line.startswith('"'):
            return 'application'
        else:
            return 'multios'
    
    def _parse_syslog(self, line: str) -> LogEntry:
        """Parse syslog format"""
        # Pattern: <month> <day> <time> <hostname> <source>[<pid>]: <level>: <message>
        pattern = r'(\w{3})\s+(\d{1,2})\s+(\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\S+)(?:\[(\d+)\])?:\s*(\w+):\s*(.+)'
        match = re.match(pattern, line)
        
        if match:
            month, day, time_str, hostname, source, pid, level, message = match.groups()
            
            # Convert to timestamp
            timestamp = self._parse_timestamp(f"{month} {day} {time_str}")
            
            return LogEntry(
                timestamp=timestamp,
                level=level.upper(),
                source=source,
                message=message,
                details={'hostname': hostname, 'pid': pid}
            )
        
        return None
    
    def _parse_nginx(self, line: str) -> LogEntry:
        """Parse nginx access log format"""
        # Pattern: IP - - [timestamp] "METHOD path" status size "referer" "user-agent"
        pattern = r'(\S+)\s+-\s+-\s+\[([^\]]+)\]\s+"([^"]+)"\s+(\d{3})\s+(\d+)\s+"([^"]*)"\s+"([^"]*)"'
        match = re.match(pattern, line)
        
        if match:
            ip, timestamp, request, status, size, referer, user_agent = match.groups()
            
            # Parse request
            request_parts = request.split()
            method = request_parts[0] if request_parts else ''
            path = request_parts[1] if len(request_parts) > 1 else ''
            
            return LogEntry(
                timestamp=self._parse_timestamp(timestamp, format='nginx'),
                level='INFO',
                source='nginx',
                message=f"{method} {path}",
                details={
                    'ip': ip,
                    'status_code': int(status),
                    'size': int(size),
                    'referer': referer,
                    'user_agent': user_agent,
                    'method': method,
                    'path': path
                }
            )
        
        return None
    
    def _parse_apache(self, line: str) -> LogEntry:
        """Parse Apache access log format"""
        return self._parse_nginx(line)  # Similar format
    
    def _parse_systemd(self, line: str) -> LogEntry:
        """Parse systemd journal format"""
        # Pattern: [timestamp] [priority] message
        pattern = r'\[([^\]]+)\]\s+<([^\>]+)>\s+(.+)'
        match = re.match(pattern, line)
        
        if match:
            timestamp, priority, message = match.groups()
            
            # Convert priority to level
            level_map = {
                '0': 'EMERG', '1': 'ALERT', '2': 'CRITICAL', '3': 'ERROR',
                '4': 'WARNING', '5': 'NOTICE', '6': 'INFO', '7': 'DEBUG'
            }
            level = level_map.get(priority, 'INFO')
            
            return LogEntry(
                timestamp=self._parse_timestamp(timestamp),
                level=level,
                source='systemd',
                message=message
            )
        
        return None
    
    def _parse_application(self, line: str) -> LogEntry:
        """Parse application logs (JSON or key-value)"""
        try:
            # Try JSON parsing first
            if line.startswith('{'):
                data = json.loads(line)
                return LogEntry(
                    timestamp=data.get('timestamp', time.time()),
                    level=data.get('level', 'INFO'),
                    source=data.get('source', 'application'),
                    message=data.get('message', ''),
                    details=data.get('details'),
                    tags=data.get('tags'),
                    user_id=data.get('user_id'),
                    session_id=data.get('session_id'),
                    trace_id=data.get('trace_id'),
                    correlation_id=data.get('correlation_id')
                )
        except json.JSONDecodeError:
            pass
        
        # Try key-value parsing
        pattern = r'(\w+)=("([^"]*)"|(\S+))\s*(.*)'
        match = re.match(pattern, line)
        
        if match:
            key, value, quoted, unquoted, rest = match.groups()
            actual_value = quoted or unquoted
            
            if key.lower() in ['level', 'loglevel']:
                level = actual_value
            elif key.lower() in ['source', 'service']:
                source = actual_value
            elif key.lower() in ['message', 'msg']:
                message = actual_value + ' ' + rest if rest else actual_value
            else:
                # Continue parsing for other fields
                message = rest if rest else line
                level = 'INFO'
                source = 'application'
        
        return LogEntry(
            timestamp=time.time(),
            level=level if 'level' in locals() else 'INFO',
            source=source if 'source' in locals() else 'application',
            message=message if 'message' in locals() else line
        )
    
    def _parse_multios(self, line: str) -> LogEntry:
        """Parse MultiOS-specific format"""
        # Pattern: [timestamp] [level] [source] message
        pattern = r'\[([^\]]+)\]\s+\[(\w+)\]\s+\[([^\]]+)\]\s+(.+)'
        match = re.match(pattern, line)
        
        if match:
            timestamp, level, source, message = match.groups()
            return LogEntry(
                timestamp=self._parse_timestamp(timestamp),
                level=level.upper(),
                source=source,
                message=message
            )
        
        # Fallback to generic parsing
        return self._parse_generic(line)
    
    def _parse_generic(self, line: str) -> LogEntry:
        """Generic log parsing"""
        # Try to extract timestamp, level, and message
        timestamp_match = re.search(self.patterns['timestamp'], line)
        level_match = re.search(self.patterns['level'], line, re.IGNORECASE)
        source_match = re.search(self.patterns['source'], line)
        
        timestamp = self._parse_timestamp(timestamp_match.group(1)) if timestamp_match else time.time()
        level = level_match.group(1).upper() if level_match else 'INFO'
        source = source_match.group(1) if source_match else 'unknown'
        
        # Extract message
        message = line
        if timestamp_match:
            message = line[timestamp_match.end():].strip()
        if level_match:
            message = message[level_match.end():].strip()
        if source_match:
            message = message[source_match.end():].strip()
        
        # Remove leading separators
        message = re.sub(r'^[\s:,-]+', '', message)
        
        return LogEntry(
            timestamp=timestamp,
            level=level,
            source=source,
            message=message
        )
    
    def _parse_timestamp(self, timestamp_str: str, format: str = None) -> float:
        """Parse timestamp string to float"""
        try:
            if format == 'nginx':
                # nginx format: 10/Oct/2000:13:55:36 -0700
                return datetime.strptime(timestamp_str, '%d/%b/%Y:%H:%M:%S %z').timestamp()
            else:
                # Try various common formats
                formats = [
                    '%Y-%m-%d %H:%M:%S.%f',
                    '%Y-%m-%d %H:%M:%S',
                    '%Y-%m-%dT%H:%M:%S.%fZ',
                    '%Y-%m-%dT%H:%M:%SZ',
                    '%Y-%m-%dT%H:%M:%S%z',
                    '%Y-%m-%dT%H:%M:%S',
                    '%Y/%m/%d %H:%M:%S',
                    '%m/%d/%Y %H:%M:%S'
                ]
                
                for fmt in formats:
                    try:
                        return datetime.strptime(timestamp_str, fmt).timestamp()
                    except ValueError:
                        continue
                
                # Fallback to current time
                return time.time()
                
        except Exception:
            return time.time()

class LogAggregator:
    """Aggregates logs from multiple sources"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.sources = {}
        self.log_handlers = []
        self.log_queue = deque(maxlen=config.get('max_queue_size', 100000))
        self.is_running = False
        self.parser = LogParser()
        
        # Storage configuration
        self.storage_backend = config.get('storage_backend', 'sqlite')
        self.retention_days = self._parse_retention(config.get('retention', '30d'))
        
        # Statistics
        self.stats = {
            'total_logs': 0,
            'parsed_logs': 0,
            'failed_logs': 0,
            'last_update': time.time()
        }
    
    def _parse_retention(self, retention_str: str) -> int:
        """Parse retention string to days"""
        if retention_str.endswith('d'):
            return int(retention_str[:-1])
        elif retention_str.endswith('h'):
            return int(retention_str[:-1]) // 24
        elif retention_str.endswith('w'):
            return int(retention_str[:-1]) * 7
        elif retention_str.endswith('m'):
            return int(retention_str[:-1]) * 30
        elif retention_str.endswith('y'):
            return int(retention_str[:-1]) * 365
        else:
            return int(retention_str) if retention_str.isdigit() else 30
    
    def add_source(self, source_name: str, source_config: Dict[str, Any]):
        """Add a log source"""
        self.sources[source_name] = {
            'config': source_config,
            'type': source_config.get('type', 'file'),
            'path': source_config.get('path'),
            'format': source_config.get('format', 'auto'),
            'enabled': source_config.get('enabled', True)
        }
        logging.info(f"Added log source: {source_name}")
    
    def add_log_handler(self, handler: Callable):
        """Add a custom log handler"""
        self.log_handlers.append(handler)
    
    def start(self):
        """Start log aggregation"""
        self.is_running = True
        
        # Start aggregation threads
        for source_name, source_config in self.sources.items():
            if source_config['enabled']:
                thread = threading.Thread(target=self._monitor_source, args=(source_name,))
                thread.daemon = True
                thread.start()
        
        # Start processing thread
        self.processing_thread = threading.Thread(target=self._process_logs)
        self.processing_thread.daemon = True
        self.processing_thread.start()
        
        # Start cleanup thread
        self.cleanup_thread = threading.Thread(target=self._cleanup_old_logs)
        self.cleanup_thread.daemon = True
        self.cleanup_thread.start()
        
        logging.info("Log aggregation started")
    
    def stop(self):
        """Stop log aggregation"""
        self.is_running = False
        
        if hasattr(self, 'processing_thread'):
            self.processing_thread.join()
        
        if hasattr(self, 'cleanup_thread'):
            self.cleanup_thread.join()
        
        logging.info("Log aggregation stopped")
    
    def _monitor_source(self, source_name: str):
        """Monitor a specific log source"""
        source_config = self.sources[source_name]
        source_type = source_config['type']
        source_path = source_config['path']
        format_type = source_config['format']
        
        if source_type == 'file':
            self._monitor_log_file(source_name, source_path, format_type)
        elif source_type == 'directory':
            self._monitor_log_directory(source_name, source_path, format_type)
        elif source_type == 'syslog':
            self._monitor_syslog(source_name, source_path)
        elif source_type == 'journal':
            self._monitor_systemd_journal(source_name)
    
    def _monitor_log_file(self, source_name: str, file_path: str, format_type: str):
        """Monitor a single log file"""
        try:
            with open(file_path, 'r') as f:
                # Move to end of file
                f.seek(0, 2)
                
                while self.is_running:
                    line = f.readline()
                    if line:
                        self._process_log_line(source_name, line.rstrip('\n\r'), format_type)
                    else:
                        time.sleep(1)
        except FileNotFoundError:
            logging.warning(f"Log file not found: {file_path}")
        except Exception as e:
            logging.error(f"Error monitoring file {file_path}: {e}")
    
    def _monitor_log_directory(self, source_name: str, dir_path: str, format_type: str):
        """Monitor a directory of log files"""
        processed_files = set()
        
        while self.is_running:
            try:
                log_files = list(Path(dir_path).glob('*.log'))
                for log_file in log_files:
                    if log_file not in processed_files:
                        self._process_log_file(source_name, str(log_file), format_type)
                        processed_files.add(log_file)
                
                # Clean up processed files set periodically
                if len(processed_files) > 1000:
                    processed_files = {f for f in processed_files if f.exists()}
                
                time.sleep(10)
            except Exception as e:
                logging.error(f"Error monitoring directory {dir_path}: {e}")
                time.sleep(5)
    
    def _process_log_file(self, source_name: str, file_path: str, format_type: str):
        """Process a single log file"""
        try:
            with open(file_path, 'r') as f:
                for line in f:
                    self._process_log_line(source_name, line.rstrip('\n\r'), format_type)
        except Exception as e:
            logging.error(f"Error processing file {file_path}: {e}")
    
    def _monitor_syslog(self, source_name: str, socket_path: str = None):
        """Monitor syslog socket"""
        try:
            import socket
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            
            if socket_path:
                sock.connect(socket_path)
            else:
                sock.connect('/dev/log')
            
            while self.is_running:
                data = sock.recv(4096)
                if data:
                    line = data.decode('utf-8', errors='ignore')
                    self._process_log_line(source_name, line, 'syslog')
        except Exception as e:
            logging.error(f"Error monitoring syslog: {e}")
    
    def _monitor_systemd_journal(self, source_name: str):
        """Monitor systemd journal"""
        try:
            import subprocess
            process = subprocess.Popen(
                ['journalctl', '-f', '--no-pager'],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            for line in process.stdout:
                if line and self.is_running:
                    self._process_log_line(source_name, line.rstrip('\n\r'), 'systemd')
        except Exception as e:
            logging.error(f"Error monitoring systemd journal: {e}")
    
    def _process_log_line(self, source_name: str, line: str, format_type: str):
        """Process a single log line"""
        self.stats['total_logs'] += 1
        
        # Parse the log line
        log_entry = self.parser.parse_line(line, format_type)
        
        if log_entry:
            log_entry.source = f"{source_name}:{log_entry.source}"
            self.stats['parsed_logs'] += 1
            
            # Add to queue
            self.log_queue.append(log_entry)
            
            # Call handlers
            for handler in self.log_handlers:
                try:
                    handler(log_entry)
                except Exception as e:
                    logging.error(f"Error in log handler: {e}")
        else:
            self.stats['failed_logs'] += 1
    
    def _process_logs(self):
        """Process logs from the queue"""
        while self.is_running:
            try:
                if self.log_queue:
                    log_entry = self.log_queue.popleft()
                    self._save_log(log_entry)
                else:
                    time.sleep(0.1)
            except Exception as e:
                logging.error(f"Error processing log: {e}")
    
    def _save_log(self, log_entry: LogEntry):
        """Save log entry to storage"""
        try:
            if self.storage_backend == 'sqlite':
                self._save_to_sqlite(log_entry)
            elif self.storage_backend == 'file':
                self._save_to_file(log_entry)
            
            self.stats['last_update'] = time.time()
        except Exception as e:
            logging.error(f"Error saving log: {e}")
    
    def _save_to_sqlite(self, log_entry: LogEntry):
        """Save to SQLite database"""
        conn = sqlite3.connect('monitoring_logs.db')
        cursor = conn.cursor()
        
        # Create table if it doesn't exist
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL,
                level TEXT,
                source TEXT,
                message TEXT,
                details TEXT,
                tags TEXT,
                user_id TEXT,
                session_id TEXT,
                trace_id TEXT,
                correlation_id TEXT,
                hash TEXT
            )
        ''')
        
        # Calculate log hash for deduplication
        log_string = f"{log_entry.timestamp}:{log_entry.level}:{log_entry.source}:{log_entry.message}"
        log_hash = hashlib.md5(log_string.encode()).hexdigest()
        
        cursor.execute('''
            INSERT OR IGNORE INTO logs (
                timestamp, level, source, message, details, tags, 
                user_id, session_id, trace_id, correlation_id, hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            log_entry.timestamp,
            log_entry.level,
            log_entry.source,
            log_entry.message,
            json.dumps(log_entry.details) if log_entry.details else None,
            json.dumps(log_entry.tags) if log_entry.tags else None,
            log_entry.user_id,
            log_entry.session_id,
            log_entry.trace_id,
            log_entry.correlation_id,
            log_hash
        ))
        
        conn.commit()
        conn.close()
    
    def _save_to_file(self, log_entry: LogEntry):
        """Save to rotating log files"""
        date_str = datetime.fromtimestamp(log_entry.timestamp).strftime('%Y-%m-%d')
        log_file = f"logs_{date_str}.log"
        
        # Compress old files
        self._compress_old_logs(date_str)
        
        with open(log_file, 'a') as f:
            f.write(json.dumps(asdict(log_entry)) + '\n')
    
    def _compress_old_logs(self, current_date_str: str):
        """Compress old log files"""
        try:
            for log_file in Path('.').glob('logs_*.log'):
                file_date = log_file.stem.split('_')[1]
                if file_date < current_date_str:
                    with open(log_file, 'rb') as f_in:
                        with gzip.open(f"{log_file}.gz", 'wb') as f_out:
                            shutil.copyfileobj(f_in, f_out)
                    log_file.unlink()
        except Exception as e:
            logging.error(f"Error compressing logs: {e}")
    
    def _cleanup_old_logs(self):
        """Clean up old log entries"""
        while self.is_running:
            try:
                cutoff_time = time.time() - (self.retention_days * 24 * 3600)
                
                if self.storage_backend == 'sqlite':
                    self._cleanup_sqlite_logs(cutoff_time)
                
                time.sleep(3600)  # Run cleanup every hour
            except Exception as e:
                logging.error(f"Error in log cleanup: {e}")
    
    def _cleanup_sqlite_logs(self, cutoff_time: float):
        """Clean up old SQLite log entries"""
        conn = sqlite3.connect('monitoring_logs.db')
        cursor = conn.cursor()
        
        cursor.execute('DELETE FROM logs WHERE timestamp < ?', (cutoff_time,))
        
        deleted_count = cursor.rowcount
        conn.commit()
        conn.close()
        
        if deleted_count > 0:
            logging.info(f"Cleaned up {deleted_count} old log entries")
    
    def query_logs(self, log_filter: LogFilter, limit: int = 1000) -> List[LogEntry]:
        """Query logs based on filter criteria"""
        try:
            if self.storage_backend == 'sqlite':
                return self._query_sqlite_logs(log_filter, limit)
            else:
                return self._query_file_logs(log_filter, limit)
        except Exception as e:
            logging.error(f"Error querying logs: {e}")
            return []
    
    def _query_sqlite_logs(self, log_filter: LogFilter, limit: int) -> List[LogEntry]:
        """Query SQLite logs"""
        conn = sqlite3.connect('monitoring_logs.db')
        cursor = conn.cursor()
        
        # Build query
        where_clauses = []
        params = []
        
        if log_filter.levels:
            placeholders = ','.join('?' * len(log_filter.levels))
            where_clauses.append(f"level IN ({placeholders})")
            params.extend(log_filter.levels)
        
        if log_filter.sources:
            placeholders = ','.join('?' * len(log_filter.sources))
            where_clauses.append(f"source IN ({placeholders})")
            params.extend(log_filter.sources)
        
        if log_filter.start_time:
            where_clauses.append("timestamp >= ?")
            params.append(log_filter.start_time)
        
        if log_filter.end_time:
            where_clauses.append("timestamp <= ?")
            params.append(log_filter.end_time)
        
        if log_filter.search_text:
            where_clauses.append("message LIKE ?")
            params.append(f"%{log_filter.search_text}%")
        
        if log_filter.user_id:
            where_clauses.append("user_id = ?")
            params.append(log_filter.user_id)
        
        # Build the query
        query = "SELECT * FROM logs"
        if where_clauses:
            query += " WHERE " + " AND ".join(where_clauses)
        query += " ORDER BY timestamp DESC LIMIT ?"
        params.append(limit)
        
        cursor.execute(query, params)
        rows = cursor.fetchall()
        
        conn.close()
        
        # Convert to LogEntry objects
        logs = []
        for row in rows:
            logs.append(LogEntry(
                timestamp=row[1],
                level=row[2],
                source=row[3],
                message=row[4],
                details=json.loads(row[5]) if row[5] else None,
                tags=json.loads(row[6]) if row[6] else None,
                user_id=row[7],
                session_id=row[8],
                trace_id=row[9],
                correlation_id=row[10]
            ))
        
        return logs
    
    def _query_file_logs(self, log_filter: LogFilter, limit: int) -> List[LogEntry]:
        """Query file-based logs"""
        # This would need to be implemented for file-based storage
        # For now, return empty list
        return []
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get aggregation statistics"""
        return {
            **self.stats,
            'sources': len(self.sources),
            'queue_size': len(self.log_queue),
            'retention_days': self.retention_days
        }
    
    def search_logs(self, query: str, hours: int = 24) -> List[LogEntry]:
        """Search logs for text query"""
        log_filter = LogFilter(
            search_text=query,
            start_time=time.time() - (hours * 3600)
        )
        return self.query_logs(log_filter)

class LogAnalyzer:
    """Analyzes log patterns and generates insights"""
    
    def __init__(self, log_aggregator: LogAggregator):
        self.aggregator = log_aggregator
        self.patterns = {}
        self.anomaly_thresholds = {}
        self.insights = deque(maxlen=1000)
        
    def analyze_log_patterns(self) -> Dict[str, Any]:
        """Analyze log patterns and trends"""
        try:
            # Get recent logs
            log_filter = LogFilter(start_time=time.time() - 86400)  # Last 24 hours
            logs = self.aggregator.query_logs(log_filter, limit=10000)
            
            analysis = {
                'timestamp': time.time(),
                'total_logs': len(logs),
                'level_distribution': self._analyze_level_distribution(logs),
                'source_distribution': self._analyze_source_distribution(logs),
                'error_patterns': self._analyze_error_patterns(logs),
                'performance_indicators': self._analyze_performance_indicators(logs),
                'trends': self._analyze_trends(logs)
            }
            
            return analysis
        except Exception as e:
            logging.error(f"Error analyzing log patterns: {e}")
            return {}
    
    def _analyze_level_distribution(self, logs: List[LogEntry]) -> Dict[str, int]:
        """Analyze log level distribution"""
        distribution = defaultdict(int)
        for log in logs:
            distribution[log.level] += 1
        return dict(distribution)
    
    def _analyze_source_distribution(self, logs: List[LogEntry]) -> Dict[str, int]:
        """Analyze log source distribution"""
        distribution = defaultdict(int)
        for log in logs:
            distribution[log.source] += 1
        return dict(distribution)
    
    def _analyze_error_patterns(self, logs: List[LogEntry]) -> Dict[str, Any]:
        """Analyze error patterns"""
        error_logs = [log for log in logs if log.level in ['ERROR', 'CRITICAL']]
        
        patterns = defaultdict(list)
        for log in error_logs:
            # Extract common error patterns
            if 'connection' in log.message.lower():
                patterns['connection_errors'].append(log)
            elif 'timeout' in log.message.lower():
                patterns['timeout_errors'].append(log)
            elif 'permission' in log.message.lower():
                patterns['permission_errors'].append(log)
            else:
                patterns['other_errors'].append(log)
        
        return {
            'total_errors': len(error_logs),
            'error_rate': len(error_logs) / len(logs) if logs else 0,
            'patterns': {k: len(v) for k, v in patterns.items()},
            'recent_errors': error_logs[-10:]  # Last 10 errors
        }
    
    def _analyze_performance_indicators(self, logs: List[LogEntry]) -> Dict[str, Any]:
        """Analyze performance indicators from logs"""
        performance_logs = []
        
        # Look for performance-related logs
        keywords = ['slow', 'latency', 'timeout', 'performance', 'response_time']
        
        for log in logs:
            if any(keyword in log.message.lower() for keyword in keywords):
                performance_logs.append(log)
        
        return {
            'performance_issues': len(performance_logs),
            'common_issues': self._find_common_issues(performance_logs),
            'performance_score': self._calculate_performance_score(logs)
        }
    
    def _analyze_trends(self, logs: List[LogEntry]) -> Dict[str, Any]:
        """Analyze trends over time"""
        # Group logs by hour
        hourly_stats = defaultdict(lambda: {'total': 0, 'errors': 0, 'warnings': 0})
        
        for log in logs:
            hour = datetime.fromtimestamp(log.timestamp).strftime('%Y-%m-%d-%H')
            hourly_stats[hour]['total'] += 1
            
            if log.level == 'ERROR':
                hourly_stats[hour]['errors'] += 1
            elif log.level == 'WARNING':
                hourly_stats[hour]['warnings'] += 1
        
        # Calculate trends
        recent_hours = sorted(hourly_stats.keys())[-24:]  # Last 24 hours
        
        return {
            'hourly_activity': {hour: hourly_stats[hour] for hour in recent_hours},
            'peak_hour': max(hourly_stats.items(), key=lambda x: x[1]['total'])[0] if hourly_stats else None,
            'error_trend': self._calculate_error_trend(hourly_stats)
        }
    
    def _find_common_issues(self, performance_logs: List[LogEntry]) -> List[Dict[str, Any]]:
        """Find common performance issues"""
        issue_counts = defaultdict(int)
        
        for log in performance_logs:
            issue_counts[log.message[:100]] += 1  # First 100 chars as issue identifier
        
        # Return top issues
        return [
            {'issue': issue, 'count': count}
            for issue, count in sorted(issue_counts.items(), key=lambda x: x[1], reverse=True)[:5]
        ]
    
    def _calculate_performance_score(self, logs: List[LogEntry]) -> int:
        """Calculate overall performance score (0-100)"""
        if not logs:
            return 100
        
        error_rate = len([log for log in logs if log.level in ['ERROR', 'CRITICAL']]) / len(logs)
        warning_rate = len([log for log in logs if log.level == 'WARNING']) / len(logs)
        
        # Score calculation: start at 100, subtract penalty for errors and warnings
        score = 100 - (error_rate * 100 * 2) - (warning_rate * 100 * 0.5)
        return max(0, min(100, int(score)))
    
    def _calculate_error_trend(self, hourly_stats: Dict) -> str:
        """Calculate error trend direction"""
        if len(hourly_stats) < 2:
            return 'insufficient_data'
        
        recent_hours = sorted(hourly_stats.keys())[-6:]  # Last 6 hours
        previous_hours = sorted(hourly_stats.keys())[-12:-6]  # Previous 6 hours
        
        recent_errors = sum(hourly_stats[hour]['errors'] for hour in recent_hours)
        previous_errors = sum(hourly_stats[hour]['errors'] for hour in previous_hours)
        
        if recent_errors > previous_errors * 1.2:
            return 'increasing'
        elif recent_errors < previous_errors * 0.8:
            return 'decreasing'
        else:
            return 'stable'

def main():
    """Main function to run log aggregation and analysis"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Load configuration
    config_file = Path(__file__).parent.parent / 'config' / 'monitoring.yaml'
    
    if config_file.exists():
        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)
    else:
        config = {
            'log_aggregation': {
                'enabled': True,
                'retention': '30d',
                'storage_backend': 'sqlite'
            }
        }
    
    # Create and start log aggregator
    aggregator = LogAggregator(config.get('log_aggregation', {}))
    
    # Add default log sources
    log_sources = [
        ('system', {'type': 'file', 'path': '/var/log/syslog', 'format': 'syslog'}),
        ('auth', {'type': 'file', 'path': '/var/log/auth.log', 'format': 'syslog'}),
        ('application', {'type': 'directory', 'path': '/var/log/multios', 'format': 'multios'})
    ]
    
    for source_name, source_config in log_sources:
        if Path(source_config['path']).exists():
            aggregator.add_source(source_name, source_config)
    
    # Create analyzer
    analyzer = LogAnalyzer(aggregator)
    
    # Add custom log handler for analysis
    def log_analysis_handler(log_entry: LogEntry):
        # Store recent logs for analysis
        pass
    
    aggregator.add_log_handler(log_analysis_handler)
    
    try:
        # Start aggregation
        aggregator.start()
        
        # Periodic analysis
        while True:
            time.sleep(3600)  # Analyze every hour
            
            analysis = analyzer.analyze_log_patterns()
            logging.info(f"Log analysis completed: {len(analysis)} metrics")
            
    except KeyboardInterrupt:
        logging.info("Shutting down log aggregation...")
        aggregator.stop()

if __name__ == '__main__':
    main()