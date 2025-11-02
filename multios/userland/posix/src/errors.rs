//! POSIX Error Handling
//! 
//! This module provides comprehensive error handling for POSIX API calls,
//! maintaining compatibility with standard POSIX errno values while providing
//! Rust-friendly error types.

use core::fmt;
use super::types::*;

/// POSIX error numbers (errno values)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    // POSIX standard errors
    Epperm = 1,           // Operation not permitted
    Enoent = 2,           // No such file or directory
    Esrch = 3,            // No such process
    Eintr = 4,            // Interrupted system call
    Eio = 5,              // I/O error
    Enxio = 6,            // No such device or address
    E2big = 7,            // Argument list too long
    Enoexec = 8,          // Exec format error
    Ebadf = 9,            // Bad file number
    Echild = 10,          // No child processes
    Eagain = 11,          // Resource temporarily unavailable
    Enomem = 12,          // Out of memory
    Eaccess = 13,         // Permission denied
    Ebadaddr = 14,        // Bad address
    Enotblk = 15,         // Block device required
    Ebusy = 16,           // Device or resource busy
    Eexist = 17,          // File exists
    Exdev = 18,           // Cross-device link
    Enodev = 19,          // No such device
    Enotdir = 20,         // Not a directory
    Eisdir = 21,          // Is a directory
    Einval = 22,          // Invalid argument
    Enfile = 23,          // File table overflow
    Emfile = 24,          // Too many open files
    Enotty = 25,          // Inappropriate ioctl for device
    Etxtbsy = 26,         // Text file busy
    Efbig = 27,           // File too large
    Enospc = 28,          // No space left on device
    Espipe = 29,          // Illegal seek
    Erofs = 30,           // Read-only file system
    Emlink = 31,          // Too many links
    Epipe = 32,           // Broken pipe
    Edom = 33,            // Math argument out of domain of func
    Erange = 34,          // Math result not representable
    Edeadlk = 36,         // Resource deadlock would occur
    Enametoolong = 37,    // File name too long
    Enolck = 39,          // No record locks available
    Enosys = 40,          // Function not implemented
    Enotempty = 41,       // Directory not empty
    Eloop = 42,           // Too many symbolic links encountered
    Enomsg = 42,          // No message of desired type
    Eidrm = 43,           // Identifier removed
    Echrng = 44,          // Channel number out of range
    El2nsync = 45,        // Level 2 not synchronized
    El3hl = 46,           // Level 3 halted
    El3rst = 47,          // Level 3 reset
    Elnrng = 48,          // Link number out of range
    Eunatch = 49,         // Protocol driver not attached
    Enocsi = 50,          // No CSI structure available
    El2hlt = 51,          // Level 2 halted
    Edeadlock = 58,       // File locking deadlock error
    Ebfont = 59,          // Bad font file format
    Enostr = 60,          // Device not a stream
    Enodata = 61,         // No data available
    Etime = 62,           // Timer expired
    Enosr = 63,           // Out of streams resources
    Enonet = 64,          // Machine is not on the network
    Enopkg = 65,          // Package not installed
    Eremot = 66,          // Remote I/O error
    Enolink = 67,         // Link has been severed
    Eadv = 68,            // Advertise error
    Esrmnt = 69,          // Srmount error
    Ecomm = 70,           // Communication error on send
    Eproto = 71,          // Protocol error
    Emultihop = 72,       // Multihop attempted
    Ebadmsg = 73,         // Bad message
    Eoverflow = 74,       // Value too large for defined data type
    Enotuniq = 75,        // Name not unique on network
    Ebadfd = 76,          // File descriptor in bad state
    Eremchg = 77,         // Remote address changed
    Elibacc = 79,         // Can not access a needed shared library
    Elibbad = 80,         // Accessing a corrupted shared library
    Elibscn = 81,         // .lib section in a.out corrupted
    Elibmax = 82,         // Attempting to link in too many shared libraries
    Elibexec = 83,        // Cannot exec a shared library directly
    Eilseq = 84,          // Illegal byte sequence
    Erestart = 85,        // Interrupted system call should be restarted
    Eustr = 86,           // Streams pipe error
    Eaddrinuse = 88,      // Address already in use
    Eaddrnotavail = 89,   // Cannot assign requested address
    Enetdown = 90,        // Network is down
    Enetunreach = 91,     // Network is unreachable
    Enetreset = 92,       // Network dropped connection because of reset
    Econnaborted = 93,    // Software caused connection abort
    Econnreset = 104,     // Connection reset by peer
    Enobufs = 105,        // No buffer space available
    Eacces = 106,         // Operation already in progress
    Eisconn = 107,        // Socket is already connected
    Enotconn = 108,       // Socket is not connected
    Eshutdown = 109,      // Cannot send after transport endpoint shutdown
    Etimedout = 110,      // Connection timed out
    Econnrefused = 111,   // Connection refused
    Ehostunreach = 113,   // No route to host
    Ealread = 114,        // Operation already in progress
    Einprogress = 115,    // Operation now in progress
    Estale = 116,         // Stale file handle
    Euclean = 117,        // Structure needs cleaning
    Enotnam = 118,        // Not a XENIX named type file
    Enavail = 119,        // No XENIX semaphores available
    Eisnam = 120,         // Is a named type file
    Eremote = 121,        // Remote I/O error
    Enomedium = 123,      // No medium found
    Emediumtype = 124,    // Wrong medium type
    Ecanceled = 125,      // Operation Canceled
    Enokey = 126,         // Required key not available
    Ekeyexpired = 127,    // Key has expired
    Ekeyrevoked = 128,    // Key has been revoked
    Ekeyrejected = 129,   // Key was rejected by service
    // Extended errors for threading and synchronization
    Eagain = 11,          // Resource temporarily unavailable (retry)
    Ewouldblock = 11,     // Operation would block
    Eownerdead = 130,     // Owner died
    Enotrecoverable = 131, // State not recoverable
    Etoomanyrefs = 132,   // Too many references: cannot splice
    Eoverflow = 74,       // Value too large for defined data type
    Ebadmsg = 73,         // Bad message
    Ewrongmsg = 133,      // Wrong message
    Elength = 134,        // Message too long
    Eproto = 71,          // Protocol error
    Eprotoall = 135,      // Protocol not available
    Eprotonosupp = 136,   // Protocol not supported
    Etimenotsupp = 137,   // Timing not supported
    Eopnotsupp = 138,     // Operation not supported
    Eprotoframe = 139,    // Protocol framing error
    Eprotorange = 140,    // Protocol out of range
    Ehalfopen = 141,      // Socket is half-open (connection being aborted)
    Eclosed = 142,        // Socket is closed
    // Unknown/undefined errno
    Unknown = 999,
}

