/**
 * Scenario Library Module
 * Manages educational debugging scenarios and problem sets
 */

class ScenarioLibrary {
    constructor() {
        this.app = null;
        this.scenarios = new Map();
        this.activeScenario = null;
        this.scenarioStates = new Map();
        this.userProgress = new Map();
        
        // Scenario categories
        this.categories = {
            'beginner': 'Beginner',
            'intermediate': 'Intermediate',
            'advanced': 'Advanced',
            'specialized': 'Specialized'
        };
        
        // Initialize built-in scenarios
        this.initializeScenarios();
    }

    setApp(app) {
        this.app = app;
    }

    initializeScenarios() {
        // Process Hang Investigation Scenario
        this.addScenario({
            id: 'process-hang',
            name: 'Process Hang Investigation',
            category: 'beginner',
            description: 'A real-world scenario where a program appears to hang and stop responding',
            difficulty: 1,
            estimatedTime: 15,
            tags: ['hang', 'process', 'debugging', 'hang'],
            learningObjectives: [
                'Identify hanging processes',
                'Use system monitoring tools',
                'Analyze process behavior',
                'Implement timeout mechanisms'
            ],
            prerequisites: [],
            files: [
                {
                    path: '/examples/hang_program.c',
                    content: this.getHangProgramSource(),
                    language: 'c'
                }
            ],
            setup: {
                commands: [
                    'gcc -o hang_program hang_program.c',
                    './hang_program &'
                ],
                environment: {
                    'HANG_TIMEOUT': '5'
                }
            },
            problems: [
                {
                    id: 'identify-hang',
                    title: 'Identify the Hanging Process',
                    description: 'The program appears to be hung. First, identify which process is not responding.',
                    hint: 'Use ps aux or top to find processes with unusual behavior',
                    solution: 'Find the process with high CPU usage or in uninterruptible sleep (D state)',
                    expectedOutput: 'Process PID and status information',
                    validation: {
                        type: 'command',
                        command: 'ps aux | grep hang_program',
                        expectedPattern: 'hang_program.*<defunct|hang_program.*R.*[0-9]{2,}%'
                    }
                },
                {
                    id: 'analyze-stack',
                    title: 'Analyze Call Stack',
                    description: 'Examine the call stack to understand where the process is stuck',
                    hint: 'Use gdb to attach and examine thread stacks',
                    solution: 'Find the function call where the process is waiting',
                    validation: {
                        type: 'gdb',
                        commands: ['attach <PID>', 'thread apply all bt'],
                        expectedPattern: 'main.*hang_program.c.*sleep'
                    }
                },
                {
                    id: 'fix-timeout',
                    title: 'Implement Timeout',
                    description: 'Add a timeout mechanism to prevent the hang',
                    hint: 'Use alarm() or setitimer() for timeout',
                    solution: 'Implement timeout using alarm(5) to prevent indefinite hanging',
                    validation: {
                        type: 'code',
                        checkFunction: 'timeoutImplements'
                    }
                }
            ],
            steps: [
                {
                    id: 1,
                    title: 'Initial Assessment',
                    description: 'Identify that the program is hung',
                    actions: ['terminal: ps aux | grep hang_program'],
                    hints: [
                        'Look for processes with high CPU usage or in sleep state',
                        'The program should be running but not producing output'
                    ]
                },
                {
                    id: 2,
                    title: 'Attach Debugger',
                    description: 'Attach GDB to the hanging process',
                    actions: ['gdb: attach <PID>'],
                    hints: [
                        'Use the PID from the previous step',
                        'Attach to examine the process state'
                    ]
                },
                {
                    id: 3,
                    title: 'Analyze Call Stack',
                    description: 'Examine where the process is stuck',
                    actions: ['gdb: thread apply all bt', 'gdb: info threads'],
                    hints: [
                        'Look for system calls like sleep() or wait()',
                        'Thread information shows which threads are active'
                    ]
                },
                {
                    id: 4,
                    title: 'Fix Implementation',
                    description: 'Implement timeout mechanism',
                    actions: ['code: add timeout using alarm()'],
                    hints: [
                        'Use alarm() to set a timeout',
                        'Handle the SIGALRM signal appropriately'
                    ]
                },
                {
                    id: 5,
                    title: 'Verification',
                    description: 'Test that the fix works',
                    actions: ['terminal: ./fixed_program'],
                    hints: [
                        'Program should complete within the timeout period',
                        'No hanging behavior should be observed'
                    ]
                }
            ]
        });

        // Memory Leak Detection Scenario
        this.addScenario({
            id: 'memory-leak',
            name: 'Memory Leak Detection',
            category: 'intermediate',
            description: 'Detect and fix memory leaks in a C program using Valgrind and manual analysis',
            difficulty: 2,
            estimatedTime: 20,
            tags: ['memory', 'leak', 'valgrind', 'c'],
            learningObjectives: [
                'Use Valgrind for memory leak detection',
                'Identify common memory leak patterns',
                'Implement proper memory management',
                'Use memory debugging tools'
            ],
            prerequisites: ['process-hang'],
            files: [
                {
                    path: '/examples/memory_leak.c',
                    content: this.getMemoryLeakSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'detect-leaks',
                    title: 'Detect Memory Leaks',
                    description: 'Run Valgrind to identify memory leaks',
                    hint: 'Use valgrind --leak-check=full to get detailed leak information',
                    solution: 'Identify the source and location of memory leaks',
                    validation: {
                        type: 'valgrind',
                        command: 'valgrind --leak-check=full ./memory_leak_program',
                        expectedPattern: 'definitely lost.*bytes in.*blocks'
                    }
                },
                {
                    id: 'analyze-leak',
                    title: 'Analyze Leak Sources',
                    description: 'Understand where and why memory is being leaked',
                    hint: 'Examine the stack traces in the Valgrind output',
                    solution: 'Identify malloc without corresponding free() calls',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'leakAnalysis'
                    }
                },
                {
                    id: 'fix-leaks',
                    title: 'Fix Memory Leaks',
                    description: 'Implement proper memory management',
                    hint: 'Ensure every malloc has a corresponding free()',
                    solution: 'Add free() calls for all dynamically allocated memory',
                    validation: {
                        type: 'valgrind',
                        command: 'valgrind --leak-check=full ./fixed_program',
                        expectedPattern: 'All heap blocks were freed -- no leaks are possible'
                    }
                }
            ]
        });

