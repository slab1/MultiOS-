//! Container Security Management
//! 
//! This module provides comprehensive security features including sandboxing,
//! privilege separation, capability management, and security policy enforcement.

use super::*;
use std::collections::HashSet;

/// Security Manager - Handles all container security operations
pub struct SecurityManager {
    security_contexts: Arc<Mutex<HashMap<String, SecurityContext>>>,
    capability_manager: CapabilityManager,
    apparmor_manager: AppArmorManager,
    seccomp_manager: SeccompManager,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            security_contexts: Arc::new(Mutex::new(HashMap::new())),
            capability_manager: CapabilityManager::new(),
            apparmor_manager: AppArmorManager::new(),
            seccomp_manager: SeccompManager::new(),
        }
    }

    /// Validate security configuration for a container
    pub fn validate_security_config(&self, security: &SecurityConfig) -> ContainerResult<()> {
        // Validate privileged mode
        if security.privileged {
            log::warn!("Container will run in privileged mode");
        }

        // Validate capability list
        for capability in &security.capabilities {
            if !self.capability_manager.is_valid_capability(capability) {
                return Err(ContainerError::SecurityViolation(
                    format!("Invalid capability: {}", capability)
                ));
            }
        }

        // Validate AppArmor profile
        if let Some(ref profile) = security.apparmor_profile {
            if !self.apparmor_manager.profile_exists(profile) {
                return Err(ContainerError::SecurityViolation(
                    format!("AppArmor profile not found: {}", profile)
                ));
            }
        }

        // Validate seccomp profile
        if let Some(ref profile) = security.seccomp_profile {
            if !self.seccomp_manager.profile_exists(profile) {
                return Err(ContainerError::SecurityViolation(
                    format!("Seccomp profile not found: {}", profile)
                ));
            }
        }

        Ok(())
    }

    /// Setup security context for a container
    pub async fn setup_security_context(&self, container_id: &str, security: &SecurityConfig) -> ContainerResult<()> {
        // Create security context
        let context = SecurityContext {
            capabilities: security.capabilities.clone(),
            apparmor_profile: security.apparmor_profile.clone(),
            seccomp_profile: security.seccomp_profile.clone(),
            uid_map: if security.user_namespace {
                Some("0 100000 65536".to_string())
            } else {
                None
            },
            gid_map: if security.user_namespace {
                Some("0 100000 65536".to_string())
            } else {
                None
            },
        };

        // Store context
        {
            let mut contexts = self.security_contexts.lock().unwrap();
            contexts.insert(container_id.to_string(), context.clone());
        }

        // Apply security configuration
        if security.privileged {
            self.apply_privileged_mode(container_id).await?;
        } else {
            self.apply_security_restrictions(container_id, security).await?;
        }

        Ok(())
    }

    /// Prepare security context for container startup
    pub async fn prepare_security_context(&self, container_id: &str) -> ContainerResult<()> {
        let contexts = self.security_contexts.lock().unwrap();
        let context = contexts.get(container_id)
            .ok_or(ContainerError::NotFound(format!("Security context for container {} not found", container_id)))?;

        // Apply capabilities
        if !context.capabilities.is_empty() {
            self.capability_manager.apply_capabilities(container_id, &context.capabilities).await?;
        }

        // Apply AppArmor profile
        if let Some(ref profile) = context.apparmor_profile {
            self.apparmor_manager.apply_profile(container_id, profile).await?;
        }

        // Apply seccomp profile
        if let Some(ref profile) = context.seccomp_profile {
            self.seccomp_manager.apply_profile(container_id, profile).await?;
        }

        Ok(())
    }

    /// Setup user namespace mapping
    pub async fn setup_user_namespace(&self, container_id: &str, uid_map: Option<String>, gid_map: Option<String>) -> ContainerResult<()> {
        // This would involve writing the UID/GID mappings to /proc/[pid]/{uid_map,gid_map}
        // and setting up the appropriate permissions
        
        if let Some(uid_map) = uid_map {
            self.write_id_mapping("uid_map", &uid_map).await?;
        }

        if let Some(gid_map) = gid_map {
            self.write_id_mapping("gid_map", &gid_map).await?;
        }

        Ok(())
    }

    /// Apply security restrictions to container
    pub async fn apply_security_restrictions(&self, container_id: &str, security: &SecurityConfig) -> ContainerResult<()> {
        // Set no_new_privileges if requested
        if security.no_new_privileges {
            self.set_no_new_privileges(container_id).await?;
        }

        // Setup user namespace if requested
        if security.user_namespace {
            self.setup_user_namespace(container_id, 
                Some("0 100000 65536".to_string()), 
                Some("0 100000 65536".to_string())).await?;
        }

        // Apply capability restrictions
        self.apply_capability_restrictions(container_id, security).await?;

        Ok(())
    }

    /// Apply privileged mode (all capabilities, minimal restrictions)
    pub async fn apply_privileged_mode(&self, container_id: &str) -> ContainerResult<()> {
        log::warn!("Container {} is running in privileged mode - this is a security risk!", container_id);

        // In privileged mode, we minimize restrictions
        // Note: This is generally not recommended for production use

        Ok(())
    }

    /// Set up no_new_privileges flag
    async fn set_no_new_privileges(&self, container_id: &str) -> ContainerResult<()> {
        // This would set the PR_SET_NO_NEW_PRIVS flag on the container process
        unsafe {
            let result = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            if result == -1 {
                return Err(ContainerError::SecurityViolation(
                    format!("Failed to set no_new_privileges for container {}", container_id)
                ));
            }
        }

        Ok(())
    }

    /// Apply capability restrictions
    async fn apply_capability_restrictions(&self, container_id: &str, security: &SecurityConfig) -> ContainerResult<()> {
        // If no specific capabilities are requested, use a minimal set
        let capabilities = if security.capabilities.is_empty() {
            vec!["CHOWN".to_string(), "SETGID".to_string(), "SETUID".to_string()]
        } else {
            security.capabilities.clone()
        };

        self.capability_manager.apply_capabilities(container_id, &capabilities).await?;
        Ok(())
    }

    /// Get security context for a container
    pub fn get_security_context(&self, container_id: &str) -> ContainerResult<SecurityContext> {
        let contexts = self.security_contexts.lock().unwrap();
        contexts.get(container_id)
            .cloned()
            .ok_or(ContainerError::NotFound(format!("Security context for container {} not found", container_id)))
    }

    /// Remove security context
    pub fn remove_security_context(&self, container_id: &str) -> ContainerResult<()> {
        let mut contexts = self.security_contexts.lock().unwrap();
        contexts.remove(container_id);
        Ok(())
    }

    // Private helper methods

    async fn write_id_mapping(&self, map_type: &str, mapping: &str) -> ContainerResult<()> {
        let path = format!("/proc/{}/{}", unsafe { libc::getpid() }, map_type);
        std::fs::write(&path, mapping)
            .map_err(|e| ContainerError::System(format!("Failed to write {}: {}", map_type, e)))?;
        Ok(())
    }
}

