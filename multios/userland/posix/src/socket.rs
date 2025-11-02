//! POSIX socket.h Compatibility
//! 
//! This module provides comprehensive socket.h compatibility for MultiOS,
//! including network socket operations, address resolution, and socket options
//! while maintaining Rust safety guarantees.

use crate::errors::*;
use crate::internal::*;
use crate::syscall;
use crate::types::*;
use crate::sys_types::*;

/// Socket address structure (generic)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr {
    pub sa_family: sa_family_t,     // Address family
    pub sa_data: [u8; 14],         // Address data
}

/// Unix domain socket address structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,    // Address family (AF_UNIX)
    pub sun_path: [u8; 108],       // Socket pathname
}

/// Internet socket address structure (IPv4)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,    // Address family (AF_INET)
    pub sin_port: in_port_t,        // Port number (network byte order)
    pub sin_addr: in_addr,          // IP address
    pub sin_zero: [u8; 8],         // Padding
}

/// Internet socket address structure (IPv6)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,   // Address family (AF_INET6)
    pub sin6_port: in_port_t,      // Port number (network byte order)
    pub sin6_flowinfo: u32,        // Traffic class and flow info
    pub sin6_addr: in6_addr,       // IPv6 address
    pub sin6_scope_id: u32,        // Scope ID
}

/// Internet address structure (IPv4)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct in_addr {
    pub s_addr: in_addr_t,         // Internet address (network byte order)
}

/// Internet address structure (IPv6)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],         // IPv6 address (16 bytes)
}

/// IP address information structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,    // IP multicast address
    pub imr_interface: in_addr,    // Local IP address
}

/// IPv6 multicast request structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ipv6_mreq {
    pub ipv6mr_multiaddr: in6_addr, // IPv6 multicast address
    pub ipv6mr_interface: u32,      // Interface index
}

/// Socket option structure for SO_LINGER
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct linger {
    pub l_onoff: i32,              // Linger active
    pub l_linger: i32,             // Linger time in seconds
}

/// Timeval structure for socket timeouts
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct timeval {
    pub tv_sec: time_t,            // Seconds
    pub tv_usec: suseconds_t,      // Microseconds
}

/// msghdr structure for sendmsg/recvmsg
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct msghdr {
    pub msg_name: *const sockaddr, // Address to send/receive from
    pub msg_namelen: socklen_t,    // Length of address data
    pub msg_iov: *const iovec,     // Scatter/gather array
    pub msg_iovlen: i32,           // Number of elements in msg_iov
    pub msg_control: *mut u8,      // Ancillary data (CMSG)
    pub msg_controllen: socklen_t, // Length of ancillary data
    pub msg_flags: i32,            // Flags on received message
}

/// iovec structure for scatter/gather I/O
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct iovec {
    pub iov_base: *mut u8,         // Base address
    pub iov_len: size_t,           // Length
}

/// cmsghdr structure for ancillary data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct cmsghdr {
    pub cmsg_len: socklen_t,       // Length of data including header
    pub cmsg_level: i32,           // Originating protocol level
    pub cmsg_type: i32,            // Protocol-specific type
    pub cmsg_data: [u8; 0],        // Data (variable length)
}

/// Protocol-independent socket creation
/// 
/// This function provides compatibility with the POSIX socket() function.
/// 
/// # Arguments
/// * `domain` - Socket domain (AF_UNIX, AF_INET, AF_INET6)
/// * `type` - Socket type (SOCK_STREAM, SOCK_DGRAM, etc.)
/// * `protocol` - Socket protocol (0 for default, or specific protocol)
/// 
/// # Returns
/// * `PosixResult<fd_t>` - Socket file descriptor on success, error on failure
pub fn socket(domain: SocketDomain, ty: SocketType, protocol: SocketProtocol) -> PosixResult<fd_t> {
    unsafe {
        let result = syscall::socket(domain, ty, protocol);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(result as fd_t)
        }
    }
}

/// Bind a socket to an address
/// 
/// This function provides compatibility with the POSIX bind() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Pointer to socket address structure
/// * `addrlen` - Length of socket address structure
/// 
/// # Returns
/// * `PosixResult<()>` - Success on bind, error on failure
pub fn bind(sockfd: fd_t, addr: &sockaddr, addrlen: socklen_t) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    unsafe {
        let result = syscall::bind(sockfd, addr as *const sockaddr, addrlen);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(())
        }
    }
}

