use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Interface configuration failed: {0}")]
    InterfaceConfigFailed(String),
    
    #[error("DNS configuration failed: {0}")]
    DnsConfigFailed(String),
    
    #[error("Hostname configuration failed: {0}")]
    HostnameConfigFailed(String),
    
    #[error("Network service failed: {0}")]
    ServiceFailed(String),
    
    #[error("Connectivity test failed: {0}")]
    ConnectivityTestFailed(String),
    
    #[error("DHCP configuration failed: {0}")]
    DhcpFailed(String),
    
    #[error("Static IP configuration failed: {0}")]
    StaticIpFailed(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other network error: {0}")]
    Other(String),
}

impl From<NetworkError> for anyhow::Error {
    fn from(error: NetworkError) -> Self {
        anyhow::anyhow!("{}", error)
    }
}