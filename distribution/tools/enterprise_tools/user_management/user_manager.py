"""
Bulk User Account Creation and Management System
"""

import os
import json
import csv
import logging
import subprocess
import hashlib
import random
import string
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import tempfile
import re

from ..core.models import UserAccount, UserRole, SiteConfig
from ..core.utils import validate_email, execute_command, generate_system_id

class UserManager:
    """Manager for bulk user account creation and management"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/users.yaml"):
        self.config_path = config_path
        self.users = {}
        self.user_groups = {}
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
        self._setup_directories()
    
    def _load_configuration(self) -> None:
        """Load user management configuration"""
        # Default configuration
        self.config = {
            'password_policy': {
                'min_length': 8,
                'require_uppercase': True,
                'require_lowercase': True,
                'require_numbers': True,
                'require_special': False
            },
            'account_settings': {
                'default_shell': '/bin/bash',
                'home_directory': '/home',
                'default_groups': ['users'],
                'password_expiry_days': 90,
                'account_expiry_days': None
            },
            'ldap_integration': {
                'enabled': False,
                'server': '',
                'base_dn': '',
                'bind_dn': '',
                'bind_password': ''
            }
        }
    
    def _setup_directories(self) -> None:
        """Create user management directories"""
        directories = [
            "/var/lib/multios-enterprise/users",
            "/var/lib/multios-enterprise/templates",
            "/var/log/multios-enterprise/users"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def create_user_account(self, user_data: Dict[str, Any]) -> Optional[str]:
        """Create a single user account"""
        try:
            # Validate user data
            validation_result = self._validate_user_data(user_data)
            if not validation_result['valid']:
                self.logger.error(f"Invalid user data: {validation_result['error']}")
                return None
            
            # Generate user ID if not provided
            user_id = user_data.get('user_id') or generate_system_id()
            
            # Create user account object
            user = UserAccount(
                user_id=user_id,
                username=user_data['username'],
                email=user_data['email'],
                full_name=user_data['full_name'],
                role=UserRole(user_data['role']),
                site_id=user_data['site_id'],
                department=user_data.get('department', ''),
                created_date=datetime.now(),
                assigned_systems=user_data.get('assigned_systems', []),
                group_memberships=user_data.get('group_memberships', []),
                license_assignments=user_data.get('license_assignments', []),
                preferences=user_data.get('preferences', {})
            )
            
            # Generate password if not provided
            password = user_data.get('password') or self._generate_password()
            
            # Create system user account
            if not self._create_system_user(user, password):
                return None
            
            # Setup user groups
            self._setup_user_groups(user)
            
            # Store user account information
            self.users[user_id] = user
            
            # Save to database/file
            self._save_user_account(user)
            
            # Log successful creation
            self.logger.info(f"Created user account: {user.username} ({user_id})")
            
            return user_id
        except Exception as e:
            self.logger.error(f"Failed to create user account: {e}")
            return None
    
    def create_bulk_users(self, users_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Create multiple user accounts in bulk"""
        results = {
            'total': len(users_data),
            'successful': 0,
            'failed': 0,
            'results': []
        }
        
        for user_data in users_data:
            user_id = self.create_user_account(user_data)
            
            result = {
                'username': user_data.get('username', 'unknown'),
                'user_id': user_id,
                'success': user_id is not None
            }
            
            results['results'].append(result)
            
            if user_id:
                results['successful'] += 1
            else:
                results['failed'] += 1
        
        self.logger.info(f"Bulk user creation completed: {results['successful']}/{results['total']} successful")
        return results
    
    def import_users_from_csv(self, csv_path: str) -> Dict[str, Any]:
        """Import users from CSV file"""
        results = {
            'total': 0,
            'successful': 0,
            'failed': 0,
            'errors': []
        }
        
        try:
            users_data = []
            
            with open(csv_path, 'r', encoding='utf-8') as csvfile:
                reader = csv.DictReader(csvfile)
                
                for row in reader:
                    users_data.append({
                        'username': row.get('username', '').strip(),
                        'email': row.get('email', '').strip(),
                        'full_name': row.get('full_name', '').strip(),
                        'role': row.get('role', 'student').strip(),
                        'site_id': row.get('site_id', '').strip(),
                        'department': row.get('department', '').strip()
                    })
            
            results['total'] = len(users_data)
            
            # Create users in bulk
            creation_results = self.create_bulk_users(users_data)
            results['successful'] = creation_results['successful']
            results['failed'] = creation_results['failed']
            
        except Exception as e:
            self.logger.error(f"Failed to import users from CSV: {e}")
            results['errors'].append(str(e))
        
        return results
    
    def _validate_user_data(self, user_data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate user account data"""
        required_fields = ['username', 'email', 'full_name', 'role', 'site_id']
        
        # Check required fields
        for field in required_fields:
            if field not in user_data or not user_data[field]:
                return {'valid': False, 'error': f'Missing required field: {field}'}
        
        # Validate username
        username = user_data['username']
        if not re.match(r'^[a-zA-Z0-9._-]+$', username):
            return {'valid': False, 'error': 'Username contains invalid characters'}
        
        if len(username) < 3 or len(username) > 32:
            return {'valid': False, 'error': 'Username must be 3-32 characters long'}
        
        # Check if username already exists
        existing_users = [u for u in self.users.values() if u.username == username]
        if existing_users:
            return {'valid': False, 'error': f'Username {username} already exists'}
        
        # Validate email
        if not validate_email(user_data['email']):
            return {'valid': False, 'error': 'Invalid email format'}
        
        # Validate role
        try:
            UserRole(user_data['role'])
        except ValueError:
            return {'valid': False, 'error': f'Invalid user role: {user_data["role"]}'}
        
        return {'valid': True, 'error': None}
    
    def _generate_password(self, length: int = None) -> str:
        """Generate a secure random password"""
        length = length or self.config['password_policy']['min_length']
        
        # Character sets
        lowercase = string.ascii_lowercase
        uppercase = string.ascii_uppercase
        digits = string.digits
        special = '!@#$%^&*'
        
        # Build character pool based on password policy
        char_pool = lowercase
        if self.config['password_policy']['require_uppercase']:
            char_pool += uppercase
        if self.config['password_policy']['require_numbers']:
            char_pool += digits
        if self.config['password_policy']['require_special']:
            char_pool += special
        
        # Generate password ensuring requirements are met
        password = []
        
        # Ensure at least one character from each required set
        if self.config['password_policy']['require_lowercase']:
            password.append(random.choice(lowercase))
        if self.config['password_policy']['require_uppercase']:
            password.append(random.choice(uppercase))
        if self.config['password_policy']['require_numbers']:
            password.append(random.choice(digits))
        if self.config['password_policy']['require_special']:
            password.append(random.choice(special))
        
        # Fill remaining length
        while len(password) < length:
            password.append(random.choice(char_pool))
        
        # Shuffle and return
        random.shuffle(password)
        return ''.join(password)
    
    def _create_system_user(self, user: UserAccount, password: str) -> bool:
        """Create user account on the system"""
        try:
            # Create user account using useradd
            useradd_cmd = [
                'useradd',
                '-c', user.full_name,
                '-d', f"{self.config['account_settings']['home_directory']}/{user.username}",
                '-m',  # Create home directory
                '-s', self.config['account_settings']['default_shell'],
                user.username
            ]
            
            result, stdout, stderr = execute_command(useradd_cmd)
            if result != 0:
                self.logger.error(f"Failed to create system user {user.username}: {stderr}")
                return False
            
            # Set password
            echo_cmd = f"echo '{user.username}:{password}' | chpasswd"
            result, stdout, stderr = execute_command(['sh', '-c', echo_cmd])
            if result != 0:
                self.logger.error(f"Failed to set password for {user.username}: {stderr}")
                # Cleanup user if password setting fails
                execute_command(['userdel', user.username])
                return False
            
            # Set account expiration if specified
            if self.config['account_settings']['account_expiry_days']:
                expiry_date = datetime.now() + timedelta(days=self.config['account_settings']['account_expiry_days'])
                expiry_timestamp = int(expiry_date.timestamp())
                
                usermod_cmd = ['usermod', '-e', str(expiry_timestamp), user.username]
                execute_command(usermod_cmd)
            
            # Setup password expiry
            if self.config['account_settings']['password_expiry_days']:
                password_age_cmd = ['chage', '-M', str(self.config['account_settings']['password_expiry_days']), user.username]
                execute_command(password_age_cmd)
            
            self.logger.info(f"Created system user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to create system user {user.username}: {e}")
            return False
    
    def _setup_user_groups(self, user: UserAccount) -> None:
        """Setup user group memberships"""
        try:
            # Add user to default groups
            for group in self.config['account_settings']['default_groups']:
                try:
                    execute_command(['usermod', '-a', '-G', group, user.username])
                except Exception:
                    pass  # Group might not exist, continue
            
            # Add user to role-based groups
            role_groups = {
                UserRole.ADMIN: ['sudo', 'admin'],
                UserRole.TEACHER: ['teachers', 'staff'],
                UserRole.STUDENT: ['students'],
                UserRole.SUPPORT: ['support'],
                UserRole.GUEST: ['guests']
            }
            
            if user.role in role_groups:
                for group in role_groups[user.role]:
                    try:
                        execute_command(['usermod', '-a', '-G', group, user.username])
                    except Exception:
                        pass  # Group might not exist, continue
            
            # Add user to custom groups from user data
            for group in user.group_memberships:
                try:
                    execute_command(['usermod', '-a', '-G', group, user.username])
                except Exception:
                    pass  # Group might not exist, continue
            
            self.logger.info(f"Setup group memberships for user {user.username}")
            
        except Exception as e:
            self.logger.error(f"Failed to setup groups for user {user.username}: {e}")
    
    def _save_user_account(self, user: UserAccount) -> None:
        """Save user account information to file"""
        try:
            user_file = Path("/var/lib/multios-enterprise/users") / f"{user.user_id}.json"
            
            user_data = {
                'user_id': user.user_id,
                'username': user.username,
                'email': user.email,
                'full_name': user.full_name,
                'role': user.role.value,
                'site_id': user.site_id,
                'department': user.department,
                'created_date': user.created_date.isoformat(),
                'last_login': user.last_login.isoformat() if user.last_login else None,
                'assigned_systems': user.assigned_systems,
                'group_memberships': user.group_memberships,
                'license_assignments': user.license_assignments,
                'preferences': user.preferences
            }
            
            with open(user_file, 'w') as f:
                json.dump(user_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save user account {user.username}: {e}")
    
    def update_user_account(self, user_id: str, updates: Dict[str, Any]) -> bool:
        """Update user account information"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            
            # Update allowed fields
            allowed_updates = ['email', 'full_name', 'role', 'department', 'assigned_systems', 
                             'group_memberships', 'license_assignments', 'preferences']
            
            for field, value in updates.items():
                if field in allowed_updates:
                    if field == 'role' and isinstance(value, str):
                        value = UserRole(value)
                    
                    setattr(user, field, value)
            
            # Update system account if needed
            if 'full_name' in updates or 'email' in updates:
                self._update_system_user(user)
            
            # Save updated user account
            self._save_user_account(user)
            
            self.logger.info(f"Updated user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to update user account {user_id}: {e}")
            return False
    
    def _update_system_user(self, user: UserAccount) -> bool:
        """Update system user account information"""
        try:
            # Update user comment (full name)
            usermod_cmd = ['usermod', '-c', user.full_name, user.username]
            result, stdout, stderr = execute_command(usermod_cmd)
            
            if result != 0:
                self.logger.error(f"Failed to update system user {user.username}: {stderr}")
                return False
            
            self.logger.info(f"Updated system user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to update system user {user.username}: {e}")
            return False
    
    def delete_user_account(self, user_id: str, remove_home: bool = True) -> bool:
        """Delete user account"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            
            # Remove system user account
            userdel_cmd = ['userdel']
            if remove_home:
                userdel_cmd.append('-r')  # Remove home directory
            
            userdel_cmd.append(user.username)
            
            result, stdout, stderr = execute_command(userdel_cmd)
            if result != 0:
                self.logger.error(f"Failed to delete system user {user.username}: {stderr}")
                return False
            
            # Remove from internal registry
            del self.users[user_id]
            
            # Remove user file
            user_file = Path("/var/lib/multios-enterprise/users") / f"{user_id}.json"
            if user_file.exists():
                user_file.unlink()
            
            self.logger.info(f"Deleted user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to delete user account {user_id}: {e}")
            return False
    
    def list_users(self, role: Optional[UserRole] = None, site_id: Optional[str] = None) -> List[UserAccount]:
        """List user accounts with optional filtering"""
        users = list(self.users.values())
        
        if role:
            users = [user for user in users if user.role == role]
        
        if site_id:
            users = [user for user in users if user.site_id == site_id]
        
        return users
    
    def get_user(self, user_id: str) -> Optional[UserAccount]:
        """Get user account by ID"""
        return self.users.get(user_id)
    
    def get_user_by_username(self, username: str) -> Optional[UserAccount]:
        """Get user account by username"""
        for user in self.users.values():
            if user.username == username:
                return user
        return None
    
    def reset_user_password(self, user_id: str, new_password: Optional[str] = None) -> bool:
        """Reset user password"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            password = new_password or self._generate_password()
            
            # Set new password
            echo_cmd = f"echo '{user.username}:{password}' | chpasswd"
            result, stdout, stderr = execute_command(['sh', '-c', echo_cmd])
            
            if result != 0:
                self.logger.error(f"Failed to reset password for {user.username}: {stderr}")
                return False
            
            self.logger.info(f"Reset password for user: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to reset password for user {user_id}: {e}")
            return False
    
    def disable_user_account(self, user_id: str) -> bool:
        """Disable user account (lock password)"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            
            # Lock user account
            usermod_cmd = ['usermod', '-L', user.username]
            result, stdout, stderr = execute_command(usermod_cmd)
            
            if result != 0:
                self.logger.error(f"Failed to disable user {user.username}: {stderr}")
                return False
            
            self.logger.info(f"Disabled user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to disable user account {user_id}: {e}")
            return False
    
    def enable_user_account(self, user_id: str) -> bool:
        """Enable user account (unlock password)"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            
            # Unlock user account
            usermod_cmd = ['usermod', '-U', user.username]
            result, stdout, stderr = execute_command(usermod_cmd)
            
            if result != 0:
                self.logger.error(f"Failed to enable user {user.username}: {stderr}")
                return False
            
            self.logger.info(f"Enabled user account: {user.username}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to enable user account {user_id}: {e}")
            return False
    
    def assign_user_to_system(self, user_id: str, system_id: str) -> bool:
        """Assign user to a system"""
        try:
            if user_id not in self.users:
                self.logger.error(f"User {user_id} not found")
                return False
            
            user = self.users[user_id]
            
            if system_id not in user.assigned_systems:
                user.assigned_systems.append(system_id)
                self._save_user_account(user)
                self.logger.info(f"Assigned user {user.username} to system {system_id}")
                return True
            
            return True  # Already assigned
            
        except Exception as e:
            self.logger.error(f"Failed to assign user to system: {e}")
            return False
    
    def generate_user_report(self, format_type: str = 'json') -> str:
        """Generate comprehensive user report"""
        try:
            report_data = {
                'generated': datetime.now().isoformat(),
                'total_users': len(self.users),
                'users_by_role': {},
                'users_by_site': {},
                'users': []
            }
            
            # Count users by role and site
            for user in self.users.values():
                # By role
                role = user.role.value
                if role not in report_data['users_by_role']:
                    report_data['users_by_role'][role] = 0
                report_data['users_by_role'][role] += 1
                
                # By site
                site = user.site_id
                if site not in report_data['users_by_site']:
                    report_data['users_by_site'][site] = 0
                report_data['users_by_site'][site] += 1
                
                # User details
                user_data = {
                    'user_id': user.user_id,
                    'username': user.username,
                    'email': user.email,
                    'full_name': user.full_name,
                    'role': user.role.value,
                    'site_id': user.site_id,
                    'department': user.department,
                    'created_date': user.created_date.isoformat(),
                    'last_login': user.last_login.isoformat() if user.last_login else None,
                    'assigned_systems_count': len(user.assigned_systems),
                    'group_memberships_count': len(user.group_memberships)
                }
                report_data['users'].append(user_data)
            
            if format_type.lower() == 'json':
                return json.dumps(report_data, indent=2)
            elif format_type.lower() == 'csv':
                return self._convert_report_to_csv(report_data)
            else:
                raise ValueError(f"Unsupported format: {format_type}")
                
        except Exception as e:
            self.logger.error(f"Failed to generate user report: {e}")
            return ""
    
    def _convert_report_to_csv(self, report_data: Dict[str, Any]) -> str:
        """Convert report data to CSV format"""
        import io
        
        output = io.StringIO()
        writer = csv.writer(output)
        
        # Write header
        writer.writerow([
            'User ID', 'Username', 'Email', 'Full Name', 'Role', 'Site ID', 
            'Department', 'Created Date', 'Last Login', 'Assigned Systems Count',
            'Group Memberships Count'
        ])
        
        # Write user data
        for user_data in report_data['users']:
            writer.writerow([
                user_data['user_id'],
                user_data['username'],
                user_data['email'],
                user_data['full_name'],
                user_data['role'],
                user_data['site_id'],
                user_data['department'],
                user_data['created_date'],
                user_data['last_login'] or '',
                user_data['assigned_systems_count'],
                user_data['group_memberships_count']
            ])
        
        return output.getvalue()
