//! MultiOS POSIX API Compatibility Layer
//! 
//! This module provides comprehensive POSIX API compatibility for MultiOS,
//! ensuring API compatibility with standard POSIX interfaces while maintaining
//! Rust safety guarantees.
//!
//! # Supported POSIX APIs
//! - stdio.h: File operations, streams, formatting
//! - unistd.h: Process management, system operations
//! - sys/types.h: Basic system type definitions  
//! - signal.h: Signal handling and management
//! - socket.h: Network socket operations
//! - pthread.h: Threading and synchronization primitives

pub mod stdio;
pub mod unistd;
pub mod sys_types;
pub mod signal;
pub mod socket;
pub mod pthread;
pub mod internal;
pub mod errors;

// Re-export commonly used types and functions
pub use stdio::*;
pub use unistd::*;
pub use sys_types::*;
pub use signal::*;
pub use socket::*;
pub use pthread::*;
pub use errors::*;

/// Core POSIX types that are used across multiple modules
/// 
/// These types provide compatibility with POSIX standard type definitions
/// while maintaining Rust safety guarantees.
pub mod types {
    use bitflags::bitflags;
    use core::fmt;

    /// Process ID type
    pub type pid_t = i32;

    /// User ID type  
    pub type uid_t = u32;

    /// Group ID type
    pub type gid_t = u32;

    /// File descriptor type
    pub type fd_t = i32;

    /// Clock ID type
    pub type clockid_t = i32;

    /// Time type
    pub type time_t = i64;

    /// Size type
    pub type size_t = usize;

    /// Signed size type
    pub type ssize_t = isize;

    /// Off-t type (for file offsets)
    pub type off_t = i64;

    /// Mode type (for file permissions)
    pub type mode_t = u32;

    /// Clock ticks per second (POSIX)
    pub const CLK_TCK: i32 = 100;

    /// Standard file descriptors
    pub const STDIN_FILENO: fd_t = 0;
    pub const STDOUT_FILENO: fd_t = 1;
    pub const STDERR_FILENO: fd_t = 2;

    /// Common file open flags
    bitflags! {
        /// File access modes for open()
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct OpenFlags: u32 {
            const READ = 0x0001;
            const WRITE = 0x0002;
            const APPEND = 0x0004;
            const CREAT = 0x0008;
            const TRUNC = 0x0010;
            const EXCL = 0x0020;
            const NOCTTY = 0x0040;
            const NONBLOCK = 0x0080;
            const DSYNC = 0x0100;
            const SYNC = 0x0200;
            const ASYNC = 0x0400;
            const DIRECT = 0x0800;
            const LARGEFILE = 0x1000;
            const DIRECTORY = 0x2000;
            const NOFOLLOW = 0x4000;
            const NOATIME = 0x8000;
        }
    }

