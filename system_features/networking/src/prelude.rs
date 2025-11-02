//! Prelude module for MultiOS Networking
//!
//! This module provides commonly used types and traits for easy importing.
//! Users can import everything with `use multios_networking::prelude::*;`

pub use crate::core::NetworkConfig;
pub use crate::core::NetworkStats;
pub use crate::core::NetworkInterface;
pub use crate::core::IpAddress;

pub use crate::sockets::{TcpSocket, UdpSocket, RawSocket, SocketAddr, SocketType, SocketError};
pub use crate::protocols::ip::IpPacket;
pub use crate::protocols::tcp::{TcpPacket, TcpConnection, TcpState};
pub use crate::protocols::udp::UdpPacket;
pub use crate::protocols::icmp::{IcmpPacket, IcmpType, IcmpCode};

pub use crate::dns::{DnsResolver, DnsRecord, DnsQuery};
pub use crate::routing::{RoutingTable, Route, RouteEntry};
pub use crate::security::{Firewall, FilterRule, SecurityPolicy};

pub use crate::drivers::NetworkDriver;
pub use crate::simulation::NetworkSimulator;

pub use crate::education::{NetworkLab, LabExercise, Tutorial};

pub use crate::{NetworkError, Result, init_networking, shutdown_networking, get_network_stats};

/// Convenient type aliases for common networking operations
pub mod types {
    use super::*;
    
    /// Socket address with IPv4
    pub type SocketAddrV4 = SocketAddr;
    
    /// IPv4 address
    pub type Ipv4Address = IpAddress;
    
    /// TCP connection handle
    pub type TcpConnectionHandle = TcpConnection;
    
    /// DNS query result
    pub type DnsResult = Result<Vec<DnsRecord>>;
}