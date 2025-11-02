//! POSIX sys/types.h Compatibility
//! 
//! This module provides comprehensive sys/types.h compatibility for MultiOS,
//! including basic system type definitions while maintaining Rust safety guarantees.

use crate::internal::*;

/// Process ID type
/// 
/// This type is used to represent process identifiers and is compatible
/// with the POSIX pid_t type.
pub type pid_t = i32;

/// Process ID types for special cases
/// 
/// These constants are used with pid_t to represent special process states
/// or operations, providing compatibility with POSIX standards.

/// Use any process ID
pub const ANY_PID: pid_t = 0;

/// Use any process in the same process group
pub const ANY_PGID: pid_t = 0;

/// Use any process in the same session
pub const ANY_SID: pid_t = 0;

/// User ID type
/// 
/// This type is used to represent user identifiers and is compatible
/// with the POSIX uid_t type.
pub type uid_t = u32;

/// Group ID type
/// 
/// This type is used to represent group identifiers and is compatible
/// with the POSIX gid_t type.
pub type gid_t = u32;

/// File descriptor type
/// 
/// This type is used to represent file descriptors and is compatible
/// with the POSIX fd_t type.
pub type fd_t = i32;

/// Special file descriptor values
/// 
/// These constants represent standard file descriptors and special
/// values used with file descriptors.

/// Standard input file descriptor
pub const STDIN_FILENO: fd_t = 0;

/// Standard output file descriptor  
pub const STDOUT_FILENO: fd_t = 1;

/// Standard error file descriptor
pub const STDERR_FILENO: fd_t = 2;

/// Invalid file descriptor
pub const INVALID_FD: fd_t = -1;

/// Time types
/// 
/// These types are used to represent various time-related values.

/// Time type (seconds since epoch)
pub type time_t = i64;

/// Nanosecond time type
pub type time_nsec_t = i64;

/// Microsecond time type  
pub type suseconds_t = i32;

/// Size and offset types
/// 
/// These types are used to represent sizes, offsets, and counts.

/// Size type (unsigned)
pub type size_t = usize;

/// Signed size type
pub type ssize_t = isize;

/// File offset type
pub type off_t = i64;

/// File offset type for 32-bit systems
pub type off32_t = i32;

/// File size type
pub type filesize_t = i64;

/// Device and inode types
/// 
/// These types are used to represent device and inode information.

/// Device ID type
pub type dev_t = u64;

/// Inode number type
pub type ino_t = u64;

/// Inode number type for 32-bit systems
pub type ino32_t = u32;

/// Mode types
/// 
/// These types are used to represent file modes and permissions.

/// File mode type
pub type mode_t = u32;

/// Permission type for access modes
pub type access_mode_t = mode_t;

/// File type mode mask
pub type file_type_mask_t = mode_t;

/// Link count type
/// 
/// This type is used to represent the number of hard links to a file.
pub type nlink_t = u64;

/// Link count type for 32-bit systems
pub type nlink32_t = u32;

/// Block types
/// 
/// These types are used to represent block-related information.

/// Block size type
pub type blksize_t = i64;

/// Block count type
pub type blkcnt_t = i64;

/// Socket address length type
pub type socklen_t = u32;

/// Socket protocol type
pub type proto_t = i32;

/// Socket address family type
pub type sa_family_t = u16;

/// Clock identifier type
/// 
/// This type is used to identify different clocks in the system.
pub type clockid_t = i32;

/// Timer identifier type
/// 
/// This type is used to identify POSIX timers.
pub type timer_t = i32;

/// Signal number type
/// 
/// This type is used to represent signal numbers.
pub type signum_t = i32;

/// Thread identifier type
/// 
/// This type is used to represent thread identifiers.
pub type pthread_t = usize;

/// Thread attribute type
pub type pthread_attr_t = usize;

/// Mutex type
pub type pthread_mutex_t = usize;

/// Mutex attribute type
pub type pthread_mutexattr_t = usize;

/// Condition variable type
pub type pthread_cond_t = usize;

/// Condition variable attribute type
pub type pthread_condattr_t = usize;

/// Read-write lock type
pub type pthread_rwlock_t = usize;

/// Read-write lock attribute type
pub type pthread_rwlockattr_t = usize;

/// Barrier type
pub type pthread_barrier_t = usize;

/// Barrier attribute type
pub type pthread_barrierattr_t = usize;

/// Spinlock type
pub type pthread_spinlock_t = usize;

/// Key type for thread-specific data
pub type pthread_key_t = usize;

/// Once type for one-time initialization
pub type pthread_once_t = usize;

/// Advanced types for system administration
/// 
/// These types are used for more advanced system operations.

/// Host identifier type
pub type hostid_t = u32;

/// Machine architecture type
pub type machine_t = u32;