/// Connect a socket to an address
/// 
/// This function provides compatibility with the POSIX connect() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Pointer to socket address structure
/// * `addrlen` - Length of socket address structure
/// 
/// # Returns
/// * `PosixResult<()>` - Success on connect, error on failure
pub fn connect(sockfd: fd_t, addr: &sockaddr, addrlen: socklen_t) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    unsafe {
        let result = syscall::connect(sockfd, addr as *const sockaddr, addrlen);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(())
        }
    }
}

/// Listen for incoming connections
/// 
/// This function provides compatibility with the POSIX listen() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `backlog` - Maximum length of pending connection queue
/// 
/// # Returns
/// * `PosixResult<()>` - Success on listen, error on failure
pub fn listen(sockfd: fd_t, backlog: i32) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if backlog < 0 {
        return Err(Errno::Einval);
    }
    
    unsafe {
        let result = syscall::listen(sockfd, backlog);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(())
        }
    }
}

/// Accept a new connection
/// 
/// This function provides compatibility with the POSIX accept() function.
/// 
/// # Arguments
/// * `sockfd` - Listening socket file descriptor
/// * `addr` - Pointer to store peer address (NULL to ignore)
/// * `addrlen` - Pointer to store length of peer address (NULL to ignore)
/// 
/// # Returns
/// * `PosixResult<fd_t>` - Connected socket file descriptor, error on failure
pub fn accept(sockfd: fd_t, addr: Option<&mut sockaddr>, addrlen: Option<&mut socklen_t>) -> PosixResult<fd_t> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    unsafe {
        let addr_ptr = addr.map_or(core::ptr::null_mut(), |a| a as *mut sockaddr);
        let len_ptr = addrlen.map_or(core::ptr::null_mut(), |l| l as *mut socklen_t);
        
        let result = syscall::accept(sockfd, addr_ptr, len_ptr);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(result as fd_t)
        }
    }
}

/// Accept a new connection with flags
/// 
/// This function provides compatibility with the POSIX accept4() function.
/// 
/// # Arguments
/// * `sockfd` - Listening socket file descriptor
/// * `addr` - Pointer to store peer address (NULL to ignore)
/// * `addrlen` - Pointer to store length of peer address (NULL to ignore)
/// * `flags` - Additional flags (SOCK_NONBLOCK, SOCK_CLOEXEC)
/// 
/// # Returns
/// * `PosixResult<fd_t>` - Connected socket file descriptor, error on failure
pub fn accept4(sockfd: fd_t, addr: Option<&mut sockaddr>, addrlen: Option<&mut socklen_t>, flags: i32) -> PosixResult<fd_t> {
    // In a real implementation, this would call syscall::accept4
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Send data on a socket
/// 
/// This function provides compatibility with the POSIX send() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `buf` - Buffer containing data to send
/// * `flags` - Send flags
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes sent, error on failure
pub fn send(sockfd: fd_t, buf: &[u8], flags: i32) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if buf.is_empty() {
        return Ok(0);
    }
    
    unsafe {
        let result = syscall::send(sockfd, buf.as_ptr(), buf.len(), flags);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(result as usize)
        }
    }
}

/// Send data to a specific address
/// 
/// This function provides compatibility with the POSIX sendto() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `buf` - Buffer containing data to send
/// * `flags` - Send flags
/// * `dest_addr` - Destination address (NULL for connected socket)
/// * `addrlen` - Length of destination address
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes sent, error on failure
pub fn sendto(sockfd: fd_t, buf: &[u8], flags: i32, dest_addr: Option<&sockaddr>, addrlen: socklen_t) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if buf.is_empty() {
        return Ok(0);
    }
    
    // In a real implementation, this would call syscall::sendto
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Send data using message structure
/// 
/// This function provides compatibility with the POSIX sendmsg() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `msg` - Pointer to message structure
/// * `flags` - Send flags
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes sent, error on failure
pub fn sendmsg(sockfd: fd_t, msg: &msghdr, flags: i32) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::sendmsg
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Receive data from a socket
/// 
/// This function provides compatibility with the POSIX recv() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `buf` - Buffer to receive data into
/// * `flags` - Receive flags
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes received, error on failure
pub fn recv(sockfd: fd_t, buf: &mut [u8], flags: i32) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if buf.is_empty() {
        return Ok(0);
    }
    
    unsafe {
        let result = syscall::recv(sockfd, buf.as_mut_ptr(), buf.len(), flags);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(result as usize)
        }
    }
}