impl Errno {
    /// Convert raw errno value to Errno enum
    pub fn from_raw(errno: i32) -> Self {
        match errno {
            0 => Errno::from(0i32), // EPERM
            1 => Errno::Eperm,
            2 => Errno::Enoent,
            3 => Errno::Esrch,
            4 => Errno::Eintr,
            5 => Errno::Eio,
            6 => Errno::Enxio,
            7 => Errno::E2big,
            8 => Errno::Enoexec,
            9 => Errno::Ebadf,
            10 => Errno::Echild,
            11 => Errno::Eagain,
            12 => Errno::Enomem,
            13 => Errno::Eaccess,
            14 => Errno::Ebadaddr,
            15 => Errno::Enotblk,
            16 => Errno::Ebusy,
            17 => Errno::Eexist,
            18 => Errno::Exdev,
            19 => Errno::Enodev,
            20 => Errno::Enotdir,
            21 => Errno::Eisdir,
            22 => Errno::Einval,
            23 => Errno::Enfile,
            24 => Errno::Emfile,
            25 => Errno::Enotty,
            26 => Errno::Etxtbsy,
            27 => Errno::Efbig,
            28 => Errno::Enospc,
            29 => Errno::Espipe,
            30 => Errno::Erofs,
            31 => Errno::Emlink,
            32 => Errno::Epipe,
            33 => Errno::Edom,
            34 => Errno::Erange,
            36 => Errno::Edeadlk,
            37 => Errno::Enametoolong,
            39 => Errno::Enolck,
            40 => Errno::Enosys,
            41 => Errno::Enotempty,
            42 => Errno::Eloop,
            43 => Errno::Eidrm,
            44 => Errno::Echrng,
            45 => Errno::El2nsync,
            46 => Errno::El3hl,
            47 => Errno::El3rst,
            48 => Errno::Elnrng,
            49 => Errno::Eunatch,
            50 => Errno::Enocsi,
            51 => Errno::El2hlt,
            58 => Errno::Edeadlock,
            59 => Errno::Ebfont,
            60 => Errno::Enostr,
            61 => Errno::Enodata,
            62 => Errno::Etime,
            63 => Errno::Enosr,
            64 => Errno::Enonet,
            65 => Errno::Enopkg,
            66 => Errno::Eremot,
            67 => Errno::Enolink,
            68 => Errno::Eadv,
            69 => Errno::Esrmnt,
            70 => Errno::Ecomm,
            71 => Errno::Eproto,
            72 => Errno::Emultihop,
            73 => Errno::Ebadmsg,
            74 => Errno::Eoverflow,
            75 => Errno::Enotuniq,
            76 => Errno::Ebadfd,
            77 => Errno::Eremchg,
            79 => Errno::Elibacc,
            80 => Errno::Elibbad,
            81 => Errno::Elibscn,
            82 => Errno::Elibmax,
            83 => Errno::Elibexec,
            84 => Errno::Eilseq,
            85 => Errno::Erestart,
            86 => Errno::Eustr,
            88 => Errno::Eaddrinuse,
            89 => Errno::Eaddrnotavail,
            90 => Errno::Enetdown,
            91 => Errno::Enetunreach,
            92 => Errno::Enetreset,
            93 => Errno::Econnaborted,
            104 => Errno::Econnreset,
            105 => Errno::Enobufs,
            106 => Errno::Eacces,
            107 => Errno::Eisconn,
            108 => Errno::Enotconn,
            109 => Errno::Eshutdown,
            110 => Errno::Etimedout,
            111 => Errno::Econnrefused,
            113 => Errno::Ehostunreach,
            114 => Errno::Ealread,
            115 => Errno::Einprogress,
            116 => Errno::Estale,
            117 => Errno::Euclean,
            118 => Errno::Enotnam,
            119 => Errno::Enavail,
            120 => Errno::Eisnam,
            121 => Errno::Eremote,
            123 => Errno::Enomedium,
            124 => Errno::Emediumtype,
            125 => Errno::Ecanceled,
            126 => Errno::Enokey,
            127 => Errno::Ekeyexpired,
            128 => Errno::Ekeyrevoked,
            129 => Errno::Ekeyrejected,
            130 => Errno::Eownerdead,
            131 => Errno::Enotrecoverable,
            132 => Errno::Etoomanyrefs,
            133 => Errno::Ewrongmsg,
            134 => Errno::Elength,
            135 => Errno::Eprotoall,
            136 => Errno::Eprotonosupp,
            137 => Errno::Etimenotsupp,
            138 => Errno::Eopnotsupp,
            139 => Errno::Eprotoframe,
            140 => Errno::Eprotorange,
            141 => Errno::Ehalfopen,
            142 => Errno::Eclosed,
            _ => Errno::Unknown,
        }
    }

