/*
 * POSIX Signal Handling API
 * MultiOS POSIX Compatibility Layer
 */

#ifndef _SIGNAL_H
#define _SIGNAL_H

#include <stddef.h>

/* Signal numbers */
#define SIGHUP     1   /* Hangup */
#define SIGINT     2   /* Interrupt */
#define SIGQUIT    3   /* Quit */
#define SIGILL     4   /* Illegal Instruction */
#define SIGTRAP    5   /* Trace/Breakpoint Trap */
#define SIGABRT    6   /* Abort */
#define SIGIOT     6   /* IOT Trap (alias for SIGABRT) */
#define SIGBUS     7   /* Bus Error */
#define SIGFPE     8   /* Floating Point Exception */
#define SIGKILL    9   /* Kill Signal (cannot be caught or ignored) */
#define SIGUSR1    10  /* User Signal 1 */
#define SIGSEGV    11  /* Segmentation Fault */
#define SIGUSR2    12  /* User Signal 2 */
#define SIGPIPE    13  /* Broken Pipe */
#define SIGALRM    14  /* Alarm Clock */
#define SIGTERM    15  /* Termination Signal */
#define SIGSTKFLT  16  /* Stack Fault */
#define SIGCHLD    17  /* Child Status Change */
#define SIGCLD     17  /* Child Status Change (alias for SIGCHLD) */
#define SIGCONT    18  /* Continue */
#define SIGSTOP    19  /* Stop (cannot be caught or ignored) */
#define SIGTSTP    20  /* Terminal Stop */
#define SIGTTIN    21  /* Terminal Read from Background Process */
#define SIGTTOU    22  /* Terminal Write from Background Process */
#define SIGURG     23  /* Urgent Condition */
#define SIGXCPU    24  /* CPU Time Limit Exceeded */
#define SIGXFSZ    25  /* File Size Limit Exceeded */
#define SIGVTALRM  26  /* Virtual Timer Expired */
#define SIGPROF    27  /* Profiling Timer Expired */
#define SIGWINCH   28  /* Window Size Changed */
#define SIGIO      29  /* I/O Possible (alias for SIGPOLL) */
#define SIGPOLL    29  /* Pollable Event */
#define SIGPWR     30  /* Power Failure */
#define SIGSYS     31  /* Bad System Call */
#define SIGUNUSED  31  /* Unused Signal */

/* Signal action flags */
#define SA_NOCLDSTOP 0x00000001  /* Do not generate SIGCHLD on child stop */
#define SA_NOCLDWAIT 0x00000002  /* Do not create zombie on child death */
#define SA_SIGINFO   0x00000004  /* Use extended signal handling */
#define SA_ONSTACK   0x08000000  /* Signal handler uses alternate stack */
#define SA_RESTART   0x10000000  /* Restart system call on signal return */
#define SA_NODEFER   0x40000000  /* Do not block signal during handler */
#define SA_RESETHAND 0x80000000  /* Reset to default on entry to handler */

/* Special signal values */
#define SIG_DFL ((void (*)(int))0)    /* Default action */
#define SIG_ERR ((void (*)(int))-1)   /* Error return */
#define SIG_HOLD ((void (*)(int))1)   /* Hold signal */

/* Signal set operations */
typedef unsigned long sigset_t;

/* Signal handler function pointer */
typedef void (*sighandler_t)(int);

/* Signal action structure for sigaction */
struct sigaction {
    void (*sa_handler)(int);
    sigset_t sa_mask;
    int sa_flags;
    void (*sa_sigaction)(int, void *, void *);
};

/* Signal information structure for sigaction */
struct siginfo_t {
    int si_signo;
    int si_errno;
    int si_code;
    pid_t si_pid;
    uid_t si_uid;
    void *si_addr;
    int si_status;
    long si_band;
};

/* Function declarations */

/* Basic signal handling */
sighandler_t signal(int signum, sighandler_t handler);
int sigaction(int signum, const struct sigaction *act, struct sigaction *oldact);
int sigprocmask(int how, const sigset_t *set, sigset_t *oldset);
int sigpending(sigset_t *set);
int sigsuspend(const sigset_t *mask);

/* Signal set operations */
int sigemptyset(sigset_t *set);
int sigfillset(sigset_t *set);
int sigaddset(sigset_t *set, int signum);
int sigdelset(sigset_t *set, int signum);
int sigismember(const sigset_t *set, int signum);

/* Signal manipulation functions */
int kill(pid_t pid, int signum);
int raise(int signum);
int alarm(unsigned int seconds);
int pause(void);
unsigned int sleep(unsigned int seconds);
int sigqueue(pid_t pid, int signum, union sigval value);

/* Advanced signal functions */
int sigaltstack(const void *ss, void *old_ss);
int sigwait(const sigset_t *set, int *sig);
int sigwaitinfo(const sigset_t *set, struct siginfo_t *info);
int sigtimedwait(const sigset_t *set, struct siginfo_t *info, const struct timespec *timeout);

/* Signal value for sigqueue */
union sigval {
    int sival_int;
    void *sival_ptr;
};

/* Additional signal codes for si_code */
#define SI_USER    0    /* Signal sent by user */
#define SI_KERNEL  0x80 /* Signal sent by kernel */

/* si_code values for SIGCHLD */
#define CLD_EXITED     1  /* Child has exited */
#define CLD_KILLED     2  /* Child was killed */
#define CLD_DUMPED     3  /* Child was killed by core dump */
#define CLD_TRAPPED    4  /* Traced child has trapped */
#define CLD_STOPPED    5  /* Child has stopped */
#define CLD_CONTINUED  6  /* Stopped child has continued */

/* Process termination signals */
#define NSIG 32  /* Number of signals */

#endif /* _SIGNAL_H */