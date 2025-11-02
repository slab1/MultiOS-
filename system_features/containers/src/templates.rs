//! Educational Container Templates
//! 
//! This module provides pre-configured container templates designed for educational
//! environments, supporting various learning scenarios and skill levels.

use super::*;
use std::collections::HashMap;

/// Template Manager - Handles educational container templates
pub struct TemplateManager {
    template_root: PathBuf,
    educational_contexts: Arc<Mutex<HashMap<String, EducationalContext>>>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        let template_root = PathBuf::from("/etc/multios/container-templates");
        
        Self {
            template_root,
            educational_contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize template manager and load default templates
    pub async fn initialize(&self) -> ContainerResult<()> {
        // Ensure template directory exists
        std::fs::create_dir_all(&self.template_root)
            .map_err(|e| ContainerError::System(format!("Failed to create template directory: {}", e)))?;

        // Load educational contexts
        self.load_educational_contexts().await?;

        // Create default templates if they don't exist
        self.create_default_templates().await?;

        Ok(())
    }

    /// Get list of available templates
    pub async fn list_templates(&self) -> ContainerResult<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        if self.template_root.exists() {
            for entry in std::fs::read_dir(&self.template_root)
                .map_err(|e| ContainerError::System(format!("Failed to read template directory: {}", e)))? {
                
                let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(template_name) = path.file_stem().and_then(|s| s.to_str()) {
                        let template_config = self.load_template(template_name).await?;
                        let context = self.get_educational_context(template_name).await?;
                        
                        templates.push(TemplateInfo {
                            id: template_name.to_string(),
                            name: template_config.name,
                            description: template_config.description,
                            difficulty_level: context.difficulty_level,
                            estimated_duration: context.estimated_duration,
                            learning_objectives: context.learning_objectives,
                            prerequisites: context.prerequisites,
                        });
                    }
                }
            }
        }

