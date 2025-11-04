//! Internal System Call Interfaces
//! 
//! This module provides the internal system call interface that links user-space
//! POSIX APIs to the kernel. It handles parameter marshalling, validation,
//! and error translation between user-space and kernel.

use crate::types::*;
use crate::errors::*;

/// System call result structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallResult {
    pub return_value: isize,
    pub error_code: i32,
}

impl SyscallResult {
    pub fn new(return_value: isize, error_code: i32) -> Self {
        Self {
            return_value,
            error_code,
        }
    }

    pub fn is_error(&self) -> bool {
        self.return_value < 0
    }

    pub fn to_posix_result<T>(&self) -> PosixResult<T>
    where
        T: From<isize>,
    {
        if self.return_value < 0 {
            Err(Errno::from_raw(self.error_code))
        } else {
            Ok(self.return_value.into())
        }
    }
}

/// Complete stat structure (aligned with POSIX stat)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: dev_t,           // Device ID
    pub st_ino: ino_t,           // Inode number
    pub st_mode: mode_t,         // File type and permissions
    pub st_nlink: nlink_t,       // Number of hard links
    pub st_uid: uid_t,           // User ID of owner
    pub st_gid: gid_t,           // Group ID of owner
    pub st_rdev: dev_t,          // Device ID (if special file)
    pub st_size: off_t,          // Total size in bytes
    pub st_blksize: blksize_t,   // Block size for filesystem I/O
    pub st_blocks: blkcnt_t,     // Number of 512-byte blocks allocated
    pub st_atime: time_t,        // Last access time
    pub st_mtime: time_t,        // Last modification time
    pub st_ctime: time_t,        // Last status change time
    pub st_atime_nsec: i64,      // Nanoseconds part of last access time
    pub st_mtime_nsec: i64,      // Nanoseconds part of last modification time
    pub st_ctime_nsec: i64,      // Nanoseconds part of last status change time
}

/// Device ID type
pub type dev_t = u64;

/// Inode number type
pub type ino_t = u64;

/// Number of links type
pub type nlink_t = u64;

/// Block size type
pub type blksize_t = i64;

/// Block count type
pub type blkcnt_t = i64;

/// Socket address structure (simplified)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr {
    pub sa_family: u16,          // Address family
    pub sa_data: [u8; 14],       // Address data
}

/// Socket address length type
pub type socklen_t = u32;

/// Time structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct timeval {
    pub tv_sec: time_t,          // Seconds
    pub tv_usec: suseconds_t,    // Microseconds
}

/// Microseconds type
pub type suseconds_t = i32;

/// Timezone structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct timezone {
    pub tz_minuteswest: i32,     // Minutes west of UTC
    pub tz_dsttime: i32,         // Type of DST correction
}

/// Timespec structure for clock operations
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct timespec {
    pub tv_sec: time_t,          // Seconds
    pub tv_nsec: i64,            // Nanoseconds
}

/// Itimerval structure for interval timers
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct itimerval {
    pub it_interval: timeval,    // Timer interval
    pub it_value: timeval,       // Timer value
}

/// Rusage structure for resource usage
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct rusage {
    pub ru_utime: timeval,       // User CPU time used
    pub ru_stime: timeval,       // System CPU time used
    pub ru_maxrss: i64,          // Maximum resident set size
    pub ru_ixrss: i64,           // Integral shared memory size
    pub ru_idrss: i64,           // Integral unshared data size
    pub ru_isrss: i64,           // Integral unshared stack size
    pub ru_minflt: i64,          // Page reclaims (soft page faults)
    pub ru_majflt: i64,          // Page faults (hard page faults)
    pub ru_nswap: i64,           // Swaps out of main memory
    pub ru_inblock: i64,         // Block input operations
    pub ru_oublock: i64,         // Block output operations
    pub ru_msgsnd: i64,          // Messages sent
    pub ru_msgrcv: i64,          // Messages received
    pub ru_nsignals: i64,        // Signals received
    pub ru_nvcsw: i64,           // Voluntary context switches
    pub ru_nivcsw: i64,          // Involuntary context switches
}

/// Signal action structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sigaction {
    pub sa_handler: usize,       // Signal handler
    pub sa_mask: sigset_t,       // Signal mask
    pub sa_flags: i32,           // Signal flags
    pub sa_restorer: usize,      // Signal restorer
}

