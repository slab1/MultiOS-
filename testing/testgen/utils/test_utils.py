"""
Test Generation Framework Utilities
Helper functions and common utilities for test generation
"""

import random
import string
import hashlib
import json
import time
import logging
from typing import Dict, List, Any, Optional, Tuple, Union
from dataclasses import dataclass
from datetime import datetime, timedelta
import uuid

class TestDataGenerator:
    """Utility class for generating test data"""
    
    @staticmethod
    def generate_random_string(length: int, charset: str = None) -> str:
        """Generate random string of specified length"""
        if charset is None:
            charset = string.ascii_letters + string.digits
        
        return ''.join(random.choice(charset) for _ in range(length))
    
    @staticmethod
    def generate_random_bytes(length: int) -> bytes:
        """Generate random bytes"""
        return bytes(random.randint(0, 255) for _ in range(length))
    
    @staticmethod
    def generate_random_int(min_val: int = 0, max_val: int = 100) -> int:
        """Generate random integer within range"""
        return random.randint(min_val, max_val)
    
    @staticmethod
    def generate_random_float(min_val: float = 0.0, max_val: float = 1.0) -> float:
        """Generate random float within range"""
        return random.uniform(min_val, max_val)
    
    @staticmethod
    def generate_random_email() -> str:
        """Generate random email address"""
        domains = ["example.com", "test.com", "demo.org", "sample.net"]
        return f"{TestDataGenerator.generate_random_string(8)}@{random.choice(domains)}"
    
    @staticmethod
    def generate_random_url() -> str:
        """Generate random URL"""
        protocols = ["http", "https"]
        hosts = ["example.com", "test.com", "demo.org"]
        paths = ["api", "data", "users", "files", "config"]
        
        protocol = random.choice(protocols)
        host = random.choice(hosts)
        path = random.choice(paths)
        
        return f"{protocol}://{host}/{path}/{TestDataGenerator.generate_random_string(6)}"
    
    @staticmethod
    def generate_random_path() -> str:
        """Generate random file system path"""
        paths = [
            "/tmp/test.txt",
            "/home/user/documents/file.pdf",
            "/var/log/application.log",
            "/etc/config/settings.ini",
            "/usr/local/bin/script.sh"
        ]
        return random.choice(paths)
    
    @staticmethod
    def generate_boundary_values(data_type: str) -> List[Any]:
        """Generate boundary values for different data types"""
        boundary_sets = {
            "integer": [-2**31, -1, 0, 1, 2**31-1],
            "unsigned_int": [0, 1, 255, 65535, 2**32-1],
            "float": [0.0, -0.0, float('inf'), float('-inf'), float('nan')],
            "string": ["", "\0", "\n", "\r", "\t"],
            "char": [chr(0), chr(127), "A", "z", "~"],
            "pointer": [0, None, 0xFFFFFFFF, -1]
        }
        
        return boundary_sets.get(data_type, [0])
    
    @staticmethod
    def generate_malicious_strings() -> List[str]:
        """Generate potentially malicious strings for security testing"""
        return [
            "../../etc/passwd",
            "'; DROP TABLE users; --",
            "<script>alert('xss')</script>",
            "{{7*7}}",
            "${7*7}",
            "$(whoami)",
            "`whoami`",
            "test' OR '1'='1",
            "../../../../etc/hosts",
            "..\\..\\..\\windows\\system32\\",
            "\x00\x01\x02\x03",
            "A" * 10000,
            "测试字符串",
            "🔥💯🎉",
            "контент",
            "محتوى"
        ]

class TestDataValidator:
    """Utility class for validating test data"""
    
    @staticmethod
    def is_valid_email(email: str) -> bool:
        """Validate email format"""
        import re
        pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        return bool(re.match(pattern, email))
    
    @staticmethod
    def is_valid_url(url: str) -> bool:
        """Validate URL format"""
        from urllib.parse import urlparse
        try:
            result = urlparse(url)
            return all([result.scheme, result.netloc])
        except Exception:
            return False
    
    @staticmethod
    def is_valid_path(path: str) -> bool:
        """Validate file system path"""
        import re
        # Basic path validation - can be enhanced
        if not path or len(path) > 4096:
            return False
        
        # Check for invalid characters
        invalid_chars = ['\0', '*', '?', '"', '<', '>', '|']
        return not any(char in path for char in invalid_chars)
    
    @staticmethod
    def validate_json_structure(data: str) -> bool:
        """Validate JSON structure"""
        try:
            json.loads(data)
            return True
        except (json.JSONDecodeError, TypeError):
            return False
    
    @staticmethod
    def is_safe_filename(filename: str) -> bool:
        """Check if filename is safe"""
        import re
        if not filename:
            return False
        
        # Windows reserved names
        reserved_names = ['CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'COM3', 'COM4', 
                         'COM5', 'COM6', 'COM7', 'COM8', 'COM9', 'LPT1', 'LPT2', 'LPT3',
                         'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9']
        
        name_upper = filename.upper()
        if name_upper in reserved_names:
            return False
        
        # Check for invalid characters
        invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*']
        return not any(char in filename for char in invalid_chars)