    /// File status flags
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct StatusFlags: u32 {
            const ACCESS_TIME = 0x0001;
            const MODIFY_TIME = 0x0002;
            const CHANGE_TIME = 0x0004;
            const LINK_COUNT = 0x0008;
            const SIZE = 0x0010;
            const BLOCKS = 0x0020;
            const BLOCK_SIZE = 0x0040;
            const TYPE = 0x0080;
            const PERMISSIONS = 0x0100;
            const UID = 0x0200;
            const GID = 0x0400;
        }
    }

    /// Common permission flags
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PermissionFlags: u32 {
            const OTHER_EXEC = 0o0001;
            const OTHER_WRITE = 0o0002;
            const OTHER_READ = 0o0004;
            const GROUP_EXEC = 0o0010;
            const GROUP_WRITE = 0o0020;
            const GROUP_READ = 0o0040;
            const USER_EXEC = 0o0100;
            const USER_WRITE = 0o0200;
            const USER_READ = 0o0400;
            const SETUID = 0o4000;
            const SETGID = 0o2000;
            const STICKY = 0o1000;
        }
    }

    /// File type flags
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct FileType: u32 {
            const FIFO = 0o010000;
            const CHAR = 0o020000;
            const DIR = 0o040000;
            const BLOCK = 0o060000;
            const REGULAR = 0o100000;
            const LINK = 0o120000;
            const SOCKET = 0o140000;
            const UNKNOWN = 0o000000;
        }
    }

    /// Standard seek modes
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SeekMode {
        Set = 0,    // SEEK_SET
        Current,    // SEEK_CUR
        End,        // SEEK_END
    }

    /// Common signal numbers (POSIX)
    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Signal {
        Null = 0,           // No signal
        Hangup = 1,         // SIGHUP
        Interrupt = 2,      // SIGINT
        Quit = 3,           // SIGQUIT
        Illegal = 4,        // SIGILL
        Trap = 5,           // SIGTRAP
        Abort = 6,          // SIGABRT
        Bus = 7,            // SIGBUS
        Floating = 8,       // SIGFPE
        Kill = 9,           // SIGKILL
        User1 = 10,         // SIGUSR1
        SegmentViolation = 11, // SIGSEGV
        User2 = 12,         // SIGUSR2
        Pipe = 13,          // SIGPIPE
        Alarm = 14,         // SIGALRM
        Terminate = 15,     // SIGTERM
        Child = 17,         // SIGCHLD
        Continue = 18,      // SIGCONT
        Stop = 19,          // SIGSTOP
        Stop2 = 20,         // SIGTSTP
        Input = 21,         // SIGTTIN
        Output = 22,        // SIGTTOU
        VirtualTime = 24,   // SIGVTALRM
        Profiling = 25,     // SIGPROF
        WindowSize = 28,    // SIGWINCH
        IO = 29,            // SIGIO
        Power = 30,         // SIGPWR
        System = 31,        // SIGSYS
    }

    /// Socket domains (address families)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SocketDomain {
        Unix = 1,           // AF_UNIX
        Inet = 2,           // AF_INET
        Inet6 = 10,         // AF_INET6
        Packet = 17,        // AF_PACKET
        Netlink = 16,       // AF_NETLINK
    }

    /// Socket types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SocketType {
        Stream = 1,         // SOCK_STREAM
        Datagram = 2,       // SOCK_DGRAM
        Raw = 3,            // SOCK_RAW
        RDM = 4,            // SOCK_RDM
        SeqPacket = 5,      // SOCK_SEQPACKET
    }

    /// Socket protocols
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SocketProtocol {
        Tcp = 6,            // IPPROTO_TCP
        Udp = 17,           // IPPROTO_UDP
        Icmp = 1,           // IPPROTO_ICMP
        Icmp6 = 58,         // IPPROTO_ICMPV6
        Raw = 255,          // IPPROTO_RAW
        Any = 0,            // IPPROTO_ANY
    }

    /// Thread attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct ThreadAttrFlags: u32 {
            const DETACHED = 0x0001;
            const STACK_SIZE = 0x0002;
            const GUARD_SIZE = 0x0004;
            const INHERIT_SCHED = 0x0008;
            const SCHED_POLICY = 0x0010;
            const SCOPE = 0x0020;
            const PROCLR_NP = 0x0040;
            const STACK_ADDR = 0x0080;
        }
    }

    /// Thread scheduling policies
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ThreadPolicy {
        Other = 0,          // SCHED_OTHER
        Fifo = 1,           // SCHED_FIFO
        RR = 2,             // SCHED_RR
        Batch = 3,          // SCHED_BATCH
        Idle = 5,           // SCHED_IDLE
        Deadline = 6,       // SCHED_DEADLINE
    }

    /// Mutex attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct MutexAttrFlags: u32 {
            const TYPE = 0x0001;
            const PRIOCEILING = 0x0002;
            const PROTOCOL = 0x0004;
            const PSHARED = 0x0008;
            const ROBUST = 0x0010;
        }
    }

    /// Mutex types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MutexType {
        Normal = 0,         // PTHREAD_MUTEX_NORMAL
        ErrorCheck = 1,     // PTHREAD_MUTEX_ERRORCHECK
        Recursive = 2,      // PTHREAD_MUTEX_RECURSIVE
        Default = 3,        // PTHREAD_MUTEX_DEFAULT
    }

    /// Condition variable attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct CondAttrFlags: u32 {
            const PSHARED = 0x0001;
            const CLOCK_ID = 0x0002;
        }
    }

    /// Read/Write lock attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct RWLockAttrFlags: u32 {
            const PSHARED = 0x0001;
        }
    }

    /// Barrier attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct BarrierAttrFlags: u32 {
            const PSHARED = 0x0001;
        }
    }

    /// Spinlock attributes
    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct SpinLockAttrFlags: u32 {
            const PSHARED = 0x0001;
        }
    }

    impl fmt::Display for SeekMode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SeekMode::Set => write!(f, "SEEK_SET"),
                SeekMode::Current => write!(f, "SEEK_CUR"),
                SeekMode::End => write!(f, "SEEK_END"),
            }
        }
    }

    impl fmt::Display for Signal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Signal::Null => "NULL",
                Signal::Hangup => "SIGHUP",
                Signal::Interrupt => "SIGINT", 
                Signal::Quit => "SIGQUIT",
                Signal::Illegal => "SIGILL",
                Signal::Trap => "SIGTRAP",
                Signal::Abort => "SIGABRT",
                Signal::Bus => "SIGBUS",
                Signal::Floating => "SIGFPE",
                Signal::Kill => "SIGKILL",
                Signal::User1 => "SIGUSR1",
                Signal::SegmentViolation => "SIGSEGV",
                Signal::User2 => "SIGUSR2",
                Signal::Pipe => "SIGPIPE",
                Signal::Alarm => "SIGALRM",
                Signal::Terminate => "SIGTERM",
                Signal::Child => "SIGCHLD",
                Signal::Continue => "SIGCONT",
                Signal::Stop => "SIGSTOP",
                Signal::Stop2 => "SIGTSTP",
                Signal::Input => "SIGTTIN",
                Signal::Output => "SIGTTOU",
                Signal::VirtualTime => "SIGVTALRM",
                Signal::Profiling => "SIGPROF",
                Signal::WindowSize => "SIGWINCH",
                Signal::IO => "SIGIO",
                Signal::Power => "SIGPWR",
                Signal::System => "SIGSYS",
            };
            write!(f, "{}", name)
        }
    }

    impl fmt::Display for SocketDomain {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SocketDomain::Unix => write!(f, "AF_UNIX"),
                SocketDomain::Inet => write!(f, "AF_INET"),
                SocketDomain::Inet6 => write!(f, "AF_INET6"),
                SocketDomain::Packet => write!(f, "AF_PACKET"),
                SocketDomain::Netlink => write!(f, "AF_NETLINK"),
            }
        }
    }

    impl fmt::Display for SocketType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SocketType::Stream => write!(f, "SOCK_STREAM"),
                SocketType::Datagram => write!(f, "SOCK_DGRAM"),
                SocketType::Raw => write!(f, "SOCK_RAW"),
                SocketType::RDM => write!(f, "SOCK_RDM"),
                SocketType::SeqPacket => write!(f, "SOCK_SEQPACKET"),
            }
        }
    }

    impl fmt::Display for SocketProtocol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SocketProtocol::Tcp => write!(f, "IPPROTO_TCP"),
                SocketProtocol::Udp => write!(f, "IPPROTO_UDP"),
                SocketProtocol::Icmp => write!(f, "IPPROTO_ICMP"),
                SocketProtocol::Icmp6 => write!(f, "IPPROTO_ICMPV6"),
                SocketProtocol::Raw => write!(f, "IPPROTO_RAW"),
                SocketProtocol::Any => write!(f, "IPPROTO_ANY"),
            }
        }
    }

    impl fmt::Display for ThreadPolicy {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ThreadPolicy::Other => write!(f, "SCHED_OTHER"),
                ThreadPolicy::Fifo => write!(f, "SCHED_FIFO"),
                ThreadPolicy::RR => write!(f, "SCHED_RR"),
                ThreadPolicy::Batch => write!(f, "SCHED_BATCH"),
                ThreadPolicy::Idle => write!(f, "SCHED_IDLE"),
                ThreadPolicy::Deadline => write!(f, "SCHED_DEADLINE"),
            }
        }
    }

    impl fmt::Display for MutexType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                MutexType::Normal => write!(f, "PTHREAD_MUTEX_NORMAL"),
                MutexType::ErrorCheck => write!(f, "PTHREAD_MUTEX_ERRORCHECK"),
                MutexType::Recursive => write!(f, "PTHREAD_MUTEX_RECURSIVE"),
                MutexType::Default => write!(f, "PTHREAD_MUTEX_DEFAULT"),
            }
        }
    }
}

