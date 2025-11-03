//! MultiOS Package Manager - Core Types and Data Structures
//! 
//! This module defines the fundamental types used throughout the package manager,
//! including package metadata, dependencies, version specifications, and repository structures.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Architecture-specific package identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    ARM64,
    RISCV64,
    Universal, // Can run on any architecture
    Custom(String), // For custom architectures
}

impl std::str::FromStr for Architecture {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x86_64" | "amd64" => Ok(Architecture::X86_64),
            "arm64" | "aarch64" => Ok(Architecture::ARM64),
            "riscv64" | "riscv" => Ok(Architecture::RISCV64),
            "universal" => Ok(Architecture::Universal),
            other => Ok(Architecture::Custom(other.to_string())),
        }
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::ARM64 => write!(f, "arm64"),
            Architecture::RISCV64 => write!(f, "riscv64"),
            Architecture::Universal => write!(f, "universal"),
            Architecture::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Package version specification with semantic versioning support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }
    
    pub fn with_pre_release(mut self, pre: impl Into<String>) -> Self {
        self.pre_release = Some(pre.into());
        self
    }
    
    pub fn with_build_metadata(mut self, meta: impl Into<String>) -> Self {
        self.build_metadata = Some(meta.into());
        self
    }
    
    /// Parse version from string (supports semantic versioning)
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 3 {
            return Err("Invalid version format".to_string());
        }
        
        let major = parts[0].parse().map_err(|_| "Invalid major version")?;
        let minor = parts[1].parse().map_err(|_| "Invalid minor version")?;
        let patch_part = parts[2];
        
        // Handle patch version with pre-release and build metadata
        let patch_split: Vec<&str> = patch_part.split(|c| c == '-' || c == '+').collect();
        let patch = patch_split[0].parse().map_err(|_| "Invalid patch version")?;
        
        let mut version = Self::new(major, minor, patch);
        
        if patch_split.len() > 1 {
            if patch_split[0].contains('+') {
                // Build metadata exists
                let build_meta = patch_split[1..].join("+");
                version.build_metadata = Some(build_meta);
            } else {
                version.pre_release = Some(patch_split[1..].join("-"));
            }
        }
        
        if let Some(build_idx) = s.find('+') {
            version.build_metadata = Some(s[build_idx + 1..].to_string());
        }
        
        Ok(version)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major &&
        self.minor == other.minor &&
        self.patch == other.patch &&
        self.pre_release == other.pre_release
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        if self.patch != other.patch {
            return Some(self.patch.cmp(&other.patch));
        }
        
        // Pre-release versions are considered less than release versions
        match (self.pre_release.is_some(), other.pre_release.is_some()) {
            (false, true) => Some(std::cmp::Ordering::Greater),
            (true, false) => Some(std::cmp::Ordering::Less),
            (true, true) => self.pre_release.partial_cmp(&other.pre_release),
            (false, false) => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref meta) = self.build_metadata {
            write!(f, "+{}", meta)?;
        }
        Ok(())
    }
}

/// Dependency specification with version constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_constraint: VersionConstraint,
    pub optional: bool,
    pub metadata: HashMap<String, String>,
}

impl Dependency {
    pub fn new(name: String, constraint: VersionConstraint) -> Self {
        Self {
            name,
            version_constraint: constraint,
            optional: false,
            metadata: HashMap::new(),
        }
    }
    
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Version constraint operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionConstraint {
    Exact(Version),
    GreaterThan(Version),
    GreaterThanOrEqual(Version),
    LessThan(Version),
    LessThanOrEqual(Version),
    Range(Box<VersionConstraint>, Box<VersionConstraint>),
    Wildcard(String), // e.g., "1.*" or "1.2.*"
}

impl VersionConstraint {
    /// Check if a version satisfies this constraint
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::GreaterThan(v) => version > v,
            VersionConstraint::GreaterThanOrEqual(v) => version >= v,
            VersionConstraint::LessThan(v) => version < v,
            VersionConstraint::LessThanOrEqual(v) => version <= v,
            VersionConstraint::Range(min, max) => min.matches(version) && max.matches(version),
            VersionConstraint::Wildcard(pattern) => {
                // Simple wildcard matching for major.minor.patch patterns
                let version_str = version.to_string();
                let pattern_regex = regex::Regex::new(&pattern.replace("*", ".*"))
                    .unwrap_or_else(|_| regex::Regex::new(&version_str).unwrap());
                pattern_regex.is_match(&version_str)
            }
        }
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Exact(v) => write!(f, "={}", v),
            VersionConstraint::GreaterThan(v) => write!(f, ">{}", v),
            VersionConstraint::GreaterThanOrEqual(v) => write!(f, ">={}", v),
            VersionConstraint::LessThan(v) => write!(f, "<{}", v),
            VersionConstraint::LessThanOrEqual(v) => write!(f, "<={}", v),
            VersionConstraint::Range(min, max) => write!(f, "{} and {}", min, max),
            VersionConstraint::Wildcard(p) => write!(f, "{}", p),
        }
    }
}

