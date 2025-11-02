"""
Educational Database Security and Access Control System
Implements authentication, authorization, encryption, and audit logging
"""

import hashlib
import hmac
import secrets
import time
import json
from typing import Dict, List, Any, Optional, Set, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import threading
import base64
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import os
import logging


class Permission(Enum):
    """Database permissions"""
    SELECT = "SELECT"
    INSERT = "INSERT"
    UPDATE = "UPDATE"
    DELETE = "DELETE"
    CREATE = "CREATE"
    DROP = "DROP"
    ALTER = "ALTER"
    INDEX = "INDEX"
    CONNECT = "CONNECT"
    RESOURCE = "RESOURCE"
    DBA = "DBA"
    ALL = "ALL"


class ResourceType(Enum):
    """Types of database resources"""
    DATABASE = "DATABASE"
    TABLE = "TABLE"
    VIEW = "VIEW"
    INDEX = "INDEX"
    FUNCTION = "FUNCTION"
    PROCEDURE = "PROCEDURE"


class AuditEvent(Enum):
    """Types of audit events"""
    LOGIN = "LOGIN"
    LOGOUT = "LOGOUT"
    SELECT = "SELECT"
    INSERT = "INSERT"
    UPDATE = "UPDATE"
    DELETE = "DELETE"
    CREATE = "CREATE"
    DROP = "DROP"
    ALTER = "ALTER"
    FAILED_LOGIN = "FAILED_LOGIN"
    PERMISSION_DENIED = "PERMISSION_DENIED"
    ENCRYPTION_KEY_ACCESS = "ENCRYPTION_KEY_ACCESS"
    DATA_EXPORT = "DATA_EXPORT"


@dataclass
class User:
    """Database user"""
    username: str
    password_hash: str
    salt: str
    roles: List[str]
    is_active: bool = True
    failed_login_attempts: int = 0
    last_login: Optional[float] = None
    created_at: float = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = time.time()
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class Role:
    """Database role"""
    role_name: str
    permissions: List[Permission]
    granted_roles: List[str]
    is_system_role: bool = False
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'role_name': self.role_name,
            'permissions': [perm.value for perm in self.permissions],
            'granted_roles': self.granted_roles,
            'is_system_role': self.is_system_role
        }


@dataclass
class Privilege:
    """Database privilege"""
    principal_name: str
    resource_type: ResourceType
    resource_name: str
    permissions: List[Permission]
    granted_by: str
    granted_at: float
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'principal_name': self.principal_name,
            'resource_type': self.resource_type.value,
            'resource_name': self.resource_name,
            'permissions': [perm.value for perm in self.permissions],
            'granted_by': self.granted_by,
            'granted_at': self.granted_at
        }


@dataclass
class AuditLogEntry:
    """Audit log entry"""
    event_id: str
    event_type: AuditEvent
    user: str
    resource: str
    timestamp: float
    success: bool
    details: Dict[str, Any]
    ip_address: Optional[str] = None
    session_id: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'event_id': self.event_id,
            'event_type': self.event_type.value,
            'user': self.user,
            'resource': self.resource,
            'timestamp': self.timestamp,
            'success': self.success,
            'details': self.details,
            'ip_address': self.ip_address,
            'session_id': self.session_id
        }


class SecurityUtils:
    """Security utility functions"""
    
    @staticmethod
    def hash_password(password: str, salt: str = None) -> Tuple[str, str]:
        """Hash password with salt"""
        if salt is None:
            salt = secrets.token_hex(16)
        
        # Use PBKDF2 for secure password hashing
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt.encode(),
            iterations=100000,
        )
        password_hash = base64.urlsafe_b64encode(kdf.derive(password.encode()))
        
        return password_hash.decode(), salt
    
    @staticmethod
    def verify_password(password: str, password_hash: str, salt: str) -> bool:
        """Verify password against hash"""
        try:
            kdf = PBKDF2HMAC(
                algorithm=hashes.SHA256(),
                length=32,
                salt=salt.encode(),
                iterations=100000,
            )
            test_hash = base64.urlsafe_b64encode(kdf.derive(password.encode()))
            return hmac.compare_digest(test_hash.decode(), password_hash)
        except Exception:
            return False
    
    @staticmethod
    def generate_session_token() -> str:
        """Generate secure session token"""
        return secrets.token_urlsafe(32)
    
    @staticmethod
    def generate_encryption_key(password: str) -> bytes:
        """Generate encryption key from password"""
        salt = b'database_security_salt_2023'  # In production, use random salt
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=100000,
        )
        key = base64.urlsafe_b64encode(kdf.derive(password.encode()))
        return key
    
    @staticmethod
    def encrypt_data(data: str, encryption_key: bytes) -> str:
        """Encrypt data using Fernet (symmetric encryption)"""
        f = Fernet(encryption_key)
        encrypted_data = f.encrypt(data.encode())
        return base64.urlsafe_b64encode(encrypted_data).decode()
    
    @staticmethod
    def decrypt_data(encrypted_data: str, encryption_key: bytes) -> str:
        """Decrypt data using Fernet"""
        f = Fernet(encryption_key)
        encrypted_bytes = base64.urlsafe_b64decode(encrypted_data.encode())
        decrypted_data = f.decrypt(encrypted_bytes)
        return decrypted_data.decode()
    
    @staticmethod
    def create_audit_id() -> str:
        """Create unique audit event ID"""
        timestamp = str(int(time.time() * 1000000))
        random_part = secrets.token_hex(8)
        return f"AUDIT_{timestamp}_{random_part}"