/// Core system call interface
/// 
/// This module provides the low-level system call interface that all POSIX
/// functions use internally. It handles parameter validation, error mapping,
/// and safe communication with the kernel.
pub mod syscall {
    use super::types::*;
    use super::errors::*;
    use core::ptr;
    
    /// System call numbers (aligned with kernel syscall numbers)
    mod numbers {
        pub const OPEN: usize = 1000;
        pub const CLOSE: usize = 1001;
        pub const READ: usize = 1002;
        pub const WRITE: usize = 1003;
        pub const SEEK: usize = 1004;
        pub const STAT: usize = 1005;
        pub const FSTAT: usize = 1006;
        pub const FSTATAT: usize = 1007;
        pub const MKDIRAT: usize = 1008;
        pub const UNLINKAT: usize = 1009;
        pub const RENAMEAT: usize = 1010;
        pub const LINKAT: usize = 1011;
        pub const SYMLINKAT: usize = 1012;
        pub const READLINKAT: usize = 1013;
        pub const CHMOD: usize = 1014;
        pub const FCHMOD: usize = 1015;
        pub const FCHMODAT: usize = 1016;
        pub const CHOWN: usize = 1017;
        pub const FCHOWN: usize = 1018;
        pub const FCHOWNAT: usize = 1019;
        pub const DUP: usize = 1020;
        pub const DUP2: usize = 1021;
        pub const DUP3: usize = 1022;
        pub const FCNTL: usize = 1023;
        pub const SYSCALLS_END: usize = 1024;