/// IP address type
pub type in_addr_t = u32;

/// IP address type for IPv6
pub type in6_addr_t = [u8; 16];

/// Port number type
pub type in_port_t = u16;

/// Service name type
pub type service_t = u16;

/// Resource ID type for getrusage
pub type id_t = i32;

/// Wait status type
/// 
/// This type is used to represent the status returned by wait() operations.
pub type wait_status_t = i32;

/// Wait status macros
/// 
/// These macros are used to decode wait status values.
pub mod wait {
    /// Extract the exit status
    pub fn status(status: wait_status_t) -> i32 {
        status & 0xFF00
    }
    
    /// Extract the signal number that caused the process to stop
    pub fn stop_signal(status: wait_status_t) -> i32 {
        (status >> 8) & 0xFF
    }
    
    /// Check if the process exited normally
    pub fn exited(status: wait_status_t) -> bool {
        (status & 0x7F) == 0
    }
    
    /// Get the exit status if the process exited normally
    pub fn exit_status(status: wait_status_t) -> i32 {
        status >> 8
    }
    
    /// Check if the process was terminated by a signal
    pub fn signalled(status: wait_status_t) -> bool {
        (status & 0x7F) != 0 && (status & 0x7F) != 0x7F
    }
    
    /// Get the signal number if the process was terminated by a signal
    pub fn term_signal(status: wait_status_t) -> i32 {
        status & 0x7F
    }
    
    /// Check if the process was stopped by a signal
    pub fn stopped(status: wait_status_t) -> bool {
        (status & 0xFF) == 0x7F
    }
    
    /// Check if the process continued after being stopped
    pub fn continued(status: wait_status_t) -> bool {
        status == 0xFFFF
    }
    
    /// Check if the process core dumped
    pub fn coredumped(status: wait_status_t) -> bool {
        (status & 0x80) != 0
    }
}

/// Utility functions for type conversions
/// 
/// These functions provide safe conversions between different types.
pub mod convert {
    use super::*;
    
    /// Convert size_t to ssize_t safely
    pub fn size_to_ssize(size: size_t) -> Option<ssize_t> {
        if size <= ssize_t::MAX as usize {
            Some(size as ssize_t)
        } else {
            None
        }
    }
    
    /// Convert ssize_t to size_t safely
    pub fn ssize_to_size(ssize: ssize_t) -> Option<size_t> {
        if ssize >= 0 {
            Some(ssize as size_t)
        } else {
            None
        }
    }
    
    /// Convert off_t to off32_t safely
    pub fn off_to_off32(off: off_t) -> Option<off32_t> {
        if off >= off32_t::MIN as off_t && off <= off32_t::MAX as off_t {
            Some(off as off32_t)
        } else {
            None
        }
    }
    
    /// Convert off32_t to off_t
    pub fn off32_to_off(off32: off32_t) -> off_t {
        off32 as off_t
    }
    
    /// Convert ino_t to ino32_t safely
    pub fn ino_to_ino32(ino: ino_t) -> Option<ino32_t> {
        if ino <= ino32_t::MAX as ino_t {
            Some(ino as ino32_t)
        } else {
            None
        }
    }
    
    /// Convert ino32_t to ino_t
    pub fn ino32_to_ino(ino32: ino32_t) -> ino_t {
        ino32 as ino_t
    }
    
    /// Convert nlink_t to nlink32_t safely
    pub fn nlink_to_nlink32(nlink: nlink_t) -> Option<nlink32_t> {
        if nlink <= nlink32_t::MAX as nlink_t {
            Some(nlink as nlink32_t)
        } else {
            None
        }
    }
    
    /// Convert nlink32_t to nlink_t
    pub fn nlink32_to_nlink(nlink32: nlink32_t) -> nlink_t {
        nlink32 as nlink_t
    }
}

/// Type aliases for common usage patterns
/// 
/// These aliases provide more descriptive names for commonly used types.

/// Process identifier (alias for pid_t)
pub type process_id_t = pid_t;

/// User identifier (alias for uid_t)
pub type user_id_t = uid_t;

/// Group identifier (alias for gid_t)
pub type group_id_t = gid_t;

/// File descriptor identifier (alias for fd_t)
pub type file_descriptor_t = fd_t;

/// Time seconds (alias for time_t)
pub type time_seconds_t = time_t;

/// Time nanoseconds (alias for time_nsec_t)
pub type time_nanoseconds_t = time_nsec_t;

/// Buffer size type (alias for size_t)
pub type buffer_size_t = size_t;

/// Buffer size type (alias for ssize_t)
pub type buffer_size_signed_t = ssize_t;

/// File offset type (alias for off_t)
pub type file_offset_t = off_t;

/// Device identifier (alias for dev_t)
pub type device_id_t = dev_t;

