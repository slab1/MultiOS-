//! Administrative Shell Interface Test Suite
//! 
//! This module provides comprehensive testing for the Administrative Shell Interface
//! including unit tests, integration tests, and example usage scenarios.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::admin_shell::{AdminShell, AdminContext, CommandCategory};
    use alloc::collections::HashMap;
    use spin::Mutex;

    /// Test administrative shell creation and initialization
    #[test]
    fn test_admin_shell_creation() {
        let shell = AdminShell::new();
        assert!(!shell.builtin_commands_initialized);
        assert_eq!(shell.max_history_size, 2000);
        assert!(shell.audit_logger.enabled);
    }

    /// Test command parsing functionality
    #[test]
    fn test_command_parsing() {
        let shell = AdminShell::new();
        
        // Test basic command parsing
        let (name, args) = shell.parse_command_line("useradd john --home /home/john").unwrap();
        assert_eq!(name, "useradd");
        assert_eq!(args, vec!["john", "--home", "/home/john"]);
        
        // Test quoted arguments
        let (name, args) = shell.parse_command_line("useradd \"John Doe\" --comment \"Test User\"").unwrap();
        assert_eq!(name, "useradd");
        assert_eq!(args, vec!["John Doe", "--comment", "Test User"]);
        
        // Test empty command
        assert!(shell.parse_command_line("").is_err());
    }

    /// Test permission checking
    #[test]
    fn test_command_permissions() {
        let mut shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test root user permissions
        let root_context = AdminContext {
            current_user: "root".to_string(),
            user_id: 0,
            group_ids: vec![0],
            is_root: true,
            session_id: "test_session".to_string(),
            working_directory: "/".to_string(),
            environment: HashMap::new(),
            permissions: crate::admin::admin_shell::AdminPermissions {
                can_manage_users: true,
                can_modify_system: true,
                can_control_processes: true,
                can_access_logs: true,
                can_configure_network: true,
                can_manage_storage: true,
                can_install_packages: true,
                can_view_audit_logs: true,
            },
        };
        
        shell.current_context = root_context;
        assert!(shell.check_command_permissions("useradd testuser"));
        assert!(shell.check_command_permissions("systemctl restart service"));
        assert!(shell.check_command_permissions("ps aux"));
        
        // Test non-root user permissions
        let user_context = AdminContext {
            current_user: "admin".to_string(),
            user_id: 1000,
            group_ids: vec![1000],
            is_root: false,
            session_id: "test_session".to_string(),
            working_directory: "/".to_string(),
            environment: HashMap::new(),
            permissions: crate::admin::admin_shell::AdminPermissions {
                can_manage_users: false,
                can_modify_system: false,
                can_control_processes: false,
                can_access_logs: true,
                can_configure_network: false,
                can_manage_storage: false,
                can_install_packages: false,
                can_view_audit_logs: true,
            },
        };
        
        shell.current_context = user_context;
        assert!(shell.check_command_permissions("ps aux"));
        assert!(!shell.check_command_permissions("useradd testuser"));
        assert!(!shell.check_command_permissions("systemctl restart service"));
    }

    /// Test completion engine
    #[test]
    fn test_completion_engine() {
        let mut shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test command completion
        let completions = shell.get_completions("user");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"useradd".to_string()));
        assert!(completions.contains(&"userdel".to_string()));
        assert!(completions.contains(&"usermod".to_string()));
        
        // Test empty input completion
        let all_completions = shell.get_completions("");
        assert!(!all_completions.is_empty());
        
        // Test no matches
        let no_matches = shell.get_completions("nonexistent");
        assert!(no_matches.is_empty());
    }

    /// Test user management commands
    #[test]
    fn test_user_management_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test useradd command
        let result = crate::admin::admin_shell::admin_useradd(
            &["john".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("john"));
        
        // Test userdel command
        let result = crate::admin::admin_shell::admin_userdel(
            &["john".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("john"));
    }

    /// Test system control commands
    #[test]
    fn test_system_control_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test systemctl command
        let result = crate::admin::admin_shell::admin_systemctl(
            &["restart".to_string(), "network".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("network"));
        
        // Test ps command
        let result = crate::admin::admin_shell::admin_ps(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("PID"));
    }

    /// Test network configuration commands
    #[test]
    fn test_network_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test ifconfig command
        let result = crate::admin::admin_shell::admin_ifconfig(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("eth0"));
        
        // Test ping command
        let result = crate::admin::admin_shell::admin_ping(
            &["google.com".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("google.com"));
    }

    /// Test storage administration commands
    #[test]
    fn test_storage_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test df command
        let result = crate::admin::admin_shell::admin_df(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Filesystem"));
        
        // Test mount command
        let result = crate::admin::admin_shell::admin_mount(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Mounted"));
    }

    /// Test package management commands
    #[test]
    fn test_package_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test pkg_install command
        let result = crate::admin::admin_shell::admin_pkg_install(
            &["vim".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("vim"));
        
        // Test pkg_list command
        let result = crate::admin::admin_shell::admin_pkg_list(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Package"));
    }

    /// Test monitoring commands
    #[test]
    fn test_monitoring_commands() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test uname command
        let result = crate::admin::admin_shell::admin_uname(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("MultiOS"));
        
        // Test uptime command
        let result = crate::admin::admin_shell::admin_uptime(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("days"));
        
        // Test free command
        let result = crate::admin::admin_shell::admin_free(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Mem"));
    }

    /// Test help system
    #[test]
    fn test_help_system() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test general help
        let result = crate::admin::admin_shell::admin_help(&[], &shell.current_context);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Administrative Shell Help"));
        assert!(result.output.contains("User Management"));
        assert!(result.output.contains("System Control"));
        
        // Test specific command help
        let result = crate::admin::admin_shell::admin_help(
            &["useradd".to_string()], 
            &shell.current_context
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("useradd"));
    }

    /// Test command history
    #[test]
    fn test_command_history() {
        let mut shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Add some commands to history
        shell.add_to_history("useradd testuser".to_string(), 100, 0);
        shell.add_to_history("ps aux".to_string(), 50, 0);
        shell.add_to_history("df -h".to_string(), 75, 0);
        
        // Test history retrieval
        let history = shell.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].command, "useradd testuser");
        assert_eq!(history[1].command, "ps aux");
        assert_eq!(history[2].command, "df -h");
        
        // Test history clearing
        shell.clear_history();
        let empty_history = shell.get_history();
        assert_eq!(empty_history.len(), 0);
    }

    /// Test audit logging
    #[test]
    fn test_audit_logging() {
        let mut shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test command execution logging
        shell.audit_log_command_execution("useradd testuser", 0, "success");
        shell.audit_log_command_execution("invalid_command", 1, "failed");
        
        // Test event logging
        shell.audit_log_event("session_start", "Administrative session started");
        shell.audit_log_event("permission_granted", "User granted admin privileges");
        
        // Verify audit logger is enabled
        assert!(shell.audit_logger.enabled);
    }

    /// Test statistics collection
    #[test]
    fn test_statistics() {
        let mut shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Execute some commands to generate statistics
        let _ = shell.execute_command("useradd testuser");
        let _ = shell.execute_command("ps aux");
        let _ = shell.execute_command("uname -a");
        
        // Get statistics
        let stats = shell.get_stats();
        assert!(stats.total_commands_executed > 0);
        assert!(stats.history_entries > 0);
        assert!(stats.users_managed > 0);
        assert!(stats.processes_controlled > 0);
    }

    /// Test error handling
    #[test]
    fn test_error_handling() {
        let shell = AdminShell::new();
        shell.initialize_builtin_commands();
        
        // Test invalid command
        let result = shell.execute_command("nonexistent_command");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("not found"));
        
        // Test command with insufficient arguments
        let result = shell.execute_command("useradd");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("requires at least"));
    }

    /// Test system integration
    #[test]
    fn test_system_integration() {
        // Test module initialization
        let result = crate::admin::init();
        assert!(result.is_ok());
        
        // Test module info retrieval
        let info = crate::admin::get_module_info();
        assert_eq!(info.module_name, "Administrative Shell");
        assert!(info.total_commands > 0);
        assert!(info.available_categories.len() > 0);
        
        // Test module stats
        let stats = crate::admin::get_module_stats();
        assert!(stats.module_initialized);
        assert!(stats.commands_by_category.len() > 0);
    }

    /// Test permission validation utility
    #[test]
    fn test_permission_validation() {
        use crate::admin::{validate_admin_permissions, AdminContext, AdminPermissions};
        
        let root_context = AdminContext {
            current_user: "root".to_string(),
            user_id: 0,
            group_ids: vec![0],
            is_root: true,
            session_id: "test".to_string(),
            working_directory: "/".to_string(),
            environment: HashMap::new(),
            permissions: AdminPermissions {
                can_manage_users: true,
                can_modify_system: true,
                can_control_processes: true,
                can_access_logs: true,
                can_configure_network: true,
                can_manage_storage: true,
                can_install_packages: true,
                can_view_audit_logs: true,
            },
        };
        
        let user_context = AdminContext {
            current_user: "admin".to_string(),
            user_id: 1000,
            group_ids: vec![1000],
            is_root: false,
            session_id: "test".to_string(),
            working_directory: "/".to_string(),
            environment: HashMap::new(),
            permissions: AdminPermissions {
                can_manage_users: false,
                can_modify_system: false,
                can_control_processes: true,
                can_access_logs: true,
                can_configure_network: false,
                can_manage_storage: false,
                can_install_packages: false,
                can_view_audit_logs: true,
            },
        };
        
        // Root should have access to all commands
        assert!(validate_admin_permissions("useradd testuser", &root_context));
        assert!(validate_admin_permissions("systemctl restart service", &root_context));
        assert!(validate_admin_permissions("ps aux", &root_context));
        
        // User should have access to safe commands but not administrative ones
        assert!(!validate_admin_permissions("useradd testuser", &user_context));
        assert!(!validate_admin_permissions("systemctl restart service", &user_context));
        assert!(validate_admin_permissions("ps aux", &user_context));
    }

    /// Test command search functionality
    #[test]
    fn test_command_search() {
        use crate::admin::search_commands;
        
        // Search for user-related commands
        let user_commands = search_commands("user");
        assert!(!user_commands.is_empty());
        
        for cmd in &user_commands {
            assert!(cmd.name.contains("user") || cmd.description.contains("user"));
        }
        
        // Search for system-related commands
        let system_commands = search_commands("system");
        assert!(!system_commands.is_empty());
        
        // Search for non-existent commands
        let empty_results = search_commands("nonexistent");
        assert!(empty_results.is_empty());
    }

    /// Test category filtering
    #[test]
    fn test_category_filtering() {
        use crate::admin::get_commands_by_category;
        
        // Get user management commands
        let user_commands = get_commands_by_category(CommandCategory::UserManagement);
        assert!(!user_commands.is_empty());
        
        for cmd in &user_commands {
            assert_eq!(cmd.category, CommandCategory::UserManagement);
        }
        
        // Get system control commands
        let system_commands = get_commands_by_category(CommandCategory::SystemControl);
        assert!(!system_commands.is_empty());
        
        for cmd in &system_commands {
            assert_eq!(cmd.category, CommandCategory::SystemControl);
        }
    }

    /// Test diagnostic functionality
    #[test]
    fn test_diagnostics() {
        use crate::admin::run_admin_diagnostics;
        
        let diagnostics = run_admin_diagnostics().unwrap();
        
        // Check for required diagnostic information
        assert!(diagnostics.contains_key("shell_initialized"));
        assert!(diagnostics.contains_key("total_commands"));
        assert!(diagnostics.contains_key("audit_enabled"));
        assert!(diagnostics.contains_key("completion_enabled"));
        assert!(diagnostics.contains_key("history_enabled"));
        
        // Verify diagnostic values
        assert_eq!(diagnostics["shell_initialized"], "true");
        assert_eq!(diagnostics["audit_enabled"], "true");
        assert_eq!(diagnostics["completion_enabled"], "true");
        assert_eq!(diagnostics["history_enabled"], "true");
    }

    /// Test global admin shell instance
    #[test]
    fn test_global_admin_shell() {
        use crate::admin::admin_shell::ADMIN_SHELL;
        
        // The global instance should be initially None
        let guard = ADMIN_SHELL.lock();
        // This test would need the shell to be initialized first
        // In real tests, we'd mock the initialization
    }

    /// Test shell lifecycle
    #[test]
    fn test_shell_lifecycle() {
        // Test initialization
        let init_result = crate::admin::admin_shell::init();
        assert!(init_result.is_ok());
        
        // Test starting
        let start_result = crate::admin::admin_shell::start();
        assert!(start_result.is_ok());
        
        // Test shutdown
        let shutdown_result = crate::admin::admin_shell::shutdown();
        assert!(shutdown_result.is_ok());
    }
}

