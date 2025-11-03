# MultiOS Code Examples Repository

## Overview

This repository contains comprehensive code examples, exercises, and hands-on tutorials for learning MultiOS development. Examples are organized by difficulty level and topic area, with progressive complexity to guide your learning journey.

## Directory Structure

```
interactive_materials/
├── code_examples/
│   ├── beginner/
│   ├── intermediate/
│   ├── advanced/
│   └── expert/
├── exercises/
│   ├── programming/
│   ├── system/
│   ├── network/
│   └── performance/
├── lab_assignments/
│   ├── basic/
│   ├── intermediate/
│   ├── advanced/
│   └── capstone/
├── quizzes/
│   ├── knowledge/
│   ├── practical/
│   └── assessment/
└── certificates/
    ├── requirements/
    ├── tracking/
    └── templates/
```

## Beginner Level Examples

### Basic System Calls

#### Hello World System Call
```c
#include <multios/multios.h>
#include <stdio.h>

int main(void) {
    // Write to standard output
    multios_write(STDOUT_FILENO, "Hello from MultiOS!\n", 19);
    
    // Exit successfully
    multios_exit(0);
    return 0;
}
```

**Compilation:**
```bash
gcc -o hello hello.c -lmultios
./hello
```

#### File Operations
```c
#include <multios/multios.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>

int main(void) {
    // Create a file
    int fd = multios_open("example.txt", 
                         MULTIOS_O_CREAT | MULTIOS_O_WRONLY | MULTIOS_O_TRUNC,
                         0644);
    
    if (fd < 0) {
        perror("Failed to create file");
        return 1;
    }
    
    // Write data
    const char *data = "Hello, MultiOS file system!\n";
    ssize_t written = multios_write(fd, data, strlen(data));
    
    if (written < 0) {
        perror("Failed to write to file");
        multios_close(fd);
        return 1;
    }
    
    // Close file
    multios_close(fd);
    
    // Read the file back
    fd = multios_open("example.txt", MULTIOS_O_RDONLY);
    if (fd < 0) {
        perror("Failed to open file for reading");
        return 1;
    }
    
    char buffer[256];
    ssize_t bytes_read = multios_read(fd, buffer, sizeof(buffer) - 1);
    if (bytes_read > 0) {
        buffer[bytes_read] = '\0';
        printf("Read from file: %s", buffer);
    }
    
    multios_close(fd);
    return 0;
}
```

### Process Management

#### Process Creation with Fork
```c
#include <multios/multios.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>

int main(void) {
    pid_t pid = multios_fork();
    
    if (pid < 0) {
        perror("Fork failed");
        return 1;
    }
    
    if (pid == 0) {
        // Child process
        printf("Child process: PID = %d, Parent PID = %d\n", 
               multios_getpid(), multios_getppid());
        
        // Execute another program
        char *args[] = {"/bin/ls", "-l", NULL};
        multios_exec("/bin/ls", args);
        
        // This code only runs if exec fails
        perror("Exec failed");
        multios_exit(1);
    } else {
        // Parent process
        printf("Parent process: PID = %d, Child PID = %d\n", 
               multios_getpid(), pid);
        
        // Wait for child to complete
        int status;
        pid_t result = multios_wait(&status);
        
        if (result > 0) {
            printf("Child process %d exited with status %d\n", 
                   result, WEXITSTATUS(status));
        }
    }
    
    return 0;
}
```

#### Process Communication with Pipes
```c
#include <multios/multios.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    int pipefd[2];
    pid_t pid;
    
    // Create pipe
    if (multios_pipe(pipefd) < 0) {
        perror("Pipe creation failed");
        return 1;
    }
    
    pid = multios_fork();
    
    if (pid < 0) {
        perror("Fork failed");
        return 1;
    }
    
    if (pid == 0) {
        // Child process - write to pipe
        close(pipefd[0]); // Close read end
        
        const char *message = "Hello from child process!\n";
        write(pipefd[1], message, strlen(message));
        close(pipefd[1]);
        
        multios_exit(0);
    } else {
        // Parent process - read from pipe
        close(pipefd[1]); // Close write end
        
        char buffer[256];
        ssize_t bytes_read = read(pipefd[0], buffer, sizeof(buffer) - 1);
        
        if (bytes_read > 0) {
            buffer[bytes_read] = '\0';
            printf("Parent received: %s", buffer);
        }
        
        close(pipefd[0]);
        
        // Wait for child
        int status;
        multios_wait(&status);
    }
    
    return 0;
}
```