    /// Convert Errno to raw errno value
    pub fn to_raw(&self) -> i32 {
        *self as i32
    }

    /// Get human-readable error name (e.g., "EPERM")
    pub fn name(&self) -> &'static str {
        match self {
            Errno::Epperm | Errno::Eperm => "EPERM",
            Errno::Enoent => "ENOENT",
            Errno::Esrch => "ESRCH",
            Errno::Eintr => "EINTR",
            Errno::Eio => "EIO",
            Errno::Enxio => "ENXIO",
            Errno::E2big => "E2BIG",
            Errno::Enoexec => "ENOEXEC",
            Errno::Ebadf => "EBADF",
            Errno::Echild => "ECHILD",
            Errno::Eagain => "EAGAIN",
            Errno::Enomem => "ENOMEM",
            Errno::Eaccess => "EACCES",
            Errno::Ebadaddr => "EFAULT",
            Errno::Enotblk => "ENOTBLK",
            Errno::Ebusy => "EBUSY",
            Errno::Eexist => "EEXIST",
            Errno::Exdev => "EXDEV",
            Errno::Enodev => "ENODEV",
            Errno::Enotdir => "ENOTDIR",
            Errno::Eisdir => "EISDIR",
            Errno::Einval => "EINVAL",
            Errno::Enfile => "ENFILE",
            Errno::Emfile => "EMFILE",
            Errno::Enotty => "ENOTTY",
            Errno::Etxtbsy => "ETXTBSY",
            Errno::Efbig => "EFBIG",
            Errno::Enospc => "ENOSPC",
            Errno::Espipe => "ESPIPE",
            Errno::Erofs => "EROFS",
            Errno::Emlink => "EMLINK",
            Errno::Epipe => "EPIPE",
            Errno::Edom => "EDOM",
            Errno::Erange => "ERANGE",
            Errno::Edeadlk => "EDEADLK",
            Errno::Enametoolong => "ENAMETOOLONG",
            Errno::Enolck => "ENOLCK",
            Errno::Enosys => "ENOSYS",
            Errno::Enotempty => "ENOTEMPTY",
            Errno::Eloop => "ELOOP",
            Errno::Eidrm => "EIDRM",
            Errno::Echrng => "ECHRNG",
            Errno::El2nsync => "EL2NSYNC",
            Errno::El3hl => "EL3HL",
            Errno::El3rst => "EL3RST",
            Errno::Elnrng => "ELNRNG",
            Errno::Eunatch => "EUNATCH",
            Errno::Enocsi => "ENOCSI",
            Errno::El2hlt => "EL2HLT",
            Errno::Edeadlock => "EDEADLOCK",
            Errno::Ebfont => "EBFONT",
            Errno::Enostr => "ENOSTR",
            Errno::Enodata => "ENODATA",
            Errno::Etime => "ETIME",
            Errno::Enosr => "ENOSR",
            Errno::Enonet => "ENONET",
            Errno::Enopkg => "ENOPKG",
            Errno::Eremot => "EREMOTE",
            Errno::Enolink => "ENOLINK",
            Errno::Eadv => "EADV",
            Errno::Esrmnt => "ESRMOUNT",
            Errno::Ecomm => "ECOMM",
            Errno::Eproto => "EPROTO",
            Errno::Emultihop => "EMULTIHOP",
            Errno::Ebadmsg => "EBADMSG",
            Errno::Eoverflow => "EOVERFLOW",
            Errno::Enotuniq => "ENOTUNIQ",
            Errno::Ebadfd => "EBADFD",
            Errno::Eremchg => "EREMCHG",
            Errno::Elibacc => "ELIBACC",
            Errno::Elibbad => "ELIBBAD",
            Errno::Elibscn => "ELIBSCN",
            Errno::Elibmax => "ELIBMAX",
            Errno::Elibexec => "ELIBEXEC",
            Errno::Eilseq => "EILSEQ",
            Errno::Erestart => "ERESTART",
            Errno::Eustr => "EUSTR",
            Errno::Eaddrinuse => "EADDRINUSE",
            Errno::Eaddrnotavail => "EADDRNOTAVAIL",
            Errno::Enetdown => "ENETDOWN",
            Errno::Enetunreach => "ENETUNREACH",
            Errno::Enetreset => "ENETRESET",
            Errno::Econnaborted => "ECONNABORTED",
            Errno::Econnreset => "ECONNRESET",
            Errno::Enobufs => "ENOBUFS",
            Errno::Eacces => "EALREADY",
            Errno::Eisconn => "EISCONN",
            Errno::Enotconn => "ENOTCONN",
            Errno::Eshutdown => "ESHUTDOWN",
            Errno::Etimedout => "ETIMEDOUT",
            Errno::Econnrefused => "ECONNREFUSED",
            Errno::Ehostunreach => "EHOSTUNREACH",
            Errno::Ealread => "EALREADY",
            Errno::Einprogress => "EINPROGRESS",
            Errno::Estale => "ESTALE",
            Errno::Euclean => "EUCLEAN",
            Errno::Enotnam => "ENOTNAM",
            Errno::Enavail => "ENAVAIL",
            Errno::Eisnam => "EISNAM",
            Errno::Eremote => "EREMOTE",
            Errno::Enomedium => "ENOMEDIUM",
            Errno::Emediumtype => "EMEDIUMTYPE",
            Errno::Ecanceled => "ECANCELED",
            Errno::Enokey => "ENOKEY",
            Errno::Ekeyexpired => "EKEYEXPIRED",
            Errno::Ekeyrevoked => "EKEYREVOKED",
            Errno::Ekeyrejected => "EKEYREJECTED",
            Errno::Eownerdead => "EOWNERDEAD",
            Errno::Enotrecoverable => "ENOTRECOVERABLE",
            Errno::Etoomanyrefs => "ETOOMANYREFS",
            Errno::Eoverflow => "EOVERFLOW",
            Errno::Ewrongmsg => "EWRONGMSG",
            Errno::Elength => "ELENGTH",
            Errno::Eprotoall => "EPROTOALL",
            Errno::Eprotonosupp => "EPROTONOSUPP",
            Errno::Etimenotsupp => "ETIMENOTSUP",
            Errno::Eopnotsupp => "EOPNOTSUPP",
            Errno::Eprotoframe => "EPROTOFRAME",
            Errno::Eprotorange => "EPROTORANGE",
            Errno::Ehalfopen => "EHALFOPEN",
            Errno::Eclosed => "ECLOSED",
            Errno::Unknown => "EUNKNOWN",
        }
    }

    /// Get human-readable error description
    pub fn description(&self) -> &'static str {
        match self {
            Errno::Epperm | Errno::Eperm => "Operation not permitted",
            Errno::Enoent => "No such file or directory",
            Errno::Esrch => "No such process",
            Errno::Eintr => "Interrupted system call",
            Errno::Eio => "Input/output error",
            Errno::Enxio => "No such device or address",
            Errno::E2big => "Argument list too long",
            Errno::Enoexec => "Exec format error",
            Errno::Ebadf => "Bad file descriptor",
            Errno::Echild => "No child processes",
            Errno::Eagain => "Resource temporarily unavailable",
            Errno::Enomem => "Out of memory",
            Errno::Eaccess => "Permission denied",
            Errno::Ebadaddr => "Bad address",
            Errno::Enotblk => "Block device required",
            Errno::Ebusy => "Device or resource busy",
            Errno::Eexist => "File exists",
            Errno::Exdev => "Invalid cross-device link",
            Errno::Enodev => "No such device",
            Errno::Enotdir => "Not a directory",
            Errno::Eisdir => "Is a directory",
            Errno::Einval => "Invalid argument",
            Errno::Enfile => "Too many open files in system",
            Errno::Emfile => "Too many open files",
            Errno::Enotty => "Inappropriate ioctl for device",
            Errno::Etxtbsy => "Text file busy",
            Errno::Efbig => "File too large",
            Errno::Enospc => "No space left on device",
            Errno::Espipe => "Illegal seek",
            Errno::Erofs => "Read-only file system",
            Errno::Emlink => "Too many links",
            Errno::Epipe => "Broken pipe",
            Errno::Edom => "Numerical argument out of domain",
            Errno::Erange => "Numerical result out of range",
            Errno::Edeadlk => "Resource deadlock avoided",
            Errno::Enametoolong => "File name too long",
            Errno::Enolck => "No locks available",
            Errno::Enosys => "Function not implemented",
            Errno::Enotempty => "Directory not empty",
            Errno::Eloop => "Too many levels of symbolic links",
            Errno::Eidrm => "Identifier removed",
            Errno::Echrng => "Channel number out of range",
            Errno::El2nsync => "Level 2 not synchronized",
            Errno::El3hl => "Level 3 halted",
            Errno::El3rst => "Level 3 reset",
            Errno::Elnrng => "Link number out of range",
            Errno::Eunatch => "Protocol driver not attached",
            Errno::Enocsi => "No CSI structure available",
            Errno::El2hlt => "Level 2 halted",
            Errno::Edeadlock => "File locking deadlock detected",
            Errno::Ebfont => "Bad font file format",
            Errno::Enostr => "Device not a stream",
            Errno::Enodata => "No data available",
            Errno::Etime => "Timer expired",
            Errno::Enosr => "Out of streams resources",
            Errno::Enonet => "Machine is not on the network",
            Errno::Enopkg => "Package not installed",
            Errno::Eremot => "Remote I/O error",
            Errno::Enolink => "Link has been severed",
            Errno::Eadv => "Advertise error",
            Errno::Esrmnt => "Srmount error",
            Errno::Ecomm => "Communication error on send",
            Errno::Eproto => "Protocol error",
            Errno::Emultihop => "Multihop attempted",
            Errno::Ebadmsg => "Bad message",
            Errno::Eoverflow => "Value too large for defined data type",
            Errno::Enotuniq => "Name not unique on network",
            Errno::Ebadfd => "File descriptor in bad state",
            Errno::Eremchg => "Remote address changed",
            Errno::Elibacc => "Cannot access a needed shared library",
            Errno::Elibbad => "Accessing a corrupted shared library",
            Errno::Elibscn => ".lib section in a.out corrupted",
            Errno::Elibmax => "Attempting to link in too many shared libraries",
            Errno::Elibexec => "Cannot exec a shared library directly",
            Errno::Eilseq => "Illegal byte sequence",
            Errno::Erestart => "Interrupted system call should be restarted",
            Errno::Eustr => "Streams pipe error",
            Errno::Eaddrinuse => "Address already in use",
            Errno::Eaddrnotavail => "Cannot assign requested address",
            Errno::Enetdown => "Network is down",
            Errno::Enetunreach => "Network is unreachable",
            Errno::Enetreset => "Network dropped connection because of reset",
            Errno::Econnaborted => "Software caused connection abort",
            Errno::Econnreset => "Connection reset by peer",
            Errno::Enobufs => "No buffer space available",
            Errno::Eacces => "Operation already in progress",
            Errno::Eisconn => "Socket is already connected",
            Errno::Enotconn => "Socket is not connected",
            Errno::Eshutdown => "Cannot send after transport endpoint shutdown",
            Errno::Etimedout => "Connection timed out",
            Errno::Econnrefused => "Connection refused",
            Errno::Ehostunreach => "No route to host",
            Errno::Ealread => "Operation already in progress",
            Errno::Einprogress => "Operation now in progress",
            Errno::Estale => "Stale file handle",
            Errno::Euclean => "Structure needs cleaning",
            Errno::Enotnam => "Not a XENIX named type file",
            Errno::Enavail => "No XENIX semaphores available",
            Errno::Eisnam => "Is a named type file",
            Errno::Eremote => "Remote I/O error",
            Errno::Enomedium => "No medium found",
            Errno::Emediumtype => "Wrong medium type",
            Errno::Ecanceled => "Operation Canceled",
            Errno::Enokey => "Required key not available",
            Errno::Ekeyexpired => "Key has expired",
            Errno::Ekeyrevoked => "Key has been revoked",
            Errno::Ekeyrejected => "Key was rejected by service",
            Errno::Eownerdead => "Owner died",
            Errno::Enotrecoverable => "State not recoverable",
            Errno::Etoomanyrefs => "Too many references: cannot splice",
            Errno::Ewrongmsg => "Wrong message type",
            Errno::Elength => "Message too long",
            Errno::Eprotoall => "Protocol not available",
            Errno::Eprotonosupp => "Protocol not supported",
            Errno::Etimenotsupp => "Timing not supported",
            Errno::Eopnotsupp => "Operation not supported",
            Errno::Eprotoframe => "Protocol framing error",
            Errno::Eprotorange => "Protocol out of range",
            Errno::Ehalfopen => "Socket is half-open (connection being aborted)",
            Errno::Eclosed => "Socket is closed",
            Errno::Unknown => "Unknown error",
        }
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.description())
    }
}