/// Inode number (alias for ino_t)
pub type inode_number_t = ino_t;

/// File mode (alias for mode_t)
pub type file_mode_t = mode_t;

/// Socket address length (alias for socklen_t)
pub type socket_address_length_t = socklen_t;

/// Clock identifier (alias for clockid_t)
pub type clock_id_t = clockid_t;

/// Thread identifier (alias for pthread_t)
pub type thread_id_t = pthread_t;

/// Maximum values for different types
/// 
/// These constants represent the maximum values for various types.

/// Maximum value for pid_t
pub const PID_MAX: pid_t = i32::MAX;

/// Maximum value for uid_t
pub const UID_MAX: uid_t = u32::MAX;

/// Maximum value for gid_t
pub const GID_MAX: gid_t = u32::MAX;

/// Maximum value for fd_t
pub const FD_MAX: fd_t = i32::MAX;

/// Maximum value for time_t
pub const TIME_MAX: time_t = i64::MAX;

/// Minimum value for time_t
pub const TIME_MIN: time_t = i64::MIN;

/// Maximum value for size_t
pub const SIZE_MAX: size_t = usize::MAX;

/// Maximum value for off_t
pub const OFF_MAX: off_t = i64::MAX;

/// Minimum value for off_t
pub const OFF_MIN: off_t = i64::MIN;

/// Maximum value for ino_t
pub const INO_MAX: ino_t = u64::MAX;

/// Maximum value for dev_t
pub const DEV_MAX: dev_t = u64::MAX;

/// Maximum value for mode_t
pub const MODE_MAX: mode_t = u32::MAX;

/// Maximum value for nlink_t
pub const NLINK_MAX: nlink_t = u64::MAX;

/// Maximum value for blksize_t
pub const BLKSIZE_MAX: blksize_t = i64::MAX;

/// Maximum value for blkcnt_t
pub const BLKCNT_MAX: blkcnt_t = i64::MAX;

/// Common constants for type ranges
/// 
/// These constants are used to represent common ranges and limits.

/// Maximum value for port numbers
pub const PORT_MAX: in_port_t = u16::MAX;

/// Maximum value for socket protocol numbers
pub const PROTO_MAX: proto_t = i32::MAX;

/// Maximum value for signal numbers
pub const SIGNAL_MAX: signum_t = i32::MAX;

/// Maximum value for clock IDs
pub const CLOCK_MAX: clockid_t = i32::MAX;

/// Maximum value for timer IDs
pub const TIMER_MAX: timer_t = i32::MAX;

/// IPv4 address type (4 bytes)
pub type ipv4_addr_t = u32;

/// IPv6 address type (16 bytes) 
pub type ipv6_addr_t = [u8; 16];

/// MAC address type (6 bytes)
pub type mac_addr_t = [u8; 6];

/// Hardware address type
pub type hw_addr_t = [u8; 8];

/// IP address family types
pub const AF_UNIX: sa_family_t = 1;     // Unix domain sockets
pub const AF_INET: sa_family_t = 2;     // Internet domain sockets
pub const AF_INET6: sa_family_t = 10;   // Internet domain sockets (IPv6)

/// Socket protocol types
pub const IPPROTO_TCP: proto_t = 6;     // Transmission Control Protocol
pub const IPPROTO_UDP: proto_t = 17;    // User Datagram Protocol
pub const IPPROTO_ICMP: proto_t = 1;    // Internet Control Message Protocol
pub const IPPROTO_ICMPV6: proto_t = 58; // Internet Control Message Protocol v6

/// File system path length limits
pub const PATH_MAX: usize = 4096;       // Maximum path length
pub const NAME_MAX: usize = 255;        // Maximum filename length

/// Utility macros for type manipulation
/// 
/// These macros provide convenient operations on types.

/// Convert a value to its absolute maximum for its type
#[macro_export]
macro_rules! type_max {
    ($t:ty) => {
        <$t>::MAX
    };
}

/// Convert a value to its absolute minimum for its type
#[macro_export]
macro_rules! type_min {
    ($t:ty) => {
        <$t>::MIN
    };
}

/// Check if a value is at its type's maximum
#[macro_export]
macro_rules! is_type_max {
    ($value:expr, $t:ty) => {
        $value == <$t>::MAX
    };
}

/// Check if a value is at its type's minimum
#[macro_export]
macro_rules! is_type_min {
    ($value:expr, $t:ty) => {
        $value == <$t>::MIN
    };
}

/// Safe cast from larger type to smaller type
#[macro_export]
macro_rules! safe_cast {
    ($value:expr, $target:ty) => {
        if $value >= <$target>::MIN as $value && $value <= <$target>::MAX as $value {
            Some($value as $target)
        } else {
            None
        }
    };
}
