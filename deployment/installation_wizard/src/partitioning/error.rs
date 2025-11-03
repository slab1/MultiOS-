use thiserror::Error;

#[derive(Error, Debug)]
pub enum PartitioningError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Partition operation failed: {0}")]
    PartitionFailed(String),
    
    #[error("Format failed: {0}")]
    FormatFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Backup failed: {0}")]
    BackupFailed(String),
    
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
    
    #[error("Encryption setup failed: {0}")]
    EncryptionFailed(String),
    
    #[error("LVM operation failed: {0}")]
    LvmFailed(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other partitioning error: {0}")]
    Other(String),
}

impl From<PartitioningError> for anyhow::Error {
    fn from(error: PartitioningError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}