#ifndef MULTIOS_UNISTD_H
#define MULTIOS_UNISTD_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>
#include <sys/select.h>
#include <sys/stat.h>

/* Process identification */
extern pid_t getpid(void);
extern pid_t getppid(void);

/* User and group identification */
extern uid_t getuid(void);
extern uid_t geteuid(void);
extern gid_t getgid(void);
extern gid_t getegid(void);
extern int setuid(uid_t uid);
extern int seteuid(uid_t euid);
extern int setgid(gid_t gid);
extern int setegid(uid_t egid);

/* Process creation and termination */
extern pid_t fork(void);
extern int execve(const char *pathname, char *const argv[], char *const envp[]);
extern int execl(const char *path, const char *arg, ...);
extern int execv(const char *path, char *const argv[]);
extern int execle(const char *path, const char *arg, ...);
extern int execve(const char *pathname, char *const argv[], char *const envp[]);
extern int execlp(const char *file, const char *arg, ...);
extern int execvp(const char *file, char *const argv[]);
extern int execvpe(const char *file, char *const argv[], char *const envp[]);
extern void _exit(int status);
extern void exit(int status);
extern int atexit(void (*function)(void));
extern int on_exit(void (*function)(int, void*), void *arg);

/* Process waiting */
extern pid_t wait(int *status);
extern pid_t waitpid(pid_t pid, int *status, int options);
extern int wait3(int *status, int options, struct rusage *rusage);
extern int wait4(pid_t pid, int *status, int options, struct rusage *rusage);

/* Process group and session management */
extern pid_t getpgrp(void);
extern int setpgrp(void);
extern pid_t setsid(void);
extern pid_t getsid(pid_t pid);
extern pid_t getpgid(pid_t pid);
extern int setpgid(pid_t pid, pid_t pgid);

/* File access and times */
extern int access(const char *pathname, int mode);
extern int faccessat(int dirfd, const char *pathname, int mode, int flags);
extern int chown(const char *pathname, uid_t owner, gid_t group);
extern int fchown(int fd, uid_t owner, gid_t group);
extern int lchown(const char *pathname, uid_t owner, gid_t group);
extern int fchownat(int dirfd, const char *pathname, uid_t owner, gid_t group, int flags);
extern int chmod(const char *pathname, mode_t mode);
extern int fchmod(int fd, mode_t mode);
extern int fchmodat(int dirfd, const char *pathname, mode_t mode, int flags);
extern mode_t umask(mode_t mask);
extern int utime(const char *filename, const struct utimbuf *times);
extern int utimes(const char *filename, const struct timeval times[2]);
extern int lutimes(const char *filename, const struct timeval times[2]);
extern int futimes(int fd, const struct timeval times[2]);
extern int futimens(int fd, const struct timespec times[2]);
extern int utimensat(int dirfd, const char *pathname, const struct timespec times[2], int flags);

/* File operations */
extern int link(const char *oldpath, const char *newpath);
extern int linkat(int olddirfd, const char *oldpath, int newdirfd, const char *newpath, int flags);
extern int symlink(const char *target, const char *linkpath);
extern int symlinkat(const char *target, int newdirfd, const char *linkpath);
extern ssize_t readlink(const char *pathname, char *buf, size_t bufsize);
extern ssize_t readlinkat(int dirfd, const char *pathname, char *buf, size_t bufsize);
extern int rename(const char *oldpath, const char *newpath);
extern int renameat(int olddirfd, const char *oldpath, int newdirfd, const char *newpath);
extern int renameat2(int olddirfd, const char *oldpath, int newdirfd, const char *newpath, unsigned int flags);
extern int unlink(const char *pathname);
extern int unlinkat(int dirfd, const char *pathname, int flags);
extern int rmdir(const char *pathname);

/* Working directory */
extern int chdir(const char *path);
extern int fchdir(int fd);
extern char *getcwd(char *buf, size_t size);
extern char *get_current_dir_name(void);

/* File descriptor operations */
extern int close(int fd);
extern int pipe(int pipefd[2]);
extern int pipe2(int pipefd[2], int flags);
extern int socketpair(int domain, int type, int protocol, int sv[2]);
extern int dup(int oldfd);
extern int dup2(int oldfd, int newfd);
extern int dup3(int oldfd, int newfd, int flags);