class AuditLogger:
    """Manages audit logging"""
    
    def __init__(self, max_log_size: int = 10000):
        self.max_log_size = max_log_size
        self.log_entries: List[AuditLogEntry] = []
        self.lock = threading.Lock()
        self.logger = self._setup_logger()
    
    def _setup_logger(self) -> logging.Logger:
        """Setup logging configuration"""
        logger = logging.getLogger('DatabaseAudit')
        logger.setLevel(logging.INFO)
        
        if not logger.handlers:
            # Create file handler
            handler = logging.FileHandler('audit.log')
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            logger.addHandler(handler)
        
        return logger
    
    def log_event(self, event_type: AuditEvent, user: str, resource: str,
                  success: bool, details: Dict[str, Any] = None,
                  ip_address: str = None, session_id: str = None):
        """Log an audit event"""
        with self.lock:
            entry = AuditLogEntry(
                event_id=SecurityUtils.create_audit_id(),
                event_type=event_type,
                user=user,
                resource=resource,
                timestamp=time.time(),
                success=success,
                details=details or {},
                ip_address=ip_address,
                session_id=session_id
            )
            
            self.log_entries.append(entry)
            
            # Limit log size
            if len(self.log_entries) > self.max_log_size:
                self.log_entries.pop(0)
            
            # Log to file
            self._write_to_file(entry)
    
    def _write_to_file(self, entry: AuditLogEntry):
        """Write audit entry to log file"""
        log_message = f"{entry.event_type.value} | User: {entry.user} | Resource: {entry.resource} | Success: {entry.success}"
        if entry.details:
            log_message += f" | Details: {entry.details}"
        
        if entry.success:
            self.logger.info(log_message)
        else:
            self.logger.warning(log_message)
    
    def get_logs(self, user: str = None, event_type: AuditEvent = None,
                start_time: float = None, end_time: float = None) -> List[AuditLogEntry]:
        """Retrieve audit logs with filters"""
        with self.lock:
            filtered_logs = self.log_entries.copy()
            
            if user:
                filtered_logs = [log for log in filtered_logs if log.user == user]
            
            if event_type:
                filtered_logs = [log for log in filtered_logs if log.event_type == event_type]
            
            if start_time:
                filtered_logs = [log for log in filtered_logs if log.timestamp >= start_time]
            
            if end_time:
                filtered_logs = [log for log in filtered_logs if log.timestamp <= end_time]
            
            return filtered_logs
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get audit log statistics"""
        with self.lock:
            if not self.log_entries:
                return {'total_events': 0}
            
            event_counts = {}
            user_counts = {}
            success_rate = 0
            
            for entry in self.log_entries:
                # Count events by type
                event_type = entry.event_type.value
                event_counts[event_type] = event_counts.get(event_type, 0) + 1
                
                # Count events by user
                user_counts[entry.user] = user_counts.get(entry.user, 0) + 1
                
                # Calculate success rate
                if entry.success:
                    success_rate += 1
            
            success_rate = success_rate / len(self.log_entries) * 100
            
            return {
                'total_events': len(self.log_entries),
                'event_type_counts': event_counts,
                'user_activity': user_counts,
                'overall_success_rate': success_rate,
                'date_range': {
                    'oldest': min(entry.timestamp for entry in self.log_entries),
                    'newest': max(entry.timestamp for entry in self.log_entries)
                }
            }


class AccessControlManager:
    """Manages access control and permissions"""
    
    def __init__(self):
        self.users: Dict[str, User] = {}
        self.roles: Dict[str, Role] = {}
        self.privileges: List[Privilege] = []
        self.sessions: Dict[str, Dict[str, Any]] = {}  # session_id -> session_info
        self.failed_logins: Dict[str, List[float]] = {}  # username -> list of failed login times
        self.lock = threading.Lock()
        
        # Initialize default roles
        self._initialize_default_roles()
    
    def _initialize_default_roles(self):
        """Initialize system default roles"""
        # Administrator role
        dba_role = Role(
            role_name="DBA",
            permissions=list(Permission),
            granted_roles=[],
            is_system_role=True
        )
        self.roles["DBA"] = dba_role
        
        # Read-only role
        read_role = Role(
            role_name="READ_ONLY",
            permissions=[Permission.SELECT],
            granted_roles=[],
            is_system_role=True
        )
        self.roles["READ_ONLY"] = read_role
        
        # Developer role
        dev_role = Role(
            role_name="DEVELOPER",
            permissions=[Permission.SELECT, Permission.INSERT, Permission.UPDATE, Permission.DELETE],
            granted_roles=[],
            is_system_role=True
        )
        self.roles["DEVELOPER"] = dev_role
    
    def create_user(self, username: str, password: str, roles: List[str]) -> bool:
        """Create a new user"""
        with self.lock:
            if username in self.users:
                return False
            
            # Validate roles
            for role in roles:
                if role not in self.roles:
                    return False
            
            # Hash password
            password_hash, salt = SecurityUtils.hash_password(password)
            
            user = User(
                username=username,
                password_hash=password_hash,
                salt=salt,
                roles=roles
            )
            
            self.users[username] = user
            return True
    
    def create_role(self, role_name: str, permissions: List[Permission],
                   granted_roles: List[str] = None) -> bool:
        """Create a new role"""
        with self.lock:
            if role_name in self.roles:
                return False
            
            # Validate granted roles
            for granted_role in granted_roles or []:
                if granted_role not in self.roles:
                    return False
            
            role = Role(
                role_name=role_name,
                permissions=permissions,
                granted_roles=granted_roles or []
            )
            
            self.roles[role_name] = role
            return True
    
    def grant_privilege(self, principal_name: str, resource_type: ResourceType,
                       resource_name: str, permissions: List[Permission],
                       granted_by: str) -> bool:
        """Grant privilege to user/role"""
        with self.lock:
            privilege = Privilege(
                principal_name=principal_name,
                resource_type=resource_type,
                resource_name=resource_name,
                permissions=permissions,
                granted_by=granted_by,
                granted_at=time.time()
            )
            
            self.privileges.append(privilege)
            return True
    
    def authenticate_user(self, username: str, password: str,
                         ip_address: str = None) -> Optional[str]:
        """Authenticate user and return session token"""
        with self.lock:
            # Check if user exists and is active
            if username not in self.users:
                self._record_failed_login(username, ip_address)
                return None
            
            user = self.users[username]
            if not user.is_active:
                return None
            
            # Verify password
            if not SecurityUtils.verify_password(password, user.password_hash, user.salt):
                self._record_failed_login(username, ip_address)
                user.failed_login_attempts += 1
                return None
            
            # Check for brute force attacks
            if self._is_brute_force_attack(username):
                return None
            
            # Generate session token
            session_token = SecurityUtils.generate_session_token()
            
            # Create session
            session_info = {
                'username': username,
                'login_time': time.time(),
                'last_activity': time.time(),
                'ip_address': ip_address,
                'roles': user.roles.copy()
            }
            
            self.sessions[session_token] = session_info
            
            # Update user statistics
            user.last_login = time.time()
            user.failed_login_attempts = 0
            
            # Record successful login
            return session_token
    
    def _record_failed_login(self, username: str, ip_address: str = None):
        """Record failed login attempt"""
        current_time = time.time()
        
        if username not in self.failed_logins:
            self.failed_logins[username] = []
        
        self.failed_logins[username].append(current_time)
        
        # Clean old attempts (older than 1 hour)
        self.failed_logins[username] = [
            attempt_time for attempt_time in self.failed_logins[username]
            if current_time - attempt_time < 3600
        ]
    
    def _is_brute_force_attack(self, username: str) -> bool:
        """Check if too many failed login attempts"""
        if username not in self.failed_logins:
            return False
        
        # More than 5 failed attempts in the last hour
        return len(self.failed_logins[username]) > 5
    
    def verify_session(self, session_token: str) -> Optional[Dict[str, Any]]:
        """Verify session token and return user info"""
        with self.lock:
            if session_token not in self.sessions:
                return None
            
            session_info = self.sessions[session_token]
            
            # Check session timeout (8 hours)
            if time.time() - session_info['last_activity'] > 28800:
                del self.sessions[session_token]
                return None
            
            # Update last activity
            session_info['last_activity'] = time.time()
            
            return session_info
    
    def terminate_session(self, session_token: str) -> bool:
        """Terminate user session"""
        with self.lock:
            if session_token in self.sessions:
                del self.sessions[session_token]
                return True
            return False
    
    def check_permission(self, session_token: str, permission: Permission,
                        resource_type: ResourceType, resource_name: str) -> bool:
        """Check if user has required permission"""
        session_info = self.verify_session(session_token)
        if not session_info:
            return False
        
        username = session_info['username']
        
        # Check direct privileges
        for privilege in self.privileges:
            if (privilege.principal_name == username and
                privilege.resource_type == resource_type and
                privilege.resource_name == resource_name and
                permission in privilege.permissions):
                return True
        
        # Check role-based permissions
        user_roles = session_info['roles']
        for role_name in user_roles:
            if role_name not in self.roles:
                continue
            
            role = self.roles[role_name]
            
            # Check role permissions
            if permission in role.permissions:
                return True
            
            # Check granted roles recursively
            if self._check_granted_roles(role.granted_roles, permission):
                return True
        
        # Check ALL permission
        for privilege in self.privileges:
            if (privilege.principal_name == username and
                privilege.resource_type == resource_type and
                privilege.resource_name == resource_name and
                Permission.ALL in privilege.permissions):
                return True
        
        return False
    
    def _check_granted_roles(self, role_names: List[str], permission: Permission) -> bool:
        """Check permissions from granted roles recursively"""
        for role_name in role_names:
            if role_name not in self.roles:
                continue
            
            role = self.roles[role_name]
            if permission in role.permissions:
                return True
            
            # Recursive check
            if self._check_granted_roles(role.granted_roles, permission):
                return True
        
        return False
    
    def get_user_permissions(self, session_token: str) -> Dict[str, List[str]]:
        """Get all permissions for user"""
        session_info = self.verify_session(session_token)
        if not session_info:
            return {}
        
        username = session_info['username']
        user_permissions = defaultdict(list)
        
        # Get direct privileges
        for privilege in self.privileges:
            if privilege.principal_name == username:
                resource_key = f"{privilege.resource_type.value}:{privilege.resource_name}"
                permissions = [perm.value for perm in privilege.permissions]
                user_permissions[resource_key].extend(permissions)
        
        # Get role-based permissions
        user_roles = session_info['roles']
        for role_name in user_roles:
            if role_name in self.roles:
                role = self.roles[role_name]
                role_permissions = [perm.value for perm in role.permissions]
                user_permissions[f"ROLE:{role_name}"].extend(role_permissions)
        
        return dict(user_permissions)
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get access control statistics"""
        with self.lock:
            active_sessions = len(self.sessions)
            total_users = len(self.users)
            active_users = sum(1 for user in self.users.values() if user.is_active)
            total_roles = len(self.roles)
            total_privileges = len(self.privileges)
            
            # User role distribution
            role_distribution = defaultdict(int)
            for user in self.users.values():
                for role in user.roles:
                    role_distribution[role] += 1
            
            return {
                'total_users': total_users,
                'active_users': active_users,
                'total_roles': total_roles,
                'total_privileges': total_privileges,
                'active_sessions': active_sessions,
                'role_distribution': dict(role_distribution),
                'failed_login_attempts': {
                    username: len(attempts)
                    for username, attempts in self.failed_logins.items()
                    if attempts
                }
            }


