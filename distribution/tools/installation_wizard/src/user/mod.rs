pub mod error;

use crate::core::config::UserConfig;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// User manager for handling user account creation and management
pub struct UserManager;

impl UserManager {
    pub fn new() -> Self {
        Self
    }

    /// Create user accounts based on provided configurations
    pub async fn create_users(&self, user_configs: Vec<UserConfig>) -> Result<()> {
        log::info!("Creating user accounts");
        
        for user_config in user_configs {
            self.create_user_account(&user_config).await?;
            
            if user_config.is_admin {
                self.add_user_to_admin_group(&user_config.username).await?;
            }
            
            // Set password if provided
            if let Some(password) = &user_config.password {
                self.set_user_password(&user_config.username, password).await?;
            }
            
            // Configure auto-login if requested
            if user_config.auto_login {
                self.configure_auto_login(&user_config.username).await?;
            }
        }
        
        // Configure default user settings
        self.configure_default_settings().await?;
        
        log::info!("User account creation completed");
        Ok(())
    }

    /// Create individual user account
    async fn create_user_account(&self, user_config: &UserConfig) -> Result<()> {
        log::info!("Creating user account: {}", user_config.username);
        
        // Use useradd to create the user account
        let mut args = vec!["-m", "-s", "/bin/bash", &user_config.username];
        
        if let Some(full_name) = &user_config.full_name {
            args.push("-c");
            args.push(full_name);
        }
        
        let output = Command::new("useradd")
            .args(&args)
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to create user '{}': {}", user_config.username,
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // Create user's home directory structure
        self.create_user_home_structure(&user_config.username).await?;
        
        // Set up user's shell configuration
        self.setup_user_shell_config(&user_config.username).await?;
        
        // Set up user's groups
        self.add_user_to_groups(&user_config.username, &["users"]).await?;
        
        Ok(())
    }

    /// Add user to admin/sudo group
    async fn add_user_to_admin_group(&self, username: &str) -> Result<()> {
        log::info!("Adding user '{}' to admin group", username);
        
        let output = Command::new("usermod")
            .args(&["-aG", "sudo", username])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to add user '{}' to sudo group: {}", username,
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Add user to additional groups
    async fn add_user_to_groups(&self, username: &str, groups: &[&str]) -> Result<()> {
        for group in groups {
            let output = Command::new("usermod")
                .args(&["-aG", group, username])
                .output()?;
                
            if !output.status.success() {
                log::warn!("Failed to add user '{}' to group '{}': {}", username, group,
                    String::from_utf8_lossy(&output.stderr));
            }
        }
        
        Ok(())
    }

    /// Set user password
    async fn set_user_password(&self, username: &str, password: &str) -> Result<()> {
        log::info!("Setting password for user '{}'", username);
        
        // Use chpasswd to set password
        let mut child = tokio::process::Command::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(format!("{}:{}", username, password).as_bytes()).await?;
        }
        
        let output = child.wait_with_output().await?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to set password for user '{}': {}", username,
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }

    /// Configure auto-login for user
    async fn configure_auto_login(&self, username: &str) -> Result<()> {
        log::info!("Configuring auto-login for user '{}'", username);
        
        // Configure auto-login for different display managers
        self.configure_lightdm_auto_login(username).await?;
        self.configure_gdm_auto_login(username).await?;
        self.configure_sddm_auto_login(username).await?;
        
        Ok(())
    }

    /// Configure auto-login for LightDM
    async fn configure_lightdm_auto_login(&self, username: &str) -> Result<()> {
        let lightdm_config = format!(
            "[Seat:*]\n\
             autologin-user={}\n\
             autologin-user-timeout=0",
            username
        );
        
        let config_path = "/etc/lightdm/lightdm.conf.d/50-autologin.conf";
        std::fs::create_dir_all(std::path::Path::new(config_path).parent().unwrap())?;
        std::fs::write(config_path, lightdm_config)?;
        
        Ok(())
    }

    /// Configure auto-login for GDM
    async fn configure_gdm_auto_login(&self, username: &str) -> Result<()> {
        let gdm_config = format!(
            "[daemon]\n\
             AutomaticLoginEnable=true\n\
             AutomaticLogin={}",
            username
        );
        
        let config_path = "/etc/gdm3/daemon.conf";
        if std::path::Path::new(config_path).exists() {
            std::fs::write(config_path, gdm_config)?;
        }
        
        Ok(())
    }

    /// Configure auto-login for SDDM
    async fn configure_sddm_auto_login(&self, username: &str) -> Result<()> {
        let sddm_config = format!(
            "[Autologin]\n\
             User={}\n\
             Session=plasma",
            username
        );
        
        let config_path = "/etc/sddm.conf.d/autologin.conf";
        std::fs::create_dir_all(std::path::Path::new(config_path).parent().unwrap())?;
        std::fs::write(config_path, sddm_config)?;
        
        Ok(())
    }