/* Seeking and positioning */
extern off_t lseek(int fd, off_t offset, int whence);
extern int fseeko(FILE *stream, off_t offset, int whence);
extern off_t ftello(FILE *stream);
extern int truncate(const char *pathname, off_t length);
extern int ftruncate(int fd, off_t length);

/* Memory management */
extern void *sbrk(intptr_t increment);
extern int brk(void *addr);
extern void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
extern void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off64_t offset);
extern int munmap(void *addr, size_t length);
extern int mprotect(void *addr, size_t len, int prot);
extern int msync(void *addr, size_t length, int flags);
extern int madvise(void *addr, size_t len, int advice);
extern int mincore(void *addr, size_t length, unsigned char *vec);
extern void *mremap(void *old_address, size_t old_size, size_t new_size, int flags, ...);
extern int remap_file_pages(void *start, size_t size, int prot, size_t pgoff, int flags);
extern int mlock(const void *addr, size_t len);
extern int munlock(const void *addr, size_t len);
extern int mlockall(int flags);
extern int munlockall(void);
extern void *shmget(key_t key, size_t size, int shmflg);
extern void *shmat(int shmid, const void *shmaddr, int shmflg);
extern int shmdt(const void *shmaddr);
extern int shmctl(int shmid, int cmd, struct shmid_ds *buf);
extern int shm_open(const char *name, int oflag, mode_t mode);
extern int shm_unlink(const char *name);

/* Synchronization */
extern int lockf(int fd, int cmd, off_t len);
extern int flock(int fd, int operation);

/* I/O control */
extern int ioctl(int fd, unsigned long request, ...);
extern int fcntl(int fd, int cmd, ...);

/* File status */
extern int fstat(int fd, struct stat *buf);
extern int lstat(const char *pathname, struct stat *buf);
extern int stat(const char *pathname, struct stat *buf);
extern int newfstatat(int dirfd, const char *pathname, struct stat *buf, int flags);
extern int fstatat64(int dirfd, const char *pathname, struct stat64 *buf, int flags);

/* Signal sending */
extern int kill(pid_t pid, int sig);
extern int killpg(int pgrp, int sig);
extern int raise(int sig);
extern int sigqueue(pid_t pid, int sig, const union sigval value);

/* Time operations */
extern time_t time(time_t *tloc);
extern int gettimeofday(struct timeval *tv, struct timezone *tz);
extern int settimeofday(const struct timeval *tv, const struct timezone *tz);
extern int stime(const time_t *t);
extern int clock_gettime(clockid_t clk_id, struct timespec *tp);
extern int clock_settime(clockid_t clk_id, const struct timespec *tp);
extern int clock_getres(clockid_t clk_id, struct timespec *tp);
extern int clock_nanosleep(clockid_t clock_id, int flags, const struct timespec *rqtp, struct timespec *rmtp);
extern int nanosleep(const struct timespec *rqtp, struct timespec *rmtp);
extern unsigned int alarm(unsigned int seconds);
extern unsigned int ualarm(unsigned int value, unsigned int interval);
extern int setitimer(int which, const struct itimerval *new_value, struct itimerval *old_value);
extern int getitimer(int which, struct itimerval *value);
extern struct tm *gmtime(const time_t *timep);
extern struct tm *localtime(const time_t *timep);
extern struct tm *gmtime_r(const time_t *timep, struct tm *result);
extern struct tm *localtime_r(const time_t *timep, struct tm *result);
extern char *asctime(const struct tm *tm);
extern char *ctime(const time_t *timep);
extern char *asctime_r(const struct tm *tm, char *buf);
extern char *ctime_r(const time_t *timep, char *buf);
extern size_t strftime(char *s, size_t max, const char *format, const struct tm *tm);
extern char *strptime(const char *s, const char *format, struct tm *tm);
extern time_t mktime(struct tm *tm);

/* Sleep functions */
extern unsigned int sleep(unsigned int seconds);
extern int usleep(useconds_t usec);
extern int pause(void);

/* System information */
extern int uname(struct utsname *buf);
extern long sysconf(int name);
extern long pathconf(const char *pathname, int name);
extern long fpathconf(int fd, int name);

