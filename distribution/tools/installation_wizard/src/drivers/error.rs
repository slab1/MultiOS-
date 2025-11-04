use thiserror::Error;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("Driver installation failed: {0}")]
    InstallationFailed(String),
    
    #[error("Driver compilation failed: {0}")]
    CompilationFailed(String),
    
    #[error("Driver not found: {0}")]
    DriverNotFound(String),
    
    #[error("Incompatible driver: {0}")]
    IncompatibleDriver(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Configuration failed: {0}")]
    ConfigurationFailed(String),
    
    #[error("Module loading failed: {0}")]
    ModuleLoadingFailed(String),
    
    #[error("Dependencies missing: {0}")]
    DependenciesMissing(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other driver error: {0}")]
    Other(String),
}

impl From<DriverError> for anyhow::Error {
    fn from(error: DriverError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}