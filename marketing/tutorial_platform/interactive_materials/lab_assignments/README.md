# MultiOS Lab Assignments Repository

## Overview

This repository contains structured laboratory assignments designed to provide hands-on experience with MultiOS development. Assignments are organized by difficulty level and cover fundamental to advanced topics in operating systems development.

## Assignment Categories

### Basic Lab Assignments (Weeks 1-5)
1. [Environment Setup and Basic Commands](#basic-lab-1)
2. [File System Operations](#basic-lab-2)
3. [Process Management](#basic-lab-3)
4. [Inter-Process Communication](#basic-lab-4)
5. [Memory Management](#basic-lab-5)

### Intermediate Lab Assignments (Weeks 6-10)
6. [Network Programming](#intermediate-lab-6)
7. [Threading and Synchronization](#intermediate-lab-7)
8. [System Monitoring](#intermediate-lab-8)
9. [Security Implementation](#intermediate-lab-9)
10. [Performance Analysis](#intermediate-lab-10)

### Advanced Lab Assignments (Weeks 11-14)
11. [Kernel Module Development](#advanced-lab-11)
12. [Device Driver Implementation](#advanced-lab-12)
13. [Network Protocol Development](#advanced-lab-13)
14. [Custom System Calls](#advanced-lab-14)

### Capstone Projects (Week 15)
15. [MultiOS Application Project](#capstone-lab)

## Basic Lab Assignments

### Basic Lab 1: Environment Setup and Basic Commands

**Duration**: 3 hours
**Prerequisites**: MultiOS installation, basic programming knowledge

#### Objectives
- Set up MultiOS development environment
- Master command-line operations
- Understand file system organization
- Practice version control

#### Part 1: Development Environment Setup
1. **Install MultiOS SDK**
   ```bash
   # Install development tools
   pkg install multios-sdk build-essential git
   
   # Verify installation
   multios-sdk --version
   ```

2. **Configure Development Environment**
   ```bash
   # Set environment variables
   export MULTIOS_ROOT=/opt/multios
   export PATH=$PATH:$MULTIOS_ROOT/bin
   
   # Create development workspace
   mkdir -p ~/multios-labs
   cd ~/multios-labs
   ```

#### Part 2: Command-Line Mastery
1. **File System Navigation**
   ```bash
   # Explore directory structure
   ls -la /
   cd /home
   pwd
   
   # Create directories and files
   mkdir lab1
   cd lab1
   touch file1.txt file2.txt
   echo "Hello MultiOS" > file1.txt
   ```

2. **File Operations**
   ```bash
   # Copy, move, and delete files
   cp file1.txt file1_copy.txt
   mv file2.txt renamed.txt
   rm renamed.txt
   
   # Search for files
   find . -name "*.txt"
   grep -r "Hello" .
   ```

#### Part 3: Version Control Setup
```bash
# Initialize Git repository
git init
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Create .gitignore
echo "*.o" > .gitignore
echo "*.out" >> .gitignore

# Commit initial setup
git add .
git commit -m "Initial lab setup"
```

#### Deliverables
1. **Working Development Environment**: Verified SDK installation and configuration
2. **Command Script**: Bash script demonstrating all required commands
3. **Git Repository**: Initialized repository with proper configuration

#### Assessment Criteria
- Environment setup completion (25%)
- Command proficiency (25%)
- Script quality (25%)
- Version control usage (25%)

#### Additional Challenges
1. Create a custom alias for frequently used commands
2. Write a shell script to automate environment setup
3. Explore advanced find and grep options

---

### Basic Lab 2: File System Operations

**Duration**: 4 hours
**Prerequisites**: Basic Lab 1 completion

#### Objectives
- Implement file operations using MultiOS system calls
- Understand file system structure
- Practice error handling
- Learn file permissions and security

#### Part 1: Basic File Operations
**Assignment**: Create a file utility program
```c
// file_ops.c
#include <multios/multios.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

int copy_file(const char *src, const char *dst) {
    int src_fd, dst_fd;
    char buffer[4096];
    ssize_t bytes_read;
    
    // Open source file
    src_fd = multios_open(src, MULTIOS_O_RDONLY);
    if (src_fd < 0) {
        perror("Failed to open source file");
        return -1;
    }
    
    // Create destination file
    dst_fd = multios_open(dst, MULTIOS_O_CREAT | MULTIOS_O_WRONLY | MULTIOS_O_TRUNC, 0644);
    if (dst_fd < 0) {
        perror("Failed to create destination file");
        multios_close(src_fd);
        return -1;
    }
    
    // Copy file contents
    while ((bytes_read = multios_read(src_fd, buffer, sizeof(buffer))) > 0) {
        ssize_t bytes_written = multios_write(dst_fd, buffer, bytes_read);
        if (bytes_written != bytes_read) {
            perror("Write error");
            multios_close(src_fd);
            multios_close(dst_fd);
            return -1;
        }
    }
    
    multios_close(src_fd);
    multios_close(dst_fd);
    
    return 0;
}

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <source> <destination>\n", argv[0]);
        return 1;
    }
    
    if (copy_file(argv[1], argv[2]) == 0) {
        printf("File copied successfully\n");
        return 0;
    } else {
        printf("File copy failed\n");
        return 1;
    }
}
```

#### Part 2: File Information
**Assignment**: Create a file info utility
```c
// file_info.c
#include <multios/multios.h>
#include <sys/stat.h>
#include <stdio.h>
#include <unistd.h>
#include <time.h>

void print_file_info(const char *filename) {
    struct stat st;
    
    if (stat(filename, &st) < 0) {
        perror("stat failed");
        return;
    }
    
    printf("File: %s\n", filename);
    printf("Size: %ld bytes\n", st.st_size);
    printf("Permissions: %o\n", st.st_mode & 0777);
    printf("Owner UID: %d\n", st.st_uid);
    printf("Group GID: %d\n", st.st_gid);
    printf("Last access: %s", ctime(&st.st_atime));
    printf("Last modification: %s", ctime(&st.st_mtime));
    printf("Last status change: %s", ctime(&st.st_ctime));
    
    if (S_ISDIR(st.st_mode)) {
        printf("Type: Directory\n");
    } else if (S_ISREG(st.st_mode)) {
        printf("Type: Regular file\n");
    } else if (S_ISLNK(st.st_mode)) {
        printf("Type: Symbolic link\n");
    } else {
        printf("Type: Special file\n");
    }
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <file1> [file2] ...\n", argv[0]);
        return 1;
    }
    
    for (int i = 1; i < argc; i++) {
        print_file_info(argv[i]);
        printf("\n");
    }
    
    return 0;
}
```

#### Part 3: File Search Utility
**Assignment**: Implement a recursive file search
```c
// file_find.c
#include <multios/multios.h>
#include <dirent.h>
#include <string.h>
#include <stdio.h>

void search_directory(const char *path, const char *search_pattern) {
    DIR *dir;
    struct dirent *entry;
    
    dir = opendir(path);
    if (!dir) {
        perror("opendir failed");
        return;
    }
    
    while ((entry = readdir(dir)) != NULL) {
        char full_path[1024];
        
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        
        snprintf(full_path, sizeof(full_path), "%s/%s", path, entry->d_name);
        
        if (strstr(entry->d_name, search_pattern)) {
            printf("Found: %s\n", full_path);
        }
        
        if (entry->d_type == DT_DIR) {
            search_directory(full_path, search_pattern);
        }
    }
    
    closedir(dir);
}

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <directory> <pattern>\n", argv[0]);
        return 1;
    }
    
    search_directory(argv[1], argv[2]);
    return 0;
}
```

#### Deliverables
1. **File Operations Program**: Complete file copy utility
2. **File Information Tool**: File details display program
3. **Search Utility**: Recursive file search program
4. **Test Report**: Testing results with various file types and sizes

#### Assessment Criteria
- Code correctness (40%)
- Error handling (20%)
- Performance (20%)
- Documentation (20%)

#### Additional Challenges
1. Implement file comparison functionality
2. Add support for different file encodings
3. Create a file archiving utility
4. Implement hard and symbolic link support

---

### Basic Lab 3: Process Management

**Duration**: 4 hours
**Prerequisites**: Basic Labs 1-2

#### Objectives
- Understand process creation and management
- Implement process communication
- Practice signal handling
- Learn about process states and scheduling

#### Part 1: Process Creation and Management
**Assignment**: Process tree demonstrator
```c
// process_tree.c
#include <multios/multios.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>
#include <stdlib.h>

void create_process_tree(int depth, int max_depth) {
    pid_t pid;
    
    if (depth >= max_depth) {
        printf("Leaf process: PID=%d, PPID=%d, depth=%d\n",
               multios_getpid(), multios_getppid(), depth);
        sleep(5);
        return;
    }
    
    for (int i = 0; i < 2; i++) {
        pid = multios_fork();
        
        if (pid == 0) {
            // Child process
            printf("Created child: PID=%d, PPID=%d, depth=%d, branch=%d\n",
                   multios_getpid(), multios_getppid(), depth + 1, i);
            create_process_tree(depth + 1, max_depth);
            exit(0);
        } else if (pid < 0) {
            perror("Fork failed");
            exit(1);
        }
    }
    
    // Parent waits for all children
    for (int i = 0; i < 2; i++) {
        int status;
        wait(&status);
    }
}

int main(int argc, char *argv[]) {
    int depth = (argc > 1) ? atoi(argv[1]) : 3;
    
    printf("Main process: PID=%d, PPID=%d\n",
           multios_getpid(), multios_getppid());
    printf("Creating process tree with depth %d\n", depth);
    
    create_process_tree(0, depth);
    
    printf("All processes completed\n");
    return 0;
}
```

#### Part 2: Signal Handling
**Assignment**: Signal handling demonstrator
```c
// signal_demo.c
#include <multios/multios.h>
#include <signal.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

volatile sig_atomic_t signal_count = 0;

void signal_handler(int signum) {
    signal_count++;
    printf("Received signal %d, count=%d\n", signum, signal_count);
    
    if (signal_count >= 3) {
        printf("Too many signals, exiting\n");
        exit(0);
    }
}

int main(void) {
    struct sigaction sa;
    
    // Set up signal handler
    sa.sa_handler = signal_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;
    
    // Register handlers for various signals
    if (sigaction(SIGINT, &sa, NULL) < 0) {
        perror("sigaction SIGINT failed");
        return 1;
    }
    
    if (sigaction(SIGTERM, &sa, NULL) < 0) {
        perror("sigaction SIGTERM failed");
        return 1;
    }
    
    printf("Signal demonstration program\n");
    printf("PID: %d\n", multios_getpid());
    printf("Send signals with: kill -INT %d or kill -TERM %d\n",
           multios_getpid(), multios_getpid());
    
    // Main loop
    while (1) {
        printf("Running... (signal count: %d)\n", signal_count);
        sleep(2);
    }
    
    return 0;
}
```

#### Part 3: Process Monitoring
**Assignment**: Process status monitor
```c
// process_monitor.c
#include <multios/multios.h>
#include <stdio.h>
#include <dirent.h>
#include <string.h>
#include <stdlib.h>

#define PROC_DIR "/proc"

typedef struct {
    int pid;
    char name[256];
    char state;
    long vm_size;
    int priority;
} process_info_t;

process_info_t *get_process_info(int pid) {
    char path[256];
    char line[256];
    FILE *fp;
    process_info_t *info;
    
    info = malloc(sizeof(process_info_t));
    if (!info) {
        return NULL;
    }
    
    info->pid = pid;
    info->name[0] = '\0';
    info->state = '?';
    info->vm_size = 0;
    info->priority = 0;
    
    // Read /proc/[pid]/stat
    snprintf(path, sizeof(path), "%s/%d/stat", PROC_DIR, pid);
    fp = fopen(path, "r");
    if (fp) {
        int dummy;
        char comm[256];
        
        if (fscanf(fp, "%d (%255[^)]) %c %d %d %d %d %d %d %lu %lu %lu %lu %lu %lu %ld %ld %ld %ld %ld %ld %ld %d %d %lu %lu %ld %lu %lu %lu %lu %lu %lu %lu %lu %lu %lu %lu %lu %lu %lu %d %d %d %d %d %d %d",
                   &dummy, comm, &info->state, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &info->priority, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &dummy, &info->vm_size) == 46) {
            strncpy(info->name, comm, sizeof(info->name) - 1);
            info->name[sizeof(info->name) - 1] = '\0';
        }
        
        fclose(fp);
    }
    
    return info;
}

void monitor_processes(void) {
    DIR *dir;
    struct dirent *entry;
    
    printf("%-8s %-20s %-6s %-12s %-8s\n", "PID", "NAME", "STATE", "VM_SIZE(KB)", "PRIORITY");
    printf("%-8s %-20s %-6s %-12s %-8s\n", "----", "---------------", "------", "------------", "--------");
    
    dir = opendir(PROC_DIR);
    if (!dir) {
        perror("Failed to open /proc");
        return;
    }
    
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_DIR) {
            int pid = atoi(entry->d_name);
            if (pid > 0) {
                process_info_t *info = get_process_info(pid);
                if (info) {
                    printf("%-8d %-20s %-6c %-12ld %-8d\n",
                           info->pid, info->name, info->state, 
                           info->vm_size, info->priority);
                    free(info);
                }
            }
        }
    }
    
    closedir(dir);
}

int main(int argc, char *argv[]) {
    if (argc > 1) {
        int pid = atoi(argv[1]);
        process_info_t *info = get_process_info(pid);
        if (info) {
            printf("Process %d information:\n", pid);
            printf("  Name: %s\n", info->name);
            printf("  State: %c\n", info->state);
            printf("  VM Size: %ld KB\n", info->vm_size);
            printf("  Priority: %d\n", info->priority);
            free(info);
        } else {
            printf("Process %d not found\n", pid);
        }
    } else {
        printf("Monitoring all processes (Press Ctrl+C to stop):\n\n");
        while (1) {
            monitor_processes();
            sleep(5);
            printf("\n");
        }
    }
    
    return 0;
}
```

#### Deliverables
1. **Process Tree Program**: Demonstrates process creation and hierarchy
2. **Signal Handler**: Shows signal handling in action
3. **Process Monitor**: Real-time process information display
4. **Documentation**: Explanation of process management concepts

#### Assessment Criteria
- Process creation accuracy (30%)
- Signal handling implementation (25%)
- Monitoring functionality (25%)
- Code documentation (20%)

---

### Basic Lab 4: Inter-Process Communication

**Duration**: 4 hours
**Prerequisites**: Basic Labs 1-3

#### Objectives
- Implement various IPC mechanisms
- Understand message passing
- Practice shared memory usage
- Learn synchronization

#### Part 1: Message Queue Implementation
**Assignment**: Client-server message queue system
```c
// msg_server.c
#include <multios/ipc.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define MSG_KEY 1234
#define MSG_TYPE 1

typedef struct {
    long msg_type;
    char msg_text[256];
} message_t;

void run_message_server(void) {
    int msgid;
    message_t msg;
    
    // Create message queue
    msgid = multios_msgget(MSG_KEY, IPC_CREAT | 0666);
    if (msgid < 0) {
        perror("msgget failed");
        return;
    }
    
    printf("Message queue server started (ID: %d)\n", msgid);
    
    // Receive messages
    while (1) {
        if (multios_msgrcv(msgid, &msg, sizeof(msg.msg_text), MSG_TYPE, 0) < 0) {
            perror("msgrcv failed");
            break;
        }
        
        printf("Received: %s\n", msg.msg_text);
        
        // Check for exit command
        if (strcmp(msg.msg_text, "quit") == 0) {
            printf("Shutting down server\n");
            break;
        }
    }
    
    // Clean up
    multios_msgctl(msgid, IPC_RMID, NULL);
}

int main(void) {
    run_message_server();
    return 0;
}
```

```c
// msg_client.c
#include <multios/ipc.h>
#include <stdio.h>
#include <string.h>

#define MSG_KEY 1234
#define MSG_TYPE 1

typedef struct {
    long msg_type;
    char msg_text[256];
} message_t;

int main(int argc, char *argv[]) {
    int msgid;
    message_t msg;
    
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <message>\n", argv[0]);
        return 1;
    }
    
    // Get message queue
    msgid = multios_msgget(MSG_KEY, 0666);
    if (msgid < 0) {
        perror("msgget failed");
        return 1;
    }
    
    // Prepare message
    msg.msg_type = MSG_TYPE;
    strncpy(msg.msg_text, argv[1], sizeof(msg.msg_text) - 1);
    msg.msg_text[sizeof(msg.msg_text) - 1] = '\0';
    
    // Send message
    if (multios_msgsnd(msgid, &msg, strlen(msg.msg_text), 0) < 0) {
        perror("msgsnd failed");
        return 1;
    }
    
    printf("Message sent: %s\n", msg.msg_text);
    return 0;
}
```

#### Part 2: Shared Memory Implementation
**Assignment**: Producer-consumer with shared memory
```c
// shm_producer.c
#include <multios/ipc.h>
#include <multios/mutex.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define SHM_KEY 5678
#define SEM_KEY 9999

typedef struct {
    char buffer[100][256];
    int count;
    int in;
    int out;
    multios_mutex_t mutex;
} shared_data_t;

int main(void) {
    int shmid;
    shared_data_t *data;
    
    // Create shared memory
    shmid = multios_shmget(SHM_KEY, sizeof(shared_data_t), IPC_CREAT | 0666);
    if (shmid < 0) {
        perror("shmget failed");
        return 1;
    }
    
    // Attach shared memory
    data = (shared_data_t *)multios_shmat(shmid, NULL, 0);
    if (data == (void *)-1) {
        perror("shmat failed");
        return 1;
    }
    
    // Initialize shared data
    data->count = 0;
    data->in = 0;
    data->out = 0;
    multios_mutex_init(&data->mutex, NULL);
    
    printf("Producer started\n");
    
    // Produce items
    for (int i = 0; i < 20; i++) {
        char item[256];
        snprintf(item, sizeof(item), "Item %d", i);
        
        multios_mutex_lock(&data->mutex);
        
        // Wait while buffer is full
        while (data->count == 100) {
            multios_mutex_unlock(&data->mutex);
            usleep(1000);
            multios_mutex_lock(&data->mutex);
        }
        
        // Add item to buffer
        strcpy(data->buffer[data->in], item);
        data->in = (data->in + 1) % 100;
        data->count++;
        
        printf("Produced: %s (count: %d)\n", item, data->count);
        
        multios_mutex_unlock(&data->mutex);
        
        usleep(50000); // Simulate work
    }
    
    printf("Producer finished\n");
    
    // Detach shared memory
    multios_shmdt(data);
    
    return 0;
}
```

#### Deliverables
1. **Message Queue System**: Working client-server with message queues
2. **Shared Memory Producer-Consumer**: Synchronized shared memory usage
3. **IPC Documentation**: Explanation of different IPC mechanisms
4. **Performance Analysis**: Comparison of different IPC methods

---

### Basic Lab 5: Memory Management

**Duration**: 4 hours
**Prerequisites**: Basic Labs 1-4

#### Objectives
- Understand dynamic memory allocation
- Implement memory management functions
- Practice memory debugging
- Learn about memory protection

#### Part 1: Custom Memory Allocator
**Assignment**: Simple memory allocator implementation
```c
// memory_allocator.c
#include <multios/memory.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>

#define ALLOCATOR_MAGIC 0x12345678
#define MIN_ALLOC_SIZE 16

typedef struct block_header {
    uint32_t magic;
    size_t size;
    int free;
    struct block_header *next;
} block_header_t;

static block_header_t *free_list = NULL;
static void *memory_pool = NULL;
static size_t pool_size = 1024 * 1024; // 1MB

void memory_pool_init(void) {
    memory_pool = multios_alloc(pool_size);
    if (!memory_pool) {
        fprintf(stderr, "Failed to allocate memory pool\n");
        return;
    }
    
    // Initialize first block
    free_list = (block_header_t *)memory_pool;
    free_list->magic = ALLOCATOR_MAGIC;
    free_list->size = pool_size - sizeof(block_header_t);
    free_list->free = 1;
    free_list->next = NULL;
    
    printf("Memory pool initialized: %zu bytes\n", pool_size);
}

void *custom_alloc(size_t size) {
    block_header_t *current, *prev;
    
    if (size < MIN_ALLOC_SIZE) {
        size = MIN_ALLOC_SIZE;
    }
    
    // Align size to 8 bytes
    size = (size + 7) & ~7;
    
    current = free_list;
    prev = NULL;
    
    // Find suitable block
    while (current) {
        if (current->free && current->size >= size) {
            // Found suitable block
            if (current->size > size + sizeof(block_header_t) + MIN_ALLOC_SIZE) {
                // Split block
                block_header_t *new_block = (block_header_t *)((char *)current + sizeof(block_header_t) + size);
                new_block->magic = ALLOCATOR_MAGIC;
                new_block->size = current->size - size - sizeof(block_header_t);
                new_block->free = 1;
                new_block->next = current->next;
                
                current->next = new_block;
                current->size = size;
            }
            
            current->free = 0;
            
            printf("Allocated %zu bytes at %p\n", size, (char *)current + sizeof(block_header_t));
            
            return (char *)current + sizeof(block_header_t);
        }
        
        prev = current;
        current = current->next;
    }
    
    fprintf(stderr, "Out of memory!\n");
    return NULL;
}

void custom_free(void *ptr) {
    block_header_t *block, *current, *prev;
    
    if (!ptr) {
        return;
    }
    
    // Get block header
    block = (block_header_t *)((char *)ptr - sizeof(block_header_t));
    
    // Validate magic number
    if (block->magic != ALLOCATOR_MAGIC) {
        fprintf(stderr, "Invalid pointer passed to custom_free\n");
        return;
    }
    
    // Mark block as free
    block->free = 1;
    
    printf("Freed %zu bytes at %p\n", block->size, ptr);
    
    // Merge with adjacent free blocks
    current = free_list;
    prev = NULL;
    
    while (current) {
        if (current->free && block != current) {
            // Check if blocks are adjacent
            if ((char *)current + sizeof(block_header_t) + current->size == (char *)block) {
                // Merge block into current
                current->size += sizeof(block_header_t) + block->size;
                block = current;
            } else if ((char *)block + sizeof(block_header_t) + block->size == (char *)current) {
                // Merge current into block
                block->size += sizeof(block_header_t) + current->size;
                if (prev) {
                    prev->next = current->next;
                } else {
                    free_list = current->next;
                }
            }
        }
        
        prev = current;
        current = current->next;
    }
}

void print_memory_stats(void) {
    block_header_t *current = free_list;
    size_t total_free = 0;
    size_t block_count = 0;
    
    printf("\nMemory Pool Statistics:\n");
    printf("------------------------\n");
    
    while (current) {
        if (current->free) {
            total_free += current->size;
            block_count++;
        }
        current = current->next;
    }
    
    printf("Total free space: %zu bytes\n", total_free);
    printf("Number of free blocks: %zu\n", block_count);
    printf("Largest free block: %zu bytes\n", total_free);
    printf("Pool utilization: %.2f%%\n", 
           100.0 * (pool_size - total_free) / pool_size);
}

int main(void) {
    memory_pool_init();
    
    // Test allocations
    char *ptr1 = custom_alloc(100);
    char *ptr2 = custom_alloc(200);
    char *ptr3 = custom_alloc(50);
    
    print_memory_stats();
    
    // Free some blocks
    custom_free(ptr2);
    custom_free(ptr1);
    
    print_memory_stats();
    
    // Test more allocations
    char *ptr4 = custom_alloc(300);
    char *ptr5 = custom_alloc(25);
    
    print_memory_stats();
    
    // Free remaining blocks
    custom_free(ptr3);
    custom_free(ptr4);
    custom_free(ptr5);
    
    print_memory_stats();
    
    multios_free(memory_pool);
    return 0;
}
```

#### Deliverables
1. **Custom Memory Allocator**: Working memory allocation implementation
2. **Memory Testing Program**: Comprehensive testing suite
3. **Memory Analysis Tool**: Tool to analyze memory usage patterns
4. **Report**: Analysis of allocator performance and efficiency

---

## Intermediate Lab Assignments

### Intermediate Lab 6: Network Programming

**Duration**: 5 hours
**Prerequisites**: Basic Labs 1-5

#### Objectives
- Implement network protocols
- Understand socket programming
- Practice client-server architecture
- Learn network security

#### Assignment: HTTP Server Implementation
[Detailed implementation continues...]

## Assessment Framework

### Grading Rubric
- **Correctness** (40%): Program produces correct results
- **Code Quality** (25%): Clean, well-documented code
- **Performance** (20%): Efficient implementation
- **Documentation** (15%): Clear explanations and comments

### Submission Requirements
1. Source code with proper documentation
2. Makefile for building the project
3. Test cases demonstrating functionality
4. README file with usage instructions
5. Performance analysis (for advanced labs)

### Late Submission Policy
- 10% deduction per day late
- Maximum 5 days late
- Zero credit after 5 days

### Collaboration Policy
- Individual work required unless specified
- Discussion of concepts is encouraged
- Code sharing is not permitted
- All sources must be cited

This laboratory assignment framework provides structured, progressive learning experiences that build from basic concepts to advanced operating systems development skills.