### Memory Management

#### Dynamic Memory Allocation
```c
#include <multios/multios.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    // Allocate memory for an array of integers
    size_t count = 10;
    int *numbers = (int *)multios_alloc(count * sizeof(int));
    
    if (!numbers) {
        perror("Memory allocation failed");
        return 1;
    }
    
    // Initialize the array
    for (size_t i = 0; i < count; i++) {
        numbers[i] = i * i;
    }
    
    // Print the array
    printf("Array of squares:\n");
    for (size_t i = 0; i < count; i++) {
        printf("%zu^2 = %d\n", i, numbers[i]);
    }
    
    // Resize the allocation
    count = 15;
    int *new_numbers = (int *)multios_realloc(numbers, count * sizeof(int));
    
    if (!new_numbers) {
        perror("Memory reallocation failed");
        multios_free(numbers);
        return 1;
    }
    
    numbers = new_numbers;
    
    // Initialize additional elements
    for (size_t i = 10; i < count; i++) {
        numbers[i] = i * i;
    }
    
    printf("\nExtended array:\n");
    for (size_t i = 0; i < count; i++) {
        printf("%zu^2 = %d\n", i, numbers[i]);
    }
    
    // Free memory
    multios_free(numbers);
    
    return 0;
}
```

#### Memory-Mapped Files
```c
#include <multios/multios.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    // Create a file for memory mapping
    int fd = multios_open("mmap_example.txt", 
                         MULTIOS_O_CREAT | MULTIOS_O_RDWR,
                         0644);
    
    if (fd < 0) {
        perror("Failed to create file");
        return 1;
    }
    
    // Write some initial data
    const char *initial_data = "Initial data in file\n";
    write(fd, initial_data, strlen(initial_data));
    
    // Memory map the file
    size_t map_size = 4096; // One page
    void *mapped = multios_mmap(NULL, map_size, 
                               PROT_READ | PROT_WRITE,
                               MAP_SHARED, fd, 0);
    
    if (mapped == MAP_FAILED) {
        perror("Memory mapping failed");
        close(fd);
        return 1;
    }
    
    // Write to the memory-mapped region
    const char *new_data = "Modified through memory mapping!\n";
    strcpy((char *)mapped, new_data);
    
    // Synchronize changes to disk
    multios_msync(mapped, map_size, MS_SYNC);
    
    printf("Written to memory-mapped file: %s", (char *)mapped);
    
    // Unmap the memory
    multios_munmap(mapped, map_size);
    close(fd);
    
    return 0;
}
```

## Intermediate Level Examples

### Threading and Synchronization

#### Thread Creation and Joining
```c
#include <multios/thread.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int start;
    int end;
    long long result;
} thread_data_t;

// Thread function
void *calculate_sum(void *arg) {
    thread_data_t *data = (thread_data_t *)arg;
    long long sum = 0;
    
    for (int i = data->start; i < data->end; i++) {
        sum += i;
    }
    
    data->result = sum;
    return NULL;
}

int main(void) {
    multios_thread_t thread1, thread2;
    thread_data_t data1, data2;
    
    // Calculate sum of 1 to 500000 in two threads
    data1.start = 1;
    data1.end = 250000;
    data1.result = 0;
    
    data2.start = 250001;
    data2.end = 500000;
    data2.result = 0;
    
    // Create threads
    if (multios_thread_create(&thread1, calculate_sum, &data1) != 0) {
        perror("Failed to create thread 1");
        return 1;
    }
    
    if (multios_thread_create(&thread2, calculate_sum, &data2) != 0) {
        perror("Failed to create thread 2");
        return 1;
    }
    
    // Wait for threads to complete
    multios_thread_join(thread1, NULL);
    multios_thread_join(thread2, NULL);
    
    long long total_sum = data1.result + data2.result;
    printf("Sum of 1 to 500000: %lld\n", total_sum);
    
    return 0;
}
```

