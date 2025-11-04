//! MultiOS User Management System
//! 
//! This module provides comprehensive user and group management functionality including:
//! - User creation, modification, and deletion
//! - Password-based authentication with optional multi-factor support
//! - Group management and role-based access control
//! - Integration with syscall interface for secure system operations
//! - Comprehensive audit logging and security monitoring

#![no_std]
#![feature(alloc)]
#![feature(core_intrinsics)]

use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::HashMap;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

#[cfg(test)]
mod tests;

/// User identifier type
pub type UserId = u32;

/// Group identifier type  
pub type GroupId = u32;

/// User and group management result
pub type UserResult<T> = Result<T, UserError>;

/// User management error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UserError {
    UserNotFound = 0,
    GroupNotFound = 1,
    UserAlreadyExists = 2,
    GroupAlreadyExists = 3,
    InvalidCredentials = 4,
    PermissionDenied = 5,
    PasswordWeak = 6,
    AccountLocked = 7,
    AccountExpired = 8,
    SessionExpired = 9,
    MultiFactorRequired = 10,
    MultiFactorFailed = 11,
    InvalidParameter = 12,
    ResourceExhausted = 13,
    ConfigurationError = 14,
    SecurityViolation = 15,
    AuditError = 16,
    DatabaseError = 17,
    NotInitialized = 18,
    OperationNotSupported = 19,
}

/// User structure representing a system user
#[derive(Debug, Clone)]
pub struct User {
    pub user_id: UserId,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub salt: Option<String>,
    pub primary_group_id: GroupId,
    pub secondary_groups: Vec<GroupId>,
    pub home_directory: String,
    pub shell: String,
    pub uid: UserId,
    pub gid: GroupId,
    pub created_time: u64,
    pub last_login_time: Option<u64>,
    pub password_last_changed: Option<u64>,
    pub account_expires: Option<u64>,
    pub is_enabled: bool,
    pub is_locked: bool,
    pub failed_login_attempts: u32,
    pub last_failed_login: Option<u64>,
    pub session_count: u32,
    pub privileges: Vec<String>,
    pub attributes: HashMap<String, String>,
}

/// Group structure representing a user group
#[derive(Debug, Clone)]
pub struct Group {
    pub group_id: GroupId,
    pub group_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub gid: GroupId,
    pub members: Vec<UserId>,
    pub created_time: u64,
    pub is_system_group: bool,
    pub privileges: Vec<String>,
    pub attributes: HashMap<String, String>,
}

/// User session information
#[derive(Debug, Clone)]
pub struct UserSession {
    pub session_id: u64,
    pub user_id: UserId,
    pub username: String,
    pub login_time: u64,
    pub last_activity: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub multi_factor_verified: bool,
    pub privileges: Vec<String>,
    pub session_data: HashMap<String, String>,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub success: bool,
    pub user_id: Option<UserId>,
    pub session_id: Option<u64>,
    pub error_code: UserError,
    pub requires_multi_factor: bool,
    pub mfa_secret: Option<String>,
    pub privileges: Vec<String>,
    pub session_timeout: u64,
}

/// Password strength requirements
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digits: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub min_age_days: Option<u32>,
    pub history_count: usize,
    pub lockout_threshold: u32,
    pub lockout_duration: u32,
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone)]
pub struct MfaConfig {
    pub enabled: bool,
    pub required: bool,
    pub methods: Vec<MfaMethod>,
    pub backup_codes: Vec<String>,
}

/// Multi-factor authentication method
#[derive(Debug, Clone)]
pub enum MfaMethod {
    Totp { secret: String, name: String },
    Sms { phone_number: String },
    Email { email: String },
    Hardware { device_id: String },
}

/// User statistics
#[derive(Debug, Clone)]
pub struct UserStats {
    pub total_users: usize,
    pub active_users: usize,
    pub locked_users: usize,
    pub system_users: usize,
    pub total_groups: usize,
    pub active_sessions: usize,
    pub total_login_attempts: u64,
    pub failed_login_attempts: u64,
    pub security_events: u64,
}

/// Global user manager instance
static USER_MANAGER: Mutex<Option<UserManager>> = Mutex::new(None);

