# Solution Guides - MultiOS Education

## 📚 Comprehensive Solution Framework

This document provides detailed solution guides for all levels of the MultiOS teaching resource library.

## 🏗️ Solution Guide Structure

### Solution Components
1. **Conceptual Overview**: Theoretical foundation
2. **Step-by-Step Implementation**: Detailed code solutions
3. **Alternative Approaches**: Multiple solution paths
4. **Common Pitfalls**: Frequent mistakes and solutions
5. **Extensions**: Additional challenges and improvements
6. **Best Practices**: Professional recommendations

### Usage Guidelines
- Solutions are provided for educational purposes
- Students should attempt problems before viewing solutions
- Alternative solutions are encouraged
- Solutions demonstrate best practices and efficiency

---

## 🎯 Beginner Level Solutions (Labs 1-30)

### Lab 01: Introduction to Command Line Interface

#### Solution Overview
This lab introduces fundamental command line operations for navigating and manipulating files in Unix-like systems.

#### Key Concepts Covered
- Directory navigation with `cd`, `pwd`, `ls`
- File and directory operations
- Permission management
- Shell environment customization

#### Implementation Solution
```bash
#!/bin/bash
# Lab 01 Solution: Comprehensive command line tutorial

# 1. Basic Navigation
echo "=== Basic Navigation Lab ==="
pwd  # Print working directory
ls -la  # List all files with details
cd /tmp && pwd  # Change directory

# 2. Directory Operations
mkdir -p lab01_practice/{documents,scripts,data}
cd lab01_practice
tree  # Show directory structure

# 3. File Operations
touch document1.txt document2.txt
echo "Hello World" > document1.txt
cp document1.txt backup.txt
ls -la

# 4. Permission Management
chmod 755 document1.txt
chmod +x script.sh
ls -l document1.txt script.sh

# 5. File Content Operations
head -5 document1.txt
tail -5 document1.txt
wc -l document1.txt
grep -n "Hello" document1.txt

echo "Lab 01 completed successfully!"
```

#### Challenge Solution
```bash
#!/bin/bash
# Challenge: Company organizational chart

# Create directory structure
mkdir -p company/{hr,engineering,sales,marketing}/{employees,projects,reports}
cd company

# Create sample files
for dept in hr engineering sales marketing; do
    for emp in alice bob charlie diana; do
        echo "Employee: $emp in $dept" > ${dept}/employees/${emp}_profile.txt
    done
done

# Display the structure
tree company/
```

#### Best Practices
1. **Always use full paths** when working in scripts
2. **Test commands** before adding to scripts
3. **Use `man` pages** for command documentation
4. **Practice error handling** with conditional statements

---

### Lab 02: File System Navigation and Management

#### Solution Overview
Advanced file system operations including search, archiving, and virtual file systems.

#### Implementation Solution
```bash
#!/bin/bash
# Lab 02 Solution: Advanced file system management

# 1. File Search Operations
echo "=== File Search Lab ==="
find /usr/bin -name "*python*" -type f 2>/dev/null
locate passwd | head -10
which python3

# 2. Symbolic Links
ln -s /usr/bin/python3 ~/python_link
ls -la ~/python_link

# 3. Archive Operations
tar -czf backup.tar.gz /etc/passwd /etc/group
tar -tzf backup.tar.gz  # List archive contents
tar -xzf backup.tar.gz  # Extract archive

# 4. Virtual File Systems
echo "=== Virtual File Systems ==="
cat /proc/cpuinfo | head -20
cat /proc/meminfo | head -10
ls /sys/class/net/  # Network interfaces

# 5. Disk Usage Analysis
df -h
du -sh /var /etc /tmp
du -sh /* 2>/dev/null | sort -hr | head -10

echo "Lab 02 completed successfully!"
```

