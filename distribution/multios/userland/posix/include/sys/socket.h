/*
 * BSD Socket API
 * MultiOS POSIX Compatibility Layer
 */

#ifndef _SYS_SOCKET_H
#define _SYS_SOCKET_H

#include <sys/types.h>

/* Socket types */
#define SOCK_STREAM    1  /* Stream socket */
#define SOCK_DGRAM     2  /* Datagram socket */
#define SOCK_RAW       3  /* Raw socket */
#define SOCK_RDM       4  /* Reliable datagram socket */
#define SOCK_SEQPACKET 5  /* Sequenced packet socket */

/* Protocol families */
#define PF_UNSPEC      0  /* Unspecified */
#define PF_UNIX        1  /* Local to host (pipes and file-domain) */
#define PF_INET        2  /* Internetwork: UDP, TCP, etc. */
#define PF_INET6       10 /* Internetwork Version 6 */
#define PF_IPX         4  /* IPX family */
#define PF_APPLETALK   5  /* AppleTalk */
#define PF_ROUTE       16 /* Internal Routing Protocol */

/* Address families (same as protocol families for simplicity) */
#define AF_UNSPEC      PF_UNSPEC
#define AF_UNIX        PF_UNIX
#define AF_INET        PF_INET
#define AF_INET6       PF_INET6
#define AF_IPX         PF_IPX
#define AF_APPLETALK   PF_APPLETALK
#define AF_ROUTE       PF_ROUTE

/* Protocol constants */
#define IPPROTO_IP     0   /* Dummy protocol for IP */
#define IPPROTO_ICMP   1   /* Internet Control Message Protocol */
#define IPPROTO_IGMP   2   /* Internet Group Management Protocol */
#define IPPROTO_TCP    6   /* Transmission Control Protocol */
#define IPPROTO_UDP    17  /* User Datagram Protocol */
#define IPPROTO_RAW    255 /* Raw IP packets */

/* Socket option levels */
#define SOL_SOCKET     1   /* Socket-level options */

/* Socket option flags */
#define SO_DEBUG        1  /* Turn on debugging info recording */
#define SO_ACCEPTCONN   2  /* Socket has had listen() called on it */
#define SO_REUSEADDR    4  /* Allow local address reuse */
#define SO_KEEPALIVE    8  /* Keep connections alive */
#define SO_DONTROUTE    16 /* Just use interface addresses */
#define SO_BROADCAST    32 /* Permit sending of broadcast messages */
#define SO_USELOOPBACK  64 /* Bypass hardware when possible */
#define SO_LINGER       128 /* Linger on close if data present */
#define SO_OOBINLINE    256 /* Leave received out-of-band data in line */
#define SO_REUSEPORT    512 /* Allow local address & port reuse */

/* Socket option values for SO_LINGER */
struct linger {
    int l_onoff;   /* Linger active */
    int l_linger;  /* Linger time in seconds */
};

/* Socket level options */
#define SO_TYPE         3    /* Get socket type */
#define SO_ERROR        4    /* Get and clear error */
#define SO_SNDBUF       5    /* Send buffer size */
#define SO_RCVBUF       6    /* Receive buffer size */
#define SO_SNDLOWAT     7    /* Send low-water mark */
#define SO_RCVLOWAT     8    /* Receive low-water mark */
#define SO_SNDTIMEO     9    /* Send timeout */
#define SO_RCVTIMEO     10   /* Receive timeout */
#define SO_RCVBUFFORCE  33   /* Receive buffer force */
#define SO_SNDBUFFORCE  32   /* Send buffer force */

/* Socket shutdown modes */
#define SHUT_RD   0  /* No more receptions */
#define SHUT_WR   1  /* No more transmissions */
#define SHUT_RDWR 2  /* No more receptions or transmissions */

/* Socket address length maximum */
#define MAXSOCKADDR    128

/* Function declarations */

/* Socket operations */
int socket(int domain, int type, int protocol);
int socketpair(int domain, int type, int protocol, int sv[2]);

/* Bind and connect */
int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);

/* Listen and accept */
int listen(int sockfd, int backlog);
int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);
int accept4(int sockfd, struct sockaddr *addr, socklen_t *addrlen, int flags);

/* Send and receive */
ssize_t send(int sockfd, const void *buf, size_t len, int flags);
ssize_t recv(int sockfd, void *buf, size_t len, int flags);
ssize_t sendto(int sockfd, const void *buf, size_t len, int flags,
               const struct sockaddr *to, socklen_t tolen);
ssize_t recvfrom(int sockfd, void *buf, size_t len, int flags,
                 struct sockaddr *from, socklen_t *fromlen);

/* Send and receive with control information */
ssize_t sendmsg(int sockfd, const struct msghdr *msg, int flags);
ssize_t recvmsg(int sockfd, struct msghdr *msg, int flags);

/* Socket control and status */
int getsockopt(int sockfd, int level, int optname,
               void *optval, socklen_t *optlen);
int setsockopt(int sockfd, int level, int optname,
               const void *optval, socklen_t optlen);
int getsockname(int sockfd, struct sockaddr *addr, socklen_t *addrlen);
int getpeername(int sockfd, struct sockaddr *addr, socklen_t *addrlen);

/* Shutdown */
int shutdown(int sockfd, int how);

