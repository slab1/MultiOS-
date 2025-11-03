//! MultiOS Package Manager - Core Error Types
//! 
//! This module defines comprehensive error types for the package manager,
//! including package operations, repository management, security validation,
//! and system-level errors.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the package manager
#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    
    #[error("Dependency resolution failed for package: {package}, error: {error}")]
    DependencyResolution { package: String, error: String },
    
    #[error("Package conflict detected: {conflict}")]
    PackageConflict { conflict: String },
    
    #[error("Repository error: {error}")]
    RepositoryError { error: String },
    
    #[error("Signature verification failed: {package}")]
    SignatureVerificationFailed { package: String },
    
    #[error("Invalid package format: {package}")]
    InvalidPackageFormat { package: String },
    
    #[error("Corrupted package file: {path}")]
    CorruptedPackage { path: PathBuf },
    
    #[error("Insufficient disk space: {required} bytes required, {available} bytes available")]
    InsufficientDiskSpace { required: u64, available: u64 },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Network error: {error}")]
    NetworkError { error: String },
    
    #[error("Download failed: {url}, error: {error}")]
    DownloadFailed { url: String, error: String },
    
    #[error("Package signing failed: {error}")]
    SigningFailed { error: String },
    
    #[error("Package validation failed: {error}")]
    ValidationFailed { error: String },
    
    #[error("Delta update error: {error}")]
    DeltaUpdateError { error: String },
    
    #[error("Rollback failed: {error}")]
    RollbackFailed { error: String },
    
    #[error("Configuration error: {error}")]
    ConfigurationError { error: String },
    
    #[error("System error: {error}")]
    SystemError { error: String },
    
    #[error("IO error: {error}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {error}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("TOML parsing error: {error}")]
    TomlError(#[from] toml::de::Error),
    
    #[error("HTTP client error: {error}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Cryptographic error: {error}")]
    CryptoError(String),
}

/// Result type for package manager operations
pub type Result<T> = std::result::Result<T, PackageError>;

/// Error context for detailed error reporting
pub struct ErrorContext {
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub repository: Option<String>,
    pub operation: String,
    pub file_path: Option<PathBuf>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        Self {
            package_name: None,
            package_version: None,
            repository: None,
            operation,
            file_path: None,
        }
    }
    
    pub fn with_package(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.package_name = Some(name.into());
        self.package_version = Some(version.into());
        self
    }
    
    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }
    
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation: {}", self.operation)?;
        if let Some(ref name) = self.package_name {
            write!(f, ", Package: {}", name)?;
        }
        if let Some(ref version) = self.package_version {
            write!(f, " v{}", version)?;
        }
        if let Some(ref repo) = self.repository {
            write!(f, ", Repository: {}", repo)?;
        }
        if let Some(ref path) = self.file_path {
            write!(f, ", File: {}", path.display())?;
        }
        Ok(())
    }
}