#### Producer-Consumer with Condition Variables
```c
#include <multios/thread.h>
#include <multios/mutex.h>
#include <multios/cond.h>
#include <stdio.h>
#include <stdlib.h>

#define BUFFER_SIZE 10

typedef struct {
    int buffer[BUFFER_SIZE];
    int count;
    int in;
    int out;
    
    multios_mutex_t mutex;
    multios_cond_t not_full;
    multios_cond_t not_empty;
} shared_data_t;

shared_data_t shared;

void producer(void *arg) {
    int id = *(int *)arg;
    
    for (int i = 0; i < 20; i++) {
        int item = id * 1000 + i;
        
        multios_mutex_lock(&shared.mutex);
        
        // Wait while buffer is full
        while (shared.count == BUFFER_SIZE) {
            multios_cond_wait(&shared.not_full, &shared.mutex);
        }
        
        // Add item to buffer
        shared.buffer[shared.in] = item;
        shared.in = (shared.in + 1) % BUFFER_SIZE;
        shared.count++;
        
        printf("Producer %d produced: %d (count: %d)\n", id, item, shared.count);
        
        multios_cond_signal(&shared.not_empty);
        multios_mutex_unlock(&shared.mutex);
        
        // Simulate some work
        multios_usleep(10000);
    }
}

void consumer(void *arg) {
    int id = *(int *)arg;
    
    for (int i = 0; i < 20; i++) {
        multios_mutex_lock(&shared.mutex);
        
        // Wait while buffer is empty
        while (shared.count == 0) {
            multios_cond_wait(&shared.not_empty, &shared.mutex);
        }
        
        // Remove item from buffer
        int item = shared.buffer[shared.out];
        shared.out = (shared.out + 1) % BUFFER_SIZE;
        shared.count--;
        
        printf("Consumer %d consumed: %d (count: %d)\n", id, item, shared.count);
        
        multios_cond_signal(&shared.not_full);
        multios_mutex_unlock(&shared.mutex);
        
        // Simulate some work
        multios_usleep(15000);
    }
}

int main(void) {
    // Initialize shared data
    shared.count = 0;
    shared.in = 0;
    shared.out = 0;
    
    multios_mutex_init(&shared.mutex, NULL);
    multios_cond_init(&shared.not_full, NULL);
    multios_cond_init(&shared.not_empty, NULL);
    
    multios_thread_t producer_thread, consumer_thread;
    int producer_id = 1, consumer_id = 1;
    
    // Create producer and consumer threads
    multios_thread_create(&producer_thread, producer, &producer_id);
    multios_thread_create(&consumer_thread, consumer, &consumer_id);
    
    // Wait for threads to complete
    multios_thread_join(producer_thread, NULL);
    multios_thread_join(consumer_thread, NULL);
    
    // Cleanup
    multios_mutex_destroy(&shared.mutex);
    multios_cond_destroy(&shared.not_full);
    multios_cond_destroy(&shared.not_empty);
    
    return 0;
}
```

### Network Programming

#### TCP Client-Server
```c
// server.c
#include <multios/network.h>
#include <multios/thread.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

typedef struct {
    int client_fd;
    struct multios_sockaddr_in client_addr;
} client_info_t;

void *handle_client(void *arg) {
    client_info_t *info = (client_info_t *)arg;
    char buffer[256];
    
    printf("New client connected from %s:%d\n",
           multios_inet_ntoa(info->client_addr.sin_addr),
           multios_ntohs(info->client_addr.sin_port));
    
    // Handle client communication
    while (1) {
        ssize_t bytes_read = multios_read(info->client_fd, buffer, sizeof(buffer) - 1);
        
        if (bytes_read <= 0) {
            break; // Client disconnected or error
        }
        
        buffer[bytes_read] = '\0';
        printf("Received: %s", buffer);
        
        // Echo back to client
        multios_write(info->client_fd, buffer, bytes_read);
    }
    
    printf("Client disconnected\n");
    multios_close(info->client_fd);
    free(info);
    return NULL;
}

int main(void) {
    int server_fd, client_fd;
    struct multios_sockaddr_in server_addr, client_addr;
    socklen_t client_len;
    
    // Create socket
    server_fd = multios_socket(MULTIOS_AF_INET, MULTIOS_SOCK_STREAM, 0);
    
    if (server_fd < 0) {
        perror("Socket creation failed");
        return 1;
    }
    
    // Configure server address
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = MULTIOS_AF_INET;
    server_addr.sin_addr.s_addr = MULTIOS_INADDR_ANY;
    server_addr.sin_port = multios_htons(8080);
    
    // Bind socket
    if (multios_bind(server_fd, (struct multios_sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("Bind failed");
        multios_close(server_fd);
        return 1;
    }
    
    // Listen for connections
    multios_listen(server_fd, 5);
    printf("Server listening on port 8080...\n");
    
    // Accept connections
    while (1) {
        client_len = sizeof(client_addr);
        client_fd = multios_accept(server_fd, (struct multios_sockaddr *)&client_addr, &client_len);
        
        if (client_fd < 0) {
            perror("Accept failed");
            continue;
        }
        
        // Create thread to handle client
        client_info_t *info = malloc(sizeof(client_info_t));
        info->client_fd = client_fd;
        info->client_addr = client_addr;
        
        multios_thread_t client_thread;
        multios_thread_create(&client_thread, handle_client, info);
        multios_thread_detach(client_thread);
    }
    
    multios_close(server_fd);
    return 0;
}
```