#### Challenge Solution
```bash
#!/bin/bash
# Challenge: Backup script for home directory

BACKUP_DIR="$HOME/backup_$(date +%Y%m%d_%H%M%S)"
SOURCE_DIR="$HOME"

echo "Creating backup from $SOURCE_DIR to $BACKUP_DIR"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup important directories
for dir in Documents Desktop Downloads Pictures; do
    if [ -d "$SOURCE_DIR/$dir" ]; then
        cp -r "$SOURCE_DIR/$dir" "$BACKUP_DIR/"
        echo "Backed up: $dir"
    fi
done

# Create archive
cd "$HOME"
tar -czf "backup_$(date +%Y%m%d_%H%M%S).tar.gz" backup_*/

# Cleanup
rm -rf backup_*/

echo "Backup completed: backup_$(date +%Y%m%d_%H%M%S).tar.gz"
```

---

### Lab 03: Text Processing and File Manipulation

#### Solution Overview
Advanced text processing using grep, sed, awk, and pipeline operations.

#### Implementation Solution
```bash
#!/bin/bash
# Lab 03 Solution: Text processing and manipulation

# 1. Basic Text Operations
echo "=== Text Processing Lab ==="
cat /etc/passwd | head -10
cat /etc/passwd | tail -5
cat /etc/passwd | wc -l

# 2. Pattern Matching with grep
grep -i "root" /etc/passwd
grep -n "^root" /etc/passwd
grep -v "nologin" /etc/passwd | head -5

# 3. Advanced Text Processing
# Process CSV-like data
cat > sample_data.csv << EOF
name,age,city
Alice,25,NYC
Bob,30,LA
Charlie,35,Chicago
Diana,28,Boston
EOF

# Extract columns with awk
awk -F',' '{print $1, $3}' sample_data.csv
awk -F',' 'NR>1 && $2>27 {print $1}' sample_data.csv

# 4. Text Transformation with sed
echo "hello world" | sed 's/hello/HELLO/'
cat sample_data.csv | sed 's/,/\t/g'  # Convert CSV to TSV

# 5. Advanced Pipeline Operations
cat /etc/passwd | grep -v nologin | awk -F':' '{print $1, $5}' | sort

echo "Lab 03 completed successfully!"
```

#### Challenge Solution
```bash
#!/bin/bash
# Challenge: Parse system log for errors

LOG_FILE="${1:-/var/log/syslog}"
ERROR_PATTERN="${2:-ERROR}"

echo "Analyzing log file: $LOG_FILE"
echo "Searching for pattern: $ERROR_PATTERN"
echo

# Extract error messages with timestamps
grep -i "$ERROR_PATTERN" "$LOG_FILE" | \
    awk '{print $1, $2, $3, $NF}' | \
    sort | uniq -c | sort -rn | head -10

# Count errors by hour
grep -i "$ERROR_PATTERN" "$LOG_FILE" | \
    awk '{print $3}' | cut -d: -f1 | sort | uniq -c

echo
echo "Error analysis completed!"
```

---

### Lab 04: Process Management Basics

#### Solution Overview
Process monitoring, control, and management using system tools.

#### Implementation Solution
```c
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
#include <signal.h>

void sigchld_handler(int sig) {
    pid_t pid;
    int status;
    while ((pid = waitpid(-1, &status, WNOHANG)) > 0) {
        printf("Child process %d terminated\n", pid);
    }
}

int main() {
    signal(SIGCHLD, sigchld_handler);
    
    printf("=== Process Management Lab ===\n");
    
    // Fork a child process
    pid_t pid = fork();
    
    if (pid == 0) {
        // Child process
        printf("Child process started (PID: %d)\n", getpid());
        sleep(5);
        printf("Child process finished\n");
        exit(0);
    } else if (pid > 0) {
        // Parent process
        printf("Parent process (PID: %d), child PID: %d\n", getpid(), pid);
        
        // Monitor child process
        int status;
        while (1) {
            pid_t result = waitpid(pid, &status, WNOHANG);
            if (result == -1) {
                perror("waitpid");
                break;
            } else if (result == 0) {
                printf("Child still running...\n");
                sleep(1);
            } else {
                if (WIFEXITED(status)) {
                    printf("Child exited with status %d\n", WEXITSTATUS(status));
                } else if (WIFSIGNALED(status)) {
                    printf("Child terminated by signal %d\n", WTERMSIG(status));
                }
                break;
            }
        }
    } else {
        perror("fork");
        exit(1);
    }
    
    printf("Process management lab completed!\n");
    return 0;
}
```

