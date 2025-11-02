//! MultiOS Networking Stack
//! 
//! A comprehensive TCP/IP network stack implementation with POSIX-compliant socket API,
//! routing capabilities, DNS resolution, and educational features.
//!
//! # Architecture Overview
//!
//! ```text
//! +------------------------+
//! |   Application Layer    |
//! |  (Socket API, DNS)     |
//! +------------------------+
//! |   Transport Layer      |
//! |   (TCP, UDP, ICMP)     |
//! +------------------------+
//! |   Network Layer        |
//! |      (IP, Routing)     |
//! +------------------------+
//! |   Data Link Layer      |
//! |  (Ethernet, WiFi)      |
//! +------------------------+
//! |   Physical Layer       |
//! |    (NIC Drivers)       |
//! +------------------------+
//! ```
//!
//! # Features
//!
//! - **Complete Network Stack**: TCP, UDP, ICMP, IP implementations
//! - **POSIX Socket API**: Full compatibility with standard socket interfaces
//! - **Routing Engine**: Dynamic routing with multiple protocols
//! - **DNS Resolution**: Built-in DNS client and resolver
//! - **Network Security**: Firewall and packet filtering capabilities
//! - **Educational Tools**: Network simulation and learning framework
//! - **Multi-Platform**: Works across different architectures
//!
//! # Quick Start
//!
//! ```rust
//! use multios_networking::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a TCP socket
//!     let socket = TcpSocket::new()?;
//!     
//!     // Connect to a remote host
//!     socket.connect("127.0.0.1:8080").await?;
//!     
//!     // Send data
//!     socket.write(b"Hello, Network!").await?;
//!     
//!     // Receive response
//!     let mut buffer = vec![0u8; 1024];
//!     let bytes_read = socket.read(&mut buffer).await?;
//!     
//!     println!("Received: {}", String::from_utf8_lossy(&buffer[..bytes_read]));
//!     Ok(())
//! }
//! ```

pub mod prelude;
pub mod core;
pub mod sockets;
pub mod protocols;
pub mod routing;
pub mod dns;
pub mod security;
pub mod drivers;
pub mod simulation;
pub mod education;

// Re-export commonly used types
pub use prelude::*;

/// Error types for networking operations
#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    #[error("Invalid IP address format")]
    InvalidAddress,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Network unreachable")]
    NetworkUnreachable,
    
    #[error("Host unreachable")]
    HostUnreachable,
    
    #[error("Port unreachable")]
    PortUnreachable,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Resource temporarily unavailable")]
    WouldBlock,
    
    #[error("Broken pipe")]
    BrokenPipe,
    
    #[error("Connection reset by peer")]
    ConnectionReset,
    
    #[error("Buffer too small")]
    BufferTooSmall,
    
    #[error("Invalid socket state")]
    InvalidState,
    
    #[error("Routing table error")]
    RoutingError,
    
    #[error("DNS resolution error: {0}")]
    DnsError(String),
    
    #[error("Firewall rule violation")]
    FirewallViolation,
    
    #[error("Security error: {0}")]
    SecurityError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other networking error: {0}")]
    Other(String),
}

impl NetworkError {
    /// Convert to std::io::Error if applicable
    pub fn as_io_error(&self) -> Option<std::io::Error> {
        match self {
            NetworkError::IoError(err) => Some(std::io::Error::new(err.kind(), self.to_string())),
            _ => None,
        }
    }
}

/// Result type for networking operations
pub type Result<T> = std::result::Result<T, NetworkError>;

/// Version information
pub const VERSION: &str = "1.0.0";

/// Initialize the networking stack
/// 
/// This function must be called before using any networking functionality.
/// It sets up the network interfaces, routing tables, and initializes
/// the network stack components.
///
/// # Arguments
///
/// * `interfaces` - List of network interfaces to initialize
/// * `config` - Network configuration
///
/// # Example
///
/// ```rust
/// use multios_networking::{init_networking, NetworkConfig};
///
/// let config = NetworkConfig::default();
/// init_networking(vec![], &config)?;
/// ```
pub fn init_networking(interfaces: Vec<String>, config: &crate::core::NetworkConfig) -> Result<()> {
    crate::core::init_core(interfaces, config)
}

/// Shutdown the networking stack
/// 
/// This function should be called when shutting down the application
/// to properly close all network connections and free resources.
pub fn shutdown_networking() -> Result<()> {
    crate::core::shutdown_core()
}

/// Get network statistics
pub fn get_network_stats() -> Result<crate::core::NetworkStats> {
    crate::core::get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_initialization() {
        let config = crate::core::NetworkConfig::default();
        let result = init_networking(vec![], &config);
        assert!(result.is_ok());
        shutdown_networking().unwrap();
    }
}