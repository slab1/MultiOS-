/*
 * POSIX Threads API
 * MultiOS POSIX Compatibility Layer
 */

#ifndef _PTHREAD_H
#define _PTHREAD_H

#include <sys/types.h>

/* Thread attributes */
typedef struct {
    int __flags;
    size_t __stacksize;
    void *__stackaddr;
    int __detachstate;
    int __schedpolicy;
    struct sched_param __schedparam;
    int __inheritsched;
    int __scope;
} pthread_attr_t;

/* Mutex attributes */
typedef struct {
    int __type;
    int __pshared;
} pthread_mutexattr_t;

/* Condition variable attributes */
typedef struct {
    int __pshared;
    clockid_t __clock;
} pthread_condattr_t;

/* Read-write lock attributes */
typedef struct {
    int __pshared;
} pthread_rwlockattr_t;

/* Barrier attributes */
typedef struct {
    int __pshared;
    unsigned int __count;
} pthread_barrierattr_t;

/* Spin lock attributes */
typedef struct {
    int __pshared;
} pthread_spinlockattr_t;

/* Mutex types */
#define PTHREAD_MUTEX_NORMAL      0
#define PTHREAD_MUTEX_RECURSIVE   1
#define PTHREAD_MUTEX_ERRORCHECK  2
#define PTHREAD_MUTEX_DEFAULT     PTHREAD_MUTEX_NORMAL

/* Mutex protocol */
#define PTHREAD_PRIO_NONE         0
#define PTHREAD_PRIO_INHERIT      1
#define PTHREAD_PRIO_PROTECT      2

/* Mutex robust */
#define PTHREAD_MUTEX_STALLED     0
#define PTHREAD_MUTEX_ROBUST      1

/* Thread scheduling policies */
#define SCHED_OTHER     0  /* Normal, non-real-time scheduling */
#define SCHED_FIFO      1  /* First-in, first-out scheduling */
#define SCHED_RR        2  /* Round-robin scheduling */

/* Thread state values */
#define PTHREAD_CREATE_JOINABLE 0  /* Thread can be joined */
#define PTHREAD_CREATE_DETACHED 1  /* Thread is detached */

/* Process shared synchronization */
#define PTHREAD_PROCESS_PRIVATE 0  /* Synchronization within process */
#define PTHREAD_PROCESS_SHARED  1  /* Synchronization across processes */

/* Thread priority bounds */
#define PTHREAD_PRIORITY_SCHEDULING_MIN  1
#define PTHREAD_PRIORITY_SCHEDULING_MAX  99

/* Initialization */
#define PTHREAD_ONCE_INIT { 0 }

#define PTHREAD_COND_INITIALIZER { { { 0, 0, 0 } } }
#define PTHREAD_MUTEX_INITIALIZER { { 0, 0, 0 } }
#define PTHREAD_RWLOCK_INITIALIZER { { 0, 0, 0 } }

/* Function return codes */
#define PTHREAD_SUCCESS         0
#define PTHREAD_ERROR_BASE     0
#define PTHREAD_BUSY           (PTHREAD_ERROR_BASE + 1)
#define PTHREAD_INVAL          (PTHREAD_ERROR_BASE + 2)
#define PTHREAD_AGAIN          (PTHREAD_ERROR_BASE + 3)
#define PTHREAD_NOMEM          (PTHREAD_ERROR_BASE + 4)
#define PTHREAD_ACCES          (PTHREAD_ERROR_BASE + 5)
#define PTHREAD_FAULT          (PTHREAD_ERROR_BASE + 6)

/* One-time initialization structure */
typedef struct {
    int __done;
    void (*__func)(void);
} pthread_once_t;

/* Mutex structure */
typedef struct {
    int __type;
    int __protocol;
    int __robust;
    int __pshared;
    void *__owner;
    int __lock;
} pthread_mutex_t;

/* Condition variable structure */
typedef struct {
    int __pshared;
    clockid_t __clock;
    int __lock;
    int __futex;
} pthread_cond_t;

/* Read-write lock structure */
typedef struct {
    int __lock;
    unsigned __readers;
    pthread_mutex_t __writelock;
    int __nr_readers;
    pthread_cond_t __readers;
    pthread_cond_t __writers;
    int __writer;
    int __shared;
} pthread_rwlock_t;

/* Barrier structure */
typedef struct {
    int __pshared;
    unsigned int __count;
    unsigned int __ceiling;
    pthread_mutex_t __lock;
    pthread_cond_t __cond;
} pthread_barrier_t;