#### Process Monitoring Script Solution
```bash
#!/bin/bash
# Challenge: Process monitoring script

PROCESS_NAME="$1"
MONITOR_INTERVAL="${2:-5}"

if [ -z "$PROCESS_NAME" ]; then
    echo "Usage: $0 <process_name> [interval_seconds]"
    exit 1
fi

echo "Monitoring process: $PROCESS_NAME (every ${MONITOR_INTERVAL}s)"
echo "Press Ctrl+C to stop"
echo

while true; do
    # Find process
    PID=$(pgrep -f "$PROCESS_NAME")
    
    if [ -n "$PID" ]; then
        CPU=$(ps -p $PID -o %cpu --no-headers | tr -d ' ')
        MEM=$(ps -p $PID -o %mem --no-headers | tr -d ' ')
        TIME=$(ps -p $PID -o time --no-headers | tr -d ' ')
        
        echo "[$(date '+%H:%M:%S')] PID: $PID, CPU: ${CPU}%, MEM: ${MEM}%, TIME: $TIME"
    else
        echo "[$(date '+%H:%M:%S')] Process '$PROCESS_NAME' not found"
    fi
    
    sleep $MONITOR_INTERVAL
done
```

---

## 🎯 Intermediate Level Solutions (Labs 31-60)

### Lab 31: Advanced Process Management and Scheduling

#### Solution Overview
Implementation of custom scheduling algorithms and priority management.

#### Implementation Solution
```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <sched.h>
#include <sys/sysinfo.h>
#include <time.h>

#define MAX_THREADS 16
#define TIME_SLICE 100000  // 100ms in microseconds

typedef struct {
    int thread_id;
    int priority;
    int burst_time;
    int remaining_time;
    int completion_time;
    pthread_t thread;
    struct timespec arrival_time;
} thread_info_t;

thread_info_t threads[MAX_THREADS];
int num_threads = 0;
pthread_mutex_t scheduler_mutex = PTHREAD_MUTEX_INITIALIZER;
int current_thread = -1;

// Priority-based scheduler
void* priority_scheduler(void* arg) {
    while (1) {
        pthread_mutex_lock(&scheduler_mutex);
        
        // Find highest priority thread
        int highest_priority = -1;
        int selected_thread = -1;
        
        for (int i = 0; i < num_threads; i++) {
            if (threads[i].remaining_time > 0 && 
                threads[i].priority > highest_priority) {
                highest_priority = threads[i].priority;
                selected_thread = i;
            }
        }
        
        if (selected_thread == -1) {
            pthread_mutex_unlock(&scheduler_mutex);
            break;
        }
        
        current_thread = selected_thread;
        
        // Execute for time slice
        usleep(TIME_SLICE);
        
        threads[selected_thread].remaining_time -= TIME_SLICE;
        current_thread = -1;
        
        pthread_mutex_unlock(&scheduler_mutex);
        
        sched_yield(); // Yield to scheduler
    }
    
    return NULL;
}

// CPU-affined worker thread
void* worker_thread(void* arg) {
    thread_info_t* info = (thread_info_t*)arg;
    
    while (info->remaining_time > 0) {
        pthread_mutex_lock(&scheduler_mutex);
        if (current_thread == info->thread_id) {
            // Performing actual work
            volatile int x = 0;
            for (int i = 0; i < 100000; i++) {
                x += i;
            }
        }
        pthread_mutex_unlock(&scheduler_mutex);
        
        usleep(1000); // Small sleep
    }
    
    printf("Thread %d completed\n", info->thread_id);
    return NULL;
}

int main() {
    printf("=== Advanced Process Management Lab ===\n");
    
    // Create scheduler thread
    pthread_t scheduler_thread;
    pthread_create(&scheduler_thread, NULL, priority_scheduler, NULL);
    
    // Create worker threads with different priorities
    num_threads = 4;
    for (int i = 0; i < num_threads; i++) {
        threads[i].thread_id = i;
        threads[i].priority = rand() % 5 + 1; // Priority 1-5
        threads[i].burst_time = 1000000 + (i * 500000); // Different burst times
        threads[i].remaining_time = threads[i].burst_time;
        clock_gettime(CLOCK_MONOTONIC, &threads[i].arrival_time);
        
        pthread_attr_t attr;
        pthread_attr_init(&attr);
        
        // Set CPU affinity to core 0
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        CPU_SET(0, &cpuset);
        pthread_attr_setaffinity_np(&attr, sizeof(cpu_set_t), &cpuset);
        
        pthread_create(&threads[i].thread, &attr, worker_thread, &threads[i]);
        pthread_attr_destroy(&attr);
    }
    
    // Wait for all threads to complete
    for (int i = 0; i < num_threads; i++) {
        pthread_join(threads[i].thread, NULL);
    }
    
    pthread_join(scheduler_thread, NULL);
    
    printf("\n=== Scheduling Results ===\n");
    printf("%-8s %-8s %-10s %-10s\n", "Thread", "Priority", "Burst Time", "Completion");
    for (int i = 0; i < num_threads; i++) {
        printf("%-8d %-8d %-10d %-10d\n", 
               threads[i].thread_id, 
               threads[i].priority, 
               threads[i].burst_time,
               threads[i].completion_time);
    }
    
    return 0;
}
```

