use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Recovery point creation failed: {0}")]
    PointCreationFailed(String),
    
    #[error("Recovery point not found: {0}")]
    PointNotFound(String),
    
    #[error("Recovery validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Recovery restore failed: {0}")]
    RestoreFailed(String),
    
    #[error("Backup operation failed: {0}")]
    BackupFailed(String),
    
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),
    
    #[error("Insufficient space for recovery: {0}")]
    InsufficientSpace(String),
    
    #[error("Invalid recovery configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other recovery error: {0}")]
    Other(String),
}

impl From<RecoveryError> for anyhow::Error {
    fn from(error: RecoveryError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}