/// User Manager - Main orchestrator for user and group operations
pub struct UserManager {
    users: RwLock<HashMap<UserId, User>>,
    groups: RwLock<HashMap<GroupId, Group>>,
    username_map: RwLock<HashMap<String, UserId>>,
    groupname_map: RwLock<HashMap<String, GroupId>>,
    sessions: RwLock<HashMap<u64, UserSession>>,
    password_policy: Mutex<PasswordPolicy>,
    mfa_config: Mutex<MfaConfig>,
    next_user_id: AtomicU32,
    next_group_id: AtomicU32,
    next_session_id: AtomicU64,
    initialized: bool,
    audit_enabled: bool,
}

impl UserManager {
    /// Create a new User Manager instance
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
            username_map: RwLock::new(HashMap::new()),
            groupname_map: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            password_policy: Mutex::new(PasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_digits: true,
                require_special_chars: true,
                max_age_days: Some(90),
                min_age_days: Some(1),
                history_count: 12,
                lockout_threshold: 5,
                lockout_duration: 1800, // 30 minutes
            }),
            mfa_config: Mutex::new(MfaConfig {
                enabled: true,
                required: false,
                methods: Vec::new(),
                backup_codes: Vec::new(),
            }),
            next_user_id: AtomicU32::new(1000), // Start UIDs from 1000
            next_group_id: AtomicU32::new(1000), // Start GIDs from 1000
            next_session_id: AtomicU64::new(1),
            initialized: false,
            audit_enabled: true,
        }
    }

    /// Initialize the user manager
    pub fn init(&mut self) -> UserResult<()> {
        if self.initialized {
            return Err(UserError::UserAlreadyExists);
        }

        // Create default system accounts
        self.create_system_accounts()?;
        
        // Create default groups
        self.create_default_groups()?;
        
        self.initialized = true;
        
        info!("User Manager initialized successfully");
        Ok(())
    }

    /// Shutdown the user manager
    pub fn shutdown(&mut self) -> UserResult<()> {
        if !self.initialized {
            return Err(UserError::NotInitialized);
        }

        // Invalidate all sessions
        let mut sessions = self.sessions.write();
        for session in sessions.values_mut() {
            session.is_active = false;
        }
        sessions.clear();

        self.initialized = false;
        info!("User Manager shutdown complete");
        Ok(())
    }

    // ==================== User Management Operations ====================

    /// Create a new user
    pub fn create_user(&self, username: &str, password: &str, email: Option<&str>) -> UserResult<UserId> {
        let username = username.to_string();
        
        // Check if user already exists
        {
            let username_map = self.username_map.read();
            if username_map.contains_key(&username) {
                return Err(UserError::UserAlreadyExists);
            }
        }

        // Validate password strength
        if let Err(e) = self.validate_password(password) {
            return Err(e);
        }

        let user_id = self.next_user_id.fetch_add(1, Ordering::SeqCst);
        
        // Hash password
        let (password_hash, salt) = self.hash_password(password)?;
        
        let user = User {
            user_id,
            username: username.clone(),
            display_name: username.clone(),
            email: email.map(|e| e.to_string()),
            password_hash: Some(password_hash),
            salt: Some(salt),
            primary_group_id: 100, // Default to users group
            secondary_groups: Vec::new(),
            home_directory: format!("/home/{}", username),
            shell: "/bin/sh".to_string(),
            uid: user_id,
            gid: 100,
            created_time: self.get_current_time(),
            last_login_time: None,
            password_last_changed: Some(self.get_current_time()),
            account_expires: None,
            is_enabled: true,
            is_locked: false,
            failed_login_attempts: 0,
            last_failed_login: None,
            session_count: 0,
            privileges: vec!["user".to_string()],
            attributes: HashMap::new(),
        };

        // Store user
        {
            let mut users = self.users.write();
            users.insert(user_id, user);
        }

        {
            let mut username_map = self.username_map.write();
            username_map.insert(username, user_id);
        }

        // Add user to primary group
        self.add_user_to_group(user_id, 100)?;

        if self.audit_enabled {
            self.log_audit_event("user_created", &username, Some(&user_id.to_string()));
        }

        info!("Created user: {} (UID: {})", username, user_id);
        Ok(user_id)
    }

    /// Modify an existing user
    pub fn modify_user(&self, user_id: UserId, updates: UserUpdates) -> UserResult<()> {
        let mut users = self.users.write();
        
        let user = users.get_mut(&user_id)
            .ok_or(UserError::UserNotFound)?;

        // Apply updates
        if let Some(display_name) = updates.display_name {
            user.display_name = display_name;
        }
        
        if let Some(email) = updates.email {
            user.email = Some(email);
        }
        
        if let Some(home_dir) = updates.home_directory {
            user.home_directory = home_dir;
        }
        
        if let Some(shell) = updates.shell {
            user.shell = shell;
        }
        
        if let Some(enabled) = updates.is_enabled {
            user.is_enabled = enabled;
        }
        
        if let Some(locked) = updates.is_locked {
            user.is_locked = locked;
        }
        
        if let Some(expires) = updates.account_expires {
            user.account_expires = Some(expires);
        }

        if self.audit_enabled {
            self.log_audit_event("user_modified", &user.username, Some(&user_id.to_string()));
        }

        Ok(())
    }

    /// Delete a user
    pub fn delete_user(&self, user_id: UserId) -> UserResult<()> {
        let mut users = self.users.write();
        let mut username_map = self.username_map.write();

        let user = users.remove(&user_id)
            .ok_or(UserError::UserNotFound)?;

        // Remove from username map
        username_map.remove(&user.username);

        // Remove from all groups
        {
            let mut groups = self.groups.write();
            for group in groups.values_mut() {
                group.members.retain(|&uid| uid != user_id);
            }
        }

        // Invalidate user's sessions
        {
            let mut sessions = self.sessions.write();
            sessions.retain(|_, session| {
                if session.user_id == user_id {
                    session.is_active = false;
                    false // Remove session
                } else {
                    true // Keep session
                }
            });
        }

        if self.audit_enabled {
            self.log_audit_event("user_deleted", &user.username, Some(&user_id.to_string()));
        }

        info!("Deleted user: {} (UID: {})", user.username, user_id);
        Ok(())
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: UserId) -> UserResult<User> {
        let users = self.users.read();
        users.get(&user_id)
            .cloned()
            .ok_or(UserError::UserNotFound)
    }

    /// Get user by username
    pub fn get_user_by_username(&self, username: &str) -> UserResult<User> {
        let username_map = self.username_map.read();
        let user_id = username_map.get(username)
            .copied()
            .ok_or(UserError::UserNotFound)?;
        
        self.get_user(user_id)
    }

    /// List all users
    pub fn list_users(&self) -> Vec<User> {
        let users = self.users.read();
        users.values().cloned().collect()
    }

    // ==================== Group Management Operations ====================

    /// Create a new group
    pub fn create_group(&self, group_name: &str, description: Option<&str>) -> UserResult<GroupId> {
        let group_name = group_name.to_string();
        
        // Check if group already exists
        {
            let groupname_map = self.groupname_map.read();
            if groupname_map.contains_key(&group_name) {
                return Err(UserError::GroupAlreadyExists);
            }
        }

        let group_id = self.next_group_id.fetch_add(1, Ordering::SeqCst);
        
        let group = Group {
            group_id,
            group_name: group_name.clone(),
            display_name: group_name.clone(),
            description: description.map(|d| d.to_string()),
            gid: group_id,
            members: Vec::new(),
            created_time: self.get_current_time(),
            is_system_group: group_id < 1000,
            privileges: Vec::new(),
            attributes: HashMap::new(),
        };

        // Store group
        {
            let mut groups = self.groups.write();
            groups.insert(group_id, group);
        }

        {
            let mut groupname_map = self.groupname_map.write();
            groupname_map.insert(group_name, group_id);
        }

        if self.audit_enabled {
            self.log_audit_event("group_created", &group_name, Some(&group_id.to_string()));
        }

        info!("Created group: {} (GID: {})", group_name, group_id);
        Ok(group_id)
    }

    /// Add user to group
    pub fn add_user_to_group(&self, user_id: UserId, group_id: GroupId) -> UserResult<()> {
        let mut groups = self.groups.write();
        
        let group = groups.get_mut(&group_id)
            .ok_or(UserError::GroupNotFound)?;

        if !group.members.contains(&user_id) {
            group.members.push(user_id);
        }

        Ok(())
    }

    /// Remove user from group
    pub fn remove_user_from_group(&self, user_id: UserId, group_id: GroupId) -> UserResult<()> {
        let mut groups = self.groups.write();
        
        let group = groups.get_mut(&group_id)
            .ok_or(UserError::GroupNotFound)?;

        group.members.retain(|&uid| uid != user_id);
        Ok(())
    }

    /// Get group by ID
    pub fn get_group(&self, group_id: GroupId) -> UserResult<Group> {
        let groups = self.groups.read();
        groups.get(&group_id)
            .cloned()
            .ok_or(UserError::GroupNotFound)
    }

    /// List all groups
    pub fn list_groups(&self) -> Vec<Group> {
        let groups = self.groups.read();
        groups.values().cloned().collect()
    }

    // ==================== Authentication Operations ====================

    /// Authenticate user with username and password
    pub fn authenticate_user(&self, username: &str, password: &str, ip_address: Option<&str>) -> UserResult<AuthResult> {
        let user = match self.get_user_by_username(username) {
            Ok(user) => user,
            Err(_) => {
                if self.audit_enabled {
                    self.log_audit_event("auth_failed_invalid_user", username, None);
                }
                return Err(UserError::InvalidCredentials);
            }
        };

        // Check if account is enabled
        if !user.is_enabled {
            if self.audit_enabled {
                self.log_audit_event("auth_failed_disabled", username, Some(&user.user_id.to_string()));
            }
            return Err(UserError::AccountLocked);
        }

        // Check if account is locked
        if user.is_locked {
            if self.audit_enabled {
                self.log_audit_event("auth_failed_locked", username, Some(&user.user_id.to_string()));
            }
            return Err(UserError::AccountLocked);
        }

        // Verify password
        let password_valid = if let (Some(hash), Some(salt)) = (&user.password_hash, &user.salt) {
            self.verify_password(password, hash, salt)?
        } else {
            false
        };

        if !password_valid {
            // Increment failed login attempts
            let mut users = self.users.write();
            if let Some(user_to_update) = users.get_mut(&user.user_id) {
                user_to_update.failed_login_attempts += 1;
                user_to_update.last_failed_login = Some(self.get_current_time());
                
                // Lock account if threshold exceeded
                if user_to_update.failed_login_attempts >= self.password_policy.lock().lockout_threshold {
                    user_to_update.is_locked = true;
                    
                    if self.audit_enabled {
                        self.log_audit_event("account_locked", username, Some(&user.user_id.to_string()));
                    }
                }
            }

            if self.audit_enabled {
                self.log_audit_event("auth_failed_invalid_password", username, Some(&user.user_id.to_string()));
            }
            
            return Err(UserError::InvalidCredentials);
        }

        // Password is valid, create session
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        
        // Update last login time
        {
            let mut users = self.users.write();
            if let Some(user_to_update) = users.get_mut(&user.user_id) {
                user_to_update.last_login_time = Some(self.get_current_time());
                user_to_update.failed_login_attempts = 0;
                user_to_update.session_count += 1;
            }
        }

        // Create session
        let session = UserSession {
            session_id,
            user_id: user.user_id,
            username: user.username.clone(),
            login_time: self.get_current_time(),
            last_activity: self.get_current_time(),
            ip_address: ip_address.map(|ip| ip.to_string()),
            user_agent: None,
            is_active: true,
            multi_factor_verified: false,
            privileges: user.privileges.clone(),
            session_data: HashMap::new(),
        };

        {
            let mut sessions = self.sessions.write();
            sessions.insert(session_id, session);
        }

        let auth_result = AuthResult {
            success: true,
            user_id: Some(user.user_id),
            session_id: Some(session_id),
            error_code: UserError::UserNotFound, // No error
            requires_multi_factor: false,
            mfa_secret: None,
            privileges: user.privileges,
            session_timeout: 3600, // 1 hour
        };

        if self.audit_enabled {
            self.log_audit_event("auth_success", username, Some(&user.user_id.to_string()));
        }

        Ok(auth_result)
    }

    /// Verify multi-factor authentication
    pub fn verify_mfa(&self, session_id: u64, code: &str) -> UserResult<bool> {
        let mut sessions = self.sessions.write();
        
        let session = sessions.get_mut(&session_id)
            .ok_or(UserError::SessionExpired)?;

        if !session.is_active {
            return Err(UserError::SessionExpired);
        }

        // Verify TOTP code (simplified implementation)
        // In real implementation, this would use proper TOTP verification
        let code_valid = self.verify_totp_code(session.user_id, code)?;

        if code_valid {
            session.multi_factor_verified = true;
            session.last_activity = self.get_current_time();
            
            if self.audit_enabled {
                self.log_audit_event("mfa_success", &session.username, Some(&session.user_id.to_string()));
            }
            
            Ok(true)
        } else {
            if self.audit_enabled {
                self.log_audit_event("mfa_failed", &session.username, Some(&session.user_id.to_string()));
            }
            
            Err(UserError::MultiFactorFailed)
        }
    }

    /// Invalidate session
    pub fn invalidate_session(&self, session_id: u64) -> UserResult<()> {
        let mut sessions = self.sessions.write();
        
        let session = sessions.remove(&session_id)
            .ok_or(UserError::SessionExpired)?;

        if self.audit_enabled {
            self.log_audit_event("session_invalidated", &session.username, Some(&session.user_id.to_string()));
        }

        Ok(())
    }

    /// Get session information
    pub fn get_session(&self, session_id: u64) -> UserResult<UserSession> {
        let sessions = self.sessions.read();
        sessions.get(&session_id)
            .cloned()
            .ok_or(UserError::SessionExpired)
    }

    // ==================== Security Operations ====================

    /// Change user password
    pub fn change_password(&self, user_id: UserId, old_password: &str, new_password: &str) -> UserResult<()> {
        let mut users = self.users.write();
        
        let user = users.get_mut(&user_id)
            .ok_or(UserError::UserNotFound)?;

        // Verify old password
        if let (Some(hash), Some(salt)) = (&user.password_hash, &user.salt) {
            if !self.verify_password(old_password, hash, salt)? {
                return Err(UserError::InvalidCredentials);
            }
        }

        // Validate new password strength
        self.validate_password(new_password)?;

        // Hash new password
        let (new_hash, new_salt) = self.hash_password(new_password)?;

        user.password_hash = Some(new_hash);
        user.salt = Some(new_salt);
        user.password_last_changed = Some(self.get_current_time());

        if self.audit_enabled {
            self.log_audit_event("password_changed", &user.username, Some(&user_id.to_string()));
        }

        Ok(())
    }

    /// Reset user password (admin function)
    pub fn reset_password(&self, user_id: UserId, new_password: &str) -> UserResult<()> {
        let mut users = self.users.write();
        
        let user = users.get_mut(&user_id)
            .ok_or(UserError::UserNotFound)?;

        // Validate new password strength
        self.validate_password(new_password)?;

        // Hash new password
        let (new_hash, new_salt) = self.hash_password(new_password)?;

        user.password_hash = Some(new_hash);
        user.salt = Some(new_salt);
        user.password_last_changed = Some(self.get_current_time());
        user.is_locked = false; // Unlock account on password reset
        user.failed_login_attempts = 0;

        if self.audit_enabled {
            self.log_audit_event("password_reset", &user.username, Some(&user_id.to_string()));
        }

        Ok(())
    }

    /// Lock user account
    pub fn lock_user(&self, user_id: UserId) -> UserResult<()> {
        let mut users = self.users.write();
        
        let user = users.get_mut(&user_id)
            .ok_or(UserError::UserNotFound)?;

        user.is_locked = true;

        if self.audit_enabled {
            self.log_audit_event("user_locked", &user.username, Some(&user_id.to_string()));
        }

        Ok(())
    }

    /// Unlock user account
    pub fn unlock_user(&self, user_id: UserId) -> UserResult<()> {
        let mut users = self.users.write();
        
        let user = users.get_mut(&user_id)
            .ok_or(UserError::UserNotFound)?;

        user.is_locked = false;
        user.failed_login_attempts = 0;

        if self.audit_enabled {
            self.log_audit_event("user_unlocked", &user.username, Some(&user_id.to_string()));
        }

        Ok(())
    }

    // ==================== Internal Helper Methods ====================

    /// Create system accounts
    fn create_system_accounts(&self) -> UserResult<()> {
        // Create root user
        let root_id = self.create_system_user("root", "System Administrator", None, 0, 0, true)?;
        
        // Create service users
        self.create_system_user("daemon", "System Daemon", None, 1, 1, true)?;
        self.create_system_user("bin", "System Bin", None, 2, 2, true)?;
        self.create_system_user("sys", "System Sys", None, 3, 3, true)?;
        
        info!("Created system accounts");
        Ok(())
    }

    /// Create a system user account
    fn create_system_user(&self, username: &str, display_name: &str, email: Option<&str>, 
                         uid: UserId, gid: GroupId, is_system: bool) -> UserResult<UserId> {
        let username_str = username.to_string();
        
        // Check if user already exists
        {
            let username_map = self.username_map.read();
            if username_map.contains_key(&username_str) {
                return Err(UserError::UserAlreadyExists);
            }
        }

        let password_hash = None; // System users typically don't have passwords
        let salt = None;
        
        let user = User {
            user_id: uid,
            username: username_str,
            display_name: display_name.to_string(),
            email: email.map(|e| e.to_string()),
            password_hash,
            salt,
            primary_group_id: gid,
            secondary_groups: Vec::new(),
            home_directory: format!("/{}", username),
            shell: "/bin/false".to_string(),
            uid,
            gid,
            created_time: self.get_current_time(),
            last_login_time: None,
            password_last_changed: None,
            account_expires: None,
            is_enabled: is_system, // System users are always enabled
            is_locked: false,
            failed_login_attempts: 0,
            last_failed_login: None,
            session_count: 0,
            privileges: if is_system { vec!["system".to_string()] } else { vec!["user".to_string()] },
            attributes: HashMap::new(),
        };

        // Store user
        {
            let mut users = self.users.write();
            users.insert(uid, user);
        }

        {
            let mut username_map = self.username_map.write();
            username_map.insert(username_str, uid);
        }

        info!("Created system user: {} (UID: {})", username, uid);
        Ok(uid)
    }

    /// Create default groups
    fn create_default_groups(&self) -> UserResult<()> {
        // Create standard groups
        self.create_group_internal("root", "System Administrators", 0, true)?;
        self.create_group_internal("users", "Regular Users", 100, false)?;
        self.create_group_internal("wheel", "Wheel Group (Administrators)", 10, false)?;
        self.create_group_internal("daemon", "System Daemons", 1, true)?;
        self.create_group_internal("bin", "System Binaries", 2, true)?;
        self.create_group_internal("sys", "System", 3, true)?;
        
        info!("Created default groups");
        Ok(())
    }

    /// Internal group creation
    fn create_group_internal(&self, group_name: &str, description: &str, gid: GroupId, is_system: bool) -> UserResult<GroupId> {
        let group_name_str = group_name.to_string();
        
        // Check if group already exists
        {
            let groupname_map = self.groupname_map.read();
            if groupname_map.contains_key(&group_name_str) {
                return Err(UserError::GroupAlreadyExists);
            }
        }
        
        let group = Group {
            group_id: gid,
            group_name: group_name_str.clone(),
            display_name: group_name.to_string(),
            description: Some(description.to_string()),
            gid,
            members: Vec::new(),
            created_time: self.get_current_time(),
            is_system_group: is_system,
            privileges: if is_system { vec!["system".to_string()] } else { Vec::new() },
            attributes: HashMap::new(),
        };

        // Store group
        {
            let mut groups = self.groups.write();
            groups.insert(gid, group);
        }

        {
            let mut groupname_map = self.groupname_map.write();
            groupname_map.insert(group_name_str, gid);
        }

        Ok(gid)
    }

    /// Validate password against policy
    fn validate_password(&self, password: &str) -> UserResult<()> {
        let policy = self.password_policy.lock();

        if password.len() < policy.min_length {
            return Err(UserError::PasswordWeak);
        }

        if policy.require_uppercase && !password.chars().any(char::is_uppercase) {
            return Err(UserError::PasswordWeak);
        }

        if policy.require_lowercase && !password.chars().any(char::is_lowercase) {
            return Err(UserError::PasswordWeak);
        }

        if policy.require_digits && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(UserError::PasswordWeak);
        }

        if policy.require_special_chars && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err(UserError::PasswordWeak);
        }

        Ok(())
    }

    /// Hash password with salt
    fn hash_password(&self, password: &str) -> UserResult<(String, String)> {
        // Simplified password hashing - in real implementation would use proper cryptographic hashing
        use core::hash::{Hash, Hasher};
        use core::hash::SipHasher13;
        
        let salt = self.generate_salt();
        let mut hasher = SipHasher13::new();
        hasher.write(password.as_bytes());
        hasher.write(salt.as_bytes());
        let hash = format!("{:x}", hasher.finish());
        
        Ok((hash, salt))
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str, salt: &str) -> UserResult<bool> {
        // Simplified password verification
        let (computed_hash, _) = self.hash_password(password)?;
        Ok(computed_hash == hash)
    }

    /// Generate random salt
    fn generate_salt(&self) -> String {
        use core::time::Duration;
        let time = self.get_current_time();
        let mut hasher = SipHasher13::new();
        hasher.write(&time.to_le_bytes());
        format!("{:x}", hasher.finish())
    }

    /// Verify TOTP code (simplified)
    fn verify_totp_code(&self, user_id: UserId, code: &str) -> UserResult<bool> {
        // Simplified TOTP verification - in real implementation would use proper TOTP
        Ok(code == "123456") // Mock TOTP code for demonstration
    }

    /// Log audit event
    fn log_audit_event(&self, event_type: &str, target: &str, details: Option<&str>) {
        info!("AUDIT: {} - Target: {} - Details: {:?}", event_type, target, details);
    }

    /// Get current time (simplified)
    fn get_current_time(&self) -> u64 {
        // In real implementation, would get time from kernel's time subsystem
        crate::hal::get_current_time()
    }

    // ==================== Statistics and Monitoring ====================

    /// Get user management statistics
    pub fn get_stats(&self) -> UserStats {
        let users = self.users.read();
        let groups = self.groups.read();
        let sessions = self.sessions.read();

        let mut active_users = 0;
        let mut locked_users = 0;
        let mut system_users = 0;

        for user in users.values() {
            if user.is_enabled {
                active_users += 1;
            }
            if user.is_locked {
                locked_users += 1;
            }
            if user.privileges.contains(&"system".to_string()) {
                system_users += 1;
            }
        }

        UserStats {
            total_users: users.len(),
            active_users,
            locked_users,
            system_users,
            total_groups: groups.len(),
            active_sessions: sessions.len(),
            total_login_attempts: 0, // Would be tracked in real implementation
            failed_login_attempts: 0, // Would be tracked in real implementation
            security_events: 0, // Would be tracked in real implementation
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> UserResult<usize> {
        let current_time = self.get_current_time();
        let timeout = 3600; // 1 hour timeout

        let mut sessions = self.sessions.write();
        let mut removed_count = 0;

        sessions.retain(|_, session| {
            if session.is_active && current_time - session.last_activity > timeout {
                session.is_active = false;
                removed_count += 1;
                false // Remove session
            } else {
                true // Keep session
            }
        });

        Ok(removed_count)
    }
}

/// User updates structure for modifying users
#[derive(Debug, Clone)]
pub struct UserUpdates {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub home_directory: Option<String>,
    pub shell: Option<String>,
    pub is_enabled: Option<bool>,
    pub is_locked: Option<bool>,
    pub account_expires: Option<u64>,
}

/// Initialize the global user manager
pub fn init_user_manager() -> UserResult<()> {
    let mut manager_guard = USER_MANAGER.lock();
    
    if manager_guard.is_some() {
        return Err(UserError::UserAlreadyExists);
    }

    let mut manager = UserManager::new();
    manager.init()?;
    
    *manager_guard = Some(manager);
    
    info!("User Manager initialized successfully");
    Ok(())
}

/// Shutdown the global user manager
pub fn shutdown_user_manager() -> UserResult<()> {
    let mut manager_guard = USER_MANAGER.lock();
    
    if let Some(mut manager) = manager_guard.take() {
        manager.shutdown()?;
    }
    
    info!("User Manager shutdown complete");
    Ok(())
}

/// Get the global user manager instance
pub fn get_user_manager() -> Option<&'static Mutex<Option<UserManager>>> {
    Some(&USER_MANAGER)
}