    /// Create user's home directory structure
    async fn create_user_home_structure(&self, username: &str) -> Result<()> {
        let home_dir = format!("/home/{}", username);
        
        // Create standard directories
        let directories = vec![
            "Desktop",
            "Documents", 
            "Downloads",
            "Music",
            "Pictures",
            "Public",
            "Templates",
            "Videos",
            ".config",
            ".local/bin",
            ".local/share",
        ];
        
        for dir in directories {
            let dir_path = format!("{}/{}", home_dir, dir);
            std::fs::create_dir_all(dir_path)?;
        }
        
        // Set proper ownership
        let output = Command::new("chown")
            .args(&["-R", &format!("{}:{}", username, username), &home_dir])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to set ownership for user home directory: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Set up user's shell configuration files
    async fn setup_user_shell_config(&self, username: &str) -> Result<()> {
        let home_dir = format!("/home/{}", username);
        
        // Create .bashrc
        let bashrc_content = r#"# MultiOS default bash configuration

# Enable color output
alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias fgrep='fgrep --color=auto'
alias egrep='egrep --color=auto'

# Add local bin to PATH
export PATH="$HOME/.local/bin:$PATH"

# PS1 prompt
export PS1='\u@\h:\w\$ '

# Auto-complete
if [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
fi
"#;
        
        std::fs::write(format!("{}/.bashrc", home_dir), bashrc_content)?;
        
        // Create .profile
        let profile_content = r#"# MultiOS default profile

# Add local bin to PATH
if [ -d "$HOME/.local/bin" ] ; then
    PATH="$HOME/.local/bin:$PATH"
fi
"#;
        
        std::fs::write(format!("{}/.profile", home_dir), profile_content)?;
        
        // Set proper permissions
        let output = Command::new("chown")
            .args(&["-R", &format!("{}:{}", username, username), &home_dir])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to set ownership for shell config files: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Configure default system settings for users
    async fn configure_default_settings(&self) -> Result<()> {
        log::info!("Configuring default user settings");
        
        // Configure system-wide user settings
        self.configure_login_manager().await?;
        self.configure_user_limits().await?;
        self.configure_shell_defaults().await?;
        
        Ok(())
    }

    /// Configure login manager settings
    async fn configure_login_manager(&self) -> Result<()> {
        // Enable and configure display manager
        let services = ["lightdm", "gdm3", "sddm"];
        
        for service in &services {
            let output = Command::new("systemctl")
                .args(&["enable", service])
                .output()?;
                
            if output.status.success() {
                // Found an enabled service
                Command::new("systemctl")
                    .args(&["set-default", "graphical.target"])
                    .output()?;
                break;
            }
        }
        
        Ok(())
    }

    /// Configure user resource limits
    async fn configure_user_limits(&self) -> Result<()> {
        let limits_config = r#"# MultiOS default user limits
* soft nofile 65536
* hard nofile 65536
* soft nproc 32768
* hard nproc 32768
"#;
        
        let limits_path = "/etc/security/limits.d/99-multios.conf";
        std::fs::create_dir_all(std::path::Path::new(limits_path).parent().unwrap())?;
        std::fs::write(limits_path, limits_config)?;
        
        Ok(())
    }

    /// Configure shell defaults
    async fn configure_shell_defaults(&self) -> Result<()> {
        // Configure global shell settings
        let shell_config = r#"# MultiOS global shell configuration

# Enable auto-completion
if [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
fi

# Set default editor
export EDITOR=nano

# Set umask
umask 022
"#;
        
        std::fs::write("/etc/bash.bashrc", shell_config)?;
        
        Ok(())
    }

    /// Validate username
    pub fn validate_username(username: &str) -> Result<()> {
        if username.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }
        
        if username.len() > 32 {
            return Err(anyhow!("Username too long (max 32 characters)"));
        }
        
        if username.starts_with('-') {
            return Err(anyhow!("Username cannot start with '-'"));
        }
        
        if username.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
            return Err(anyhow!("Username contains invalid characters"));
        }
        
        // Check if username is reserved
        let reserved_users = ["root", "daemon", "bin", "sys", "sync", "games", "man", "lp", 
                             "mail", "news", "uucp", "proxy", "www-data", "backup", "list", 
                             "irc", "gnats", "nobody", "systemd", "syslog"];
        
        if reserved_users.contains(&username) {
            return Err(anyhow!("Username '{}' is reserved", username));
        }
        
        Ok(())
    }

    /// Validate password
    pub fn validate_password(password: &str) -> Result<()> {
        if password.is_empty() {
            return Err(anyhow!("Password cannot be empty"));
        }
        
        if password.len() < 6 {
            return Err(anyhow!("Password too short (minimum 6 characters)"));
        }
        
        if password.len() > 128 {
            return Err(anyhow!("Password too long (maximum 128 characters)"));
        }
        
        // Check for common weak patterns
        let weak_patterns = ["123", "password", "qwerty", "abc"];
        let password_lower = password.to_lowercase();
        
        for pattern in &weak_patterns {
            if password_lower.contains(pattern) {
                log::warn!("Password may be weak (contains common pattern: {})", pattern);
                break;
            }
        }
        
        Ok(())
    }

    /// Get recommended user configuration
    pub fn get_recommended_config(username: &str) -> UserConfig {
        UserConfig {
            username: username.to_string(),
            full_name: Some(format!("{} User", username)),
            password: None, // User should set their own password
            is_admin: false,
            auto_login: false,
        }
    }
}

/// User information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub full_name: Option<String>,
    pub home_directory: String,
    pub shell: String,
    pub groups: Vec<String>,
    pub is_admin: bool,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

/// User group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_name: String,
    pub gid: u32,
    pub members: Vec<String>,
    pub is_system_group: bool,
}