        // Deadlock Scenario
        this.addScenario({
            id: 'deadlock',
            name: 'Deadlock Analysis',
            category: 'advanced',
            description: 'Debug and resolve deadlock situations in multi-threaded programs',
            difficulty: 3,
            estimatedTime: 25,
            tags: ['deadlock', 'threads', 'mutex', 'race-condition'],
            learningObjectives: [
                'Identify deadlock conditions',
                'Analyze thread states and locks',
                'Implement deadlock prevention strategies',
                'Use thread debugging tools'
            ],
            prerequisites: ['memory-leak'],
            files: [
                {
                    path: '/examples/deadlock_program.c',
                    content: this.getDeadlockSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'reproduce-deadlock',
                    title: 'Reproduce Deadlock',
                    description: 'Run the program and observe deadlock behavior',
                    hint: 'The program will hang when threads deadlock',
                    solution: 'Confirm the program hangs due to circular wait',
                    validation: {
                        type: 'behavior',
                        check: 'programHangs',
                        timeout: 10
                    }
                },
                {
                    id: 'analyze-threads',
                    title: 'Analyze Thread States',
                    description: 'Use GDB to examine thread states and lock acquisition',
                    hint: 'Use info threads and thread apply all bt',
                    solution: 'Identify threads waiting for locks held by other threads',
                    validation: {
                        type: 'gdb',
                        commands: ['info threads', 'thread apply all bt'],
                        expectedPattern: 'pthread_mutex_lock.*resource.*deadlock_program.c'
                    }
                },
                {
                    id: 'identify-cause',
                    title: 'Identify Deadlock Cause',
                    description: 'Analyze backtraces to find the lock ordering issue',
                    hint: 'Look for threads acquiring locks in different orders',
                    solution: 'Find circular dependency between resources',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'deadlockAnalysis'
                    }
                },
                {
                    id: 'fix-deadlock',
                    title: 'Implement Deadlock Fix',
                    description: 'Fix the lock ordering or add timeout mechanisms',
                    hint: 'Ensure consistent lock acquisition order or use timeouts',
                    solution: 'Implement consistent lock ordering or pthread_mutex_timedlock',
                    validation: {
                        type: 'behavior',
                        check: 'programCompletes',
                        timeout: 5
                    }
                }
            ]
        });

        // Race Condition Scenario
        this.addScenario({
            id: 'race-condition',
            name: 'Race Condition Debugging',
            category: 'advanced',
            description: 'Debug race conditions in concurrent programs',
            difficulty: 3,
            estimatedTime: 30,
            tags: ['race-condition', 'concurrency', 'threads', 'synchronization'],
            learningObjectives: [
                'Identify race conditions',
                'Use thread debugging tools',
                'Implement proper synchronization',
                'Test concurrent programs'
            ],
            prerequisites: ['deadlock'],
            files: [
                {
                    path: '/examples/race_condition.c',
                    content: this.getRaceConditionSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'reproduce-race',
                    title: 'Reproduce Race Condition',
                    description: 'Run the program multiple times to observe inconsistent results',
                    hint: 'Race conditions may not occur every time - try multiple runs',
                    solution: 'Observe inconsistent output across multiple runs',
                    validation: {
                        type: 'multiple-runs',
                        command: './race_program',
                        runs: 5,
                        check: 'inconsistentResults'
                    }
                },
                {
                    id: 'analyze-thread-interleaving',
                    title: 'Analyze Thread Interleaving',
                    description: 'Use GDB to examine thread scheduling and shared data access',
                    hint: 'Set breakpoints and observe thread switching',
                    solution: 'Identify the specific interleaving that causes problems',
                    validation: {
                        type: 'gdb',
                        commands: ['break shared_variable_update', 'run', 'continue'],
                        expectedPattern: 'Thread.*shared_variable_update.*deadlock_program.c'
                    }
                },
                {
                    id: 'identify-critical-section',
                    title: 'Identify Critical Section',
                    description: 'Find the code section that needs protection',
                    hint: 'Look for accesses to shared variables without synchronization',
                    solution: 'Identify the critical section in shared_variable_update',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'criticalSectionAnalysis'
                    }
                },
                {
                    id: 'implement-synchronization',
                    title: 'Implement Synchronization',
                    description: 'Add proper mutex or other synchronization mechanisms',
                    hint: 'Protect the critical section with pthread_mutex_lock/unlock',
                    solution: 'Add mutex protection around shared variable accesses',
                    validation: {
                        type: 'multiple-runs',
                        command: './fixed_program',
                        runs: 10,
                        check: 'consistentResults'
                    }
                }
            ]
        });

        // Kernel Panic Analysis Scenario
        this.addScenario({
            id: 'kernel-panic',
            name: 'Kernel Panic Analysis',
            category: 'advanced',
            description: 'Debug kernel panics using crash dumps and kernel debugging tools',
            difficulty: 4,
            estimatedTime: 35,
            tags: ['kernel', 'panic', 'crash-dump', 'system'],
            learningObjectives: [
                'Analyze kernel crash dumps',
                'Identify panic causes',
                'Use crash utility for debugging',
                'Understand kernel debugging'
            ],
            prerequisites: ['race-condition'],
            files: [
                {
                    path: '/examples/kernel_panic.txt',
                    content: this.getKernelPanicDump(),
                    language: 'text'
                }
            ],
            problems: [
                {
                    id: 'examine-panic-dump',
                    title: 'Examine Panic Dump',
                    description: 'Analyze the kernel panic dump to understand what happened',
                    hint: 'Look at the panic message and stack trace',
                    solution: 'Identify the function that caused the panic',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'panicDumpAnalysis'
                    }
                },
                {
                    id: 'use-crash-utility',
                    title: 'Use Crash Utility',
                    description: 'Use the crash utility to analyze the kernel dump',
                    hint: 'crash vmcore vmlinux for analyzing kernel dumps',
                    solution: 'Extract detailed information about the panic',
                    validation: {
                        type: 'crash-utility',
                        command: 'crash vmcore vmlinux',
                        expectedPattern: 'PID.*CPU.*TASK.*STATE'
                    }
                },
                {
                    id: 'analyze-call-stack',
                    title: 'Analyze Call Stack',
                    description: 'Examine the kernel call stack at panic time',
                    hint: 'Use bt command in crash utility',
                    solution: 'Identify the sequence of function calls leading to panic',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'callStackAnalysis'
                    }
                },
                {
                    id: 'identify-root-cause',
                    title: 'Identify Root Cause',
                    description: 'Determine the root cause of the kernel panic',
                    hint: 'Look for NULL pointer dereferences, invalid memory access, etc.',
                    solution: 'Identify the specific bug that caused the panic',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'rootCauseAnalysis'
                    }
                }
            ]
        });

        // System Call Tracing Scenario
        this.addScenario({
            id: 'system-call',
            name: 'System Call Tracing',
            category: 'intermediate',
            description: 'Trace and analyze system calls to understand program behavior',
            difficulty: 2,
            estimatedTime: 20,
            tags: ['syscall', 'strace', 'tracing', 'system'],
            learningObjectives: [
                'Use strace to trace system calls',
                'Analyze system call patterns',
                'Identify performance issues',
                'Debug system-level problems'
            ],
            prerequisites: ['process-hang'],
            files: [
                {
                    path: '/examples/syscall_test.c',
                    content: this.getSyscallTestSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'basic-tracing',
                    title: 'Basic System Call Tracing',
                    description: 'Use strace to trace system calls made by the program',
                    hint: 'strace ./program to see all system calls',
                    solution: 'Generate a trace of system calls',
                    validation: {
                        type: 'strace',
                        command: 'strace ./syscall_test',
                        expectedPattern: 'execve.*syscall_test'
                    }
                },
                {
                    id: 'analyze-performance',
                    title: 'Analyze Performance Issues',
                    description: 'Identify slow system calls or excessive call patterns',
                    hint: 'Look for repeated calls or long-duration system calls',
                    solution: 'Find performance bottlenecks in system call usage',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'performanceAnalysis'
                    }
                },
                {
                    id: 'debug-syscall-error',
                    title: 'Debug System Call Errors',
                    description: 'Identify and fix system call errors',
                    hint: 'Look for failed system calls in the trace',
                    solution: 'Fix system calls that return errors',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'errorAnalysis'
                    }
                }
            ]
        });

        // Thread Synchronization Scenario
        this.addScenario({
            id: 'thread-sync',
            name: 'Thread Synchronization',
            category: 'intermediate',
            description: 'Debug and fix thread synchronization issues',
            difficulty: 3,
            estimatedTime: 25,
            tags: ['threads', 'synchronization', 'mutex', 'condition-variables'],
            learningObjectives: [
                'Debug thread synchronization issues',
                'Use thread debugging tools',
                'Implement proper synchronization primitives',
                'Test multi-threaded programs'
            ],
            prerequisites: ['memory-leak'],
            files: [
                {
                    path: '/examples/thread_sync.c',
                    content: this.getThreadSyncSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'identify-sync-issue',
                    title: 'Identify Synchronization Issue',
                    description: 'Run the program and identify synchronization problems',
                    hint: 'Look for race conditions or deadlocks',
                    solution: 'Identify the specific synchronization issue',
                    validation: {
                        type: 'behavior',
                        check: 'syncIssueDetected'
                    }
                },
                {
                    id: 'analyze-thread-interaction',
                    title: 'Analyze Thread Interaction',
                    description: 'Use GDB to examine thread interactions',
                    hint: 'Examine thread states and lock usage',
                    solution: 'Understand how threads interact and where problems occur',
                    validation: {
                        type: 'gdb',
                        commands: ['info threads', 'thread apply all bt'],
                        expectedPattern: 'pthread_cond_wait.*pthread_mutex_lock'
                    }
                },
                {
                    id: 'implement-proper-sync',
                    title: 'Implement Proper Synchronization',
                    description: 'Fix the synchronization using mutexes and condition variables',
                    hint: 'Use pthread_mutex_lock/unlock and pthread_cond_wait/signal',
                    solution: 'Implement correct synchronization mechanism',
                    validation: {
                        type: 'behavior',
                        check: 'programWorksCorrectly'
                    }
                }
            ]
        });

        // Disk I/O Performance Scenario
        this.addScenario({
            id: 'disk-io',
            name: 'Disk I/O Performance Debugging',
            category: 'specialized',
            description: 'Debug and optimize disk I/O performance issues',
            difficulty: 3,
            estimatedTime: 30,
            tags: ['disk', 'i/o', 'performance', 'optimization'],
            learningObjectives: [
                'Analyze disk I/O patterns',
                'Use I/O profiling tools',
                'Optimize disk performance',
                'Understand filesystem behavior'
            ],
            prerequisites: ['system-call'],
            files: [
                {
                    path: '/examples/disk_io_test.c',
                    content: this.getDiskIOSource(),
                    language: 'c'
                }
            ],
            problems: [
                {
                    id: 'measure-io-performance',
                    title: 'Measure I/O Performance',
                    description: 'Use tools to measure current disk I/O performance',
                    hint: 'Use iostat, iotop, or similar tools',
                    solution: 'Get baseline performance metrics',
                    validation: {
                        type: 'measurement',
                        check: 'performanceMeasured'
                    }
                },
                {
                    id: 'analyze-io-patterns',
                    title: 'Analyze I/O Patterns',
                    description: 'Examine the program's I/O patterns and identify issues',
                    hint: 'Look for excessive small I/O operations or synchronous I/O',
                    solution: 'Identify inefficient I/O patterns',
                    validation: {
                        type: 'analysis',
                        checkFunction: 'ioPatternAnalysis'
                    }
                },
                {
                    id: 'optimize-io',
                    title: 'Optimize I/O Operations',
                    description: 'Implement I/O optimizations',
                    hint: 'Use buffered I/O, larger block sizes, or async I/O',
                    solution: 'Implement efficient I/O operations',
                    validation: {
                        type: 'performance',
                        check: 'performanceImproved'
                    }
                }
            ]
        });
    }

    addScenario(scenario) {
        this.scenarios.set(scenario.id, scenario);
    }

    async loadScenario(scenarioId) {
        const scenario = this.scenarios.get(scenarioId);
        if (!scenario) {
            throw new Error(`Scenario ${scenarioId} not found`);
        }

        try {
            // Initialize scenario
            const scenarioInstance = new ScenarioInstance(scenario, this);
            await scenarioInstance.initialize();
            
            this.activeScenario = scenarioInstance;
            return scenarioInstance;
        } catch (error) {
            console.error(`Failed to load scenario ${scenarioId}:`, error);
            throw error;
        }
    }

    getScenario(scenarioId) {
        return this.scenarios.get(scenarioId);
    }

    getAllScenarios() {
        return Array.from(this.scenarios.values());
    }

    getScenariosByCategory(category) {
        return this.getAllScenarios().filter(scenario => 
            scenario.category === category
        );
    }

    getScenariosByDifficulty(difficulty) {
        return this.getAllScenarios().filter(scenario => 
            scenario.difficulty === difficulty
        );
    }

    searchScenarios(query) {
        const lowercaseQuery = query.toLowerCase();
        return this.getAllScenarios().filter(scenario =>
            scenario.name.toLowerCase().includes(lowercaseQuery) ||
            scenario.description.toLowerCase().includes(lowercaseQuery) ||
            scenario.tags.some(tag => tag.toLowerCase().includes(lowercaseQuery))
        );
    }

    // Mock file content generators
    getHangProgramSource() {
        return `#include <stdio.h>
#include <unistd.h>
#include <signal.h>

void timeout_handler(int sig) {
    printf("Timeout occurred!\\n");
    exit(1);
}

int main() {
    // Set up timeout
    signal(SIGALRM, timeout_handler);
    alarm(3); // 3 second timeout
    
    printf("Starting program...\\n");
    
    // This will cause the program to hang without timeout
    printf("About to hang...\\n");
    while(1) {
        // Infinite loop - simulates hanging
        sleep(1);
    }
    
    printf("This should never be reached\\n");
    return 0;
}`;
    }

    getMemoryLeakSource() {
        return `#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void allocate_memory() {
    // Memory leak - malloc without corresponding free
    char *buffer = malloc(1024);
    strcpy(buffer, "This memory will be leaked");
    printf("Allocated memory: %s\\n", buffer);
    // Missing: free(buffer);
}

int main() {
    printf("Memory leak demo\\n");
    
    for (int i = 0; i < 100; i++) {
        allocate_memory();
    }
    
    printf("Program completed\\n");
    return 0;
}`;
    }

    getDeadlockSource() {
        return `#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

pthread_mutex_t mutex1 = PTHREAD_MUTEX_INITIALIZER;
pthread_mutex_t mutex2 = PTHREAD_MUTEX_INITIALIZER;

void* thread1_func(void* arg) {
    pthread_mutex_lock(&mutex1);
    printf("Thread 1 acquired mutex1\\n");
    sleep(1);
    pthread_mutex_lock(&mutex2); // Will deadlock with thread 2
    printf("Thread 1 acquired both mutexes\\n");
    pthread_mutex_unlock(&mutex2);
    pthread_mutex_unlock(&mutex1);
    return NULL;
}

void* thread2_func(void* arg) {
    pthread_mutex_lock(&mutex2);
    printf("Thread 2 acquired mutex2\\n");
    sleep(1);
    pthread_mutex_lock(&mutex1); // Will deadlock with thread 1
    printf("Thread 2 acquired both mutexes\\n");
    pthread_mutex_unlock(&mutex1);
    pthread_mutex_unlock(&mutex2);
    return NULL;
}

int main() {
    pthread_t thread1, thread2;
    
    pthread_create(&thread1, NULL, thread1_func, NULL);
    pthread_create(&thread2, NULL, thread2_func, NULL);
    
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);
    
    return 0;
}`;
    }

    getRaceConditionSource() {
        return `#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

int shared_counter = 0;
pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;

void* increment_counter(void* arg) {
    int thread_id = *(int*)arg;
    
    for (int i = 0; i < 1000; i++) {
        // Race condition: no synchronization
        int temp = shared_counter;
        temp++;
        shared_counter = temp;
        printf("Thread %d: counter = %d\\n", thread_id, shared_counter);
    }
    
    return NULL;
}

int main() {
    pthread_t threads[5];
    int thread_ids[5];
    
    for (int i = 0; i < 5; i++) {
        thread_ids[i] = i;
        pthread_create(&threads[i], NULL, increment_counter, &thread_ids[i]);
    }
    
    for (int i = 0; i < 5; i++) {
        pthread_join(threads[i], NULL);
    }
    
    printf("Final counter value: %d (expected: 5000)\\n", shared_counter);
    return 0;
}`;
    }

    getKernelPanicDump() {
        return `Kernel panic - not syncing: Fatal exception in interrupt
Pid: 1234, comm: systemd Not tainted 3.10.0-123.el7.x86_64 #1
Call Trace:
[<ffffffff8100129a>] show_regs+0x6a/0x70
[<ffffffff8105f2e9>] panic+0x1cb/0x1d7
[<ffffffff8105f8ba>] printk+0x45/0x4a
[<ffffffff81234567>] null_pointer_deref+0x50/0x80
[<ffffffff81234901>] interrupt_handler+0x23/0x40
[<ffffffff81012345>] handle_irq_event_percpu+0x65/0x1a0
[<ffffffff81012456>] handle_irq_event+0x3b/0x60
[<ffffffff81067890>] generic_handle_irq+0x30/0x40
[<ffffffff81078901>] __handle_irq+0x21/0x30
[<ffffffff81023456>] do_IRQ+0x46/0xe0
[<ffffffff81034567>] ret_from_intr+0x0/0x29
[<ffffffff81000000>] restore_args+0x0/0x30
Code: 48 8b 45 f8 48 85 c0 74 1e 48 8b 45 f0 8b 00 48 89 45 f8 48 c7 45 f0 00 00 00 00 48 8b 45 f0 c9 c3 0f 1f 00 48 8b 45 f8 48 85 c0 74 0c 48 8b 45 f0
RIP: 0010:[<ffffffff81234567>]  [<ffffffff81234567>] null_pointer_deref+0x50/0x80
RSP: 0018:ffff88001fc03e78  EFLAGS: 00010246
RAX: 0000000000000000 RBX: ffff88001fc03f18 RCX: 0000000000000001
RDX: 0000000000000000 RSI: ffff88001fc03f18 RDI: 0000000000000000
RBP: ffff88001fc03e78 R08: 0000000000000001 R09: 0000000000000000
R10: ffff88001fc03dd0 R11: 0000000000000206 R12: ffff88001fc03f18
---[ end trace 12345678abcdef01 ]---`;
    }

    getSyscallTestSource() {
        return `#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>

int main() {
    printf("Starting syscall test\\n");
    
    // Read from file
    int fd = open("test.txt", O_RDONLY);
    if (fd == -1) {
        perror("open");
        return 1;
    }
    
    char buffer[1024];
    ssize_t bytes_read = read(fd, buffer, sizeof(buffer));
    printf("Read %zd bytes\\n", bytes_read);
    
    close(fd);
    
    // Write to file
    fd = open("output.txt", O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return 1;
    }
    
    const char* message = "Hello, syscall world!\\n";
    ssize_t bytes_written = write(fd, message, 20);
    printf("Wrote %zd bytes\\n", bytes_written);
    
    close(fd);
    
    printf("Syscall test completed\\n");
    return 0;
}`;
    }

    getThreadSyncSource() {
        return `#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

pthread_cond_t condition = PTHREAD_COND_INITIALIZER;
pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;
int ready = 0;
int data = 0;

void* producer(void* arg) {
    for (int i = 0; i < 10; i++) {
        pthread_mutex_lock(&mutex);
        data = i * 10;
        ready = 1;
        printf("Produced: %d\\n", data);
        pthread_cond_signal(&condition);
        pthread_mutex_unlock(&mutex);
        usleep(100000); // 100ms
    }
    return NULL;
}

void* consumer(void* arg) {
    for (int i = 0; i < 10; i++) {
        pthread_mutex_lock(&mutex);
        while (!ready) {
            pthread_cond_wait(&condition, &mutex);
        }
        printf("Consumed: %d\\n", data);
        ready = 0;
        pthread_mutex_unlock(&mutex);
        usleep(150000); // 150ms
    }
    return NULL;
}

int main() {
    pthread_t prod, cons;
    
    pthread_create(&prod, NULL, producer, NULL);
    pthread_create(&cons, NULL, consumer, NULL);
    
    pthread_join(prod, NULL);
    pthread_join(cons, NULL);
    
    return 0;
}`;
    }

    getDiskIOSource() {
        return `#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>

#define BUFFER_SIZE 1024
#define ITERATIONS 1000

int main() {
    printf("Starting disk I/O test\\n");
    
    // Inefficient: small writes
    int fd = open("slow_io.dat", O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return 1;
    }
    
    char buffer[BUFFER_SIZE];
    memset(buffer, 'A', BUFFER_SIZE);
    
    for (int i = 0; i < ITERATIONS; i++) {
        ssize_t written = write(fd, buffer, BUFFER_SIZE);
        if (written != BUFFER_SIZE) {
            perror("write");
            break;
        }
    }
    
    close(fd);
    
    // Read test
    fd = open("slow_io.dat", O_RDONLY);
    if (fd == -1) {
        perror("open");
        return 1;
    }
    
    for (int i = 0; i < ITERATIONS; i++) {
        ssize_t bytes_read = read(fd, buffer, BUFFER_SIZE);
        if (bytes_read != BUFFER_SIZE) {
            break;
        }
    }
    
    close(fd);
    
    printf("Disk I/O test completed\\n");
    return 0;
}`;
    }

    // Progress tracking
    getUserProgress() {
        return Object.fromEntries(this.userProgress);
    }

    updateProgress(scenarioId, progressData) {
        this.userProgress.set(scenarioId, {
            ...progressData,
            lastUpdated: new Date()
        });
    }

    getScenarioProgress(scenarioId) {
        return this.userProgress.get(scenarioId);
    }

    // Export/Import
    exportScenarioData() {
        return {
            progress: this.getUserProgress(),
            scenarios: this.getAllScenarios()
        };
    }

    importScenarioData(data) {
        try {
            if (data.progress) {
                Object.entries(data.progress).forEach(([id, progress]) => {
                    this.userProgress.set(id, progress);
                });
            }
            
            if (data.app) {
                this.app.showNotification('Scenario data imported', 'success');
            }
        } catch (error) {
            if (this.app) {
                this.app.showNotification(`Failed to import scenario data: ${error.message}`, 'error');
            }
        }
    }

    destroy() {
        this.scenarios.clear();
        this.scenarioStates.clear();
        this.userProgress.clear();
        this.activeScenario = null;
    }
}