        Ok(templates)
    }

    /// Get template by ID
    pub async fn get_template(&self, template_id: &str) -> ContainerResult<TemplateConfig> {
        self.load_template(template_id).await
    }

    /// Create container from template with customization
    pub async fn create_from_template(&self, template_id: &str, customizations: TemplateCustomizations) -> ContainerResult<ContainerConfig> {
        // Load template configuration
        let mut template_config = self.load_template(template_id).await?;

        // Apply customizations
        self.apply_customizations(&mut template_config, &customizations)?;

        // Convert to container configuration
        let container_config = self.template_to_container_config(&template_config, template_id)?;

        Ok(container_config)
    }

    /// Get educational context for a template
    pub async fn get_educational_context(&self, template_id: &str) -> ContainerResult<EducationalContext> {
        let contexts = self.educational_contexts.lock().unwrap();
        contexts.get(template_id)
            .cloned()
            .ok_or(ContainerError::TemplateError(format!("Educational context for template {} not found", template_id)))
    }

    // Private helper methods

    async fn load_template(&self, template_id: &str) -> ContainerResult<TemplateConfig> {
        let template_path = self.template_root.join(format!("{}.json", template_id));
        
        if !template_path.exists() {
            return Err(ContainerError::TemplateError(format!("Template {} not found", template_id)));
        }

        let content = std::fs::read_to_string(&template_path)
            .map_err(|e| ContainerError::System(format!("Failed to read template: {}", e)))?;

        let template_config: TemplateConfig = serde_json::from_str(&content)
            .map_err(|e| ContainerError::System(format!("Failed to parse template: {}", e)))?;

        Ok(template_config)
    }

    async fn load_educational_contexts(&self) -> ContainerResult<()> {
        let contexts_path = self.template_root.join("contexts");
        
        if contexts_path.exists() {
            for entry in std::fs::read_dir(&contexts_path)
                .map_err(|e| ContainerError::System(format!("Failed to read contexts directory: {}", e)))? {
                
                let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(context_id) = path.file_stem().and_then(|s| s.to_str()) {
                        let content = std::fs::read_to_string(&path)
                            .map_err(|e| ContainerError::System(format!("Failed to read context: {}", e)))?;

                        let context: EducationalContext = serde_json::from_str(&content)
                            .map_err(|e| ContainerError::System(format!("Failed to parse context: {}", e)))?;

                        {
                            let mut contexts = self.educational_contexts.lock().unwrap();
                            contexts.insert(context_id.to_string(), context);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn create_default_templates(&self) -> ContainerResult<()> {
        // Create beginner programming template
        self.create_beginner_programming_template().await?;
        
        // Create system administration template
        self.create_system_admin_template().await?;
        
        // Create network security template
        self.create_network_security_template().await?;
        
        // Create web development template
        self.create_web_development_template().await?;
        
        // Create database administration template
        self.create_database_admin_template().await?;

        Ok(())
    }

    async fn create_beginner_programming_template(&self) -> ContainerResult<()> {
        let template_id = "beginner-programming";
        let template_config = TemplateConfig {
            name: "Beginner Programming Environment".to_string(),
            description: "Complete development environment for learning programming fundamentals".to_string(),
            difficulty_level: DifficultyLevel::Beginner,
            estimated_duration: Duration::from_hours(2),
            learning_objectives: vec![
                "Understand basic programming concepts".to_string(),
                "Learn to write and run simple programs".to_string(),
                "Practice debugging techniques".to_string(),
                "Understand file organization".to_string(),
            ],
            prerequisites: vec!["Basic computer literacy".to_string()],
            image_base: "ubuntu:20.04".to_string(),
            entrypoint: Some(vec!["/bin/bash".to_string()]),
            command: None,
            environment: vec![
                ("LANG".to_string(), "C.UTF-8".to_string()),
                ("PS1".to_string(), "\\u@\\h:\\w\\$ ".to_string()),
            ],
            packages: vec![
                "build-essential".to_string(),
                "python3".to_string(),
                "python3-pip".to_string(),
                "git".to_string(),
                "vim".to_string(),
                "curl".to_string(),
                "wget".to_string(),
                "tree".to_string(),
                "nano".to_string(),
            ],
            network_mode: NetworkMode::Bridge,
            resource_limits: ResourceLimits {
                cpu_cores: Some(1.0),
                memory_bytes: Some(1024 * 1024 * 1024), // 1GB
                disk_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
                network_bandwidth: Some(10 * 1024 * 1024), // 10Mbps
                file_descriptors: Some(1024),
                processes: Some(64),
            },
            security: SecurityConfig {
                privileged: false,
                capabilities: vec![
                    "CHOWN".to_string(),
                    "SETGID".to_string(),
                    "SETUID".to_string(),
                ],
                apparmor_profile: Some("multios-programming".to_string()),
                seccomp_profile: Some("default".to_string()),
                read_only_root: true,
                no_new_privileges: true,
                user_namespace: true,
            },
            educational_tools: vec![
                "python3-interactive".to_string(),
                "git-tutorial".to_string(),
                "debugger-setup".to_string(),
            ],
            volumes: vec![],
            ports: vec![],
        };

        let context = EducationalContext {
            learning_objectives: template_config.learning_objectives.clone(),
            difficulty_level: template_config.difficulty_level.clone(),
            estimated_duration: template_config.estimated_duration,
            prerequisites: template_config.prerequisites.clone(),
            evaluation_criteria: vec![
                "Can compile and run basic programs".to_string(),
                "Understands variable usage".to_string(),
                "Can use basic debugging tools".to_string(),
                "Demonstrates proper code organization".to_string(),
            ],
            related_topics: vec![
                "Data types and structures".to_string(),
                "Control flow".to_string(),
                "Functions and modules".to_string(),
                "Version control basics".to_string(),
            ],
        };

        self.save_template(template_id, &template_config).await?;
        self.save_context(template_id, &context).await?;

        Ok(())
    }

    async fn create_system_admin_template(&self) -> ContainerResult<()> {
        let template_id = "system-admin";
        let template_config = TemplateConfig {
            name: "System Administration Lab".to_string(),
            description: "Hands-on environment for learning system administration skills".to_string(),
            difficulty_level: DifficultyLevel::Intermediate,
            estimated_duration: Duration::from_hours(4),
            learning_objectives: vec![
                "Understand Linux system administration".to_string(),
                "Learn user and group management".to_string(),
                "Practice service configuration".to_string(),
                "Master log management".to_string(),
            ],
            prerequisites: vec![
                "Basic Linux command line knowledge".to_string(),
                "Understanding of file permissions".to_string(),
            ],
            image_base: "ubuntu:20.04".to_string(),
            entrypoint: Some(vec!["/bin/bash".to_string()]),
            command: None,
            environment: vec![
                ("LANG".to_string(), "C.UTF-8".to_string()),
                ("TERM".to_string(), "xterm-256color".to_string()),
            ],
            packages: vec![
                "sudo".to_string(),
                "nginx".to_string(),
                "mysql-server".to_string(),
                "redis".to_string(),
                "cron".to_string(),
                "rsync".to_string(),
                "htop".to_string(),
                "sysstat".to_string(),
                "logrotate".to_string(),
                "fail2ban".to_string(),
            ],
            network_mode: NetworkMode::Bridge,
            resource_limits: ResourceLimits {
                cpu_cores: Some(2.0),
                memory_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
                disk_bytes: Some(20 * 1024 * 1024 * 1024), // 20GB
                network_bandwidth: Some(50 * 1024 * 1024), // 50Mbps
                file_descriptors: Some(2048),
                processes: Some(128),
            },
            security: SecurityConfig {
                privileged: false,
                capabilities: vec![
                    "CHOWN".to_string(),
                    "SETGID".to_string(),
                    "SETUID".to_string(),
                    "NET_BIND_SERVICE".to_string(),
                    "SYS_CHROOT".to_string(),
                ],
                apparmor_profile: Some("multios-system-admin".to_string()),
                seccomp_profile: Some("default".to_string()),
                read_only_root: true,
                no_new_privileges: true,
                user_namespace: true,
            },
            educational_tools: vec![
                "service-manager".to_string(),
                "user-management".to_string(),
                "log-analyzer".to_string(),
                "performance-monitor".to_string(),
            ],
            volumes: vec![],
            ports: vec![],
        };

        let context = EducationalContext {
            learning_objectives: template_config.learning_objectives.clone(),
            difficulty_level: template_config.difficulty_level.clone(),
            estimated_duration: template_config.estimated_duration,
            prerequisites: template_config.prerequisites.clone(),
            evaluation_criteria: vec![
                "Can manage users and groups effectively".to_string(),
                "Understands service management".to_string(),
                "Can analyze system logs".to_string(),
                "Demonstrates security best practices".to_string(),
            ],
            related_topics: vec![
                "File system management".to_string(),
                "Network configuration".to_string(),
                "Process management".to_string(),
                "Security hardening".to_string(),
            ],
        };

        self.save_template(template_id, &template_config).await?;
        self.save_context(template_id, &context).await?;

        Ok(())
    }

    async fn create_network_security_template(&self) -> ContainerResult<()> {
        let template_id = "network-security";
        let template_config = TemplateConfig {
            name: "Network Security Lab".to_string(),
            description: "Secure environment for practicing network security techniques".to_string(),
            difficulty_level: DifficultyLevel::Advanced,
            estimated_duration: Duration::from_hours(6),
            learning_objectives: vec![
                "Understand network security fundamentals".to_string(),
                "Practice penetration testing techniques".to_string(),
                "Learn network monitoring and analysis".to_string(),
                "Master firewall configuration".to_string(),
            ],
            prerequisites: vec![
                "Network fundamentals knowledge".to_string(),
                "Basic Linux administration".to_string(),
                "Understanding of security concepts".to_string(),
            ],
            image_base: "kalilinux/kali-rolling".to_string(),
            entrypoint: Some(vec!["/bin/bash".to_string()]),
            command: None,
            environment: vec![
                ("LANG".to_string(), "C.UTF-8".to_string()),
                ("DISPLAY".to_string(), ":0".to_string()),
            ],
            packages: vec![
                "nmap".to_string(),
                "wireshark".to_string(),
                "tcpdump".to_string(),
                "netcat".to_string(),
                "openssl".to_string(),
                "iptables".to_string(),
                "fail2ban".to_string(),
                "snort".to_string(),
                "metasploit-framework".to_string(),
            ],
            network_mode: NetworkMode::Custom("isolated".to_string()),
            resource_limits: ResourceLimits {
                cpu_cores: Some(4.0),
                memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4GB
                disk_bytes: Some(30 * 1024 * 1024 * 1024), // 30GB
                network_bandwidth: Some(100 * 1024 * 1024), // 100Mbps
                file_descriptors: Some(4096),
                processes: Some(256),
            },
            security: SecurityConfig {
                privileged: false,
                capabilities: vec![
                    "CHOWN".to_string(),
                    "SETGID".to_string(),
                    "SETUID".to_string(),
                    "NET_RAW".to_string(),
                    "SYS_ADMIN".to_string(),
                ],
                apparmor_profile: Some("multios-security".to_string()),
                seccomp_profile: Some("security".to_string()),
                read_only_root: false,
                no_new_privileges: false,
                user_namespace: true,
            },
            educational_tools: vec![
                "network-analyzer".to_string(),
                "security-scanner".to_string(),
                "packet-capture".to_string(),
                "firewall-manager".to_string(),
            ],
            volumes: vec![],
            ports: vec![],
        };

        let context = EducationalContext {
            learning_objectives: template_config.learning_objectives.clone(),
            difficulty_level: template_config.difficulty_level.clone(),
            estimated_duration: template_config.estimated_duration,
            prerequisites: template_config.prerequisites.clone(),
            evaluation_criteria: vec![
                "Can perform network reconnaissance".to_string(),
                "Understands common vulnerabilities".to_string(),
                "Can implement security controls".to_string(),
                "Demonstrates ethical hacking practices".to_string(),
            ],
            related_topics: vec![
                "Cryptography basics".to_string(),
                "Wireless security".to_string(),
                "Incident response".to_string(),
                "Security auditing".to_string(),
            ],
        };

        self.save_template(template_id, &template_config).await?;
        self.save_context(template_id, &context).await?;

        Ok(())
    }

    async fn create_web_development_template(&self) -> ContainerResult<()> {
        let template_id = "web-development";
        let template_config = TemplateConfig {
            name: "Web Development Environment".to_string(),
            description: "Full-stack web development environment with modern tools".to_string(),
            difficulty_level: DifficultyLevel::Intermediate,
            estimated_duration: Duration::from_hours(3),
            learning_objectives: vec![
                "Set up complete web development environment".to_string(),
                "Learn front-end development workflows".to_string(),
                "Practice backend API development".to_string(),
                "Understand containerized deployment".to_string(),
            ],
            prerequisites: vec![
                "Basic programming knowledge".to_string(),
                "Understanding of web concepts".to_string(),
            ],
            image_base: "node:18-alpine".to_string(),
            entrypoint: Some(vec!["/bin/sh".to_string()]),
            command: None,
            environment: vec![
                ("NODE_ENV".to_string(), "development".to_string()),
                ("LANG".to_string(), "C.UTF-8".to_string()),
            ],
            packages: vec![
                "git".to_string(),
                "nginx".to_string(),
                "postgresql".to_string(),
                "redis".to_string(),
                "yarn".to_string(),
                "typescript".to_string(),
                "eslint".to_string(),
                "webpack".to_string(),
            ],
            network_mode: NetworkMode::Bridge,
            resource_limits: ResourceLimits {
                cpu_cores: Some(2.0),
                memory_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
                disk_bytes: Some(15 * 1024 * 1024 * 1024), // 15GB
                network_bandwidth: Some(50 * 1024 * 1024), // 50Mbps
                file_descriptors: Some(2048),
                processes: Some(128),
            },
            security: SecurityConfig {
                privileged: false,
                capabilities: vec![
                    "CHOWN".to_string(),
                    "SETGID".to_string(),
                    "SETUID".to_string(),
                    "NET_BIND_SERVICE".to_string(),
                ],
                apparmor_profile: Some("multios-web-dev".to_string()),
                seccomp_profile: Some("default".to_string()),
                read_only_root: true,
                no_new_privileges: true,
                user_namespace: true,
            },
            educational_tools: vec![
                "live-reload".to_string(),
                "api-tester".to_string(),
                "code-linter".to_string(),
                "build-optimizer".to_string(),
            ],
            volumes: vec![],
            ports: vec![
                PortMapping {
                    container_port: 3000,
                    host_port: 3000,
                    protocol: "tcp".to_string(),
                },
                PortMapping {
                    container_port: 8080,
                    host_port: 8080,
                    protocol: "tcp".to_string(),
                },
            ],
        };

        let context = EducationalContext {
            learning_objectives: template_config.learning_objectives.clone(),
            difficulty_level: template_config.difficulty_level.clone(),
            estimated_duration: template_config.estimated_duration,
            prerequisites: template_config.prerequisites.clone(),
            evaluation_criteria: vec![
                "Can create functional web applications".to_string(),
                "Understands development workflows".to_string(),
                "Demonstrates good coding practices".to_string(),
                "Can containerize applications".to_string(),
            ],
            related_topics: vec![
                "JavaScript frameworks".to_string(),
                "REST API design".to_string(),
                "Database integration".to_string(),
                "DevOps fundamentals".to_string(),
            ],
        };

        self.save_template(template_id, &template_config).await?;
        self.save_context(template_id, &context).await?;

        Ok(())
    }

    async fn create_database_admin_template(&self) -> ContainerResult<()> {
        let template_id = "database-admin";
        let template_config = TemplateConfig {
            name: "Database Administration Lab".to_string(),
            description: "Comprehensive database administration and optimization environment".to_string(),
            difficulty_level: DifficultyLevel::Advanced,
            estimated_duration: Duration::from_hours(5),
            learning_objectives: vec![
                "Master database administration fundamentals".to_string(),
                "Learn performance optimization techniques".to_string(),
                "Practice backup and recovery procedures".to_string(),
                "Understand database security best practices".to_string(),
            ],
            prerequisites: vec![
                "SQL knowledge".to_string(),
                "Database fundamentals".to_string(),
                "Basic system administration".to_string(),
            ],
            image_base: "postgres:14".to_string(),
            entrypoint: Some(vec!["/bin/bash".to_string()]),
            command: None,
            environment: vec![
                ("POSTGRES_DB".to_string(), "lab_db".to_string()),
                ("POSTGRES_USER".to_string(), "lab_user".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "lab_pass".to_string()),
            ],
            packages: vec![
                "postgresql-14".to_string(),
                "postgresql-contrib-14".to_string(),
                "pgbench".to_string(),
                "pgadmin4".to_string(),
                "mysqldump".to_string(),
                "redis-tools".to_string(),
                "mongodb".to_string(),
            ],
            network_mode: NetworkMode::Bridge,
            resource_limits: ResourceLimits {
                cpu_cores: Some(3.0),
                memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4GB
                disk_bytes: Some(50 * 1024 * 1024 * 1024), // 50GB
                network_bandwidth: Some(100 * 1024 * 1024), // 100Mbps
                file_descriptors: Some(4096),
                processes: Some(256),
            },
            security: SecurityConfig {
                privileged: false,
                capabilities: vec![
                    "CHOWN".to_string(),
                    "SETGID".to_string(),
                    "SETUID".to_string(),
                    "DAC_OVERRIDE".to_string(),
                ],
                apparmor_profile: Some("multios-database".to_string()),
                seccomp_profile: Some("default".to_string()),
                read_only_root: true,
                no_new_privileges: true,
                user_namespace: true,
            },
            educational_tools: vec![
                "query-analyzer".to_string(),
                "performance-monitor".to_string(),
                "backup-manager".to_string(),
                "security-scanner".to_string(),
            ],
            volumes: vec![],
            ports: vec![
                PortMapping {
                    container_port: 5432,
                    host_port: 5432,
                    protocol: "tcp".to_string(),
                },
                PortMapping {
                    container_port: 8080,
                    host_port: 8080,
                    protocol: "tcp".to_string(),
                },
            ],
        };

        let context = EducationalContext {
            learning_objectives: template_config.learning_objectives.clone(),
            difficulty_level: template_config.difficulty_level.clone(),
            estimated_duration: template_config.estimated_duration,
            prerequisites: template_config.prerequisites.clone(),
            evaluation_criteria: vec![
                "Can perform database administration tasks".to_string(),
                "Understands performance tuning".to_string(),
                "Can implement backup strategies".to_string(),
                "Demonstrates security best practices".to_string(),
            ],
            related_topics: vec![
                "Database design".to_string(),
                "Query optimization".to_string(),
                "Data warehousing".to_string(),
                "High availability".to_string(),
            ],
        };

        self.save_template(template_id, &template_config).await?;
        self.save_context(template_id, &context).await?;

        Ok(())
    }

    async fn save_template(&self, template_id: &str, config: &TemplateConfig) -> ContainerResult<()> {
        let template_path = self.template_root.join(format!("{}.json", template_id));
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ContainerError::System(format!("Failed to serialize template: {}", e)))?;
        
        std::fs::write(&template_path, content)
            .map_err(|e| ContainerError::System(format!("Failed to write template: {}", e)))?;

        Ok(())
    }

    async fn save_context(&self, template_id: &str, context: &EducationalContext) -> ContainerResult<()> {
        let contexts_dir = self.template_root.join("contexts");
        std::fs::create_dir_all(&contexts_dir)
            .map_err(|e| ContainerError::System(format!("Failed to create contexts directory: {}", e)))?;

        let context_path = contexts_dir.join(format!("{}.json", template_id));
        let content = serde_json::to_string_pretty(context)
            .map_err(|e| ContainerError::System(format!("Failed to serialize context: {}", e)))?;
        
        std::fs::write(&context_path, content)
            .map_err(|e| ContainerError::System(format!("Failed to write context: {}", e)))?;

        Ok(())
    }

    fn apply_customizations(&self, template_config: &mut TemplateConfig, customizations: &TemplateCustomizations) -> ContainerResult<()> {
        if let Some(ref name) = customizations.name {
            template_config.name = name.clone();
        }

        if let Some(ref description) = customizations.description {
            template_config.description = description.clone();
        }

        if let Some(ref entrypoint) = customizations.entrypoint {
            template_config.entrypoint = Some(vec![entrypoint.clone()]);
        }

        if let Some(ref packages) = customizations.packages {
            template_config.packages = packages.clone();
        }

        if let Some(ref environment) = customizations.environment {
            template_config.environment = environment.clone();
        }

        Ok(())
    }

    fn template_to_container_config(&self, template_config: &TemplateConfig, template_id: &str) -> ContainerResult<ContainerConfig> {
        Ok(ContainerConfig {
            container_id: uuid::Uuid::new_v4().to_string(),
            name: template_config.name.clone(),
            image: template_config.image_base.clone(),
            command: template_config.command.clone().unwrap_or_default(),
            environment: template_config.environment.iter().cloned().collect(),
            ports: template_config.ports.clone(),
            volumes: vec![],
            resource_limits: template_config.resource_limits.clone(),
            security: template_config.security.clone(),
            network: NetworkConfig {
                network_mode: template_config.network_mode.clone(),
                bridge_name: Some("multios-br0".to_string()),
                ip_address: None,
                mac_address: None,
                dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
                port_mappings: template_config.ports.clone(),
            },
            namespace_mode: NamespaceMode::default(),
            template_id: Some(template_id.to_string()),
            created_at: Utc::now(),
        })
    }
}

/// Container template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub difficulty_level: DifficultyLevel,
    pub estimated_duration: Duration,
    pub learning_objectives: Vec<String>,
    pub prerequisites: Vec<String>,
    pub image_base: String,
    pub entrypoint: Option<Vec<String>>,
    pub command: Option<Vec<String>>,
    pub environment: Vec<(String, String)>,
    pub packages: Vec<String>,
    pub network_mode: NetworkMode,
    pub resource_limits: ResourceLimits,
    pub security: SecurityConfig,
    pub educational_tools: Vec<String>,
    pub volumes: Vec<VolumeMapping>,
    pub ports: Vec<PortMapping>,
}

/// Template customizations
#[derive(Debug, Clone, Default)]
pub struct TemplateCustomizations {
    pub name: Option<String>,
    pub description: Option<String>,
    pub entrypoint: Option<String>,
    pub packages: Option<Vec<String>>,
    pub environment: Option<Vec<(String, String)>>,
    pub custom_labels: Option<HashMap<String, String>>,
}

/// Template information for listing
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty_level: DifficultyLevel,
    pub estimated_duration: Duration,
    pub learning_objectives: Vec<String>,
    pub prerequisites: Vec<String>,
}

/// Learning path configuration
#[derive(Debug, Clone)]
pub struct LearningPath {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty_progression: Vec<String>,
    pub estimated_total_duration: Duration,
    pub prerequisites: Vec<String>,
    pub learning_outcomes: Vec<String>,
}

/// Assessment configuration
#[derive(Debug, Clone)]
pub struct AssessmentConfig {
    pub practical_exercises: Vec<PracticalExercise>,
    pub theoretical_questions: Vec<TheoreticalQuestion>,
    pub evaluation_criteria: Vec<EvaluationCriterion>,
}

/// Practical exercise definition
#[derive(Debug, Clone)]
pub struct PracticalExercise {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub estimated_duration: Duration,
    pub instructions: String,
    pub expected_outputs: Vec<String>,
    pub hints: Vec<String>,
}

/// Theoretical question definition
#[derive(Debug, Clone)]
pub struct TheoreticalQuestion {
    pub id: String,
    pub question: String,
    pub question_type: QuestionType,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub difficulty: DifficultyLevel,
}

/// Question types
#[derive(Debug, Clone)]
pub enum QuestionType {
    MultipleChoice,
    TrueFalse,
    ShortAnswer,
    Essay,
}

/// Evaluation criterion
#[derive(Debug, Clone)]
pub struct EvaluationCriterion {
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub levels: Vec<EvaluationLevel>,
}

/// Evaluation levels
#[derive(Debug, Clone)]
pub struct EvaluationLevel {
    pub name: String,
    pub description: String,
    pub score_range: (f32, f32),
}