/// Signal mask type
pub type sigset_t = u64;

/// Signal information structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct siginfo {
    pub si_signo: i32,           // Signal number
    pub si_code: i32,            // Signal code
    pub si_errno: i32,           // Error number
    pub si_addr: usize,          // Faulting address
    pub si_band: i64,            // Band event
    pub si_fd: i32,              // File descriptor
    pub si_timer1: i32,          // Timer ID
    pub si_timer2: i32,          // Timer ID
    pub si_uid: uid_t,           // User ID
    pub si_pid: pid_t,           // Process ID
    pub si_addr_lsb: i16,        // Address least significant bit
    pub si_call_addr: usize,     // Faulting instruction address
    pub si_syscall: i32,         // System call number
    pub si_arch: u32,            // Architecture
}

/// File system information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct statvfs {
    pub f_bsize: u64,            // File system block size
    pub f_frsize: u64,           // Fragment size
    pub f_blocks: u64,           // Total number of blocks
    pub f_bfree: u64,            // Free blocks
    pub f_bavail: u64,           // Available blocks (for non-root)
    pub f_files: u64,            // Total number of file nodes
    pub f_ffree: u64,            // Free file nodes
    pub f_favail: u64,           // Available file nodes (for non-root)
    pub f_flag: u64,             // Mount flags
    pub f_namemax: u64,          // Maximum filename length
}

/// Flock structure for file locking
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct flock {
    pub l_type: i16,             // Lock type (F_RDLCK, F_WRLCK, F_UNLCK)
    pub l_whence: i16,           // How to interpret l_start
    pub l_start: off_t,          // Starting offset for lock
    pub l_len: off_t,            // Number of bytes to lock
    pub l_pid: pid_t,            // Process holding the lock
}

/// Directory entry structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct dirent {
    pub d_ino: ino_t,            // Inode number
    pub d_off: off_t,            // Offset to next dirent
    pub d_reclen: u16,           // Length of this dirent
    pub d_type: u8,              // File type
    pub d_name: [u8; 256],       // Filename
}

/// Pollfd structure for poll/select
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct pollfd {
    pub fd: fd_t,                // File descriptor
    pub events: i16,             // Requested events
    pub revents: i16,            // Returned events
}

/// Termios structure for terminal I/O
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct termios {
    pub c_iflag: u32,            // Input modes
    pub c_oflag: u32,            // Output modes
    pub c_cflag: u32,            // Control modes
    pub c_lflag: u32,            // Local modes
    pub c_line: u8,              // Line discipline
    pub c_cc: [u8; 32],          // Special characters
    pub c_ispeed: u32,           // Input speed
    pub c_ospeed: u32,           // Output speed
}

/// Utsname structure for system information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct utsname {
    pub sysname: [u8; 65],       // System name
    pub nodename: [u8; 65],      // Node name
    pub release: [u8; 65],       // Release
    pub version: [u8; 65],       // Version
    pub machine: [u8; 65],       // Machine
    pub domainname: [u8; 65],    // Domain name
}

/// Clockid structure for clock operations
pub type clockid_t = i32;

/// Timer structure for POSIX timers
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct itimerspec {
    pub it_interval: timespec,   // Timer interval
    pub it_value: timespec,      // Timer value
}

/// Timer identifier type
pub type timer_t = i32;

/// Signal event structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sigevent {
    pub sigev_notify: i32,       // Notification method
    pub sigev_signo: i32,        // Signal number
    pub sigev_value: u64,        // Signal value
    pub sigev_notify_function: usize, // Notification function
    pub sigev_notify_attributes: usize, // Notification attributes
}

/// Memory region information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct vm_region {
    pub start: usize,            // Region start address
    pub end: usize,              // Region end address
    pub protection: i32,         // Protection flags
    pub flags: u32,              // Region flags
    pub offset: off_t,           // File offset (if mapped)
    pub pathname: [u8; 256],     // Mapped file path (if any)
}