#### Performance Analysis Solution
```bash
#!/bin/bash
# Challenge: Real-time task scheduling system

echo "=== Real-Time Task Scheduling Analysis ==="

# Compile and run scheduling test
gcc -pthread -O2 scheduler_test.c -o scheduler_test
./scheduler_test > scheduling_results.txt

# Analyze scheduling behavior
echo "Scheduling Performance Analysis:"
echo "==============================="

# Calculate average waiting time
awk '
BEGIN { total_wait = 0; count = 0 }
{
    if (NF >= 4) {
        burst = $3
        wait = burst - 1000000  # Simplified calculation
        total_wait += wait
        count++
    }
}
END { if (count > 0) print "Average Wait Time:", total_wait/count/1000, "ms" }
' scheduling_results.txt

# Generate Gantt chart
echo
echo "Scheduling Gantt Chart:"
echo "======================"
awk '
BEGIN { gantt = "" }
{
    if (NF >= 2) {
        priority = $2
        gantt = gantt sprintf("Thread %d: |%s|\n", $1, repeat("=", priority * 10))
    }
}
END { print gantt }
' scheduling_results.txt

echo "Real-time scheduling analysis completed!"
```

---

### Lab 32: Memory Management Deep Dive

#### Solution Overview
Implementation of custom memory allocators with NUMA awareness and performance monitoring.

