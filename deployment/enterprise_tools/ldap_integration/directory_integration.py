"""
Active Directory and LDAP Integration for MultiOS Enterprise
"""

import os
import json
import logging
import ldap3
from typing import Dict, List, Optional, Any
from datetime import datetime

from ..core.models import UserAccount, UserRole
from ..core.utils import validate_email

class DirectoryIntegration:
    """Manager for Active Directory and LDAP integration"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/ldap.yaml"):
        self.config_path = config_path
        self.ldap_server = None
        self.connection = None
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
    
    def _load_configuration(self) -> None:
        """Load LDAP configuration"""
        self.config = {
            'ldap': {
                'server': 'ldap://localhost:389',
                'base_dn': 'dc=example,dc=edu',
                'bind_dn': 'cn=admin,dc=example,dc=edu',
                'bind_password': '',
                'use_ssl': False,
                'verify_ssl': True
            },
            'active_directory': {
                'enabled': False,
                'domain': 'example.edu',
                'server': 'ldap://dc.example.edu',
                'base_ou': 'OU=Users,DC=example,DC=edu',
                'bind_user': 'admin@example.edu',
                'bind_password': ''
            },
            'sync': {
                'auto_sync': False,
                'sync_interval_hours': 24,
                'create_missing_users': False,
                'update_existing_users': True,
                'sync_groups': True
            }
        }
        
        # Load configuration file if exists
        if os.path.exists(self.config_path):
            try:
                import yaml
                with open(self.config_path, 'r') as f:
                    loaded_config = yaml.safe_load(f)
                self.config.update(loaded_config)
            except Exception as e:
                self.logger.warning(f"Failed to load LDAP config: {e}")
    
    def configure_ldap(self, server_url: str, base_dn: str, bind_dn: str, 
                      bind_password: str, use_ssl: bool = True) -> bool:
        """Configure LDAP connection settings"""
        try:
            self.config['ldap']['server'] = server_url
            self.config['ldap']['base_dn'] = base_dn
            self.config['ldap']['bind_dn'] = bind_dn
            self.config['ldap']['bind_password'] = bind_password
            self.config['ldap']['use_ssl'] = use_ssl
            
            self.logger.info("LDAP configuration updated")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to configure LDAP: {e}")
            return False
    
    def configure_active_directory(self, domain: str, server: str, base_ou: str,
                                  bind_user: str, bind_password: str) -> bool:
        """Configure Active Directory connection settings"""
        try:
            self.config['active_directory']['enabled'] = True
            self.config['active_directory']['domain'] = domain
            self.config['active_directory']['server'] = server
            self.config['active_directory']['base_ou'] = base_ou
            self.config['active_directory']['bind_user'] = bind_user
            self.config['active_directory']['bind_password'] = bind_password
            
            self.logger.info("Active Directory configuration updated")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to configure Active Directory: {e}")
            return False
    
    def test_connection(self) -> bool:
        """Test LDAP/AD connection"""
        try:
            # Test LDAP connection
            if self.config['ldap']['server']:
                server = ldap3.Server(self.config['ldap']['server'])
                connection = ldap3.Connection(
                    server,
                    user=self.config['ldap']['bind_dn'],
                    password=self.config['ldap']['bind_password'],
                    auto_bind=True
                )
                
                if connection.bind():
                    self.logger.info("LDAP connection test successful")
                    connection.unbind()
                    return True
            
            # Test Active Directory connection
            if self.config['active_directory']['enabled']:
                ad_server = ldap3.Server(self.config['active_directory']['server'])
                ad_connection = ldap3.Connection(
                    ad_server,
                    user=self.config['active_directory']['bind_user'],
                    password=self.config['active_directory']['bind_password'],
                    auto_bind=True
                )
                
                if ad_connection.bind():
                    self.logger.info("Active Directory connection test successful")
                    ad_connection.unbind()
                    return True
            
            return False
            
        except Exception as e:
            self.logger.error(f"LDAP/AD connection test failed: {e}")
            return False
    
    def sync_users(self) -> bool:
        """Synchronize users from LDAP/AD"""
        try:
            synced_users = 0
            updated_users = 0
            
            # Sync from LDAP
            if self.config['ldap']['server']:
                ldap_users = self._sync_from_ldap()
                synced_users += len(ldap_users)
                updated_users += sum(1 for user in ldap_users if user.get('updated'))
            
            # Sync from Active Directory
            if self.config['active_directory']['enabled']:
                ad_users = self._sync_from_active_directory()
                synced_users += len(ad_users)
                updated_users += sum(1 for user in ad_users if user.get('updated'))
            
            self.logger.info(f"User synchronization completed: {synced_users} users processed")
            return True
            
        except Exception as e:
            self.logger.error(f"User synchronization failed: {e}")
            return False
    
    def _sync_from_ldap(self) -> List[Dict[str, Any]]:
        """Synchronize users from LDAP directory"""
        try:
            server = ldap3.Server(self.config['ldap']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['ldap']['bind_dn'],
                password=self.config['ldap']['bind_password'],
                auto_bind=True
            )
            
            # Search for users
            connection.search(
                search_base=self.config['ldap']['base_dn'],
                search_filter='(objectClass=person)',
                attributes=['cn', 'mail', 'uid', 'givenName', 'sn', 'memberOf']
            )
            
            synced_users = []
            for entry in connection.entries:
                user_data = {
                    'username': str(entry.uid) if hasattr(entry, 'uid') else str(entry.cn),
                    'email': str(entry.mail) if hasattr(entry, 'mail') else '',
                    'full_name': str(entry.cn) if hasattr(entry, 'cn') else '',
                    'first_name': str(entry.givenName) if hasattr(entry, 'givenName') else '',
                    'last_name': str(entry.sn) if hasattr(entry, 'sn') else '',
                    'groups': self._extract_ldap_groups(entry),
                    'source': 'ldap',
                    'updated': False
                }
                
                synced_users.append(user_data)
            
            connection.unbind()
            self.logger.info(f"Synced {len(synced_users)} users from LDAP")
            return synced_users
            
        except Exception as e:
            self.logger.error(f"LDAP synchronization failed: {e}")
            return []
    
    def _sync_from_active_directory(self) -> List[Dict[str, Any]]:
        """Synchronize users from Active Directory"""
        try:
            server = ldap3.Server(self.config['active_directory']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['active_directory']['bind_user'],
                password=self.config['active_directory']['bind_password'],
                auto_bind=True
            )
            
            # Search for users in the specified OU
            connection.search(
                search_base=self.config['active_directory']['base_ou'],
                search_filter='(&(objectClass=user)(objectCategory=person))',
                attributes=['cn', 'mail', 'sAMAccountName', 'givenName', 'sn', 'memberOf']
            )
            
            synced_users = []
            for entry in connection.entries:
                user_data = {
                    'username': str(entry.sAMAccountName),
                    'email': str(entry.mail) if hasattr(entry, 'mail') else '',
                    'full_name': str(entry.cn),
                    'first_name': str(entry.givenName) if hasattr(entry, 'givenName') else '',
                    'last_name': str(entry.sn) if hasattr(entry, 'sn') else '',
                    'groups': self._extract_ad_groups(entry),
                    'source': 'active_directory',
                    'updated': False
                }
                
                synced_users.append(user_data)
            
            connection.unbind()
            self.logger.info(f"Synced {len(synced_users)} users from Active Directory")
            return synced_users
            
        except Exception as e:
            self.logger.error(f"Active Directory synchronization failed: {e}")
            return []
    
    def _extract_ldap_groups(self, entry) -> List[str]:
        """Extract group memberships from LDAP entry"""
        groups = []
        try:
            if hasattr(entry, 'memberOf'):
                for group_dn in entry.memberOf.values:
                    # Extract group name from DN
                    group_name = group_dn.split(',')[0].split('=')[1].strip()
                    groups.append(group_name)
        except Exception:
            pass
        return groups
    
    def _extract_ad_groups(self, entry) -> List[str]:
        """Extract group memberships from AD entry"""
        groups = []
        try:
            if hasattr(entry, 'memberOf'):
                for group_dn in entry.memberOf.values:
                    # Extract group name from DN
                    group_name = group_dn.split(',')[0].split('=')[1].strip()
                    groups.append(group_name)
        except Exception:
            pass
        return groups
    
    def map_groups_to_roles(self, groups: List[str]) -> UserRole:
        """Map LDAP/AD groups to user roles"""
        # Define group-to-role mappings
        admin_groups = ['admins', 'domain admins', 'it staff']
        teacher_groups = ['teachers', 'faculty', 'instructors']
        support_groups = ['support', 'help desk', 'technicians']
        
        # Check for admin groups
        for group in groups:
            if any(admin_group in group.lower() for admin_group in admin_groups):
                return UserRole.ADMIN
        
        # Check for teacher groups
        for group in groups:
            if any(teacher_group in group.lower() for teacher_group in teacher_groups):
                return UserRole.TEACHER
        
        # Check for support groups
        for group in groups:
            if any(support_group in group.lower() for support_group in support_groups):
                return UserRole.SUPPORT
        
        # Default to student
        return UserRole.STUDENT
    
    def create_user_in_directory(self, user_account: UserAccount) -> bool:
        """Create user in LDAP/AD directory"""
        try:
            if self.config['active_directory']['enabled']:
                return self._create_user_in_ad(user_account)
            elif self.config['ldap']['server']:
                return self._create_user_in_ldap(user_account)
            else:
                self.logger.warning("No directory service configured")
                return False
                
        except Exception as e:
            self.logger.error(f"Failed to create user in directory: {e}")
            return False
    
    def _create_user_in_ldap(self, user_account: UserAccount) -> bool:
        """Create user in LDAP directory"""
        try:
            server = ldap3.Server(self.config['ldap']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['ldap']['bind_dn'],
                password=self.config['ldap']['bind_password'],
                auto_bind=True
            )
            
            # Create user entry
            user_dn = f"uid={user_account.username},{self.config['ldap']['base_dn']}"
            user_attributes = {
                'objectClass': ['inetOrgPerson', 'posixAccount', 'shadowAccount'],
                'uid': user_account.username,
                'cn': user_account.full_name,
                'givenName': user_account.full_name.split()[0] if user_account.full_name else user_account.username,
                'sn': user_account.full_name.split()[-1] if user_account.full_name else user_account.username,
                'mail': user_account.email,
                'uidNumber': self._get_next_uid_number(),
                'gidNumber': 1000,
                'homeDirectory': f'/home/{user_account.username}',
                'loginShell': '/bin/bash'
            }
            
            connection.add(user_dn, attributes=user_attributes)
            
            if connection.result['result'] == 0:
                self.logger.info(f"Created LDAP user: {user_account.username}")
                connection.unbind()
                return True
            else:
                self.logger.error(f"Failed to create LDAP user: {connection.result}")
                return False
                
        except Exception as e:
            self.logger.error(f"LDAP user creation failed: {e}")
            return False
    
    def _create_user_in_ad(self, user_account: UserAccount) -> bool:
        """Create user in Active Directory"""
        try:
            server = ldap3.Server(self.config['active_directory']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['active_directory']['bind_user'],
                password=self.config['active_directory']['bind_password'],
                auto_bind=True
            )
            
            # Create user entry
            user_dn = f"CN={user_account.username},{self.config['active_directory']['base_ou']}"
            user_attributes = {
                'objectClass': ['top', 'person', 'organizationalPerson', 'user'],
                'cn': user_account.full_name,
                'givenName': user_account.full_name.split()[0] if user_account.full_name else user_account.username,
                'sn': user_account.full_name.split()[-1] if user_account.full_name else user_account.username,
                'mail': user_account.email,
                'sAMAccountName': user_account.username,
                'userAccountControl': 512  # Normal account
            }
            
            connection.add(user_dn, attributes=user_attributes)
            
            if connection.result['result'] == 0:
                self.logger.info(f"Created AD user: {user_account.username}")
                connection.unbind()
                return True
            else:
                self.logger.error(f"Failed to create AD user: {connection.result}")
                return False
                
        except Exception as e:
            self.logger.error(f"AD user creation failed: {e}")
            return False
    
    def _get_next_uid_number(self) -> int:
        """Get next available UID number"""
        # This would normally query the directory for existing UIDs
        # For now, return a placeholder
        return 10000 + len(self.config)  # Simple increment
    
    def get_directory_users(self) -> List[Dict[str, Any]]:
        """Get list of users from directory"""
        try:
            users = []
            
            # Get LDAP users
            if self.config['ldap']['server']:
                ldap_users = self._sync_from_ldap()
                users.extend(ldap_users)
            
            # Get AD users
            if self.config['active_directory']['enabled']:
                ad_users = self._sync_from_active_directory()
                users.extend(ad_users)
            
            return users
            
        except Exception as e:
            self.logger.error(f"Failed to get directory users: {e}")
            return []
    
    def update_user_in_directory(self, user_account: UserAccount) -> bool:
        """Update user in directory service"""
        try:
            if self.config['active_directory']['enabled']:
                return self._update_user_in_ad(user_account)
            elif self.config['ldap']['server']:
                return self._update_user_in_ldap(user_account)
            else:
                return False
                
        except Exception as e:
            self.logger.error(f"Failed to update user in directory: {e}")
            return False
    
    def _update_user_in_ldap(self, user_account: UserAccount) -> bool:
        """Update user in LDAP directory"""
        try:
            server = ldap3.Server(self.config['ldap']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['ldap']['bind_dn'],
                password=self.config['ldap']['bind_password'],
                auto_bind=True
            )
            
            user_dn = f"uid={user_account.username},{self.config['ldap']['base_dn']}"
            update_attributes = {
                'mail': user_account.email,
                'cn': user_account.full_name
            }
            
            connection.modify(user_dn, update_attributes)
            
            if connection.result['result'] == 0:
                self.logger.info(f"Updated LDAP user: {user_account.username}")
                connection.unbind()
                return True
            else:
                self.logger.error(f"Failed to update LDAP user: {connection.result}")
                return False
                
        except Exception as e:
            self.logger.error(f"LDAP user update failed: {e}")
            return False
    
    def _update_user_in_ad(self, user_account: UserAccount) -> bool:
        """Update user in Active Directory"""
        try:
            server = ldap3.Server(self.config['active_directory']['server'])
            connection = ldap3.Connection(
                server,
                user=self.config['active_directory']['bind_user'],
                password=self.config['active_directory']['bind_password'],
                auto_bind=True
            )
            
            user_dn = f"CN={user_account.username},{self.config['active_directory']['base_ou']}"
            update_attributes = {
                'mail': user_account.email,
                'cn': user_account.full_name
            }
            
            connection.modify(user_dn, update_attributes)
            
            if connection.result['result'] == 0:
                self.logger.info(f"Updated AD user: {user_account.username}")
                connection.unbind()
                return True
            else:
                self.logger.error(f"Failed to update AD user: {connection.result}")
                return False
                
        except Exception as e:
            self.logger.error(f"AD user update failed: {e}")
            return False