/**
 * Scenario Instance Class
 * Represents an active scenario with runtime state
 */
class ScenarioInstance {
    constructor(scenarioData, scenarioLibrary) {
        this.data = scenarioData;
        this.library = scenarioLibrary;
        this.app = scenarioLibrary.app;
        this.currentStep = 0;
        this.completedSteps = new Set();
        this.userInteractions = [];
        this.startTime = null;
        this.endTime = null;
    }

    async initialize() {
        this.startTime = new Date();
        
        // Set up the scenario environment
        await this.setupEnvironment();
        
        // Load files if needed
        await this.loadFiles();
        
        console.log(`📋 Initialized scenario: ${this.data.name}`);
        return this;
    }

    async setupEnvironment() {
        try {
            // Execute setup commands
            for (const command of this.data.setup?.commands || []) {
                await this.executeCommand(command);
            }
            
            // Set environment variables
            for (const [key, value] of Object.entries(this.data.setup?.environment || {})) {
                process.env[key] = value;
            }
        } catch (error) {
            console.warn('Failed to setup scenario environment:', error);
        }
    }

    async loadFiles() {
        if (!this.data.files) return;
        
        for (const file of this.data.files) {
            try {
                await this.loadFile(file.path, file.content);
            } catch (error) {
                console.warn(`Failed to load file ${file.path}:`, error);
            }
        }
    }

