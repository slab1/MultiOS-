use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User creation failed: {0}")]
    UserCreationFailed(String),
    
    #[error("Password configuration failed: {0}")]
    PasswordConfigFailed(String),
    
    #[error("Invalid username: {0}")]
    InvalidUsername(String),
    
    #[error("Invalid password: {0}")]
    InvalidPassword(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Auto-login configuration failed: {0}")]
    AutoLoginFailed(String),
    
    #[error("Home directory creation failed: {0}")]
    HomeDirFailed(String),
    
    #[error("Group configuration failed: {0}")]
    GroupConfigFailed(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other user error: {0}")]
    Other(String),
}

impl From<UserError> for anyhow::Error {
    fn from(error: UserError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}