        // Process management
        pub const FORK: usize = 2000;
        pub const EXECVE: usize = 2001;
        pub const EXIT: usize = 2002;
        pub const WAIT4: usize = 2003;
        pub const KILL: usize = 2004;
        pub const GETPID: usize = 2005;
        pub const GETPPID: usize = 2006;
        pub const GETPGRP: usize = 2007;
        pub const SETPGRP: usize = 2008;
        pub const SETSID: usize = 2009;
        pub const GETSID: usize = 2010;
        pub const GETPGID: usize = 2011;
        pub const SETPGID: usize = 2012;

        // Memory management
        pub const BRK: usize = 3000;
        pub const MMAP: usize = 3001;
        pub const MUNMAP: usize = 3002;
        pub const MPROTECT: usize = 3003;
        pub const MSYNC: usize = 3004;
        pub const MADVISE: usize = 3005;
        pub const MINCORE: usize = 3006;
        pub const MREMAP: usize = 3007;

        // File system operations
        pub const ACCESS: usize = 4000;
        pub const FACCESSAT: usize = 4001;
        pub const CHDIR: usize = 4002;
        pub const FCHDIR: usize = 4003;
        pub const GETCWD: usize = 4004;

        // Directory operations
        pub const GETDENTS64: usize = 4100;
        pub const MOUNT: usize = 4101;
        pub const UMOUNT: usize = 4102;
        pub const UMOUNT2: usize = 4103;

        // Time operations
        pub const TIME: usize = 5000;
        pub const GETTIMEOFDAY: usize = 5001;
        pub const SETTIMEOFDAY: usize = 5002;
        pub const CLOCK_GETTIME: usize = 5003;
        pub const CLOCK_SETTIME: usize = 5004;
        pub const CLOCK_GETRES: usize = 5005;
        pub const CLOCK_NANOSLEEP: usize = 5006;

        // Signal operations
        pub const RT_SIGACTION: usize = 6000;
        pub const RT_SIGPROCMASK: usize = 6001;
        pub const RT_SIGPENDING: usize = 6002;
        pub const RT_SIGSUSPEND: usize = 6003;
        pub const RT_SIGWAITINFO: usize = 6004;
        pub const RT_SIGQUEUEINFO: usize = 6005;
        pub const RT_SIGRETURN: usize = 6006;