/// Package metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub architecture: Architecture,
    pub dependencies: Vec<Dependency>,
    pub provides: HashSet<String>,
    pub conflicts: Vec<Dependency>,
    pub files: Vec<PackageFile>,
    pub metadata: PackageMetadata,
    pub checksum: String,
    pub signature: Option<PackageSignature>,
    pub delta_info: Option<DeltaInfo>,
}

impl Package {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            description: String::new(),
            architecture: Architecture::Universal,
            dependencies: Vec::new(),
            provides: HashSet::new(),
            conflicts: Vec::new(),
            files: Vec::new(),
            metadata: PackageMetadata::default(),
            checksum: String::new(),
            signature: None,
            delta_info: None,
        }
    }
    
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    
    pub fn with_architecture(mut self, arch: Architecture) -> Self {
        self.architecture = arch;
        self
    }
    
    pub fn with_dependency(mut self, dependency: Dependency) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Package file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub path: String,
    pub size: u64,
    pub permissions: u32,
    pub checksum: String,
    pub file_type: FileType,
}

/// Type of file in a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Regular,
    Directory,
    Symlink { target: String },
    Executable,
    Configuration,
    Data,
}

/// Additional package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub author: Option<String>,
    pub maintainer: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub source_url: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub install_scripts: InstallScripts,
    pub security_info: SecurityInfo,
    pub build_info: BuildInfo,
    pub custom_fields: HashMap<String, String>,
}

impl Default for PackageMetadata {
    fn default() -> Self {
        Self {
            author: None,
            maintainer: None,
            license: None,
            homepage: None,
            source_url: None,
            categories: Vec::new(),
            tags: Vec::new(),
            install_scripts: InstallScripts::default(),
            security_info: SecurityInfo::default(),
            build_info: BuildInfo::default(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Installation and removal scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallScripts {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
    pub pre_update: Option<String>,
    pub post_update: Option<String>,
}

impl Default for InstallScripts {
    fn default() -> Self {
        Self {
            pre_install: None,
            post_install: None,
            pre_remove: None,
            post_remove: None,
            pre_update: None,
            post_update: None,
        }
    }
}

/// Security information for package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub cve_references: Vec<String>,
    pub security_audit_date: Option<DateTime<Utc>>,
    pub vulnerability_status: VulnerabilityStatus,
    pub trusted_publisher: bool,
    pub checksum_algorithm: String,
}

impl Default for SecurityInfo {
    fn default() -> Self {
        Self {
            cve_references: Vec::new(),
            security_audit_date: None,
            vulnerability_status: VulnerabilityStatus::Unknown,
            trusted_publisher: false,
            checksum_algorithm: "sha256".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityStatus {
    Safe,
    Minor,
    Moderate,
    High,
    Critical,
    Unknown,
}

/// Build information for package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_timestamp: DateTime<Utc>,
    pub compiler_version: Option<String>,
    pub target_triple: String,
    pub build_flags: Vec<String>,
    pub source_checksum: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            build_timestamp: Utc::now(),
            compiler_version: None,
            target_triple: "unknown".to_string(),
            build_flags: Vec::new(),
            source_checksum: String::new(),
        }
    }
}

/// Package signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSignature {
    pub algorithm: String,
    pub public_key_id: String,
    pub signature_data: Vec<u8>,
    pub certificate_chain: Vec<Vec<u8>>,
    pub timestamp: DateTime<Utc>,
}

/// Delta update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaInfo {
    pub base_version: Version,
    pub delta_size: u64,
    pub delta_checksum: String,
    pub patch_algorithm: String,
    pub compression: String,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub mirror_urls: Vec<String>,
    pub enabled: bool,
    pub priority: i32,
    pub architectures: Vec<Architecture>,
    pub metadata_url: String,
    pub signing_key: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub update_interval: std::time::Duration,
    pub security_info: RepositorySecurityInfo,
}

impl Repository {
    pub fn new(name: String, base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            base_url,
            mirror_urls: Vec::new(),
            enabled: true,
            priority: 0,
            architectures: vec![Architecture::Universal],
            metadata_url: "".to_string(),
            signing_key: None,
            last_updated: None,
            update_interval: std::time::Duration::from_secs(3600),
            security_info: RepositorySecurityInfo::default(),
        }
    }
}

/// Repository security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySecurityInfo {
    pub require_signature: bool,
    pub allow_unsigned: bool,
    pub trusted_keys: HashSet<String>,
    pub certificate_pins: Vec<String>,
    pub tls_verification: bool,
}

impl Default for RepositorySecurityInfo {
    fn default() -> Self {
        Self {
            require_signature: false,
            allow_unsigned: true,
            trusted_keys: HashSet::new(),
            certificate_pins: Vec::new(),
            tls_verification: true,
        }
    }
}