/// Capability Manager - Handles Linux capability management
pub struct CapabilityManager {
    allowed_capabilities: HashSet<String>,
    default_denylist: HashSet<String>,
}

impl CapabilityManager {
    fn new() -> Self {
        let mut allowed_capabilities = HashSet::new();
        let mut default_denylist = HashSet::new();

        // Define allowed capabilities for containers
        allowed_capabilities.insert("CHOWN".to_string());
        allowed_capabilities.insert("DAC_OVERRIDE".to_string());
        allowed_capabilities.insert("FOWNER".to_string());
        allowed_capabilities.insert("FSETID".to_string());
        allowed_capabilities.insert("KILL".to_string());
        allowed_capabilities.insert("SETGID".to_string());
        allowed_capabilities.insert("SETUID".to_string());
        allowed_capabilities.insert("SETPCAP".to_string());
        allowed_capabilities.insert("NET_BIND_SERVICE".to_string());
        allowed_capabilities.insert("SYS_CHROOT".to_string());

        // Define capabilities that should be denied by default
        default_denylist.insert("AUDIT_CONTROL".to_string());
        default_denylist.insert("AUDIT_WRITE".to_string());
        default_denylist.insert("BLOCK_SUSPEND".to_string());
        default_denylist.insert("DAC_READ_SEARCH".to_string());
        default_denylist.insert("EXECVE".to_string());
        default_denylist.insert("IPC_LOCK".to_string());
        default_denylist.insert("IPC_OWNER".to_string());
        default_denylist.insert("LINUX_IMMUTABLE".to_string());
        default_denylist.insert("MAC_ADMIN".to_string());
        default_denylist.insert("MAC_OVERRIDE".to_string());
        default_denylist.insert("MKNOD".to_string());
        default_denylist.insert("NET_ADMIN".to_string());
        default_denylist.insert("NET_RAW".to_string());
        default_denylist.insert("PERFMON".to_string());
        default_denylist.insert("SETFCAP".to_string());
        default_denylist.insert("SETPCAP".to_string());
        default_denylist.insert("SYS_ADMIN".to_string());
        default_denylist.insert("SYS_BOOT".to_string());
        default_denylman.insert("SYSLOG".to_string());
        default_denylist.insert("SYS_MODULE".to_string());
        default_denylist.insert("SYS_NICE".to_string());
        default_denylist.insert("SYS_PACCT".to_string());
        default_denyllist.insert("SYS_PTRACE".to_string());
        default_denylist.insert("SYS_RAWIO".to_string());
        default_denylist.insert("SYS_RESOURCE".to_string());
        default_denyllist.insert("SYS_TIME".to_string());
        default_denylist.insert("SYS_TTY_CONFIG".to_string());
        default_denylist.insert("WAKE_ALARM".to_string());

        Self {
            allowed_capabilities,
            default_denylist,
        }
    }