/// CPU information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct cpuinfo {
    pub processor: i32,          // Processor number
    pub vendor_id: [u8; 13],    // Vendor ID
    pub cpu_family: i32,         // CPU family
    pub model: i32,              // Model
    pub model_name: [u8; 65],   // Model name
    pub stepping: i32,           // Stepping
    pub cpu_mhz: f64,            // CPU frequency in MHz
    pub cache_size_kb: u64,      // Cache size in KB
    pub physical_id: i32,        // Physical processor ID
    pub siblings: i32,           // Sibling processors
    pub core_id: i32,            // Core ID
    pub cpu_cores: i32,          // CPU cores
    pub apicid: i32,             // APIC ID
    pub fpu: i32,                // FPU present
    pub fpu_exception: i32,      // FPU exception support
    pub cpuid_level: i32,        // CPUID level
    pub wp: i32,                 // Write protection
    pub flags: [u8; 1024],       // CPU flags
}

/// System memory information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sysinfo {
    pub uptime: time_t,          // Seconds since boot
    pub loads: [u64; 3],         // 1, 5, and 15 minute load averages
    pub totalram: u64,           // Total usable main memory size
    pub freeram: u64,            // Available memory size
    pub sharedram: u64,          // Amount of shared memory
    pub bufferram: u64,          // Memory used by buffers
    pub totalswap: u64,          // Total swap space size
    pub freeswap: u64,           // Swap space still available
    pub procs: u16,              // Number of current processes
    pub totalhigh: u64,          // Total high memory size
    pub freehigh: u64,           // Available high memory size
    pub mem_unit: u32,           // Memory unit size in bytes
}

/// File status flags for fstat
pub const S_IFMT: mode_t = 0o170000;   // File type mask
pub const S_IFREG: mode_t = 0o100000;  // Regular file
pub const S_IFDIR: mode_t = 0o040000;  // Directory
pub const S_IFLNK: mode_t = 0o120000;  // Symbolic link
pub const S_IFBLK: mode_t = 0o060000;  // Block device
pub const S_IFCHR: mode_t = 0o020000;  // Character device
pub const S_IFIFO: mode_t = 0o010000;  // FIFO
pub const S_IFSOCK: mode_t = 0o140000; // Socket

/// File access modes
pub const F_OK: mode_t = 0;    // Test for existence
pub const X_OK: mode_t = 1;    // Test for execute permission
pub const W_OK: mode_t = 2;    // Test for write permission
pub const R_OK: mode_t = 4;    // Test for read permission

/// Seek modes
pub const SEEK_SET: i32 = 0;   // Seek relative to beginning of file
pub const SEEK_CUR: i32 = 1;   // Seek relative to current file position
pub const SEEK_END: i32 = 2;   // Seek relative to end of file
pub const SEEK_DATA: i32 = 3;  // Seek to next data
pub const SEEK_HOLE: i32 = 4;  // Seek to next hole

/// File lock types
pub const F_RDLCK: i16 = 0;    // Shared or read lock
pub const F_WRLCK: i16 = 1;    // Exclusive or write lock
pub const F_UNLCK: i16 = 2;    // Unlock

/// File control commands
pub const F_DUPFD: i32 = 0;    // Duplicate file descriptor
pub const F_GETFD: i32 = 1;    // Get file descriptor flags
pub const F_SETFD: i32 = 2;    // Set file descriptor flags
pub const F_GETFL: i32 = 3;    // Get file status flags
pub const F_SETFL: i32 = 4;    // Set file status flags
pub const F_GETLK: i32 = 5;    // Get record locking information
pub const F_SETLK: i32 = 6;    // Set record locking information
pub const F_SETLKW: i32 = 7;   // Set record locking information and wait

/// File descriptor flags
pub const FD_CLOEXEC: i32 = 1; // Close on exec

/// Socket address families
pub const AF_UNIX: i32 = 1;    // Unix domain sockets
pub const AF_INET: i32 = 2;    // Internet domain sockets
pub const AF_INET6: i32 = 10;  // Internet domain sockets (IPv6)
pub const AF_PACKET: i32 = 17; // Packet family

/// Socket types
pub const SOCK_STREAM: i32 = 1;    // Stream socket
pub const SOCK_DGRAM: i32 = 2;     // Datagram socket
pub const SOCK_RAW: i32 = 3;       // Raw socket
pub const SOCK_RDM: i32 = 4;       // Reliable datagram socket
pub const SOCK_SEQPACKET: i32 = 5; // Sequenced packet socket

