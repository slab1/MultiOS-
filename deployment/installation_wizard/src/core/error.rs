use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Installation configuration error: {0}")]
    Configuration(#[from] crate::core::config::ConfigError),
    
    #[error("Installation state error: {0}")]
    State(String),
    
    #[error("Installation progress error: {0}")]
    Progress(String),
    
    #[error("Installation manager error: {0}")]
    Manager(String),
    
    #[error("Hardware detection error: {0}")]
    Hardware(#[from] crate::hardware::error::HardwareError),
    
    #[error("Partitioning error: {0}")]
    Partitioning(#[from] crate::partitioning::error::PartitioningError),
    
    #[error("Driver error: {0}")]
    Drivers(#[from] crate::drivers::error::DriverError),
    
    #[error("Network error: {0}")]
    Network(#[from] crate::network::error::NetworkError),
    
    #[error("User management error: {0}")]
    User(#[from] crate::user::error::UserError),
    
    #[error("Recovery error: {0}")]
    Recovery(#[from] crate::recovery::error::RecoveryError),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<CoreError> for anyhow::Error {
    fn from(error: CoreError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}