/// Receive data from a specific address
/// 
/// This function provides compatibility with the POSIX recvfrom() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `buf` - Buffer to receive data into
/// * `flags` - Receive flags
/// * `from_addr` - Pointer to store source address (NULL to ignore)
/// * `addrlen` - Pointer to store length of source address (NULL to ignore)
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes received, error on failure
pub fn recvfrom(sockfd: fd_t, buf: &mut [u8], flags: i32, from_addr: Option<&mut sockaddr>, addrlen: Option<&mut socklen_t>) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if buf.is_empty() {
        return Ok(0);
    }
    
    // In a real implementation, this would call syscall::recvfrom
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Receive data using message structure
/// 
/// This function provides compatibility with the POSIX recvmsg() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `msg` - Pointer to message structure
/// * `flags` - Receive flags
/// 
/// # Returns
/// * `PosixResult<usize>` - Number of bytes received, error on failure
pub fn recvmsg(sockfd: fd_t, msg: &mut msghdr, flags: i32) -> PosixResult<usize> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::recvmsg
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Shut down part of a socket connection
/// 
/// This function provides compatibility with the POSIX shutdown() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `how` - How to shut down (SHUT_RD, SHUT_WR, SHUT_RDWR)
/// 
/// # Returns
/// * `PosixResult<()>` - Success on shutdown, error on failure
pub fn shutdown(sockfd: fd_t, how: i32) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    if how != SHUT_RD && how != SHUT_WR && how != SHUT_RDWR {
        return Err(Errno::Einval);
    }
    
    unsafe {
        let result = syscall::shutdown(sockfd, how);
        if result < 0 {
            Err(Errno::from_raw(-result))
        } else {
            Ok(())
        }
    }
}