/* Host and network conversion functions */
unsigned long inet_addr(const char *cp);
char *inet_ntoa(struct in_addr in);
const char *inet_ntop(int af, const void *src, char *dst, socklen_t size);
int inet_pton(int af, const char *src, void *dst);

/* Message header structure for sendmsg/recvmsg */
struct iovec {
    void  *iov_base;  /* Base address */
    size_t iov_len;   /* Length */
};

struct msghdr {
    void         *msg_name;       /* Optional address */
    socklen_t     msg_namelen;    /* Size of address */
    struct iovec *msg_iov;        /* Scatter/gather array */
    size_t        msg_iovlen;     /* Number of elements in msg_iov */
    void         *msg_control;    /* Ancillary data, see below */
    socklen_t     msg_controllen; /* Ancillary data buffer length */
    int           msg_flags;      /* Flags on received message */
};

/* Ancillary data types */
struct cmsghdr {
    socklen_t cmsg_len;    /* Data byte count, including header */
    int       cmsg_level;  /* Originating protocol */
    int       cmsg_type;   /* Protocol-specific type */
};

/* Ancillary data macros */
#define CMSG_DATA(cmsg) ((unsigned char *)((cmsg) + 1))
#define CMSG_NXTHDR(mhdr, cmsg) \
    (((unsigned char *)(cmsg) + ((cmsg)->cmsg_len + sizeof(struct cmsghdr) <= \
      (mhdr)->msg_control + (mhdr)->msg_controllen) ? \
      (struct cmsghdr *)((unsigned char *)(cmsg) + (cmsg)->cmsg_len) : \
      (struct cmsghdr *)0))

#define CMSG_FIRSTHDR(mhdr) \
    ((mhdr)->msg_control ? (struct cmsghdr *)(mhdr)->msg_control : \
     (struct cmsghdr *)0)

/* Ancillary data levels */
#define SOL_SOCKET     1   /* Socket-level options */
#define IPPROTO_IPV6   41  /* IPv6 protocol */
#define IPPROTO_TCP    6   /* Transmission Control Protocol */

/* Ancillary data types */
#define SCM_RIGHTS     0x01  /* Pass file descriptors */
#define SCM_CREDENTIALS 0x02 /* Credentials structure */

/* Credentials structure for SCM_CREDENTIALS */
struct ucred {
    pid_t pid;  /* Process ID */
    uid_t uid;  /* User ID */
    gid_t gid;  /* Group ID */
};

/* Socket flags for recv and send */
#define MSG_OOB       0x01  /* Process out-of-band data */
#define MSG_PEEK      0x02  /* Peek at incoming data */
#define MSG_DONTROUTE 0x04  /* Send without using routing tables */
#define MSG_EOR       0x08  /* Data completes record */
#define MSG_TRUNC     0x20  /* Data discarded before delivery */
#define MSG_CTRUNC    0x40  /* Control data truncated */
#define MSG_WAITALL   0x100 /* Wait for complete request */
#define MSG_NOSIGNAL  0x400 /* Do not generate SIGPIPE */

/* Standard flags for accept4 */
#define SOCK_NONBLOCK 0x80000000  /* Non-blocking socket */
#define SOCK_CLOEXEC  0x80000000  /* Close-on-exec socket */

/* Protocol-independent hostname/address resolution */
struct addrinfo {
    int ai_flags;           /* Input flags */
    int ai_family;          /* Protocol family for socket */
    int ai_socktype;        /* Socket type */
    int ai_protocol;        /* Protocol */
    socklen_t ai_addrlen;   /* Length of socket address */
    char *ai_canonname;     /* Canonical name */
    struct sockaddr *ai_addr; /* Socket address */
    struct addrinfo *ai_next; /* Next entry in list */
};

/* Address information flags */
#define AI_PASSIVE     0x0001  /* Socket address is intended for bind() */
#define AI_CANONNAME   0x0002  /* Return canonical name */
#define AI_NUMERICHOST 0x0004  /* Nodename must be numeric */
#define AI_NUMERICSERV 0x0008  /* Servicename must be numeric */
#define AI_V4MAPPED    0x0010  /* If no IPv6 addresses found, return IPv4-mapped IPv6 addresses */
#define AI_ALL         0x0020  /* Return both IPv4 and IPv6 addresses */
#define AI_ADDRCONFIG  0x0040  /* Use configuration of the system */

/* Address information errors */
#define EAI_AGAIN      1  /* Temporary failure in name resolution */
#define EAI_BADFLAGS   2  /* Invalid value for ai_flags */
#define EAI_FAIL       3  /* Non-recoverable failure in name resolution */
#define EAI_FAMILY     4  /* ai_family not supported */
#define EAI_MEMORY     5  /* Memory allocation failure */
#define EAI_NODATA     6  /* No address associated with nodename */
#define EAI_NONAME     7  /* Name or service not known */
#define EAI_SERVICE    8  /* Servname not supported for ai_socktype */
#define EAI_SOCKTYPE   9  /* ai_socktype not supported */
#define EAI_SYSTEM     10 /* System error returned in errno */

/* Host and service resolution */
int getaddrinfo(const char *node, const char *service,
                const struct addrinfo *hints,
                struct addrinfo **res);
void freeaddrinfo(struct addrinfo *res);
const char *gai_strerror(int error);
int getnameinfo(const struct sockaddr *sa, socklen_t salen,
                char *host, size_t hostlen,
                char *serv, size_t servlen,
                int flags);

#endif /* _SYS_SOCKET_H */