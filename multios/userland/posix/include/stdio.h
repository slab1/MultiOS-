#ifndef MULTIOS_STDIO_H
#define MULTIOS_STDIO_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

/* File descriptor constants */
#define STDIN_FILENO    0
#define STDOUT_FILENO   1
#define STDERR_FILENO   2

/* File status flags */
#define O_RDONLY        0x00000001
#define O_WRONLY        0x00000002
#define O_RDWR          0x00000003
#define O_APPEND        0x00000004
#define O_CREAT         0x00000008
#define O_TRUNC         0x00000010
#define O_EXCL          0x00000020
#define O_NOCTTY        0x00000040
#define O_NONBLOCK      0x00000080
#define O_DSYNC         0x00000100
#define O_SYNC          0x00000200
#define O_ASYNC         0x00000400
#define O_DIRECT        0x00000800
#define O_DIRECTORY     0x00002000
#define O_NOFOLLOW      0x00004000
#define O_NOATIME       0x00008000

/* File access modes for access() */
#define F_OK            0
#define X_OK            1
#define W_OK            2
#define R_OK            4

/* Seek constants */
#define SEEK_SET        0
#define SEEK_CUR        1
#define SEEK_END        2
#define SEEK_DATA       3
#define SEEK_HOLE       4

/* Buffer modes */
#define _IOFBF          0
#define _IOLBF          1
#define _IONBF          2

/* EOF indicator */
#define EOF             -1

/* Maximum path and name lengths */
#define PATH_MAX        4096
#define NAME_MAX        255

/* File type masks */
#define S_IFMT          0o170000
#define S_IFREG         0o100000
#define S_IFDIR         0o040000
#define S_IFLNK         0o120000
#define S_IFBLK         0o060000
#define S_IFCHR         0o020000
#define S_IFIFO         0o010000
#define S_IFSOCK        0o140000

/* File mode bits */
#define S_IRUSR         0o00400
#define S_IWUSR         0o00200
#define S_IXUSR         0o00100
#define S_IRGRP         0o00040
#define S_IWGRP         0o00020
#define S_IXGRP         0o00010
#define S_IROTH         0o00004
#define S_IWOTH         0o00002
#define S_IXOTH         0o00001

/* File lock types */
#define F_RDLCK         0
#define F_WRLCK         1
#define F_UNLCK         2

/* File control commands */
#define F_DUPFD         0
#define F_GETFD         1
#define F_SETFD         2
#define F_GETFL         3
#define F_SETFL         4
#define F_GETLK         5
#define F_SETLK         6
#define F_SETLKW        7

/* File descriptor flags */
#define FD_CLOEXEC      1

/* File structure (simplified) */
typedef struct {
    int fd;                     /* File descriptor */
    unsigned int flags;         /* File status flags */
    mode_t mode;                /* File mode */
    off_t offset;               /* Current file offset */
    int eof;                    /* End of file flag */
    int error;                  /* Error code */
    void *buffer;               /* I/O buffer */
    size_t buf_size;            /* Buffer size */
    size_t buf_pos;             /* Buffer position */
    size_t buf_count;           /* Buffer count */
} FILE;

/* File status structure */
typedef struct {
    dev_t st_dev;               /* Device ID */
    ino_t st_ino;               /* Inode number */
    mode_t st_mode;             /* File type and permissions */
    nlink_t st_nlink;           /* Number of hard links */
    uid_t st_uid;               /* User ID of owner */
    gid_t st_gid;               /* Group ID of owner */
    dev_t st_rdev;              /* Device ID (if special file) */
    off_t st_size;              /* Total size in bytes */
    blksize_t st_blksize;       /* Block size for filesystem I/O */
    blkcnt_t st_blocks;         /* Number of 512-byte blocks allocated */
    time_t st_atime;            /* Last access time */
    time_t st_mtime;            /* Last modification time */
    time_t st_ctime;            /* Last status change time */
    long st_atime_nsec;         /* Nanoseconds part of last access time */
    long st_mtime_nsec;         /* Nanoseconds part of last modification time */
    long st_ctime_nsec;         /* Nanoseconds part of last status change time */
} stat;

/* Flock structure for file locking */
typedef struct {
    short l_type;               /* Lock type */
    short l_whence;             /* How to interpret l_start */
    off_t l_start;              /* Starting offset for lock */
    off_t l_len;                /* Number of bytes to lock */
    pid_t l_pid;                /* Process holding the lock */
} flock;

/* Directory entry structure */
typedef struct {
    ino_t d_ino;                /* Inode number */
    off_t d_off;                /* Offset to next dirent */
    unsigned short d_reclen;    /* Length of this dirent */
    unsigned char d_type;       /* File type */
    char d_name[256];           /* Filename */
} dirent;

/* Directory stream type */
typedef void DIR;

/* Standard streams */
#define stdin   ((FILE*)0)
#define stdout  ((FILE*)1)
#define stderr  ((FILE*)2)