```c
// client.c
#include <multios/network.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(void) {
    int sock_fd;
    struct multios_sockaddr_in server_addr;
    char buffer[256];
    
    // Create socket
    sock_fd = multios_socket(MULTIOS_AF_INET, MULTIOS_SOCK_STREAM, 0);
    
    if (sock_fd < 0) {
        perror("Socket creation failed");
        return 1;
    }
    
    // Configure server address
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = MULTIOS_AF_INET;
    server_addr.sin_port = multios_htons(8080);
    
    // Convert IP address
    if (multios_inet_pton(MULTIOS_AF_INET, "127.0.0.1", &server_addr.sin_addr) <= 0) {
        perror("Invalid address");
        multios_close(sock_fd);
        return 1;
    }
    
    // Connect to server
    if (multios_connect(sock_fd, (struct multios_sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("Connection failed");
        multios_close(sock_fd);
        return 1;
    }
    
    printf("Connected to server\n");
    
    // Communication loop
    while (1) {
        printf("Enter message: ");
        if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
            break;
        }
        
        // Send to server
        multios_write(sock_fd, buffer, strlen(buffer));
        
        // Receive response
        ssize_t bytes_read = multios_read(sock_fd, buffer, sizeof(buffer) - 1);
        if (bytes_read <= 0) {
            break;
        }
        
        buffer[bytes_read] = '\0';
        printf("Server response: %s", buffer);
    }
    
    multios_close(sock_fd);
    return 0;
}
```

## Advanced Level Examples

### Device Driver Development