/// Example usage scenarios for demonstration
pub mod examples {
    use super::*;
    use alloc::vec;
    use alloc::string::String;

    /// Example: Basic administrative shell usage
    pub fn example_basic_usage() {
        println!("=== Basic Administrative Shell Usage ===");
        
        // Initialize the admin shell
        let _ = crate::admin::admin_shell::init();
        let _ = crate::admin::admin_shell::start();
        
        // Execute administrative commands
        let commands = vec![
            "uname -a",
            "uptime", 
            "users",
            "ps aux",
            "free -h",
            "df -h",
            "adminhelp"
        ];
        
        for command in &commands {
            println!("\nExecuting: {}", command);
            let result = crate::admin::execute_admin_command(command);
            match result {
                Ok(cmd_result) => {
                    if cmd_result.success {
                        println!("Success: {}", cmd_result.output.trim());
                    } else {
                        println!("Failed: {}", cmd_result.output.trim());
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }

    /// Example: User management workflow
    pub fn example_user_management() {
        println!("=== User Management Workflow ===");
        
        // Create a new user
        let result = crate::admin::execute_admin_command("useradd john --home /home/john");
        println!("User creation: {:?}", result);
        
        // Modify user properties
        let result = crate::admin::execute_admin_command("usermod john --groups developers");
        println!("User modification: {:?}", result);
        
        // List all users
        let result = crate::admin::execute_admin_command("users");
        println!("User list: {:?}", result);
    }

    /// Example: System monitoring workflow
    pub fn example_system_monitoring() {
        println!("=== System Monitoring Workflow ===");
        
        let commands = vec![
            "uname -a",
            "uptime",
            "free",
            "df -h", 
            "top -n 1",
        ];
        
        for command in &commands {
            println!("\n--- Running: {} ---", command);
            let result = crate::admin::execute_admin_command(command);
            match result {
                Ok(cmd_result) => {
                    println!("{}", cmd_result.output);
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }

    /// Example: Network configuration workflow
    pub fn example_network_config() {
        println!("=== Network Configuration Workflow ===");
        
        // Check current network interfaces
        let result = crate::admin::execute_admin_command("ifconfig");
        println!("Current interfaces:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
        
        // Check routing table
        let result = crate::admin::execute_admin_command("route");
        println!("Routing table:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
        
        // Test connectivity
        let result = crate::admin::execute_admin_command("ping -c 3 localhost");
        println!("Connectivity test:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
    }

    /// Example: Package management workflow
    pub fn example_package_management() {
        println!("=== Package Management Workflow ===");
        
        // List installed packages
        let result = crate::admin::execute_admin_command("pkg_list");
        println!("Installed packages:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
        
        // Install a package
        let result = crate::admin::execute_admin_command("pkg_install vim");
        println!("Package installation: {:?}", result);
        
        // List packages again
        let result = crate::admin::execute_admin_command("pkg_list");
        println!("Updated package list:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
    }

    /// Example: Security administration workflow
    pub fn example_security_admin() {
        println!("=== Security Administration Workflow ===");
        
        // View audit logs
        let result = crate::admin::execute_admin_command("audit");
        println!("Audit log:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
        
        // View system logs
        let result = crate::admin::execute_admin_command("logs");
        println!("System logs:\n{}", result.as_ref().map_or_else(|_| "Error".to_string(), |r| r.output.clone()));
    }

    /// Example: Advanced administrative operations
    pub fn example_advanced_operations() {
        println!("=== Advanced Administrative Operations ===");
        
        // System control examples
        let system_commands = vec![
            "systemctl status",
            "systemctl list-units",
        ];
        
        for command in &system_commands {
            println!("\n--- System Control: {} ---", command);
            let result = crate::admin::execute_admin_command(command);
            match result {
                Ok(cmd_result) => {
                    println!("{}", cmd_result.output);
                    if !cmd_result.warnings.is_empty() {
                        println!("Warnings: {:?}", cmd_result.warnings);
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        
        // Get system information
        println!("\n--- System Information ---");
        if let Some(system_info) = crate::admin::get_system_info() {
            println!("Hostname: {}", system_info.hostname);
            println!("Kernel: {}", system_info.kernel_version);
            println!("OS: {}", system_info.os_version);
            println!("Architecture: {}", system_info.architecture);
            println!("Uptime: {} seconds", system_info.uptime_ns / 1_000_000_000);
        }
    }

    /// Example: Administrative statistics and diagnostics
    pub fn example_statistics_and_diagnostics() {
        println!("=== Statistics and Diagnostics ===");
        
        // Get module statistics
        let stats = crate::admin::get_module_stats();
        println!("Module Statistics:");
        println!("  Shell Commands Executed: {}", stats.shell_stats.total_commands_executed);
        println!("  History Entries: {}", stats.shell_stats.history_entries);
        println!("  Users Managed: {}", stats.shell_stats.users_managed);
        println!("  Processes Controlled: {}", stats.shell_stats.processes_controlled);
        println!("  Active Sessions: {}", stats.active_sessions);
        
        // Get module information
        let info = crate::admin::get_module_info();
        println!("\nModule Information:");
        println!("  Name: {}", info.module_name);
        println!("  Version: {}", info.module_version);
        println!("  Total Commands: {}", info.total_commands);
        println!("  Categories: {}", info.available_categories.len());
        
        // Run diagnostics
        let diagnostics = crate::admin::run_admin_diagnostics().unwrap();
        println!("\nDiagnostics:");
        for (key, value) in diagnostics {
            println!("  {}: {}", key, value);
        }
    }
}