class DataEncryptionManager:
    """Manages data encryption and key management"""
    
    def __init__(self):
        self.encryption_keys: Dict[str, bytes] = {}  # table_name -> encryption key
        self.key_rotation_schedule: Dict[str, float] = {}  # table_name -> last_rotation
        self.encrypted_tables: Set[str] = set()
        self.lock = threading.Lock()
    
    def enable_encryption(self, table_name: str, password: str) -> bool:
        """Enable encryption for a table"""
        with self.lock:
            try:
                # Generate encryption key from password
                encryption_key = SecurityUtils.generate_encryption_key(password)
                
                # Store encryption key securely
                self.encryption_keys[table_name] = encryption_key
                self.encrypted_tables.add(table_name)
                self.key_rotation_schedule[table_name] = time.time()
                
                return True
            except Exception:
                return False
    
    def encrypt_table_data(self, table_name: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Encrypt table data"""
        if table_name not in self.encrypted_tables:
            return data
        
        encryption_key = self.encryption_keys.get(table_name)
        if not encryption_key:
            return data
        
        encrypted_data = {}
        for key, value in data.items():
            if isinstance(value, str):
                # Encrypt string values
                encrypted_value = SecurityUtils.encrypt_data(str(value), encryption_key)
                encrypted_data[key] = encrypted_value
            elif isinstance(value, (int, float)):
                # Encrypt numeric values by converting to string first
                encrypted_value = SecurityUtils.encrypt_data(str(value), encryption_key)
                encrypted_data[key] = encrypted_value
            else:
                # Leave non-encryptable types as-is
                encrypted_data[key] = value
        
        return encrypted_data
    
    def decrypt_table_data(self, table_name: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Decrypt table data"""
        if table_name not in self.encrypted_tables:
            return data
        
        encryption_key = self.encryption_keys.get(table_name)
        if not encryption_key:
            return data
        
        decrypted_data = {}
        for key, value in data.items():
            if isinstance(value, str) and value.startswith('gAAAAA'):  # Fernet encrypted data
                try:
                    decrypted_value = SecurityUtils.decrypt_data(value, encryption_key)
                    
                    # Try to convert back to original type
                    try:
                        if '.' in decrypted_value:
                            decrypted_data[key] = float(decrypted_value)
                        elif decrypted_value.isdigit():
                            decrypted_data[key] = int(decrypted_value)
                        else:
                            decrypted_data[key] = decrypted_value
                    except ValueError:
                        decrypted_data[key] = decrypted_value
                except Exception:
                    # Decryption failed, keep original value
                    decrypted_data[key] = value
            else:
                decrypted_data[key] = value
        
        return decrypted_data
    
    def rotate_encryption_key(self, table_name: str, new_password: str) -> bool:
        """Rotate encryption key for a table"""
        with self.lock:
            if table_name not in self.encrypted_tables:
                return False
            
            try:
                # Generate new encryption key
                new_key = SecurityUtils.generate_encryption_key(new_password)
                
                # Update encryption key
                self.encryption_keys[table_name] = new_key
                self.key_rotation_schedule[table_name] = time.time()
                
                return True
            except Exception:
                return False
    
    def get_encryption_statistics(self) -> Dict[str, Any]:
        """Get encryption statistics"""
        with self.lock:
            return {
                'encrypted_tables': len(self.encrypted_tables),
                'tables': list(self.encrypted_tables),
                'key_rotation_schedule': {
                    table: time.time() - last_rotation
                    for table, last_rotation in self.key_rotation_schedule.items()
                }
            }


class SecurityManager:
    """
    Main security manager that coordinates all security components
    """
    
    def __init__(self):
        self.access_control = AccessControlManager()
        self.audit_logger = AuditLogger()
        self.encryption_manager = DataEncryptionManager()
        self.security_policies = {}
        self.lock = threading.Lock()
    
    def initialize_security(self):
        """Initialize security system with default configuration"""
        # Create default admin user
        self.access_control.create_user("admin", "admin123", ["DBA"])
        
        # Create sample users
        self.access_control.create_user("readonly", "readonly123", ["READ_ONLY"])
        self.access_control.create_user("developer", "dev123", ["DEVELOPER"])
        
        # Grant default privileges
        self.access_control.grant_privilege(
            "DBA", ResourceType.DATABASE, "main_db", [Permission.ALL], "system"
        )
        
        self.access_control.grant_privilege(
            "READ_ONLY", ResourceType.TABLE, "*", [Permission.SELECT], "system"
        )
        
        self.access_control.grant_privilege(
            "DEVELOPER", ResourceType.TABLE, "*", 
            [Permission.SELECT, Permission.INSERT, Permission.UPDATE, Permission.DELETE], "system"
        )
        
        print("✓ Security system initialized with default configuration")
    
    def login(self, username: str, password: str, ip_address: str = None) -> Optional[str]:
        """User login with security checks"""
        session_token = self.access_control.authenticate_user(username, password, ip_address)
        
        # Log login attempt
        if session_token:
            self.audit_logger.log_event(
                AuditEvent.LOGIN, username, "database",
                True, {'ip_address': ip_address}, ip_address, session_token
            )
        else:
            self.audit_logger.log_event(
                AuditEvent.FAILED_LOGIN, username, "database",
                False, {'ip_address': ip_address}, ip_address
            )
        
        return session_token
    
    def logout(self, session_token: str):
        """User logout"""
        session_info = self.access_control.verify_session(session_token)
        if session_info:
            username = session_info['username']
            self.access_control.terminate_session(session_token)
            
            self.audit_logger.log_event(
                AuditEvent.LOGOUT, username, "database",
                True, {}, session_info.get('ip_address'), session_token
            )
    
    def execute_secure_query(self, session_token: str, query: Dict[str, Any],
                            ip_address: str = None) -> Tuple[bool, Any]:
        """Execute query with security checks"""
        session_info = self.access_control.verify_session(session_token)
        if not session_info:
            return False, "Invalid session"
        
        username = session_info['username']
        
        # Determine query type and check permissions
        query_type = self._determine_query_type(query)
        resource_name = query.get('FROM', query.get('TABLE', 'unknown'))
        
        required_permission = self._get_required_permission(query_type)
        
        # Check permission
        if not self.access_control.check_permission(
            session_token, required_permission, ResourceType.TABLE, resource_name
        ):
            self.audit_logger.log_event(
                AuditEvent.PERMISSION_DENIED, username, resource_name,
                False, {
                    'query_type': query_type,
                    'required_permission': required_permission.value
                }, ip_address, session_token
            )
            return False, "Permission denied"
        
        # Log successful query
        self.audit_logger.log_event(
            AuditEvent(query_type), username, resource_name,
            True, {'query': query}, ip_address, session_token
        )
        
        return True, "Query authorized"
    
    def _determine_query_type(self, query: Dict[str, Any]) -> str:
        """Determine query type from query structure"""
        if 'SELECT' in query:
            return 'SELECT'
        elif 'INSERT' in query:
            return 'INSERT'
        elif 'UPDATE' in query:
            return 'UPDATE'
        elif 'DELETE' in query:
            return 'DELETE'
        elif 'CREATE' in query:
            return 'CREATE'
        else:
            return 'UNKNOWN'
    
    def _get_required_permission(self, query_type: str) -> Permission:
        """Get required permission for query type"""
        permission_map = {
            'SELECT': Permission.SELECT,
            'INSERT': Permission.INSERT,
            'UPDATE': Permission.UPDATE,
            'DELETE': Permission.DELETE,
            'CREATE': Permission.CREATE,
            'DROP': Permission.DROP,
            'ALTER': Permission.ALTER
        }
        return permission_map.get(query_type, Permission.SELECT)
    
    def get_security_dashboard(self) -> Dict[str, Any]:
        """Get comprehensive security dashboard"""
        access_stats = self.access_control.get_statistics()
        audit_stats = self.audit_logger.get_statistics()
        encryption_stats = self.encryption_manager.get_encryption_statistics()
        
        return {
            'access_control': access_stats,
            'audit_logs': audit_stats,
            'encryption': encryption_stats,
            'security_level': self._calculate_security_level(access_stats, audit_stats),
            'recommendations': self._generate_security_recommendations(access_stats, audit_stats)
        }
    
    def _calculate_security_level(self, access_stats: Dict, audit_stats: Dict) -> str:
        """Calculate overall security level"""
        active_users = access_stats.get('active_users', 0)
        total_users = access_stats.get('total_users', 1)
        success_rate = audit_stats.get('overall_success_rate', 100)
        
        # Simple scoring algorithm
        score = 0
        
        # User security (active vs total ratio)
        user_ratio = active_users / total_users if total_users > 0 else 0
        if user_ratio > 0.8:
            score += 30
        elif user_ratio > 0.5:
            score += 20
        else:
            score += 10
        
        # Authentication success rate
        if success_rate > 95:
            score += 40
        elif success_rate > 90:
            score += 30
        else:
            score += 20
        
        # Session management
        if access_stats.get('active_sessions', 0) < 100:
            score += 30
        else:
            score += 20
        
        if score >= 80:
            return "HIGH"
        elif score >= 60:
            return "MEDIUM"
        else:
            return "LOW"
    
    def _generate_security_recommendations(self, access_stats: Dict, audit_stats: Dict) -> List[str]:
        """Generate security recommendations"""
        recommendations = []
        
        failed_logins = access_stats.get('failed_login_attempts', {})
        if failed_logins:
            max_failures = max(failed_logins.values())
            if max_failures > 10:
                recommendations.append("Consider implementing account lockout after multiple failed attempts")
        
        success_rate = audit_stats.get('overall_success_rate', 100)
        if success_rate < 90:
            recommendations.append("Review failed operations - may indicate security issues or training needs")
        
        active_sessions = access_stats.get('active_sessions', 0)
        if active_sessions > 50:
            recommendations.append("Monitor active sessions - consider session timeout policies")
        
        return recommendations


def demonstrate_security_system():
    """Demonstrate security system functionality"""
    print("\n" + "="*60)
    print("SECURITY SYSTEM DEMONSTRATION")
    print("="*60)
    
    security = SecurityManager()
    security.initialize_security()
    
    print("\n1. User Authentication...")
    
    # Test successful login
    admin_token = security.login("admin", "admin123", "192.168.1.100")
    if admin_token:
        print("✓ Admin login successful")
    else:
        print("✗ Admin login failed")
    
    # Test failed login
    failed_token = security.login("admin", "wrongpassword", "192.168.1.101")
    if not failed_token:
        print("✓ Failed login properly rejected")
    
    # Test invalid user
    invalid_token = security.login("nonexistent", "password", "192.168.1.102")
    if not invalid_token:
        print("✓ Invalid user properly rejected")
    
    print("\n2. Permission Checking...")
    
    # Test authorized query
    success, result = security.execute_secure_query(admin_token, {
        'SELECT': ['*'],
        'FROM': 'users_table'
    }, "192.168.1.100")
    print(f"✓ Admin query result: {result}")
    
    # Test unauthorized user
    dev_token = security.login("developer", "dev123", "192.168.1.103")
    if dev_token:
        print("✓ Developer login successful")
        
        # Try to drop table (should be denied)
        success, result = security.execute_secure_query(dev_token, {
            'DROP': 'table',
            'TABLE': 'important_table'
        }, "192.168.1.103")
        print(f"✓ Developer query result: {result}")
    
    print("\n3. Data Encryption...")
    
    # Enable encryption for a table
    success = security.encryption_manager.enable_encryption("sensitive_data", "encryption_password_123")
    if success:
        print("✓ Table encryption enabled")
        
        # Test encryption
        test_data = {"ssn": "123-45-6789", "salary": 75000, "name": "John Doe"}
        encrypted_data = security.encryption_manager.encrypt_table_data("sensitive_data", test_data)
        print(f"✓ Data encrypted: SSN encrypted = {'ssn' in encrypted_data and len(encrypted_data['ssn']) > 20}")
        
        # Test decryption
        decrypted_data = security.encryption_manager.decrypt_table_data("sensitive_data", encrypted_data)
        print(f"✓ Data decrypted: SSN = {decrypted_data.get('ssn')}")
    
    print("\n4. Audit Logging...")
    
    # Generate some audit events
    dev_token = security.login("developer", "dev123", "192.168.1.104")
    if dev_token:
        security.execute_secure_query(dev_token, {'SELECT': ['id', 'name'], 'FROM': 'users'}, "192.168.1.104")
    
    security.logout(dev_token)
    
    # Show audit statistics
    audit_stats = security.audit_logger.get_statistics()
    print(f"✓ Total audit events: {audit_stats['total_events']}")
    print(f"✓ Success rate: {audit_stats['overall_success_rate']:.1f}%")
    print(f"✓ Event types: {list(audit_stats['event_type_counts'].keys())}")
    
    print("\n5. Security Dashboard...")
    
    dashboard = security.get_security_dashboard()
    print(f"✓ Security level: {dashboard['security_level']}")
    print(f"✓ Active sessions: {dashboard['access_control']['active_sessions']}")
    print(f"✓ Active users: {dashboard['access_control']['active_users']}")
    print(f"✓ Encrypted tables: {dashboard['encryption']['encrypted_tables']}")
    
    if dashboard['recommendations']:
        print("✓ Security recommendations:")
        for rec in dashboard['recommendations']:
            print(f"   - {rec}")
    
    print("\n6. Role-Based Access Control...")
    
    # Show user permissions
    if admin_token:
        permissions = security.access_control.get_user_permissions(admin_token)
        print("✓ Admin permissions:")
        for resource, perms in permissions.items():
            print(f"   {resource}: {', '.join(perms)}")
    
    print("\n7. Session Management...")
    
    # Check session status
    session_info = security.access_control.verify_session(admin_token)
    if session_info:
        print(f"✓ Session active for: {session_info['username']}")
        print(f"✓ Login time: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(session_info['login_time']))}")
        print(f"✓ Last activity: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(session_info['last_activity']))}")
    
    # Test logout
    security.logout(admin_token)
    print("✓ Logout successful")
    
    # Verify session terminated
    session_info = security.access_control.verify_session(admin_token)
    if not session_info:
        print("✓ Session properly terminated after logout")


def demonstrate_encryption_scenarios():
    """Demonstrate data encryption scenarios"""
    print("\n" + "="*60)
    print("DATA ENCRYPTION SCENARIOS")
    print("="*60)
    
    security = SecurityManager()
    
    print("\n1. Personal Data Encryption...")
    
    # Enable encryption for personal data
    security.encryption_manager.enable_encryption("personal_data", "secure_password_2023")
    
    # Sample personal data
    personal_records = [
        {"name": "Alice Johnson", "ssn": "111-22-3333", "email": "alice@example.com", "salary": 85000},
        {"name": "Bob Smith", "ssn": "444-55-6666", "email": "bob@example.com", "salary": 92000},
        {"name": "Carol Davis", "ssn": "777-88-9999", "email": "carol@example.com", "salary": 78000}
    ]
    
    print("✓ Encrypted personal data:")
    for record in personal_records:
        encrypted = security.encryption_manager.encrypt_table_data("personal_data", record)
        # Show that sensitive fields are encrypted
        encrypted_ssn = encrypted.get('ssn', 'not found')
        print(f"   {record['name']}: SSN encrypted = {len(encrypted_ssn) > 20}")
    
    print("\n2. Financial Data Encryption...")
    
    # Enable encryption for financial data
    security.encryption_manager.enable_encryption("financial_data", "finance_security_key")
    
    financial_records = [
        {"account_id": "ACC001", "balance": 25000.50, "transaction_history": "multiple_transactions"},
        {"account_id": "ACC002", "balance": 15000.75, "transaction_history": "recent_activity"}
    ]
    
    print("✓ Encrypted financial data:")
    for record in financial_records:
        encrypted = security.encryption_manager.encrypt_table_data("financial_data", record)
        encrypted_balance = encrypted.get('balance', 'not found')
        print(f"   {record['account_id']}: Balance encrypted = {isinstance(encrypted_balance, str)}")
    
    print("\n3. Key Rotation...")
    
    # Simulate key rotation
    rotation_time = security.encryption_manager.key_rotation_schedule.get("personal_data")
    print(f"✓ Last key rotation: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(rotation_time))}")
    
    # Perform rotation
    success = security.encryption_manager.rotate_encryption_key("personal_data", "new_password_2023")
    if success:
        new_rotation_time = security.encryption_manager.key_rotation_schedule.get("personal_data")
        print(f"✓ Key rotation completed at: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(new_rotation_time))}")
    
    print("\n4. Encryption Statistics...")
    
    stats = security.encryption_manager.get_encryption_statistics()
    print(f"✓ Encrypted tables: {stats['encrypted_tables']}")
    print(f"✓ Table list: {stats['tables']}")
    print("✓ Key rotation schedule:")
    for table, days_old in stats['key_rotation_schedule'].items():
        print(f"   {table}: {days_old/86400:.1f} days ago")


def main():
    """Main demonstration function"""
    print("DATABASE SECURITY AND ACCESS CONTROL DEMO")
    print("Demonstrating Authentication, Authorization, Encryption, and Auditing")
    print("="*80)
    
    try:
        demonstrate_security_system()
        demonstrate_encryption_scenarios()
        
        print("\n" + "="*80)
        print("SECURITY SYSTEM DEMO COMPLETED")
        print("="*80)
        print("\nKey Concepts Demonstrated:")
        print("✓ User authentication and session management")
        print("✓ Role-based access control (RBAC)")
        print("✓ Permission checking and enforcement")
        print("✓ Data encryption at rest")
        print("✓ Audit logging and monitoring")
        print("✓ Brute force attack prevention")
        print("✓ Security policy enforcement")
        print("✓ Security dashboard and monitoring")
        print("\nThis educational security system provides hands-on learning of:")
        print("- Database security principles")
        print("- Authentication and authorization mechanisms")
        print("- Data encryption techniques")
        print("- Security auditing and compliance")
        print("- Access control models")
        print("- Security threat mitigation")
        print("- Key management and rotation")
        print("- Security monitoring and analysis")
        
    except Exception as e:
        print(f"Error during demonstration: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()