impl std::error::Error for Errno {
    fn description(&self) -> &str {
        self.description()
    }
}

/// Result type for POSIX API calls
pub type PosixResult<T> = Result<T, Errno>;

/// Thread-local error variable (errno)
#[cfg(feature = "std")]
thread_local! {
    static errno: std::cell::RefCell<i32> = std::cell::RefCell::new(0);
}

/// Get the current errno value
#[cfg(feature = "std")]
pub fn get_errno() -> i32 {
    errno.with(|e| *e.borrow())
}

/// Set the errno value
#[cfg(feature = "std")]
pub fn set_errno(err: i32) {
    errno.with(|e| *e.borrow_mut() = err);
}

/// Clear the errno value (set to 0)
#[cfg(feature = "std")]
pub fn clear_errno() {
    errno.with(|e| *e.borrow_mut() = 0);
}

/// Macro to convert system call result to PosixResult
#[macro_export]
macro_rules! posix_result {
    ($result:expr) => {
        if $result < 0 {
            Err($crate::errors::Errno::from_raw(-$result))
        } else {
            Ok($result)
        }
    };
    ($result:expr, $cast_type:ty) => {
        if $result < 0 {
            Err($crate::errors::Errno::from_raw(-$result))
        } else {
            Ok($result as $cast_type)
        }
    };
}

