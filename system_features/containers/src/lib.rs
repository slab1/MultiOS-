//! MultiOS Lightweight Container Support System
//! 
//! This module provides comprehensive container support for educational environments,
//! including isolation, resource management, and security features.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub mod container;
pub mod runtime;
pub mod namespaces;
pub mod resource_manager;
pub mod image_manager;
pub mod network_manager;
pub mod security;
pub mod templates;
pub mod lifecycle;
pub mod orchestration;

pub use container::*;
pub use runtime::*;
pub use namespaces::*;
pub use resource_manager::*;
pub use image_manager::*;
pub use network_manager::*;
pub use security::*;
pub use templates::*;
pub use lifecycle::*;
pub use orchestration::*;

use anyhow::Result;
use chrono::{DateTime, Utc};

/// Container system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub container_id: String,
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
    pub resource_limits: ResourceLimits,
    pub security: SecurityConfig,
    pub network: NetworkConfig,
    pub namespace_mode: NamespaceMode,
    pub template_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            container_id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            image: String::new(),
            command: vec![],
            environment: HashMap::new(),
            ports: vec![],
            volumes: vec![],
            resource_limits: ResourceLimits::default(),
            security: SecurityConfig::default(),
            network: NetworkConfig::default(),
            namespace_mode: NamespaceMode::default(),
            template_id: None,
            created_at: Utc::now(),
        }
    }
}

/// Resource limits for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: Option<f64>,
    pub memory_bytes: Option<u64>,
    pub disk_bytes: Option<u64>,
    pub network_bandwidth: Option<u64>,
    pub file_descriptors: Option<u64>,
    pub processes: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: None,
            memory_bytes: None,
            disk_bytes: None,
            network_bandwidth: None,
            file_descriptors: None,
            processes: None,
        }
    }
}

/// Security configuration for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub privileged: bool,
    pub capabilities: Vec<String>,
    pub apparmor_profile: Option<String>,
    pub seccomp_profile: Option<String>,
    pub read_only_root: bool,
    pub no_new_privileges: bool,
    pub user_namespace: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            privileged: false,
            capabilities: vec![],
            apparmor_profile: None,
            seccomp_profile: None,
            read_only_root: true,
            no_new_privileges: true,
            user_namespace: false,
        }
    }
}

/// Network configuration for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_mode: NetworkMode,
    pub bridge_name: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub dns_servers: Vec<String>,
    pub port_mappings: Vec<PortMapping>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_mode: NetworkMode::Bridge,
            bridge_name: Some("multios-br0".to_string()),
            ip_address: None,
            mac_address: None,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            port_mappings: vec![],
        }
    }
}

/// Network modes for containers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Custom(String),
}

impl Default for NetworkMode {
    fn default() -> Self {
        NetworkMode::Bridge
    }
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String,
}

/// Volume mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: PathBuf,
    pub container_path: PathBuf,
    pub read_only: bool,
}

/// Namespace mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceMode {
    pub pid: bool,
    pub network: bool,
    pub mount: bool,
    pub ipc: bool,
    pub uts: bool,
    pub user: bool,
    pub cgroup: bool,
}

impl Default for NamespaceMode {
    fn default() -> Self {
        Self {
            pid: true,
            network: true,
            mount: true,
            ipc: true,
            uts: true,
            user: true,
            cgroup: true,
        }
    }
}

/// Container state information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
    Exited,
    Error,
}

/// Container statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub disk_usage: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub uptime: Duration,
    pub pid_count: u64,
}

/// Educational use case information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalContext {
    pub learning_objectives: Vec<String>,
    pub difficulty_level: DifficultyLevel,
    pub estimated_duration: Duration,
    pub prerequisites: Vec<String>,
    pub evaluation_criteria: Vec<String>,
    pub related_topics: Vec<String>,
}

/// Difficulty levels for educational content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// MultiOS Container System Error Types
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("Container not found: {0}")]
    NotFound(String),
    #[error("Container already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("System error: {0}")]
    System(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Orchestration error: {0}")]
    OrchestrationError(String),
}

pub type ContainerResult<T> = Result<T, ContainerError>;