/// Socket protocol numbers
pub const IPPROTO_TCP: i32 = 6;    // Transmission Control Protocol
pub const IPPROTO_UDP: i32 = 17;   // User Datagram Protocol
pub const IPPROTO_ICMP: i32 = 1;   // Internet Control Message Protocol
pub const IPPROTO_ICMPV6: i32 = 58; // Internet Control Message Protocol v6

/// Socket option levels
pub const SOL_SOCKET: i32 = 1;     // Socket level

/// Socket options
pub const SO_REUSEADDR: i32 = 2;   // Allow reuse of local addresses
pub const SO_REUSEPORT: i32 = 15;  // Allow reuse of port
pub const SO_TYPE: i32 = 3;        // Get socket type
pub const SO_ERROR: i32 = 4;       // Get and clear error
pub const SO_DONTROUTE: i32 = 5;   // Bypass routing table lookup
pub const SO_BROADCAST: i32 = 6;   // Permit sending of broadcast messages
pub const SO_SNDBUF: i32 = 7;      // Send buffer size
pub const SO_RCVBUF: i32 = 8;      // Receive buffer size
pub const SO_SNDBUFFORCE: i32 = 32; // Send buffer force
pub const SO_RCVBUFFORCE: i32 = 33; // Receive buffer force
pub const SO_KEEPALIVE: i32 = 9;   // Keep connections alive
pub const SO_OOBINLINE: i32 = 10;  // Keep out-of-band data in-band
pub const SO_LINGER: i32 = 13;     // Linger on close if unsent data present
pub const SO_RCVLOWAT: i32 = 18;   // Receive low-water mark
pub const SO_SNDLOWAT: i32 = 19;   // Send low-water mark
pub const SO_RCVTIMEO: i32 = 20;   // Receive timeout
pub const SO_SNDTIMEO: i32 = 21;   // Send timeout
pub const SO_TIMESTAMP: i32 = 29;  // Timestamp incoming packets
pub const SO_ACCEPTCONN: i32 = 30; // Socket has accept() call
pub const SO_PROTOCOL: i32 = 38;   // Protocol number
pub const SO_DOMAIN: i32 = 39;     // Domain name

/// Socket shutdown modes
pub const SHUT_RD: i32 = 0;        // Further receptions disallowed
pub const SHUT_WR: i32 = 1;        // Further transmissions disallowed
pub const SHUT_RDWR: i32 = 2;      // Further receptions and transmissions disallowed

/// Signal handling options
pub const SA_NOCLDSTOP: i32 = 1;   // Do not generate SIGCHLD on child stop
pub const SA_NOCLDWAIT: i32 = 2;   // Do not create zombie processes
pub const SA_SIGINFO: i32 = 4;     // Signal handler takes 3 arguments
pub const SA_ONSTACK: i32 = 134217728; // Signal handler uses alternate stack
pub const SA_RESTART: i32 = 268435456; // System call restart after signal
pub const SA_NODEFER: i32 = 1073741824; // Signal not blocked during handler

/// Signal set operations
pub const SIG_BLOCK: i32 = 0;      // Block signals
pub const SIG_UNBLOCK: i32 = 1;    // Unblock signals
pub const SIG_SETMASK: i32 = 2;    // Set signal mask

/// Clone flags for threading
pub const CLONE_VM: i64 = 0x00000100;      // Share memory space
pub const CLONE_FS: i64 = 0x00000200;      // Share filesystem info
pub const CLONE_FILES: i64 = 0x00000400;   // Share file descriptor table
pub const CLONE_SIGHAND: i64 = 0x00000800; // Share signal handlers
pub const CLONE_PTRACE: i64 = 0x00002000;  // Trace this process
pub const CLONE_VFORK: i64 = 0x00004000;   // Wait for fork completion
pub const CLONE_PARENT: i64 = 0x00008000;  // Set parent to same as forker
pub const CLONE_THREAD: i64 = 0x00010000;  // Add to parent's thread group
pub const CLONE_NEWNS: i64 = 0x00020000;   // Clone with new namespace
pub const CLONE_SYSVSEM: i64 = 0x00040000; // Share System V semaphore adjustment
pub const CLONE_SETTLS: i64 = 0x00080000;  // Create TLS for child
pub const CLONE_PARENT_SETTID: i64 = 0x00100000; // Set parent TID
pub const CLONE_CHILD_CLEARTID: i64 = 0x00200000; // Clear TID in child
pub const CLONE_DETACHED: i64 = 0x00400000; // Unused, ignored
pub const CLONE_CHILD_SETTID: i64 = 0x01000000; // Set TID in child
pub const CLONE_NEWUTS: i64 = 0x04000000;   // Clone with new UTS namespace
pub const CLONE_NEWIPC: i64 = 0x08000000;   // Clone with new IPC namespace
pub const CLONE_NEWUSER: i64 = 0x10000000;  // Clone with new user namespace
pub const CLONE_NEWPID: i64 = 0x20000000;   // Clone with new PID namespace
pub const CLONE_NEWNET: i64 = 0x40000000;   // Clone with new network namespace
pub const CLONE_IO: i64 = 0x80000000;       // Clone with new I/O context