/// Macro to check if errno indicates retry is needed
#[macro_export]
macro_rules! is_retryable_error {
    ($errno:expr) => {
        matches!($errno, Errno::Eintr | Errno::Eagain | Errno::Etimedout)
    };
}

/// Common error handling patterns
#[cfg(feature = "std")]
pub mod helpers {
    use super::*;
    
    /// Handle a system call that may return retryable errors
    pub fn retry_on_eintr<T, F>(f: F) -> PosixResult<T>
    where
        F: Fn() -> PosixResult<T>,
    {
        loop {
            return match f() {
                Ok(result) => Ok(result),
                Err(Errno::Eintr) => continue,  // Retry on interrupt
                Err(err) => Err(err),           // Propagate other errors
            };
        }
    }

    /// Handle a system call that may return EAGAIN/EWOULDBLOCK
    pub fn retry_on_eagain<T, F>(f: F) -> PosixResult<T>
    where
        F: Fn() -> PosixResult<T>,
    {
        loop {
            return match f() {
                Ok(result) => Ok(result),
                Err(Errno::Eagain) => continue,  // Retry on would block
                Err(err) => Err(err),           // Propagate other errors
            };
        }
    }

    /// Helper to convert Result<(), Errno> to Result<i32, Errno> where success = 0
    pub fn unit_to_zero(result: PosixResult<()>) -> PosixResult<i32> {
        result.map(|_| 0)
    }

    /// Helper to handle system calls that return negative values for errors
    pub fn handle_syscall_result<T>(result: isize) -> PosixResult<T>
    where
        T: From<isize>,
    {
        if result < 0 {
            Err(Errno::from_raw(-(result as i32)))
        } else {
            Ok(result.into())
        }
    }
}
