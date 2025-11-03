"""
Utility functions for MultiOS Enterprise Deployment System
"""

import hashlib
import json
import yaml
import subprocess
import ipaddress
import logging
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from pathlib import Path
import uuid
import psutil
import socket
import requests

def generate_system_id() -> str:
    """Generate a unique system identifier"""
    return str(uuid.uuid4())

def calculate_md5(file_path: str) -> str:
    """Calculate MD5 hash of a file"""
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def validate_ip_address(ip: str) -> bool:
    """Validate if string is a valid IP address"""
    try:
        ipaddress.ip_address(ip)
        return True
    except ValueError:
        return False

def validate_mac_address(mac: str) -> bool:
    """Validate if string is a valid MAC address"""
    try:
        # Remove common separators
        mac = mac.replace(':', '').replace('-', '').replace('.', '')
        if len(mac) != 12:
            return False
        int(mac, 16)  # Try to convert to int
        return True
    except ValueError:
        return False

def get_system_info() -> Dict[str, Any]:
    """Get current system information"""
    try:
        return {
            'hostname': socket.gethostname(),
            'ip_address': get_local_ip(),
            'cpu_count': psutil.cpu_count(),
            'memory_total': psutil.virtual_memory().total,
            'disk_total': psutil.disk_usage('/').total,
            'timestamp': datetime.now().isoformat()
        }
    except Exception as e:
        logging.error(f"Failed to get system info: {e}")
        return {}

def get_local_ip() -> str:
    """Get local IP address"""
    try:
        # Connect to a remote address to determine local IP
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
            s.connect(("8.8.8.8", 80))
            return s.getsockname()[0]
    except Exception:
        return "127.0.0.1"

def load_config(config_path: str) -> Dict[str, Any]:
    """Load configuration from JSON or YAML file"""
    try:
        with open(config_path, 'r') as f:
            if config_path.endswith('.yaml') or config_path.endswith('.yml'):
                return yaml.safe_load(f)
            else:
                return json.load(f)
    except Exception as e:
        logging.error(f"Failed to load config from {config_path}: {e}")
        return {}

def save_config(config_path: str, config: Dict[str, Any]) -> bool:
    """Save configuration to JSON or YAML file"""
    try:
        with open(config_path, 'w') as f:
            if config_path.endswith('.yaml') or config_path.endswith('.yml'):
                yaml.dump(config, f, default_flow_style=False, indent=2)
            else:
                json.dump(config, f, indent=2)
        return True
    except Exception as e:
        logging.error(f"Failed to save config to {config_path}: {e}")
        return False

def execute_command(cmd: List[str], timeout: int = 300) -> Tuple[int, str, str]:
    """Execute shell command and return exit code, stdout, stderr"""
    try:
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True, 
            timeout=timeout,
            check=False
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", f"Command timed out after {timeout} seconds"
    except Exception as e:
        return -1, "", str(e)

def format_size(size_bytes: int) -> str:
    """Format bytes to human readable format"""
    if size_bytes == 0:
        return "0B"
    
    size_names = ["B", "KB", "MB", "GB", "TB", "PB"]
    i = 0
    while size_bytes >= 1024 and i < len(size_names) - 1:
        size_bytes /= 1024.0
        i += 1
    
    return f"{size_bytes:.1f}{size_names[i]}"

def format_duration(seconds: int) -> str:
    """Format duration in seconds to human readable format"""
    days = seconds // 86400
    hours = (seconds % 86400) // 3600
    minutes = (seconds % 3600) // 60
    seconds = seconds % 60
    
    if days > 0:
        return f"{days}d {hours}h {minutes}m"
    elif hours > 0:
        return f"{hours}h {minutes}m"
    elif minutes > 0:
        return f"{minutes}m {seconds}s"
    else:
        return f"{seconds}s"

def is_port_open(host: str, port: int) -> bool:
    """Check if a port is open on a host"""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(1)
            result = sock.connect_ex((host, port))
            return result == 0
    except Exception:
        return False

def ping_host(host: str) -> bool:
    """Ping a host to check connectivity"""
    try:
        result = subprocess.run(
            ['ping', '-c', '1', '-W', '3', host],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0
    except Exception:
        return False

def download_file(url: str, dest_path: str, progress_callback=None) -> bool:
    """Download a file with optional progress callback"""
    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()
        
        total_size = int(response.headers.get('content-length', 0))
        downloaded = 0
        
        with open(dest_path, 'wb') as f:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    downloaded += len(chunk)
                    
                    if progress_callback and total_size > 0:
                        progress = (downloaded / total_size) * 100
                        progress_callback(progress)
        
        return True
    except Exception as e:
        logging.error(f"Failed to download {url}: {e}")
        return False

def backup_file(file_path: str, backup_dir: str) -> Optional[str]:
    """Create a backup of a file"""
    try:
        file_path = Path(file_path)
        backup_dir = Path(backup_dir)
        backup_dir.mkdir(parents=True, exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_name = f"{file_path.stem}_{timestamp}{file_path.suffix}"
        backup_path = backup_dir / backup_name
        
        import shutil
        shutil.copy2(file_path, backup_path)
        
        return str(backup_path)
    except Exception as e:
        logging.error(f"Failed to backup {file_path}: {e}")
        return None

def calculate_network_broadcast(network_cidr: str) -> str:
    """Calculate broadcast address for a network"""
    try:
        network = ipaddress.IPv4Network(network_cidr, strict=False)
        return str(network.broadcast_address)
    except Exception:
        return ""

def parse_csv_line(line: str) -> List[str]:
    """Parse a CSV line handling quoted fields"""
    result = []
    current_field = ""
    in_quotes = False
    
    i = 0
    while i < len(line):
        char = line[i]
        
        if char == '"':
            in_quotes = not in_quotes
        elif char == ',' and not in_quotes:
            result.append(current_field.strip())
            current_field = ""
        else:
            current_field += char
        
        i += 1
    
    result.append(current_field.strip())
    return result

def validate_email(email: str) -> bool:
    """Validate email address format"""
    import re
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return re.match(pattern, email) is not None

def create_directory_structure(base_path: str, structure: Dict[str, Any]) -> bool:
    """Create directory structure from dictionary"""
    try:
        base_path = Path(base_path)
        base_path.mkdir(parents=True, exist_ok=True)
        
        for name, content in structure.items():
            path = base_path / name
            
            if isinstance(content, dict):
                path.mkdir(exist_ok=True)
                create_directory_structure(str(path), content)
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                if isinstance(content, str) and content:
                    path.write_text(content)
        
        return True
    except Exception as e:
        logging.error(f"Failed to create directory structure: {e}")
        return False

def set_logging_level(level: str) -> None:
    """Set logging level"""
    numeric_level = getattr(logging, level.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError(f'Invalid log level: {level}')
    
    logging.basicConfig(
        level=numeric_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler('/var/log/multios-enterprise.log'),
            logging.StreamHandler()
        ]
    )