#### Character Device Driver
```c
#include <multios/driver.h>
#include <multios/char_device.h>
#include <multios/mutex.h>
#include <multios/atomic.h>
#include <stdio.h>
#include <string.h>

#define DEVICE_NAME "example_device"
#define BUFFER_SIZE 1024

typedef struct {
    char buffer[BUFFER_SIZE];
    size_t read_pos;
    size_t write_pos;
    size_t count;
    multios_mutex_t mutex;
    multios_atomic_t open_count;
} example_device_t;

static example_device_t *example_dev;

static ssize_t example_read(struct multios_char_device *dev,
                           char *buffer, size_t count) {
    example_device_t *priv = dev->private_data;
    size_t bytes_read = 0;
    
    multios_mutex_lock(&priv->mutex);
    
    if (priv->count == 0) {
        multios_mutex_unlock(&priv->mutex);
        return 0; // No data available
    }
    
    // Read data from buffer
    while (count > 0 && priv->count > 0) {
        buffer[bytes_read++] = priv->buffer[priv->read_pos];
        priv->read_pos = (priv->read_pos + 1) % BUFFER_SIZE;
        priv->count--;
        count--;
    }
    
    multios_mutex_unlock(&priv->mutex);
    
    return bytes_read;
}

static ssize_t example_write(struct multios_char_device *dev,
                            const char *buffer, size_t count) {
    example_device_t *priv = dev->private_data;
    size_t bytes_written = 0;
    
    multios_mutex_lock(&priv->mutex);
    
    // Check if enough space
    if (priv->count + count > BUFFER_SIZE) {
        multios_mutex_unlock(&priv->mutex);
        return -ENOSPC; // No space
    }
    
    // Write data to buffer
    while (count > 0) {
        size_t space_available = BUFFER_SIZE - priv->count;
        size_t to_write = (count < space_available) ? count : space_available;
        
        for (size_t i = 0; i < to_write; i++) {
            priv->buffer[priv->write_pos] = buffer[bytes_written + i];
            priv->write_pos = (priv->write_pos + 1) % BUFFER_SIZE;
        }
        
        priv->count += to_write;
        bytes_written += to_write;
        count -= to_write;
    }
    
    multios_mutex_unlock(&priv->mutex);
    
    return bytes_written;
}

static int example_open(struct multios_char_device *dev) {
    example_device_t *priv = dev->private_data;
    multios_atomic_inc(&priv->open_count);
    return 0;
}

static int example_release(struct multios_char_device *dev) {
    example_device_t *priv = dev->private_data;
    multios_atomic_dec(&priv->open_count);
    return 0;
}

static struct multios_char_device_ops example_char_ops = {
    .read = example_read,
    .write = example_write,
    .open = example_open,
    .release = example_release,
};

int example_driver_init(void) {
    // Allocate device structure
    example_dev = malloc(sizeof(example_device_t));
    if (!example_dev) {
        return -ENOMEM;
    }
    
    // Initialize device
    memset(example_dev, 0, sizeof(example_device_t));
    multios_mutex_init(&example_dev->mutex, NULL);
    multios_atomic_set(&example_dev->open_count, 0);
    
    // Create character device
    struct multios_char_device *char_dev = multios_char_device_create(DEVICE_NAME, &example_char_ops);
    if (!char_dev) {
        free(example_dev);
        return -ENOMEM;
    }
    
    char_dev->private_data = example_dev;
    
    printf("Example driver loaded\n");
    return 0;
}

void example_driver_exit(void) {
    if (example_dev) {
        multios_char_device_destroy(example_dev);
        multios_mutex_destroy(&example_dev->mutex);
        free(example_dev);
    }
    
    printf("Example driver unloaded\n");
}
```

### Kernel Module Development

#### Basic Kernel Module
```c
#include <multios/module.h>
#include <multios/kernel.h>
#include <multios/device.h>
#include <stdio.h>

static int __init example_module_init(void) {
    printk(KERN_INFO "Example module: Loading module\n");
    
    // Module initialization code
    return 0;
}

static void __exit example_module_exit(void) {
    printk(KERN_INFO "Example module: Unloading module\n");
    
    // Module cleanup code
}

module_init(example_module_init);
module_exit(example_module_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MultiOS Developer");
MODULE_DESCRIPTION("Example kernel module");
MODULE_VERSION("1.0");
```

#### Advanced Kernel Module with Proc Interface
```c
#include <multios/module.h>
#include <multios/kernel.h>
#include <multios/proc.h>
#include <multios/atomic.h>
#include <multios/slab.h>

#define PROC_NAME "example_module"
#define MAX_COUNTER 1000000

static atomic_t counter = ATOMIC_INIT(0);

static ssize_t proc_read(struct multios_proc_file *file, char __user *buffer,
                        size_t count, loff_t *ppos) {
    char buf[256];
    int len;
    
    if (*ppos > 0) {
        return 0; // Already read
    }
    
    len = sprintf(buf, "Counter value: %d\n", atomic_read(&counter));
    
    if (copy_to_user(buffer, buf, len)) {
        return -EFAULT;
    }
    
    *ppos = len;
    return len;
}

static ssize_t proc_write(struct multios_proc_file *file, const char __user *buffer,
                         size_t count, loff_t *ppos) {
    char buf[256];
    int len;
    int value;
    
    len = (count < sizeof(buf) - 1) ? count : sizeof(buf) - 1;
    
    if (copy_from_user(buf, buffer, len)) {
        return -EFAULT;
    }
    
    buf[len] = '\0';
    
    if (sscanf(buf, "%d", &value) == 1) {
        if (value >= 0 && value <= MAX_COUNTER) {
            atomic_set(&counter, value);
            printk(KERN_INFO "Counter set to %d\n", value);
        } else {
            printk(KERN_WARNING "Invalid counter value: %d\n", value);
            return -EINVAL;
        }
    } else {
        printk(KERN_WARNING "Invalid input format\n");
        return -EINVAL;
    }
    
    return count;
}

static struct multios_proc_ops example_proc_ops = {
    .proc_read = proc_read,
    .proc_write = proc_write,
};

static int __init example_module_init(void) {
    struct multios_proc_file *proc_file;
    
    printk(KERN_INFO "Example module: Initializing\n");
    
    // Create proc entry
    proc_file = multios_proc_create(PROC_NAME, &example_proc_ops);
    if (!proc_file) {
        printk(KERN_ERR "Failed to create proc entry\n");
        return -ENOMEM;
    }
    
    printk(KERN_INFO "Example module: Module loaded successfully\n");
    return 0;
}

static void __exit example_module_exit(void) {
    multios_proc_remove(PROC_NAME);
    printk(KERN_INFO "Example module: Module unloaded\n");
}

module_init(example_module_init);
module_exit(example_module_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MultiOS Developer");
MODULE_DESCRIPTION("Example module with proc interface");
MODULE_VERSION("2.0");
```