/// Get socket name (local address)
/// 
/// This function provides compatibility with the POSIX getsockname() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Pointer to store local address
/// * `addrlen` - Pointer to length of local address buffer
/// 
/// # Returns
/// * `PosixResult<()>` - Success on getsockname, error on failure
pub fn getsockname(sockfd: fd_t, addr: &mut sockaddr, addrlen: &mut socklen_t) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::getsockname
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Get peer name (remote address)
/// 
/// This function provides compatibility with the POSIX getpeername() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Pointer to store peer address
/// * `addrlen` - Pointer to length of peer address buffer
/// 
/// # Returns
/// * `PosixResult<()>` - Success on getpeername, error on failure
pub fn getpeername(sockfd: fd_t, addr: &mut sockaddr, addrlen: &mut socklen_t) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::getpeername
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Get socket option
/// 
/// This function provides compatibility with the POSIX getsockopt() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `level` - Protocol level (SOL_SOCKET, IPPROTO_TCP, etc.)
/// * `optname` - Option name
/// * `optval` - Pointer to store option value
/// * `optlen` - Pointer to length of option value buffer
/// 
/// # Returns
/// * `PosixResult<()>` - Success on getsockopt, error on failure
pub fn getsockopt(sockfd: fd_t, level: i32, optname: i32, optval: &mut [u8], optlen: &mut socklen_t) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::getsockopt
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Set socket option
/// 
/// This function provides compatibility with the POSIX setsockopt() function.
/// 
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `level` - Protocol level (SOL_SOCKET, IPPROTO_TCP, etc.)
/// * `optname` - Option name
/// * `optval` - Pointer to option value
/// * `optlen` - Length of option value
/// 
/// # Returns
/// * `PosixResult<()>` - Success on setsockopt, error on failure
pub fn setsockopt(sockfd: fd_t, level: i32, optname: i32, optval: &[u8]) -> PosixResult<()> {
    if sockfd < 0 {
        return Err(Errno::Ebadf);
    }
    
    // In a real implementation, this would call syscall::setsockopt
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// Create a pair of connected sockets
/// 
/// This function provides compatibility with the POSIX socketpair() function.
/// 
/// # Arguments
/// * `domain` - Socket domain
/// * `type` - Socket type
/// * `protocol` - Socket protocol
/// * `sv` - Array to store socket pair file descriptors
/// 
/// # Returns
/// * `PosixResult<()>` - Success on socketpair, error on failure
pub fn socketpair(domain: i32, ty: i32, protocol: i32, sv: &mut [fd_t; 2]) -> PosixResult<()> {
    if domain != AF_UNIX {
        return Err(Errno::Eafnosupport);
    }
    
    // In a real implementation, this would call syscall::socketpair
    // For now, return not implemented
    Err(Errno::Enosys)
}

/// IP address conversion functions
/// 
/// These functions provide IP address conversion utilities.
pub mod inet {
    use super::*;
    
    /// Convert IPv4 address string to binary form
    /// 
    /// This function provides compatibility with the POSIX inet_addr() function.
    /// 
    /// # Arguments
    /// * `cp` - IPv4 address string
    /// 
    /// # Returns
    /// * `PosixResult<in_addr_t>` - Binary IP address in network byte order
    pub fn addr(cp: &str) -> PosixResult<in_addr_t> {
        // Simple IPv4 address parsing
        let parts: Vec<&str> = cp.split('.').collect();
        if parts.len() != 4 {
            return Err(Errno::Einval);
        }
        
        let mut result: in_addr_t = 0;
        for (i, part) in parts.iter().enumerate() {
            let num = part.parse::<u32>().map_err(|_| Errno::Einval)?;
            if num > 255 {
                return Err(Errno::Einval);
            }
            result |= num << (24 - i * 8);
        }
        
        Ok(htonl(result))
    }
    
    /// Convert IPv4 address from binary form to string
    /// 
    /// This function provides compatibility with the POSIX inet_ntoa() function.
    /// 
    /// # Arguments
    /// * `inp` - Binary IP address in network byte order
    /// 
    /// # Returns
    /// * `String` - IPv4 address string
    pub fn ntoa(inp: in_addr_t) -> String {
        let addr = htonl(inp);
        format!("{}.{}.{}.{}", 
            (addr >> 24) & 0xFF,
            (addr >> 16) & 0xFF,
            (addr >> 8) & 0xFF,
            addr & 0xFF
        )
    }
    
    /// Convert IPv6 address string to binary form
    /// 
    /// This function provides compatibility with the POSIX inet_pton() function.
    /// 
    /// # Arguments
    /// * `family` - Address family (AF_INET or AF_INET6)
    /// * `src` - IPv6 address string
    /// * `dst` - Buffer to store binary address
    /// 
    /// # Returns
    /// * `PosixResult<i32>` - 1 on success, 0 if invalid, error on failure
    pub fn pton(family: i32, src: &str, dst: &mut [u8]) -> PosixResult<i32> {
        if family == AF_INET {
            // IPv4 address
            if dst.len() < 4 {
                return Err(Errno::Eoverflow);
            }
            
            let addr = addr(src)?;
            unsafe {
                core::ptr::write_unaligned(dst.as_mut_ptr() as *mut u32, addr.to_be());
            }
            Ok(1)
        } else if family == AF_INET6 {
            // IPv6 address - simplified implementation
            if dst.len() < 16 {
                return Err(Errno::Eoverflow);
            }
            
            // Basic IPv6 parsing (simplified)
            let parts: Vec<&str> = src.split(':').collect();
            if parts.len() != 8 {
                return Ok(0); // Invalid format
            }
            
            for (i, part) in parts.iter().enumerate() {
                if i < 4 {
                    // First 4 parts (16 bytes total)
                    let num = part.parse::<u64>().unwrap_or(0);
                    let bytes = num.to_be_bytes();
                    let offset = i * 2;
                    dst[offset..offset + 2].copy_from_slice(&bytes[6..8]);
                    dst[offset + 1] = bytes[7];
                    dst[offset + 2] = bytes[6];
                    dst[offset + 3] = bytes[5];
                    dst[offset + 4] = bytes[4];
                    dst[offset + 5] = bytes[3];
                    dst[offset + 6] = bytes[2];
                    dst[offset + 7] = bytes[1];
                    dst[offset + 8] = bytes[0];
                }
            }
            Ok(1)
        } else {
            Err(Errno::Eafnosupport)
        }
    }
    
    /// Convert IPv6 address from binary form to string
    /// 
    /// This function provides compatibility with the POSIX inet_ntop() function.
    /// 
    /// # Arguments
    /// * `family` - Address family (AF_INET or AF_INET6)
    /// * `src` - Binary address
    /// * `dst` - Buffer to store string
    /// * `size` - Size of destination buffer
    /// 
    /// # Returns
    /// * `PosixResult<*const u8>` - Pointer to destination string, error on failure
    pub fn ntop(family: i32, src: &[u8], dst: &mut [u8]) -> PosixResult<*const u8> {
        if family == AF_INET {
            if src.len() < 4 {
                return Err(Errno::Eoverflow);
            }
            if dst.len() < 16 {
                return Err(Errno::Eoverflow);
            }
            
            let addr = u32::from_be_bytes([src[0], src[1], src[2], src[3]]);
            let addr_str = ntoa(addr);
            
            if dst.len() < addr_str.len() + 1 {
                return Err(Errno::Eoverflow);
            }
            
            dst[..addr_str.len()].copy_from_slice(addr_str.as_bytes());
            dst[addr_str.len()] = 0;
            
            Ok(dst.as_ptr())
        } else if family == AF_INET6 {
            if src.len() < 16 {
                return Err(Errno::Eoverflow);
            }
            if dst.len() < 46 {
                return Err(Errno::Eoverflow);
            }
            
            // Simplified IPv6 formatting
            let parts: Vec<String> = (0..16).step_by(2)
                .map(|i| format!("{:02x}", (src[i] as u16) << 8 | src[i + 1] as u16))
                .collect();
            
            let addr_str = parts.join(":");
            if dst.len() < addr_str.len() + 1 {
                return Err(Errno::Eoverflow);
            }
            
            dst[..addr_str.len()].copy_from_slice(addr_str.as_bytes());
            dst[addr_str.len()] = 0;
            
            Ok(dst.as_ptr())
        } else {
            Err(Errno::Eafnosupport)
        }
    }
}

/// Address family constants
pub const AF_UNIX: sa_family_t = 1;         // Unix domain sockets
pub const AF_INET: sa_family_t = 2;         // Internet domain sockets (IPv4)
pub const AF_INET6: sa_family_t = 10;       // Internet domain sockets (IPv6)
pub const AF_IPX: sa_family_t = 4;          // IPX protocols
pub const AF_APPLETALK: sa_family_t = 5;    // AppleTalk
pub const AF_NETLINK: sa_family_t = 16;     // Kernel netlink socket
pub const AF_X25: sa_family_t = 9;          // ITU-T X.25 / ISO-8208
pub const AF_AX25: sa_family_t = 3;         // Amateur radio AX.25
pub const AF_ATMPVC: sa_family_t = 8;       // Access to raw ATM PVCs
pub const AF_PACKET: sa_family_t = 17;      // Low level packet interface
pub const AF_ALG: sa_family_t = 26;         // Linux Kernel crypto API
pub const AF_UNSPEC: sa_family_t = 0;       // Unspecified

/// Socket type constants
pub const SOCK_STREAM: i32 = 1;             // Stream (connection) socket
pub const SOCK_DGRAM: i32 = 2;              // Datagram (connectionless) socket
pub const SOCK_RAW: i32 = 3;                // Raw protocol interface
pub const SOCK_RDM: i32 = 4;                // Reliably-delivered messages
pub const SOCK_SEQPACKET: i32 = 5;          // Sequenced, reliable, connection-based

/// Socket flags
pub const SOCK_NONBLOCK: i32 = 0x0400;      // Non-blocking socket
pub const SOCK_CLOEXEC: i32 = 0x80000;      // Close-on-exec socket

/// Protocol family (usually same as address family)
pub const PF_UNSPEC: i32 = 0;
pub const PF_UNIX: i32 = 1;
pub const PF_INET: i32 = 2;
pub const PF_INET6: i32 = 10;

/// Protocol constants
pub const IPPROTO_IP: i32 = 0;              // Dummy protocol for IP
pub const IPPROTO_ICMP: i32 = 1;            // Internet Control Message Protocol
pub const IPPROTO_IGMP: i32 = 2;            // Internet Group Management Protocol
pub const IPPROTO_TCP: i32 = 6;             // Transmission Control Protocol
pub const IPPROTO_PUP: i32 = 12;            // PUP protocol
pub const IPPROTO_UDP: i32 = 17;            // User Datagram Protocol
pub const IPPROTO_IDP: i32 = 22;            // XNS IDP protocol
pub const IPPROTO_RAW: i32 = 255;           // Raw IP packet
pub const IPPROTO_IPV6: i32 = 41;           // IPv6-in-IPv4 tunnelling

/// Socket shutdown modes
pub const SHUT_RD: i32 = 0;                 // Further receptions disallowed
pub const SHUT_WR: i32 = 1;                 // Further transmissions disallowed  
pub const SHUT_RDWR: i32 = 2;               // Further receptions and transmissions disallowed

/// Socket option levels
pub const SOL_SOCKET: i32 = 1;              // Socket-level options
pub const IPPROTO_TCP: i32 = 6;             // TCP protocol options
pub const IPPROTO_IP: i32 = 0;              // IP protocol options
pub const IPPROTO_IPV6: i32 = 41;           // IPv6 protocol options

/// Socket options (for SOL_SOCKET level)
pub const SO_REUSEADDR: i32 = 2;            // Allow reuse of local addresses
pub const SO_TYPE: i32 = 3;                 // Get socket type
pub const SO_ERROR: i32 = 4;                // Get and clear error
pub const SO_DONTROUTE: i32 = 5;            // Bypass routing table lookup
pub const SO_BROADCAST: i32 = 6;            // Permit sending of broadcast messages
pub const SO_SNDBUF: i32 = 7;               // Send buffer size
pub const SO_RCVBUF: i32 = 8;               // Receive buffer size
pub const SO_SNDBUFFORCE: i32 = 32;         // Send buffer force
pub const SO_RCVBUFFORCE: i32 = 33;         // Receive buffer force
pub const SO_KEEPALIVE: i32 = 9;            // Keep connections alive
pub const SO_OOBINLINE: i32 = 10;           // Keep out-of-band data in-band
pub const SO_NO_CHECK: i32 = 11;            // Disable checksums
pub const SO_PRIORITY: i32 = 12;            // Set the priority for all packets
pub const SO_LINGER: i32 = 13;              // Linger on close if unsent data present
pub const SO_BSDCOMPAT: i32 = 14;           // BSD bug fixes
pub const SO_REUSEPORT: i32 = 15;           // Allow reuse of port
pub const SO_PASSCRED: i32 = 16;            // Pass credentials
pub const SO_PEERCRED: i32 = 17;            // Get peer credentials
pub const SO_RCVLOWAT: i32 = 18;            // Receive low-water mark
pub const SO_SNDLOWAT: i32 = 19;            // Send low-water mark
pub const SO_RCVTIMEO: i32 = 20;            // Receive timeout
pub const SO_SNDTIMEO: i32 = 21;            // Send timeout
pub const SO_TIMESTAMP: i32 = 29;           // Timestamp incoming packets
pub const SO_ACCEPTCONN: i32 = 30;          // Socket has accept() call
pub const SO_PROTOCOL: i32 = 38;            // Protocol number
pub const SO_DOMAIN: i32 = 39;              // Domain name

/// TCP socket options
pub const TCP_NODELAY: i32 = 1;             // Disable Nagle algorithm
pub const TCP_MAXSEG: i32 = 2;              // Maximum segment size
pub const TCP_CORK: i32 = 3;                // Don't send partial frames
pub const TCP_KEEPIDLE: i32 = 4;            // Time before keepalive probes are sent
pub const TCP_KEEPINTVL: i32 = 5;           // Time between keepalive probes
pub const TCP_KEEPCNT: i32 = 6;             // Number of keepalive probes
pub const TCP_SYNCNT: i32 = 7;              // Number of SYN retransmits
pub const TCP_LINGER2: i32 = 8;             // Lifetime of orphaned FIN-WAIT-2 state
pub const TCP_DEFER_ACCEPT: i32 = 9;        // Wake up listener only when data arrives
pub const TCP_WINDOW_CLAMP: i32 = 10;       // Bound advertised window
pub const TCP_INFO: i32 = 11;               // Information about this socket
pub const TCP_QUICKACK: i32 = 12;           // Enable quickack mode

/// MSG flags for sendmsg/recvmsg
pub const MSG_OOB: i32 = 0x01;              // Process out-of-band data
pub const MSG_PEEK: i32 = 0x02;             // Peek at incoming data
pub const MSG_DONTROUTE: i32 = 0x04;        // Send without using routing tables
pub const MSG_TRUNC: i32 = 0x08;            // Normal data truncated
pub const MSG_CTRUNC: i32 = 0x10;           // Control data truncated
pub const MSG_WAITALL: i32 = 0x100;         // Wait for complete request
pub const MSG_DONTWAIT: i32 = 0x40;         // Non-blocking operation
pub const MSG_EOF: i32 = 0x100;             // Data completes transaction
pub const MSG_NOSIGNAL: i32 = 0x4000;       // Do not generate SIGPIPE
pub const MSG_MORE: i32 = 0x8000;           // More data coming

/// Address utility functions
pub mod addr {
    use super::*;
    
    /// Get the size of a socket address structure
    pub fn sizeof(sa_family: sa_family_t) -> usize {
        match sa_family {
            AF_UNIX => core::mem::size_of::<sockaddr_un>(),
            AF_INET => core::mem::size_of::<sockaddr_in>(),
            AF_INET6 => core::mem::size_of::<sockaddr_in6>(),
            _ => core::mem::size_of::<sockaddr>(),
        }
    }
    
    /// Create a Unix domain socket address
    pub fn unix(path: &str) -> PosixResult<sockaddr_un> {
        if path.len() > 107 {
            return Err(Errno::Enametoolong);
        }
        
        let mut addr = sockaddr_un {
            sun_family: AF_UNIX as sa_family_t,
            sun_path: [0; 108],
        };
        
        addr.sun_path[..path.len()].copy_from_slice(path.as_bytes());
        
        Ok(addr)
    }
    
    /// Create an IPv4 socket address
    pub fn ipv4(port: u16, addr: in_addr_t) -> sockaddr_in {
        sockaddr_in {
            sin_family: AF_INET as sa_family_t,
            sin_port: port.to_be(),
            sin_addr: in_addr { s_addr: addr },
            sin_zero: [0; 8],
        }
    }
    
    /// Create an IPv6 socket address
    pub fn ipv6(port: u16, addr: in6_addr, flowinfo: u32, scope_id: u32) -> sockaddr_in6 {
        sockaddr_in6 {
            sin6_family: AF_INET6 as sa_family_t,
            sin6_port: port.to_be(),
            sin6_flowinfo: flowinfo,
            sin6_addr: addr,
            sin6_scope_id: scope_id,
        }
    }
    
    /// Create an IPv4 loopback address
    pub fn ipv4_loopback(port: u16) -> sockaddr_in {
        let loopback = htonl(0x7F000001); // 127.0.0.1
        ipv4(port, loopback)
    }
    
    /// Create an IPv6 loopback address
    pub fn ipv6_loopback(port: u16) -> sockaddr_in6 {
        let mut loopback = in6_addr { s6_addr: [0; 16] };
        loopback.s6_addr[15] = 1; // ::1
        ipv6(port, loopback, 0, 0)
    }
    
    /// Create an IPv4 any address (INADDR_ANY)
    pub fn ipv4_any(port: u16) -> sockaddr_in {
        ipv4(port, 0) // 0.0.0.0
    }
    
    /// Create an IPv6 any address (IN6ADDR_ANY)
    pub fn ipv6_any(port: u16) -> sockaddr_in6 {
        let any = in6_addr { s6_addr: [0; 16] };
        ipv6(port, any, 0, 0)
    }
    
    /// Get the length of a socket address structure
    pub fn get_length(addr: &sockaddr) -> socklen_t {
        match addr.sa_family {
            AF_UNIX => {
                let unix_addr = addr as *const sockaddr as *const sockaddr_un;
                unsafe {
                    let unix_addr = &*unix_addr;
                    let mut len = 2; // sun_family + null terminator
                    while len < 108 && unix_addr.sun_path[len - 2] != 0 {
                        len += 1;
                    }
                    len as socklen_t
                }
            }
            AF_INET => core::mem::size_of::<sockaddr_in>() as socklen_t,
            AF_INET6 => core::mem::size_of::<sockaddr_in6>() as socklen_t,
            _ => core::mem::size_of::<sockaddr>() as socklen_t,
        }
    }
}