        // Socket operations
        pub const SOCKET: usize = 7000;
        pub const BIND: usize = 7001;
        pub const CONNECT: usize = 7002;
        pub const LISTEN: usize = 7003;
        pub const ACCEPT: usize = 7004;
        pub const ACCEPT4: usize = 7005;
        pub const GETSOCKNAME: usize = 7006;
        pub const GETPEERNAME: usize = 7007;
        pub const SEND: usize = 7008;
        pub const RECV: usize = 7009;
        pub const SENDTO: usize = 7010;
        pub const RECVFROM: usize = 7011;
        pub const SHUTDOWN: usize = 7012;
        pub const SETSOCKOPT: usize = 7013;
        pub const GETSOCKOPT: usize = 7014;
        pub const SOCKETPAIR: usize = 7015;

        // Thread operations
        pub const CLONE: usize = 8000;
        pub const SET_TID_ADDRESS: usize = 8001;
        pub const SET_ROBUST_LIST: usize = 8002;
        pub const GET_ROBUST_LIST: usize = 8003;
        pub const FUTEX: usize = 8004;
        pub const SET_ROBUST_LIST: usize = 8005;
        pub const RT_SIGQUEUEINFO: usize = 8006;
        pub const RT_SIGTIMEDWAIT: usize = 8007;
        pub const RT_SIGSUSPEND: usize = 8008;

        // File descriptor operations
        pub const SELECT: usize = 9000;
        pub const POLL: usize = 9001;
        pub const EPOLL_CREATE: usize = 9002;
        pub const EPOLL_CTL: usize = 9003;
        pub const EPOLL_WAIT: usize = 9004;
        pub const EPOLL_PWAIT: usize = 9005;
    }

    /// Perform a system call with parameter validation and error handling
    unsafe extern "C" fn syscall6(
        num: usize,
        arg0: usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
        arg5: usize,
    ) -> usize;