#### Implementation Solution
```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <sys/mman.h>
#include <numa.h>
#include <numaif.h>

#define MEMORY_POOL_SIZE (1024 * 1024 * 1024) // 1GB
#define CHUNK_SIZE 4096
#define MAX_CHUNKS (MEMORY_POOL_SIZE / CHUNK_SIZE)

// Memory chunk structure
typedef struct chunk {
    size_t size;
    int is_free;
    struct chunk* next;
    int numa_node;
} chunk_t;

// NUMA-aware memory pool
typedef struct {
    void* pool_base;
    size_t pool_size;
    chunk_t* free_list;
    int num_numa_nodes;
    size_t allocated;
    size_t freed;
    pthread_mutex_t lock;
} memory_pool_t;

memory_pool_t global_pool;

// Initialize NUMA-aware memory pool
int init_memory_pool(memory_pool_t* pool, size_t size) {
    pool->pool_size = size;
    pool->allocated = 0;
    pool->freed = 0;
    pool->num_numa_nodes = numa_num_configured_nodes();
    
    // Allocate memory on node 0 first
    pool->pool_base = mmap(NULL, size, 
                          PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS | MAP_POPULATE,
                          -1, 0);
    
    if (pool->pool_base == MAP_FAILED) {
        perror("mmap failed");
        return -1;
    }
    
    // Initialize free list
    pool->free_list = (chunk_t*)pool->pool_base;
    pool->free_list->size = size - sizeof(chunk_t);
    pool->free_list->is_free = 1;
    pool->free_list->next = NULL;
    pool->free_list->numa_node = 0;
    
    pthread_mutex_init(&pool->lock, NULL);
    return 0;
}

// NUMA-aware allocation
void* numa_alloc(memory_pool_t* pool, size_t size, int preferred_node) {
    pthread_mutex_lock(&pool->lock);
    
    // Align size to chunk size
    size = (size + sizeof(chunk_t) + CHUNK_SIZE - 1) & ~(CHUNK_SIZE - 1);
    
    chunk_t* current = pool->free_list;
    chunk_t* prev = NULL;
    
    // Find suitable chunk on preferred NUMA node
    while (current) {
        if (current->is_free && current->size >= size && 
            current->numa_node == preferred_node) {
            
            // Split chunk if necessary
            if (current->size > size + sizeof(chunk_t)) {
                chunk_t* new_chunk = (chunk_t*)((char*)current + sizeof(chunk_t) + size);
                new_chunk->size = current->size - sizeof(chunk_t) - size;
                new_chunk->is_free = 1;
                new_chunk->next = current->next;
                new_chunk->numa_node = current->numa_node;
                
                current->size = size;
                current->next = new_chunk;
            }
            
            // Remove from free list
            if (prev) {
                prev->next = current->next;
            } else {
                pool->free_list = current->next;
            }
            
            current->is_free = 0;
            pool->allocated += current->size;
            
            pthread_mutex_unlock(&pool->lock);
            return (char*)current + sizeof(chunk_t);
        }
        
        prev = current;
        current = current->next;
    }
    
    pthread_mutex_unlock(&pool->lock);
    return NULL; // Allocation failed
}

// Memory deallocation
void numa_free(void* ptr) {
    if (!ptr) return;
    
    chunk_t* chunk = (chunk_t*)((char*)ptr - sizeof(chunk_t));
    
    pthread_mutex_lock(&global_pool.lock);
    chunk->is_free = 1;
    global_pool.freed += chunk->size;
    
    // Add to free list (simplified - no coalescing)
    chunk->next = global_pool.free_list;
    global_pool.free_list = chunk;
    
    pthread_mutex_unlock(&global_pool.lock);
}

// Performance monitoring
void print_memory_stats(memory_pool_t* pool) {
    printf("Memory Pool Statistics:\n");
    printf("=======================\n");
    printf("Pool Size: %zu bytes\n", pool->pool_size);
    printf("Allocated: %zu bytes\n", pool->allocated);
    printf("Freed: %zu bytes\n", pool->freed);
    printf("Available: %zu bytes\n", pool->pool_size - pool->allocated + pool->freed);
    printf("NUMA Nodes: %d\n", pool->num_numa_nodes);
}

int main() {
    printf("=== Memory Management Deep Dive ===\n");
    
    // Initialize memory pool
    if (init_memory_pool(&global_pool, MEMORY_POOL_SIZE) != 0) {
        fprintf(stderr, "Failed to initialize memory pool\n");
        return 1;
    }
    
    printf("Memory pool initialized successfully\n");
    print_memory_stats(&global_pool);
    
    // Test NUMA-aware allocation
    printf("\n=== Testing NUMA-aware allocation ===\n");
    
    void* ptrs[10];
    for (int i = 0; i < 10; i++) {
        int node = i % 2; // Alternate between nodes 0 and 1
        ptrs[i] = numa_alloc(&global_pool, 1024 * 1024, node);
        if (ptrs[i]) {
            printf("Allocated 1MB on node %d: %p\n", node, ptrs[i]);
        }
    }
    
    print_memory_stats(&global_pool);
    
    // Free half of the allocations
    printf("\n=== Freeing memory ===\n");
    for (int i = 0; i < 5; i++) {
        numa_free(ptrs[i]);
        printf("Freed pointer %p\n", ptrs[i]);
    }
    
    print_memory_stats(&global_pool);
    
    // Performance test
    printf("\n=== Performance Test ===\n");
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < 1000; i++) {
        void* ptr = numa_alloc(&global_pool, 4096, 0);
        if (ptr) {
            memset(ptr, 0, 4096);
            numa_free(ptr);
        }
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (end.tv_sec - start.tv_sec) + 
                    (end.tv_nsec - start.tv_nsec) / 1e9;
    
    printf("1000 allocations/deallocations took %.6f seconds\n", elapsed);
    printf("Average time per operation: %.6f ms\n", elapsed * 1000);
    
    return 0;
}
```

