use thiserror::Error;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("CPU detection error: {0}")]
    Cpu(String),
    
    #[error("Memory detection error: {0}")]
    Memory(String),
    
    #[error("Storage detection error: {0}")]
    Storage(String),
    
    #[error("Network detection error: {0}")]
    Network(String),
    
    #[error("Graphics detection error: {0}")]
    Graphics(String),
    
    #[error("Boot system detection error: {0}")]
    Boot(String),
    
    #[error("Audio detection error: {0}")]
    Audio(String),
    
    #[error("Input device detection error: {0}")]
    Input(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("System call error: {0}")]
    SystemCall(String),
    
    #[error("Incompatible hardware: {0}")]
    Incompatible(String),
    
    #[error("Other hardware error: {0}")]
    Other(String),
}

impl From<HardwareError> for anyhow::Error {
    fn from(error: HardwareError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}