    async loadFile(path, content) {
        // In a real implementation, this would write files to the filesystem
        console.log(`📁 Loading file: ${path}`);
        
        // Mock file loading
        return {
            path,
            content,
            loaded: true,
            timestamp: new Date()
        };
    }

    async executeCommand(command) {
        // Mock command execution
        console.log(`⚡ Executing: ${command}`);
        return {
            stdout: `Mock output for: ${command}`,
            stderr: '',
            exitCode: 0
        };
    }

    getCurrentStep() {
        return this.data.steps?.[this.currentStep];
    }

    getNextStep() {
        return this.data.steps?.[this.currentStep + 1];
    }

    async completeStep(stepId) {
        this.completedSteps.add(stepId);
        this.currentStep++;
        
        const progress = this.library.getScenarioProgress(this.data.id) || {};
        progress.completedSteps = Array.from(this.completedSteps);
        progress.currentStep = this.currentStep;
        
        this.library.updateProgress(this.data.id, progress);
        
        console.log(`✅ Completed step ${stepId} in scenario ${this.data.name}`);
    }

    recordInteraction(interaction) {
        this.userInteractions.push({
            ...interaction,
            timestamp: new Date()
        });
    }

    async reset() {
        this.currentStep = 0;
        this.completedSteps.clear();
        this.userInteractions = [];
        this.startTime = new Date();
        this.endTime = null;
        
        console.log(`🔄 Reset scenario: ${this.data.name}`);
    }

    async complete() {
        this.endTime = new Date();
        
        const progress = this.library.getScenarioProgress(this.data.id) || {};
        progress.completed = this.endTime;
        progress.duration = this.endTime - this.startTime;
        progress.userInteractions = this.userInteractions;
        
        this.library.updateProgress(this.data.id, progress);
        
        console.log(`🎉 Completed scenario: ${this.data.name}`);
    }

    getProgress() {
        const totalSteps = this.data.steps?.length || 0;
        const completedSteps = this.completedSteps.size;
        const duration = this.endTime || new Date() - this.startTime;
        
        return {
            totalSteps,
            completedSteps,
            currentStep: this.currentStep,
            percentageComplete: totalSteps > 0 ? (completedSteps / totalSteps) * 100 : 0,
            duration,
            isCompleted: !!this.endTime
        };
    }

    exportState() {
        return {
            scenarioId: this.data.id,
            data: this.data,
            currentStep: this.currentStep,
            completedSteps: Array.from(this.completedSteps),
            userInteractions: this.userInteractions,
            startTime: this.startTime,
            endTime: this.endTime,
            progress: this.getProgress()
        };
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ScenarioLibrary, ScenarioInstance };
}