class TestMetrics:
    """Utility class for tracking test metrics"""
    
    def __init__(self):
        self.start_time = None
        self.end_time = None
        self.test_count = 0
        self.passed_count = 0
        self.failed_count = 0
        self.error_count = 0
        self.skipped_count = 0
        
    def start_timer(self):
        """Start test execution timer"""
        self.start_time = time.time()
    
    def stop_timer(self):
        """Stop test execution timer"""
        self.end_time = time.time()
    
    def get_duration(self) -> float:
        """Get test execution duration in seconds"""
        if self.start_time and self.end_time:
            return self.end_time - self.start_time
        return 0.0
    
    def increment_test_count(self):
        """Increment test count"""
        self.test_count += 1
    
    def increment_passed(self):
        """Increment passed test count"""
        self.passed_count += 1
        self.increment_test_count()
    
    def increment_failed(self):
        """Increment failed test count"""
        self.failed_count += 1
        self.increment_test_count()
    
    def increment_error(self):
        """Increment error test count"""
        self.error_count += 1
        self.increment_test_count()
    
    def increment_skipped(self):
        """Increment skipped test count"""
        self.skipped_count += 1
        self.increment_test_count()
    
    def get_pass_rate(self) -> float:
        """Get pass rate percentage"""
        if self.test_count == 0:
            return 0.0
        return (self.passed_count / self.test_count) * 100
    
    def get_fail_rate(self) -> float:
        """Get failure rate percentage"""
        if self.test_count == 0:
            return 0.0
        return (self.failed_count / self.test_count) * 100
    
    def get_error_rate(self) -> float:
        """Get error rate percentage"""
        if self.test_count == 0:
            return 0.0
        return (self.error_count / self.test_count) * 100
    
    def get_summary(self) -> Dict[str, Any]:
        """Get test execution summary"""
        return {
            "duration_seconds": self.get_duration(),
            "total_tests": self.test_count,
            "passed": self.passed_count,
            "failed": self.failed_count,
            "errors": self.error_count,
            "skipped": self.skipped_count,
            "pass_rate": self.get_pass_rate(),
            "fail_rate": self.get_fail_rate(),
            "error_rate": self.get_error_rate()
        }

class TestReporter:
    """Utility class for generating test reports"""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
    
    def generate_html_report(self, test_results: List[Dict[str, Any]], 
                           output_file: str = "test_report.html") -> str:
        """Generate HTML test report"""
        html_content = """
<!DOCTYPE html>
<html>
<head>
    <title>MultiOS Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .summary { background-color: #e8f5e8; padding: 15px; margin: 10px 0; border-radius: 5px; }
        .test-case { margin: 10px 0; padding: 10px; border: 1px solid #ddd; border-radius: 3px; }
        .passed { background-color: #d4edda; }
        .failed { background-color: #f8d7da; }
        .error { background-color: #fff3cd; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }
        .metric { background-color: #f8f9fa; padding: 10px; border-radius: 3px; text-align: center; }
    </style>
</head>
<body>
    <div class="header">
        <h1>MultiOS Test Execution Report</h1>
        <p>Generated: {}</p>
    </div>
    
    <div class="summary">
        <h2>Summary</h2>
        <div class="metrics">
            <div class="metric">
                <h3>{}</h3>
                <p>Total Tests</p>
            </div>
            <div class="metric">
                <h3>{}</h3>
                <p>Passed</p>
            </div>
            <div class="metric">
                <h3>{}</h3>
                <p>Failed</p>
            </div>
            <div class="metric">
                <h3>{:.1f}%</h3>
                <p>Pass Rate</p>
            </div>
        </div>
    </div>
    
    <h2>Test Cases</h2>
""".format(
            datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            len(test_results),
            len([t for t in test_results if t.get("status") == "pass"]),
            len([t for t in test_results if t.get("status") in ["fail", "error"]]),
            sum(1 for t in test_results if t.get("status") == "pass") / len(test_results) * 100 if test_results else 0
        )
        
        # Add test case details
        for test_case in test_results:
            status = test_case.get("status", "unknown")
            status_class = "passed" if status == "pass" else "failed" if status == "fail" else "error"
            
            html_content += f"""
    <div class="test-case {status_class}">
        <h3>{test_case.get("name", "Unknown Test")}</h3>
        <p><strong>ID:</strong> {test_case.get("id", "N/A")}</p>
        <p><strong>Type:</strong> {test_case.get("type", "N/A")}</p>
        <p><strong>Component:</strong> {test_case.get("component", "N/A")}</p>
        <p><strong>Status:</strong> {status.upper()}</p>
        <p><strong>Priority:</strong> {test_case.get("priority", "N/A")}</p>
        <p><strong>Description:</strong> {test_case.get("description", "N/A")}</p>
    </div>
"""
        
        html_content += """
</body>
</html>
"""
        
        with open(output_file, 'w') as f:
            f.write(html_content)
        
        return output_file
    
    def generate_json_report(self, test_results: List[Dict[str, Any]], 
                           output_file: str = "test_report.json") -> str:
        """Generate JSON test report"""
        report = {
            "report_info": {
                "generated_at": datetime.now().isoformat(),
                "framework": "MultiOS Test Generation Framework",
                "version": "1.0.0"
            },
            "summary": {
                "total_tests": len(test_results),
                "passed": len([t for t in test_results if t.get("status") == "pass"]),
                "failed": len([t for t in test_results if t.get("status") in ["fail", "error"]]),
                "pass_rate": sum(1 for t in test_results if t.get("status") == "pass") / len(test_results) * 100 if test_results else 0
            },
            "test_cases": test_results
        }
        
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        return output_file
    
    def generate_xml_report(self, test_results: List[Dict[str, Any]], 
                          output_file: str = "test_report.xml") -> str:
        """Generate JUnit XML test report"""
        xml_content = '<?xml version="1.0" encoding="UTF-8"?>\n'
        xml_content += '<testsuite name="MultiOS Test Suite">\n'
        
        for test_case in test_results:
            status = test_case.get("status", "unknown")
            status_attr = "pass" if status == "pass" else "fail"
            
            xml_content += f'  <testcase classname="{test_case.get("component", "Unknown")}" '
            xml_content += f'id="{test_case.get("id", "")}" '
            xml_content += f'name="{test_case.get("name", "")}">\n'
            
            if status != "pass":
                xml_content += f'    <failure message="{test_case.get("description", "Test failed")}"/>\n'
            
            xml_content += '  </testcase>\n'
        
        xml_content += '</testsuite>'
        
        with open(output_file, 'w') as f:
            f.write(xml_content)
        
        return output_file