    /// Wrapper macro for system calls
    macro_rules! syscall {
        ($num:expr) => {{
            unsafe { syscall6($num, 0, 0, 0, 0, 0, 0) }
        }};
        ($num:expr, $a0:expr) => {{
            unsafe { syscall6($num, $a0, 0, 0, 0, 0, 0) }
        }};
        ($num:expr, $a0:expr, $a1:expr) => {{
            unsafe { syscall6($num, $a0, $a1, 0, 0, 0, 0) }
        }};
        ($num:expr, $a0:expr, $a1:expr, $a2:expr) => {{
            unsafe { syscall6($num, $a0, $a1, $a2, 0, 0, 0) }
        }};
        ($num:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr) => {{
            unsafe { syscall6($num, $a0, $a1, $a2, $a3, 0, 0) }
        }};
        ($num:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
            unsafe { syscall6($num, $a0, $a1, $a2, $a3, $a4, 0) }
        }};
        ($num:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
            unsafe { syscall6($num, $a0, $a1, $a2, $a3, $a4, $a5) }
        }};
    }

    /// File descriptor operations
    pub fn open(path: *const u8, flags: OpenFlags, mode: mode_t) -> Result<fd_t, Errno> {
        let result = syscall!(numbers::OPEN, path as usize, flags.bits(), mode);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as fd_t)
        }
    }

    pub fn close(fd: fd_t) -> Result<(), Errno> {
        let result = syscall!(numbers::CLOSE, fd as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    pub fn read(fd: fd_t, buf: *mut u8, count: size_t) -> Result<ssize_t, Errno> {
        let result = syscall!(numbers::READ, fd as usize, buf as usize, count);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as ssize_t)
        }
    }

    pub fn write(fd: fd_t, buf: *const u8, count: size_t) -> Result<ssize_t, Errno> {
        let result = syscall!(numbers::WRITE, fd as usize, buf as usize, count);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as ssize_t)
        }
    }

    pub fn lseek(fd: fd_t, offset: off_t, whence: SeekMode) -> Result<off_t, Errno> {
        let result = syscall!(numbers::SEEK, fd as usize, offset as usize, whence as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as off_t)
        }
    }

    pub fn fstat(fd: fd_t, statbuf: *mut Stat) -> Result<(), Errno> {
        let result = syscall!(numbers::FSTAT, fd as usize, statbuf as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    pub fn dup(oldfd: fd_t) -> Result<fd_t, Errno> {
        let result = syscall!(numbers::DUP, oldfd as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as fd_t)
        }
    }

    pub fn dup2(oldfd: fd_t, newfd: fd_t) -> Result<fd_t, Errno> {
        let result = syscall!(numbers::DUP2, oldfd as usize, newfd as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as fd_t)
        }
    }

    // Process management
    pub fn fork() -> Result<pid_t, Errno> {
        let result = syscall!(numbers::FORK);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as pid_t)
        }
    }

    pub fn execve(path: *const u8, argv: *const *const u8, envp: *const *const u8) -> Result<!, Errno> {
        let result = syscall!(numbers::EXECVE, path as usize, argv as usize, envp as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            // This should never return
            unreachable!()
        }
    }

    pub fn exit(status: i32) -> ! {
        unsafe { syscall!(numbers::EXIT, status as usize); }
        loop {} // Never return
    }

    pub fn getpid() -> pid_t {
        syscall!(numbers::GETPID) as pid_t
    }

    pub fn getppid() -> pid_t {
        syscall!(numbers::GETPPID) as pid_t
    }

    pub fn kill(pid: pid_t, sig: i32) -> Result<(), Errno> {
        let result = syscall!(numbers::KILL, pid as usize, sig as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    // Memory management
    pub fn brk(addr: usize) -> Result<usize, Errno> {
        let result = syscall!(numbers::BRK, addr);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result)
        }
    }

    pub fn mmap(addr: usize, length: size_t, prot: i32, flags: i32, fd: fd_t, offset: off_t) -> Result<usize, Errno> {
        let result = syscall!(numbers::MMAP, addr, length, prot as usize, flags as usize, fd as usize, offset as usize);
        if result == usize::MAX {
            Err(Errno::from_raw(-1))
        } else {
            Ok(result)
        }
    }

    pub fn munmap(addr: usize, length: size_t) -> Result<(), Errno> {
        let result = syscall!(numbers::MUNMAP, addr, length);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    // Socket operations
    pub fn socket(domain: SocketDomain, ty: SocketType, protocol: SocketProtocol) -> Result<fd_t, Errno> {
        let result = syscall!(numbers::SOCKET, domain as usize, ty as usize, protocol as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as fd_t)
        }
    }

    pub fn bind(sockfd: fd_t, addr: *const sockaddr, addrlen: socklen_t) -> Result<(), Errno> {
        let result = syscall!(numbers::BIND, sockfd as usize, addr as usize, addrlen);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    pub fn connect(sockfd: fd_t, addr: *const sockaddr, addrlen: socklen_t) -> Result<(), Errno> {
        let result = syscall!(numbers::CONNECT, sockfd as usize, addr as usize, addrlen);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    pub fn listen(sockfd: fd_t, backlog: i32) -> Result<(), Errno> {
        let result = syscall!(numbers::LISTEN, sockfd as usize, backlog as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    pub fn accept(sockfd: fd_t, addr: *mut sockaddr, addrlen: *mut socklen_t) -> Result<fd_t, Errno> {
        let result = syscall!(numbers::ACCEPT, sockfd as usize, addr as usize, addrlen as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as fd_t)
        }
    }

    pub fn send(sockfd: fd_t, buf: *const u8, len: size_t, flags: i32) -> Result<ssize_t, Errno> {
        let result = syscall!(numbers::SEND, sockfd as usize, buf as usize, len, flags as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as ssize_t)
        }
    }

    pub fn recv(sockfd: fd_t, buf: *mut u8, len: size_t, flags: i32) -> Result<ssize_t, Errno> {
        let result = syscall!(numbers::RECV, sockfd as usize, buf as usize, len, flags as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as ssize_t)
        }
    }

    pub fn shutdown(sockfd: fd_t, how: i32) -> Result<(), Errno> {
        let result = syscall!(numbers::SHUTDOWN, sockfd as usize, how as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }

    // Time operations
    pub fn time(tloc: *mut time_t) -> Result<time_t, Errno> {
        let result = syscall!(numbers::TIME, tloc as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result as time_t)
        }
    }

    pub fn gettimeofday(tv: *mut timeval, tz: *mut timezone) -> Result<(), Errno> {
        let result = syscall!(numbers::GETTIMEOFDAY, tv as usize, tz as usize);
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(())
        }
    }
}