    /// Check if a capability is valid
    pub fn is_valid_capability(&self, capability: &str) -> bool {
        self.allowed_capabilities.contains(capability)
    }

    /// Apply capabilities to a container
    pub async fn apply_capabilities(&self, container_id: &str, capabilities: &[String]) -> ContainerResult<()> {
        let cgroup_path = format!("/sys/fs/cgroup/multios/cpu/{}", container_id);
        
        // Create capability bounding set
        let mut effective_capabilities = HashSet::new();
        
        // Add requested capabilities
        for cap in capabilities {
            if self.allowed_capabilities.contains(cap) {
                effective_capabilities.insert(cap.clone());
            } else {
                log::warn!("Capability {} not allowed for container {}", cap, container_id);
            }
        }

        // Remove denied capabilities
        let mut final_capabilities = Vec::new();
        for cap in &effective_capabilities {
            if !self.default_denylist.contains(cap) {
                final_capabilities.push(cap.clone());
            } else {
                log::warn!("Capability {} denied for container {}", cap, container_id);
            }
        }

        // Apply capabilities using libcap or direct kernel calls
        self.set_container_capabilities(container_id, &final_capabilities).await?;

        Ok(())
    }

    async fn set_container_capabilities(&self, container_id: &str, capabilities: &[String]) -> ContainerResult<()> {
        // This would use libcap or similar to set the capability bounding set
        // For now, we'll write to a mock capability file
        let cap_file = format!("/sys/fs/cgroup/multios/caps/{}/capabilities", container_id);
        
        // Ensure directory exists
        let dir_path = std::path::Path::new(&cap_file).parent().unwrap();
        std::fs::create_dir_all(dir_path)
            .map_err(|e| ContainerError::System(format!("Failed to create caps directory: {}", e)))?;

        let capability_str = capabilities.join(",");
        std::fs::write(&cap_file, capability_str)
            .map_err(|e| ContainerError::System(format!("Failed to write capabilities: {}", e)))?;

        Ok(())
    }
}

/// AppArmor Manager - Handles AppArmor profile management
pub struct AppArmorManager {
    profile_dir: PathBuf,
}

impl AppArmorManager {
    fn new() -> Self {
        Self {
            profile_dir: PathBuf::from("/etc/multios/apparmor.d"),
        }
    }

    /// Check if an AppArmor profile exists
    pub fn profile_exists(&self, profile_name: &str) -> bool {
        let profile_path = self.profile_dir.join(profile_name);
        profile_path.exists()
    }

    /// Apply an AppArmor profile to a container
    pub async fn apply_profile(&self, container_id: &str, profile_name: &str) -> ContainerResult<()> {
        if !self.profile_exists(profile_name) {
            return Err(ContainerError::SecurityViolation(
                format!("AppArmor profile {} not found", profile_name)
            ));
        }

        // Load and apply the profile
        let profile_path = self.profile_dir.join(profile_name);
        
        // Write container-specific profile
        let container_profile = self.generate_container_profile(container_id, &profile_path).await?;
        
        // Apply the profile using aa-exec or similar
        self.load_and_enforce_profile(container_id, &container_profile).await?;

        Ok(())
    }

    async fn generate_container_profile(&self, container_id: &str, base_profile: &PathBuf) -> ContainerResult<PathBuf> {
        // Read base profile
        let content = std::fs::read_to_string(base_profile)
            .map_err(|e| ContainerError::System(format!("Failed to read AppArmor profile: {}", e)))?;

        // Modify profile for container
        let container_profile_content = format!(
            "# Container profile for {}\n{}",
            container_id,
            content.replace("multios-container", &format!("multios-container-{}", container_id))
        );

        // Write container-specific profile
        let container_profile_path = self.profile_dir.join(format!("multios-container-{}", container_id));
        std::fs::write(&container_profile_path, container_profile_content)
            .map_err(|e| ContainerError::System(format!("Failed to write container profile: {}", e)))?;

        Ok(container_profile_path)
    }