## Expert Level Examples

### Custom System Call Implementation

#### System Call Handler
```c
#include <multios/kernel.h>
#include <multios/thread.h>
#include <multios/mm.h>
#include <multios/errno.h>

/**
 * Custom system call: multios_get_thread_info
 * Returns information about the current thread
 */
int64_t multios_syscall_get_thread_info(struct multios_thread_info *info) {
    struct multios_thread *current;
    
    if (!info) {
        return -EFAULT;
    }
    
    current = multios_get_current_thread();
    if (!current) {
        return -ESRCH;
    }
    
    // Copy thread information to user space
    struct multios_thread_info kernel_info;
    kernel_info.pid = current->pid;
    kernel_info.tid = current->tid;
    kernel_info.state = current->state;
    kernel_info.priority = current->priority;
    kernel_info.stack_usage = current->stack_used;
    
    if (copy_to_user(info, &kernel_info, sizeof(kernel_info))) {
        return -EFAULT;
    }
    
    return 0;
}

/**
 * System call: multios_cache_flush
 * Flushes CPU caches for memory region
 */
int64_t multios_syscall_cache_flush(void *addr, size_t size) {
    struct multios_thread *current;
    
    if (!addr || size == 0) {
        return -EINVAL;
    }
    
    current = multios_get_current_thread();
    if (!current) {
        return -ESRCH;
    }
    
    // Validate memory address
    if (!multios_access_ok(addr, size)) {
        return -EFAULT;
    }
    
    // Perform cache flush (architecture-specific)
    multios_cache_flush_range((unsigned long)addr, size);
    
    return 0;
}

/**
 * System call registration
 */
static struct multios_syscall_table syscall_table[] = {
    { .name = "get_thread_info", .handler = multios_syscall_get_thread_info },
    { .name = "cache_flush", .handler = multios_syscall_cache_flush },
    // Add more system calls here
};

static int __init syscall_init(void) {
    int i;
    
    for (i = 0; i < ARRAY_SIZE(syscall_table); i++) {
        if (multios_syscall_register(&syscall_table[i]) < 0) {
            printk(KERN_ERR "Failed to register syscall %s\n", 
                   syscall_table[i].name);
            return -1;
        }
    }
    
    printk(KERN_INFO "Custom system calls registered\n");
    return 0;
}

static void __exit syscall_exit(void) {
    int i;
    
    for (i = 0; i < ARRAY_SIZE(syscall_table); i++) {
        multios_syscall_unregister(syscall_table[i].name);
    }
    
    printk(KERN_INFO "Custom system calls unregistered\n");
}

module_init(syscall_init);
module_exit(syscall_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MultiOS Developer");
MODULE_DESCRIPTION("Custom system call implementation");
```

### Virtual Memory Manager Extension