---

### Lab 40: Device Driver Programming Basics

#### Solution Overview
Complete character device driver implementation with interrupt handling and sysfs interface.

#### Implementation Solution
```c
/*
 * Basic Character Device Driver
 * Demonstrates device registration, file operations, and sysfs interface
 */

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/device.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/interrupt.h>
#include <linux/irq.h>
#include <linux/workqueue.h>

#define DEVICE_NAME "mychar_device"
#define CLASS_NAME "mychar_class"
#define BUFFER_SIZE 1024

static int major_number;
static struct class* char_class = NULL;
static struct device* char_device = NULL;
static struct cdev char_cdev;

// Device data
static char* device_buffer;
static int buffer_size = BUFFER_SIZE;
static atomic_t device_open_count = ATOMIC_INIT(0);

// Work queue for deferred processing
static struct workqueue_struct* work_queue;
static struct work_struct deferred_work;

// IRQ and interrupt handling (simulated)
static int irq_number = 1; // Simulated IRQ number
static int interrupt_count = 0;

// Device file operations
static int device_open(struct inode* inode, struct file* filp) {
    if (atomic_inc_return(&device_open_count) == 1) {
        printk(KERN_INFO "mychar_device: Device opened\n");
    }
    
    try_module_get(THIS_MODULE);
    return 0;
}

static int device_release(struct inode* inode, struct file* filp) {
    atomic_dec(&device_open_count);
    printk(KERN_INFO "mychar_device: Device closed\n");
    
    module_put(THIS_MODULE);
    return 0;
}

static ssize_t device_read(struct file* filp, char __user* buffer, size_t len, loff_t* offset) {
    size_t bytes_to_read = len;
    
    if (*offset >= buffer_size) {
        return 0;
    }
    
    if (*offset + bytes_to_read > buffer_size) {
        bytes_to_read = buffer_size - *offset;
    }
    
    if (copy_to_user(buffer, device_buffer + *offset, bytes_to_read)) {
        return -EFAULT;
    }
    
    *offset += bytes_to_read;
    return bytes_to_read;
}

static ssize_t device_write(struct file* filp, const char __user* buffer, size_t len, loff_t* offset) {
    size_t bytes_to_write = len;
    
    if (*offset >= buffer_size) {
        return -ENOSPC;
    }
    
    if (*offset + bytes_to_write > buffer_size) {
        bytes_to_write = buffer_size - *offset;
    }
    
    if (copy_from_user(device_buffer + *offset, buffer, bytes_to_write)) {
        return -EFAULT;
    }
    
    *offset += bytes_to_write;
    return bytes_to_write;
}

// Sysfs show function
static ssize_t device_show(struct device* dev, struct device_attribute* attr, char* buf) {
    ssize_t ret = 0;
    
    if (strcmp(attr->attr.name, "buffer_size") == 0) {
        ret = sprintf(buf, "%d\n", buffer_size);
    } else if (strcmp(attr->attr.name, "open_count") == 0) {
        ret = sprintf(buf, "%d\n", atomic_read(&device_open_count));
    } else if (strcmp(attr->attr.name, "interrupt_count") == 0) {
        ret = sprintf(buf, "%d\n", interrupt_count);
    }
    
    return ret;
}

// Sysfs store function
static ssize_t device_store(struct device* dev, struct device_attribute* attr, const char* buf, size_t count) {
    ssize_t ret = count;
    
    if (strcmp(attr->attr.name, "buffer_size") == 0) {
        sscanf(buf, "%d", &buffer_size);
        printk(KERN_INFO "mychar_device: Buffer size changed to %d\n", buffer_size);
    }
    
    return ret;
}

// Device attributes
static DEVICE_ATTR(buffer_size, S_IRUGO | S_IWUSR, device_show, device_store);
static DEVICE_ATTR(open_count, S_IRUGO, device_show, NULL);
static DEVICE_ATTR(interrupt_count, S_IRUGO, device_show, NULL);

// Deferred work function
static void deferred_work_handler(struct work_struct* work) {
    printk(KERN_INFO "mychar_device: Processing deferred work\n");
    // Add your deferred processing logic here
}

// Interrupt handler
static irqreturn_t device_irq_handler(int irq, void* dev_id) {
    interrupt_count++;
    
    // Schedule deferred work
    if (work_queue) {
        queue_work(work_queue, &deferred_work);
    }
    
    printk(KERN_INFO "mychar_device: Interrupt received, count: %d\n", interrupt_count);
    
    return IRQ_HANDLED;
}

// File operations structure
static struct file_operations fops = {
    .owner = THIS_MODULE,
    .open = device_open,
    .release = device_release,
    .read = device_read,
    .write = device_write,
};

// Module initialization
static int __init mychar_init(void) {
    int ret;
    
    printk(KERN_INFO "mychar_device: Initializing device driver\n");
    
    // Allocate device number
    major_number = register_chrdev(0, DEVICE_NAME, &fops);
    if (major_number < 0) {
        printk(KERN_ALERT "mychar_device: Failed to register a major number\n");
        return major_number;
    }
    
    printk(KERN_INFO "mychar_device: Registered with major number %d\n", major_number);
    
    // Register device class
    char_class = class_create(THIS_MODULE, CLASS_NAME);
    if (IS_ERR(char_class)) {
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mychar_device: Failed to create device class\n");
        return PTR_ERR(char_class);
    }
    
    // Register device driver
    char_device = device_create(char_class, NULL, MKDEV(major_number, 0), NULL, DEVICE_NAME);
    if (IS_ERR(char_device)) {
        class_destroy(char_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mychar_device: Failed to create device\n");
        return PTR_ERR(char_device);
    }
    
    // Initialize character device
    cdev_init(&char_cdev, &fops);
    char_cdev.owner = THIS_MODULE;
    
    ret = cdev_add(&char_cdev, MKDEV(major_number, 0), 1);
    if (ret < 0) {
        device_destroy(char_class, MKDEV(major_number, 0));
        class_destroy(char_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mychar_device: Failed to add character device\n");
        return ret;
    }
    
    // Create device attributes
    ret = device_create_file(char_device, &dev_attr_buffer_size);
    if (ret) {
        printk(KERN_WARNING "mychar_device: Failed to create buffer_size attribute\n");
    }
    
    ret = device_create_file(char_device, &dev_attr_open_count);
    if (ret) {
        printk(KERN_WARNING "mychar_device: Failed to create open_count attribute\n");
    }
    
    ret = device_create_file(char_device, &dev_attr_interrupt_count);
    if (ret) {
        printk(KERN_WARNING "mychar_device: Failed to create interrupt_count attribute\n");
    }
    
    // Allocate device buffer
    device_buffer = kmalloc(BUFFER_SIZE, GFP_KERNEL);
    if (!device_buffer) {
        printk(KERN_ALERT "mychar_device: Failed to allocate memory\n");
        return -ENOMEM;
    }
    
    memset(device_buffer, 0, BUFFER_SIZE);
    
    // Create work queue
    work_queue = create_workqueue("mychar_work_queue");
    if (!work_queue) {
        printk(KERN_ALERT "mychar_device: Failed to create work queue\n");
        kfree(device_buffer);
        return -ENOMEM;
    }
    
    INIT_WORK(&deferred_work, deferred_work_handler);
    
    // Request interrupt (simulated)
    ret = request_irq(irq_number, device_irq_handler, IRQF_SHARED, DEVICE_NAME, (void*)&char_device);
    if (ret) {
        printk(KERN_WARNING "mychar_device: Failed to request interrupt %d\n", irq_number);
    }
    
    printk(KERN_INFO "mychar_device: Device driver loaded successfully\n");
    return 0;
}

// Module cleanup
static void __exit mychar_exit(void) {
    printk(KERN_INFO "mychar_device: Cleaning up device driver\n");
    
    // Free interrupt
    free_irq(irq_number, (void*)&char_device);
    
    // Destroy work queue
    if (work_queue) {
        destroy_workqueue(work_queue);
    }
    
    // Remove device attributes
    device_remove_file(char_device, &dev_attr_buffer_size);
    device_remove_file(char_device, &dev_attr_open_count);
    device_remove_file(char_device, &dev_attr_interrupt_count);
    
    // Free device buffer
    kfree(device_buffer);
    
    // Remove character device
    cdev_del(&char_cdev);
    
    // Destroy device
    device_destroy(char_class, MKDEV(major_number, 0));
    class_destroy(char_class);
    
    // Unregister device number
    unregister_chrdev(major_number, DEVICE_NAME);
    
    printk(KERN_INFO "mychar_device: Device driver unloaded\n");
}

module_init(mychar_init);
module_exit(mychar_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MultiOS Education");
MODULE_DESCRIPTION("Basic Character Device Driver");
MODULE_VERSION("1.0");
```