/* Spin lock structure */
typedef struct {
    int __pshared;
    volatile unsigned __lock;
} pthread_spinlock_t;

/* Thread-local storage key */
typedef unsigned long pthread_key_t;

/* Thread identifier */
typedef unsigned long pthread_t;

/* Scheduling parameters */
struct sched_param {
    int sched_priority;  /* Scheduling priority */
};

/* Thread function pointer */
typedef void *(*pthread_start_routine)(void *);

/* Function declarations */

/* Thread management */
int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   pthread_start_routine start_routine, void *arg);
int pthread_join(pthread_t thread, void **value_ptr);
int pthread_detach(pthread_t thread);
int pthread_cancel(pthread_t thread);
int pthread_setcancelstate(int state, int *oldstate);
int pthread_setcanceltype(int type, int *oldtype);
void pthread_testcancel(void);
void pthread_exit(void *value_ptr);
pthread_t pthread_self(void);

/* Thread attributes */
int pthread_attr_init(pthread_attr_t *attr);
int pthread_attr_destroy(pthread_attr_t *attr);
int pthread_attr_setdetachstate(pthread_attr_t *attr, int detachstate);
int pthread_attr_getdetachstate(const pthread_attr_t *attr, int *detachstate);
int pthread_attr_setschedparam(pthread_attr_t *attr, const struct sched_param *param);
int pthread_attr_getschedparam(const pthread_attr_t *attr, struct sched_param *param);
int pthread_attr_setschedpolicy(pthread_attr_t *attr, int policy);
int pthread_attr_getschedpolicy(const pthread_attr_t *attr, int *policy);
int pthread_attr_setinheritsched(pthread_attr_t *attr, int inherit);
int pthread_attr_getinheritsched(const pthread_attr_t *attr, int *inherit);
int pthread_attr_setscope(pthread_attr_t *attr, int scope);
int pthread_attr_getscope(const pthread_attr_t *attr, int *scope);
int pthread_attr_setstacksize(pthread_attr_t *attr, size_t stacksize);
int pthread_attr_getstacksize(const pthread_attr_t *attr, size_t *stacksize);
int pthread_attr_setstackaddr(pthread_attr_t *attr, void *stackaddr);
int pthread_attr_getstackaddr(const pthread_attr_t *attr, void **stackaddr);

/* Scheduling */
int pthread_setschedparam(pthread_t thread, int policy,
                          const struct sched_param *param);
int pthread_getschedparam(pthread_t thread, int *policy,
                          struct sched_param *param);
int pthread_setschedprio(pthread_t thread, int prio);
int pthread_getschedprio(pthread_t thread);

/* Mutex operations */
int pthread_mutex_init(pthread_mutex_t *mutex, const pthread_mutexattr_t *attr);
int pthread_mutex_destroy(pthread_mutex_t *mutex);
int pthread_mutex_lock(pthread_mutex_t *mutex);
int pthread_mutex_trylock(pthread_mutex_t *mutex);
int pthread_mutex_timedlock(pthread_mutex_t *mutex,
                           const struct timespec *abstime);
int pthread_mutex_unlock(pthread_mutex_t *mutex);

/* Mutex attributes */
int pthread_mutexattr_init(pthread_mutexattr_t *attr);
int pthread_mutexattr_destroy(pthread_mutexattr_t *attr);
int pthread_mutexattr_setpshared(pthread_mutexattr_t *attr, int pshared);
int pthread_mutexattr_getpshared(const pthread_mutexattr_t *attr, int *pshared);
int pthread_mutexattr_settype(pthread_mutexattr_t *attr, int type);
int pthread_mutexattr_gettype(const pthread_mutexattr_t *attr, int *type);
int pthread_mutexattr_setprotocol(pthread_mutexattr_t *attr, int protocol);
int pthread_mutexattr_getprotocol(const pthread_mutexattr_t *attr, int *protocol);
int pthread_mutexattr_setrobust(pthread_mutexattr_t *attr, int robust);
int pthread_mutexattr_getrobust(const pthread_mutexattr_t *attr, int *robust);

/* Condition variables */
int pthread_cond_init(pthread_cond_t *cond, const pthread_condattr_t *attr);
int pthread_cond_destroy(pthread_cond_t *cond);
int pthread_cond_wait(pthread_cond_t *cond, pthread_mutex_t *mutex);
int pthread_cond_timedwait(pthread_cond_t *cond, pthread_mutex_t *mutex,
                          const struct timespec *abstime);