/* Function declarations */

/* File operations */
extern int open(const char *pathname, int flags, mode_t mode);
extern int close(int fd);
extern ssize_t read(int fd, void *buf, size_t count);
extern ssize_t write(int fd, const void *buf, size_t count);
extern off_t lseek(int fd, off_t offset, int whence);
extern int fstat(int fd, struct stat *buf);
extern int stat(const char *pathname, struct stat *buf);
extern int dup(int oldfd);
extern int dup2(int oldfd, int newfd);
extern int ftruncate(int fd, off_t length);
extern int access(const char *pathname, int mode);
extern int unlink(const char *pathname);
extern int link(const char *oldpath, const char *newpath);
extern int symlink(const char *target, const char *linkpath);
extern ssize_t readlink(const char *pathname, char *buf, size_t bufsize);
extern int rename(const char *oldpath, const char *newpath);
extern int chmod(const char *pathname, mode_t mode);
extern int chown(const char *pathname, uid_t owner, gid_t group);

/* Directory operations */
extern int chdir(const char *pathname);
extern char *getcwd(char *buf, size_t size);
extern int mkdir(const char *pathname, mode_t mode);
extern int rmdir(const char *pathname);
extern DIR *opendir(const char *name);
extern int closedir(DIR *dirp);
extern struct dirent *readdir(DIR *dirp);

/* File descriptor operations */
extern int fcntl(int fd, int cmd, ...);
extern int ioctl(int fd, unsigned long request, ...);
extern int select(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval *timeout);
extern int poll(struct pollfd *fds, nfds_t nfds, int timeout);

/* File and stream operations */
extern FILE *fopen(const char *pathname, const char *mode);
extern int fclose(FILE *stream);
extern size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
extern size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
extern int fseek(FILE *stream, long offset, int whence);
extern long ftell(FILE *stream);
extern void rewind(FILE *stream);
extern int fgetpos(FILE *stream, fpos_t *pos);
extern int fsetpos(FILE *stream, const fpos_t *pos);
extern int feof(FILE *stream);
extern int ferror(FILE *stream);
extern void clearerr(FILE *stream);
extern int fileno(FILE *stream);
extern FILE *fdopen(int fd, const char *mode);
extern int fflush(FILE *stream);
extern void setbuf(FILE *stream, char *buf);
extern int setvbuf(FILE *stream, char *buf, int mode, size_t size);

/* Character I/O */
extern int fgetc(FILE *stream);
extern char *fgets(char *s, int size, FILE *stream);
extern int fputc(int c, FILE *stream);
extern int fputs(const char *s, FILE *stream);
extern int getc(FILE *stream);
extern int getchar(void);
extern char *gets(char *s);
extern int putc(int c, FILE *stream);
extern int putchar(int c);
extern int puts(const char *s);
extern int ungetc(int c, FILE *stream);

/* Formatted I/O */
extern int printf(const char *format, ...);
extern int fprintf(FILE *stream, const char *format, ...);
extern int sprintf(char *str, const char *format, ...);
extern int snprintf(char *str, size_t size, const char *format, ...);
extern int vprintf(const char *format, va_list ap);
extern int vfprintf(FILE *stream, const char *format, va_list ap);
extern int vsprintf(char *str, const char *format, va_list ap);
extern int vsnprintf(char *str, size_t size, const char *format, va_list ap);
extern int scanf(const char *format, ...);
extern int fscanf(FILE *stream, const char *format, ...);
extern int sscanf(const char *str, const char *format, ...);

/* Temporary files */
extern FILE *tmpfile(void);
extern char *tmpnam(char *s);
extern int mkstemp(char *template);

/* File positioning and size */
extern int fseeko(FILE *stream, off_t offset, int whence);
extern off_t ftello(FILE *stream);

/* File status */
extern int fstatat(int dirfd, const char *pathname, struct stat *buf, int flags);
extern int newfstatat(int dirfd, const char *pathname, struct stat *buf, int flags);

/* File operations with flags */
extern int openat(int dirfd, const char *pathname, int flags, mode_t mode);
extern int faccessat(int dirfd, const char *pathname, int mode, int flags);

/* Special file types */
extern int mkfifo(const char *pathname, mode_t mode);
extern int mknod(const char *pathname, mode_t mode, dev_t dev);

/* Directory entry operations */
extern int readdir_r(DIR *dirp, struct dirent *entry, struct dirent **result);
extern void seekdir(DIR *dirp, long loc);
extern long telldir(DIR *dirp);

/* System V IPC */
extern int msgget(key_t key, int msgflg);
extern int msgsnd(int msqid, const void *msgp, size_t msgsz, int msgflg);
extern ssize_t msgrcv(int msqid, void *msgp, size_t msgsz, long msgtyp, int msgflg);
extern int msgctl(int msqid, int cmd, struct msqid_ds *buf);

#endif /* MULTIOS_STDIO_H */
