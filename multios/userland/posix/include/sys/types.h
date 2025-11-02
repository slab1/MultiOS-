/*
 * POSIX Type Definitions
 * MultiOS POSIX Compatibility Layer
 */

#ifndef _SYS_TYPES_H
#define _SYS_TYPES_H

#include <stddef.h>

/* Basic integral types */
typedef unsigned char   u_char;
typedef unsigned short  u_short;
typedef unsigned int    u_int;
typedef unsigned long   u_long;

/* System types */
typedef unsigned char   u_int8_t;
typedef unsigned short  u_int16_t;
typedef unsigned int    u_int32_t;
typedef unsigned long   u_int64_t;

/* Signed integral types */
typedef signed char     int8_t;
typedef signed short    int16_t;
typedef signed int      int32_t;
typedef signed long     int64_t;

/* Standard integer types from stdint.h */
typedef int8_t    int8;
typedef int16_t   int16;
typedef int32_t   int32;
typedef int64_t   int64;

typedef uint8_t   uint8;
typedef uint16_t  uint16;
typedef uint32_t  uint32;
typedef uint64_t  uint64;

/* Process ID types */
typedef long        pid_t;    /* Process ID */
typedef unsigned int uid_t;   /* User ID */
typedef unsigned int gid_t;   /* Group ID */
typedef long        ppid_t;   /* Parent Process ID */

/* File system types */
typedef unsigned long mode_t;     /* File mode bits */
typedef unsigned long ino_t;      /* Inode number */
typedef unsigned long dev_t;      /* Device number */
typedef unsigned long nlink_t;    /* Number of hard links */
typedef unsigned long blksize_t;  /* Block size for I/O */
typedef unsigned long blkcnt_t;   /* Block count */
typedef unsigned long fsblkcnt_t; /* File system block count */
typedef unsigned long fsfilcnt_t; /* File system file count */

/* Time types */
typedef long time_t;         /* Time in seconds */
typedef long suseconds_t;    /* Microseconds */
typedef struct timespec {
    time_t  tv_sec;   /* Seconds */
    long    tv_nsec;  /* Nanoseconds */
} timespec;

typedef struct timeval {
    time_t      tv_sec;      /* Seconds */
    suseconds_t tv_usec;     /* Microseconds */
} timeval;

typedef struct tm {
    int tm_sec;    /* Seconds (0-59) */
    int tm_min;    /* Minutes (0-59) */
    int tm_hour;   /* Hours (0-23) */
    int tm_mday;   /* Day of month (1-31) */
    int tm_mon;    /* Month (0-11) */
    int tm_year;   /* Year - 1900 */
    int tm_wday;   /* Day of week (0-6, Sunday = 0) */
    int tm_yday;   /* Day of year (0-365) */
    int tm_isdst;  /* Daylight savings time flag */
    long tm_gmtoff; /* Offset from UTC in seconds */
    const char *tm_zone; /* Timezone abbreviation */
} tm;

/* Size and offset types */
typedef long        off_t;    /* File size or offset */
typedef unsigned long size_t;  /* Size of objects */
typedef long        ssize_t;  /* Size or error return */

/* Select and poll types */
typedef unsigned long fd_set;     /* File descriptor set */
typedef unsigned int  nfds_t;     /* Number of file descriptors */

/* Socket address types */
typedef unsigned char  sa_family_t;    /* Socket address family */
typedef unsigned short in_port_t;      /* Port number */
typedef unsigned int   socklen_t;      /* Socket address length */

/* Address family definitions */
typedef unsigned int   in_addr_t;      /* Internet address */
typedef struct in_addr {
    in_addr_t s_addr;  /* Internet address */
} in_addr;

/* IPv6 address structure */
struct in6_addr {
    unsigned char s6_addr[16];  /* IPv6 address */
};
typedef struct in6_addr in6_addr_t;

/* Socket address structures */
struct sockaddr {
    sa_family_t sa_family;  /* Address family */
    char sa_data[14];       /* Address data */
};

/* Internet socket address */
struct sockaddr_in {
    sa_family_t    sin_family;  /* Address family (AF_INET) */
    in_port_t      sin_port;    /* Port number */
    struct in_addr sin_addr;    /* Internet address */
    unsigned char  sin_zero[8]; /* Padding */
};

/* IPv6 socket address */
struct sockaddr_in6 {
    sa_family_t     sin6_family;   /* Address family (AF_INET6) */
    in_port_t       sin6_port;     /* Port number */
    unsigned long   sin6_flowinfo; /* Flow information */
    struct in6_addr sin6_addr;     /* IPv6 address */
    unsigned long   sin6_scope_id; /* Scope ID */
};

/* Generic socket address */
struct sockaddr_storage {
    sa_family_t ss_family;     /* Address family */
    char __ss_padding[128 - sizeof(sa_family_t) - sizeof(unsigned long)];
    unsigned long __ss_align;  /* Force structure alignment */
};

/* Signal types */
typedef unsigned long sigset_t;  /* Signal set */
typedef struct siginfo_t siginfo_t;

/* Resource limits */
typedef unsigned long rlim_t;    /* Resource limit value */
typedef unsigned long id_t;      /* General ID type */

/* I/O control types */
typedef unsigned int  iovec_t;   /* I/O vector element */
typedef unsigned int  caddr_t;   /* Core address */
typedef unsigned char u_char;
typedef unsigned short u_short;
typedef unsigned int u_int;

/* Flag types */
typedef unsigned long  fflags_t;  /* File flags */
typedef unsigned long  fixpt_t;   /* Fixed point number */

/* Clock types */
typedef unsigned long clock_t;    /* Clock ticks */
typedef unsigned long clockid_t;  /* Clock identifier */

/* Event notification types */
typedef unsigned long event_t;    /* Event identifier */
typedef unsigned long key_t;      /* Key for interprocess communication */

/* Complex number types */
typedef struct {
    double real;
    double imag;
} complex_t;

/* IEEE 754 floating point types */
typedef float  float_t;   /* float */
typedef double double_t;  /* double */

/* Basic constants */
#define NULL ((void *)0)
#define _POSIX_PATH_MAX  256
#define _POSIX_NAME_MAX  255

/* Maximum values */
#define SSIZE_MAX LONG_MAX
#define SIZE_MAX  SIZE_MAX

#endif /* _SYS_TYPES_H */