int pthread_cond_signal(pthread_cond_t *cond);
int pthread_cond_broadcast(pthread_cond_t *cond);

/* Condition variable attributes */
int pthread_condattr_init(pthread_condattr_t *attr);
int pthread_condattr_destroy(pthread_condattr_t *attr);
int pthread_condattr_setpshared(pthread_condattr_t *attr, int pshared);
int pthread_condattr_getpshared(const pthread_condattr_t *attr, int *pshared);
int pthread_condattr_setclock(pthread_condattr_t *attr, clockid_t clock_id);
int pthread_condattr_getclock(const pthread_condattr_t *attr, clockid_t *clock_id);

/* Read-write locks */
int pthread_rwlock_init(pthread_rwlock_t *rwlock,
                       const pthread_rwlockattr_t *attr);
int pthread_rwlock_destroy(pthread_rwlock_t *rwlock);
int pthread_rwlock_rdlock(pthread_rwlock_t *rwlock);
int pthread_rwlock_tryrdlock(pthread_rwlock_t *rwlock);
int pthread_rwlock_timedrdlock(pthread_rwlock_t *rwlock,
                              const struct timespec *abstime);
int pthread_rwlock_wrlock(pthread_rwlock_t *rwlock);
int pthread_rwlock_trywrlock(pthread_rwlock_t *rwlock);
int pthread_rwlock_timedwrlock(pthread_rwlock_t *rwlock,
                              const struct timespec *abstime);
int pthread_rwlock_unlock(pthread_rwlock_t *rwlock);

/* Read-write lock attributes */
int pthread_rwlockattr_init(pthread_rwlockattr_t *attr);
int pthread_rwlockattr_destroy(pthread_rwlockattr_t *attr);
int pthread_rwlockattr_setpshared(pthread_rwlockattr_t *attr, int pshared);
int pthread_rwlockattr_getpshared(const pthread_rwlockattr_t *attr,
                                 int *pshared);

/* Barriers */
int pthread_barrier_init(pthread_barrier_t *barrier,
                        const pthread_barrierattr_t *attr,
                        unsigned int count);
int pthread_barrier_destroy(pthread_barrier_t *barrier);
int pthread_barrier_wait(pthread_barrier_t *barrier);

/* Barrier attributes */
int pthread_barrierattr_init(pthread_barrierattr_t *attr);
int pthread_barrierattr_destroy(pthread_barrierattr_t *attr);
int pthread_barrierattr_setpshared(pthread_barrierattr_t *attr, int pshared);
int pthread_barrierattr_getpshared(const pthread_barrierattr_t *attr,
                                  int *pshared);

/* Spin locks */
int pthread_spin_init(pthread_spinlock_t *lock, int pshared);
int pthread_spin_destroy(pthread_spinlock_t *lock);
int pthread_spin_lock(pthread_spinlock_t *lock);
int pthread_spin_trylock(pthread_spinlock_t *lock);
int pthread_spin_unlock(pthread_spinlock_t *lock);

/* Spin lock attributes */
int pthread_spinlockattr_init(pthread_spinlockattr_t *attr);
int pthread_spinlockattr_destroy(pthread_spinlockattr_t *attr);
int pthread_spinlockattr_setpshared(pthread_spinlockattr_t *attr, int pshared);
int pthread_spinlockattr_getpshared(const pthread_spinlockattr_t *attr,
                                   int *pshared);

/* Thread-local storage */
int pthread_key_create(pthread_key_t *key, void (*destructor)(void *));
int pthread_key_delete(pthread_key_t key);
int pthread_setspecific(pthread_key_t key, const void *value);
void *pthread_getspecific(pthread_key_t key);

/* One-time initialization */
int pthread_once(pthread_once_t *once_control, void (*init_routine)(void));

/* Signal handling */
int pthread_kill(pthread_t thread, int sig);

/* Thread-specific signal mask */
int pthread_sigmask(int how, const sigset_t *set, sigset_t *oldset);

/* Conformance */
int pthread_getconcurrency(void);
int pthread_setconcurrency(int level);

/* Thread priority */
int pthread_getpriority_np(pthread_t thread, int policy, int *prio);
int pthread_setpriority_np(pthread_t thread, int policy, int prio);

/* Robust mutex */
int pthread_mutex_consistent(pthread_mutex_t *mutex);

/* Non-portable extensions */
int pthread_yield(void);

#endif /* _PTHREAD_H */