#### Custom Memory Allocator
```c
#include <multios/mm.h>
#include <multios/spinlock.h>
#include <multios/slab.h>
#include <multios/list.h>

#define ALLOCATOR_MAGIC 0xDEADBEEF
#define MIN_ORDER 3  // 8 bytes
#define MAX_ORDER 20 // 1MB

typedef struct free_area {
    struct list_head free_list[MMAX_ORDER];
    unsigned long nr_free;
} free_area_t;

typedef struct alloc_header {
    unsigned int magic;
    unsigned int order;
    struct list_head list;
} alloc_header_t;

static free_area_t free_areas[MAX_ORDER];
static DEFINE_SPINLOCK(allocator_lock);

static void add_to_free_list(void *ptr, unsigned int order) {
    alloc_header_t *header = (alloc_header_t *)ptr - 1;
    header->magic = ALLOCATOR_MAGIC;
    header->order = order;
    
    list_add(&header->list, &free_areas[order].free_list);
    free_areas[order].nr_free++;
}

static void *remove_from_free_list(unsigned int order) {
    if (list_empty(&free_areas[order].free_list)) {
        return NULL;
    }
    
    alloc_header_t *header = list_first_entry(&free_areas[order].free_list,
                                             alloc_header_t, list);
    list_del(&header->list);
    free_areas[order].nr_free--;
    
    return (void *)header + sizeof(alloc_header_t);
}

void *custom_alloc(size_t size) {
    unsigned int order;
    unsigned long flags;
    void *ptr;
    
    if (size == 0) {
        return NULL;
    }
    
    // Calculate required order
    size += sizeof(alloc_header_t);
    for (order = MIN_ORDER; order <= MAX_ORDER; order++) {
        if ((PAGE_SIZE << order) >= size) {
            break;
        }
    }
    
    if (order > MAX_ORDER) {
        return NULL; // Request too large
    }
    
    spin_lock_irqsave(&allocator_lock, flags);
    
    // Try to allocate from free list
    ptr = remove_from_free_list(order);
    if (ptr) {
        spin_unlock_irqrestore(&allocator_lock, flags);
        return ptr;
    }
    
    // Split higher order blocks
    for (int i = order + 1; i <= MAX_ORDER; i++) {
        void *block = remove_from_free_list(i);
        if (block) {
            // Split block and add remainder to free list
            size_t block_size = PAGE_SIZE << i;
            size_t half_size = block_size >> 1;
            
            add_to_free_list(block + half_size, i - 1);
            ptr = block;
            order = i - 1;
            break;
        }
    }
    
    spin_unlock_irqrestore(&allocator_lock, flags);
    return ptr;
}

void custom_free(void *ptr) {
    unsigned int flags;
    alloc_header_t *header;
    unsigned int order;
    
    if (!ptr) {
        return;
    }
    
    header = (alloc_header_t *)ptr - 1;
    
    // Validate magic number
    if (header->magic != ALLOCATOR_MAGIC) {
        // Invalid pointer, could be corruption
        return;
    }
    
    order = header->order;
    
    spin_lock_irqsave(&allocator_lock, flags);
    add_to_free_list(ptr, order);
    
    // Try to merge with adjacent free blocks
    // (Implementation would be more complex in practice)
    
    spin_unlock_irqrestore(&allocator_lock, flags);
}

static int __init allocator_init(void) {
    // Initialize free area lists
    for (int i = 0; i <= MAX_ORDER; i++) {
        INIT_LIST_HEAD(&free_areas[i].free_list);
        free_areas[i].nr_free = 0;
    }
    
    printk(KERN_INFO "Custom memory allocator initialized\n");
    return 0;
}

module_init(allocator_init);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MultiOS Developer");
MODULE_DESCRIPTION("Custom memory allocator");
```

## Code Quality Standards

### Documentation Requirements
- All functions must have Doxygen-style comments
- Include parameter descriptions
- Document return values
- Add usage examples

### Error Handling
- Always check return values
- Use appropriate error codes
- Clean up resources in error paths
- Log errors appropriately

### Memory Management
- Free all allocated memory
- Check allocation failures
- Use appropriate allocation functions
- Avoid memory leaks

### Thread Safety
- Protect shared data with locks
- Avoid deadlocks
- Use appropriate synchronization primitives
- Consider lock-free alternatives

### Performance Considerations
- Choose appropriate algorithms
- Minimize system calls
- Use efficient data structures
- Profile code for bottlenecks

---

This repository provides a comprehensive foundation for learning MultiOS development through practical examples and exercises. Each example builds upon previous concepts while introducing new challenges and real-world applications.