    async fn load_and_enforce_profile(&self, container_id: &str, profile_path: &PathBuf) -> ContainerResult<()> {
        unsafe {
            // Load the profile
            let load_cmd = format!("apparmor_parser --replace {}\n", profile_path.display());
            let result = libc::system(load_cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                log::warn!("Failed to load AppArmor profile for container {} (may not be available)", container_id);
            }

            // Apply the profile to the process
            let profile_name = format!("multios-container-{}", container_id);
            let profile_bytes = profile_name.as_bytes();
            
            let result = libc::aa_change_profile(profile_bytes.as_ptr() as *const libc::c_char);
            if result == -1 {
                log::warn!("Failed to change AppArmor profile for container {}", container_id);
            }
        }

        Ok(())
    }
}

/// Seccomp Manager - Handles seccomp filter management
pub struct SeccompManager {
    profiles_dir: PathBuf,
}

impl SeccompManager {
    fn new() -> Self {
        Self {
            profiles_dir: PathBuf::from("/etc/multios/seccomp.d"),
        }
    }

    /// Check if a seccomp profile exists
    pub fn profile_exists(&self, profile_name: &str) -> bool {
        let profile_path = self.profiles_dir.join(format!("{}.json", profile_name));
        profile_path.exists()
    }

    /// Apply a seccomp profile to a container
    pub async fn apply_profile(&self, container_id: &str, profile_name: &str) -> ContainerResult<()> {
        if !self.profile_exists(profile_name) {
            return Err(ContainerError::SecurityViolation(
                format!("Seccomp profile {} not found", profile_name)
            ));
        }

        let profile_path = self.profiles_dir.join(format!("{}.json", profile_name));
        let profile_content = std::fs::read_to_string(&profile_path)
            .map_err(|e| ContainerError::System(format!("Failed to read seccomp profile: {}", e)))?;

        let seccomp_filter: SeccompFilter = serde_json::from_str(&profile_content)
            .map_err(|e| ContainerError::System(format!("Failed to parse seccomp profile: {}", e)))?;

        // Apply seccomp filter
        self.apply_seccomp_filter(container_id, &seccomp_filter).await?;

        Ok(())
    }

    async fn apply_seccomp_filter(&self, container_id: &str, filter: &SeccompFilter) -> ContainerResult<()> {
        // Serialize filter to binary format and apply using prctl
        let filter_data = self.serialize_seccomp_filter(filter)?;
        
        unsafe {
            let result = libc::prctl(
                libc::PR_SET_SECCOMP,
                libc::SECCOMP_MODE_FILTER,
                filter_data.as_ptr(),
                0,
                0
            );

            if result == -1 {
                return Err(ContainerError::SecurityViolation(
                    format!("Failed to apply seccomp filter for container {}", container_id)
                ));
            }
        }

        Ok(())
    }

    fn serialize_seccomp_filter(&self, filter: &SeccompFilter) -> Result<Vec<u8>, ContainerError> {
        // This would serialize the seccomp filter to the binary format expected by the kernel
        // Simplified implementation
        let mut data = Vec::new();
        
        // Write filter structure (simplified)
        for rule in &filter.default_action {
            data.extend_from_slice(&rule.action.to_le_bytes());
            data.extend_from_slice(&rule.syscall_number.to_le_bytes());
            data.extend_from_slice(&rule.args_count.to_le_bytes());
            
            for arg in &rule.args {
                data.extend_from_slice(&arg.value.to_le_bytes());
                data.extend_from_slice(&arg.mask.to_le_bytes());
                data.extend_from_slice(&arg.op.to_le_bytes());
            }
        }

        Ok(data)
    }
}

/// Security context information
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub capabilities: Vec<String>,
    pub apparmor_profile: Option<String>,
    pub seccomp_profile: Option<String>,
    pub uid_map: Option<String>,
    pub gid_map: Option<String>,
}

/// Seccomp filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompFilter {
    pub default_action: Vec<SeccompRule>,
    pub architectures: Vec<String>,
}

/// Seccomp rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompRule {
    pub action: u32,
    pub syscall_number: i32,
    pub args_count: u8,
    pub args: Vec<SeccompArg>,
}

/// Seccomp argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompArg {
    pub index: u8,
    pub value: u64,
    pub mask: u64,
    pub op: u8,
}

/// Security audit information
#[derive(Debug, Clone)]
pub struct SecurityAudit {
    pub container_id: String,
    pub timestamp: SystemTime,
    pub event_type: SecurityEvent,
    pub details: HashMap<String, String>,
}

/// Security event types
#[derive(Debug, Clone)]
pub enum SecurityEvent {
    PrivilegeEscalationAttempt,
    UnauthorizedCapability,
    AppArmorViolation,
    SeccompViolation,
    NamespaceBreach,
    FilesystemAccessViolation,
    NetworkPolicyViolation,
}