#### User-space Test Program
```c
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

#define DEVICE "/dev/mychar_device"

int main() {
    int fd;
    char write_buffer[100];
    char read_buffer[100];
    
    printf("=== Character Device Driver Test ===\n");
    
    // Open device
    fd = open(DEVICE, O_RDWR);
    if (fd < 0) {
        perror("Failed to open device");
        return 1;
    }
    
    printf("Device opened successfully\n");
    
    // Write to device
    strcpy(write_buffer, "Hello from user space!");
    ssize_t bytes_written = write(fd, write_buffer, strlen(write_buffer));
    if (bytes_written < 0) {
        perror("Failed to write to device");
        close(fd);
        return 1;
    }
    
    printf("Wrote %zd bytes to device\n", bytes_written);
    
    // Read from device
    memset(read_buffer, 0, sizeof(read_buffer));
    ssize_t bytes_read = read(fd, read_buffer, sizeof(read_buffer) - 1);
    if (bytes_read < 0) {
        perror("Failed to read from device");
        close(fd);
        return 1;
    }
    
    printf("Read %zd bytes from device: %s\n", bytes_read, read_buffer);
    
    // Close device
    close(fd);
    printf("Device closed successfully\n");
    
    return 0;
}
```

#### Makefile for Building
```makefile
obj-m += mychar_device.o

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean

install:
	sudo insmod mychar_device.ko

uninstall:
	sudo rmmod mychar_device

test:
	gcc test_program.c -o test_program
	sudo ./test_program

.PHONY: all clean install uninstall test
```

---

## 📚 Solution Guide Usage Instructions

### For Educators
1. **Review Solutions First**: Understand the solution approaches before teaching
2. **Encourage Alternatives**: Students should explore multiple solution paths
3. **Focus on Best Practices**: Emphasize code quality and design principles
4. **Adapt for Your Environment**: Modify solutions to fit your specific requirements

### For Students
1. **Attempt First**: Always try to solve problems before viewing solutions
2. **Understand the Logic**: Don't just copy code, understand the concepts
3. **Experiment**: Modify solutions and explore alternatives
4. **Document Your Work**: Keep notes on what you learn

### For Self-Assessment
1. **Compare Approaches**: Review multiple solution approaches
2. **Identify Improvements**: Look for optimization opportunities
3. **Test Thoroughly**: Verify solutions work in different scenarios
4. **Extend Solutions**: Add features and improvements

---

This comprehensive solution guide provides detailed solutions for the MultiOS teaching resource library, demonstrating best practices and multiple approaches to each problem. The solutions serve as both teaching aids and reference materials for students and educators.