class FileUtils:
    """Utility functions for file operations in testing"""
    
    @staticmethod
    def ensure_directory(path: str) -> bool:
        """Ensure directory exists, create if necessary"""
        import os
        try:
            os.makedirs(path, exist_ok=True)
            return True
        except Exception as e:
            logging.error(f"Failed to create directory {path}: {e}")
            return False
    
    @staticmethod
    def clean_test_files(pattern: str = "/tmp/test_*", max_age_hours: int = 24) -> int:
        """Clean up old test files"""
        import os
        import glob
        import time
        
        count = 0
        current_time = time.time()
        max_age_seconds = max_age_hours * 3600
        
        for file_path in glob.glob(pattern):
            try:
                if current_time - os.path.getmtime(file_path) > max_age_seconds:
                    os.remove(file_path)
                    count += 1
            except Exception as e:
                logging.warning(f"Failed to remove {file_path}: {e}")
        
        return count
    
    @staticmethod
    def calculate_file_hash(file_path: str) -> str:
        """Calculate hash of file content"""
        hash_md5 = hashlib.md5()
        try:
            with open(file_path, "rb") as f:
                for chunk in iter(lambda: f.read(4096), b""):
                    hash_md5.update(chunk)
            return hash_md5.hexdigest()
        except Exception as e:
            logging.error(f"Failed to calculate hash for {file_path}: {e}")
            return ""

class StringUtils:
    """String utility functions for testing"""
    
    @staticmethod
    def truncate_string(text: str, max_length: int = 100, suffix: str = "...") -> str:
        """Truncate string to maximum length"""
        if len(text) <= max_length:
            return text
        
        return text[:max_length - len(suffix)] + suffix
    
    @staticmethod
    def sanitize_filename(filename: str) -> str:
        """Sanitize filename for cross-platform compatibility"""
        import re
        
        # Remove invalid characters
        invalid_chars = r'[<>:"/\\|?*]'
        sanitized = re.sub(invalid_chars, '_', filename)
        
        # Remove leading/trailing dots and spaces
        sanitized = sanitized.strip('. ')
        
        # Ensure not empty
        if not sanitized:
            sanitized = "unnamed"
        
        # Limit length
        if len(sanitized) > 255:
            name, ext = os.path.splitext(sanitized)
            sanitized = name[:255-len(ext)] + ext
        
        return sanitized
    
    @staticmethod
    def generate_test_id(prefix: str = "test") -> str:
        """Generate unique test ID"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        uuid_short = str(uuid.uuid4())[:8]
        return f"{prefix}_{timestamp}_{uuid_short}"

# Add missing os import
import os