/* Resource usage */
extern int getrusage(int who, struct rusage *usage);
extern int vlimit(int resource, int value);
extern int vlimit(int resource, int value);

/* Resource limits */
extern int getrlimit(int resource, struct rlimit *rlim);
extern int setrlimit(int resource, const struct rlimit *rlim);
extern int prlimit(pid_t pid, int resource, const struct rlimit *new_limit, struct rlimit *old_limit);
extern int prlimit64(pid_t pid, int resource, const struct rlimit64 *new_limit, struct rlimit64 *old_limit);

/* Password and group database access */
extern char *getlogin(void);
extern int getlogin_r(char *buf, size_t bufsize);
extern char *getlogin_r(char *buf, size_t bufsize);
extern int setlogin(const char *name);
extern struct passwd *getpwnam(const char *name);
extern struct passwd *getpwuid(uid_t uid);
extern int getpwnam_r(const char *name, struct passwd *pwd, char *buf, size_t buflen, struct passwd **result);
extern int getpwuid_r(uid_t uid, struct passwd *pwd, char *buf, size_t buflen, struct passwd **result);
extern struct group *getgrnam(const char *name);
extern struct group *getgrgid(gid_t gid);
extern int getgrnam_r(const char *name, struct group *grp, char *buf, size_t buflen, struct group **result);
extern int getgrgid_r(gid_t gid, struct group *grp, char *buf, size_t buflen, struct group **result);
extern int setpwent(void);
extern struct passwd *getpwent(void);
extern void endpwent(void);
extern int setgrent(void);
extern struct group *getgrent(void);
extern void endgrent(void);

/* Terminal identification */
extern char *ttyname(int fd);
extern int ttyname_r(int fd, char *buf, size_t buflen);
extern int isatty(int fd);

/* Mode and permission checking */
extern int access(const char *pathname, int mode);
extern int faccessat(int dirfd, const char *pathname, int mode, int flags);

/* Constants */
#define _POSIX_VERSION        200809L
#define _POSIX2_VERSION       200809L
#define _XOPEN_VERSION        700
#define _XOPEN_UNIX           1
#define _POSIX_C_SOURCE       200809L

/* Path and name limits */
#define _POSIX_PATH_MAX       255
#define _POSIX_NAME_MAX       255
#define PATH_MAX              4096
#define NAME_MAX              255

/* General limits */
#define _POSIX_CHILD_MAX      25
#define _POSIX_LINK_MAX       8
#define _POSIX_MAX_CANON      255
#define _POSIX_MAX_INPUT      255
#define _POSIX_NGROUPS_MAX    8
#define _POSIX_OPEN_MAX       20
#define _POSIX_PIPE_BUF       512
#define _POSIX_RE_DUP_MAX     255
#define _POSIX_STREAM_MAX     8
#define _POSIX_TZNAME_MAX     6

/* Job control */
#define _POSIX_JOB_CONTROL    1
#define _POSIX_SAVED_IDS      1

/* Child process limits */
#define CHILD_MAX             _POSIX_CHILD_MAX

/* Open file limits */
#define OPEN_MAX              _POSIX_OPEN_MAX

/* Number of groups */
#define NGROUPS_MAX           _POSIX_NGROUPS_MAX

/* File system limits */
#define LINK_MAX              _POSIX_LINK_MAX
#define PIPE_BUF              _POSIX_PIPE_BUF
#define STREAM_MAX            _POSIX_STREAM_MAX

/* System name length limits */
#define TZNAME_MAX            _POSIX_TZNAME_MAX
#define _POSIX_TZNAME_MAX     _POSIX_TZNAME_MAX

/* Timer resolution limits */
#define _POSIX_TIMER_MAX      32
#define _POSIX_DELAYTIMER_MAX 32
#define _POSIX_AIO_LISTIO_MAX 2
#define _POSIX_AIO_MAX        1

/* Error handling */
extern const char *const sys_errlist[];
extern int sys_nerr;
extern int errno;

/* Process termination */
extern void abort(void);
extern int at_quick_exit(void (*function)(void));
extern void quick_exit(int status);

#endif /* MULTIOS_UNISTD_H */