/// Clone protection flags
pub const CLONE_ONE_SHOT: i64 = 0x00000000; // One-shot thread (unused)

/// Utility functions for parameter validation and conversion
pub mod utils {
    use super::*;
    
    /// Validate file descriptor
    pub fn validate_fd(fd: fd_t) -> bool {
        fd >= 0 && fd <= i32::MAX as fd_t
    }
    
    /// Validate pointer alignment
    pub fn is_aligned<T>(ptr: *const T) -> bool {
        (ptr as usize) % core::mem::align_of::<T>() == 0
    }
    
    /// Convert between host and network byte order
    pub fn htons(val: u16) -> u16 {
        // MultiOS assumes little-endian, so this is a placeholder
        // In a real implementation, this would check endianness
        val
    }
    
    pub fn htonl(val: u32) -> u32 {
        // MultiOS assumes little-endian, so this is a placeholder
        // In a real implementation, this would check endianness
        val
    }
    
    pub fn ntohs(val: u16) -> u16 {
        htons(val)
    }
    
    pub fn ntohl(val: u32) -> u32 {
        htonl(val)
    }
    
    /// Convert host byte order to network byte order for 64-bit values
    pub fn htonq(val: u64) -> u64 {
        // MultiOS assumes little-endian, so this is a placeholder
        // In a real implementation, this would check endianness
        val
    }
    
    pub fn ntohq(val: u64) -> u64 {
        htonq(val)
    }
    
    /// Safe copy from user space
    pub unsafe fn copy_from_user<T>(src: *const T, dest: *mut T, count: usize) -> PosixResult<()> {
        if src.is_null() || dest.is_null() || count == 0 {
            return Err(Errno::Ebadaddr);
        }
        
        // In a real implementation, this would check if the user pointer
        // is within the allowed address space
        core::ptr::copy_nonoverlapping(src, dest, count);
        Ok(())
    }
    
    /// Safe copy to user space
    pub unsafe fn copy_to_user<T>(src: *const T, dest: *mut T, count: usize) -> PosixResult<()> {
        if src.is_null() || dest.is_null() || count == 0 {
            return Err(Errno::Ebadaddr);
        }
        
        // In a real implementation, this would check if the user pointer
        // is within the allowed address space
        core::ptr::copy_nonoverlapping(src, dest, count);
        Ok(())
    }
    
    /// Convert C string to Rust string slice
    pub unsafe fn c_str_to_str(c_str: *const u8) -> PosixResult<&str> {
        if c_str.is_null() {
            return Err(Errno::Ebadaddr);
        }
        
        // Find null terminator
        let mut len = 0;
        while len < usize::MAX {
            if *c_str.add(len) == 0 {
                break;
            }
            len += 1;
        }
        
        if len == usize::MAX {
            return Err(Errno::Enametoolong);
        }
        
        core::str::from_utf8(core::slice::from_raw_parts(c_str, len))
            .map_err(|_| Errno::Einval)
    }
    
    /// Convert Rust string to C string buffer
    pub fn str_to_c_str(s: &str, buf: &mut [u8]) -> PosixResult<usize> {
        if buf.len() < s.len() + 1 {
            return Err(Errno::Eoverflow);
        }
        
        buf[..s.len()].copy_from_slice(s.as_bytes());
        buf[s.len()] = 0;
        Ok(s.len())
    }
}
