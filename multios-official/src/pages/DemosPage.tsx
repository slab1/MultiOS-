import React, { useState } from 'react';

const DemosPage = () => {
  const [activeDemo, setActiveDemo] = useState('terminal');

  const demoCategories = [
    { id: 'terminal', label: 'Interactive Terminal', icon: '💻' },
    { id: 'process', label: 'Process Management', icon: '⚙️' },
    { id: 'memory', label: 'Memory Visualization', icon: '🧠' },
    { id: 'filesystem', label: 'File System Explorer', icon: '📁' },
    { id: 'drivers', label: 'Driver Management', icon: '🔧' }
  ];

  const demoContent = {
    terminal: {
      title: "Interactive Terminal",
      description: "Experience MultiOS command line interface with real-time feedback and system commands.",
      features: [
        "Real-time command execution",
        "System information display",
        "File and process navigation",
        "Built-in help system"
      ],
      code: `MultiOS Shell v1.0.0

$ uname -a
MultiOS 1.0.0 x86_64 MultiOS
$ free -h
              total        used        free      shared  buff/cache   available
Mem:           8G         1.2G         6.1G         256K         700M         6.6G
$ ps aux
PID   USER     COMMAND
1     root     /sbin/init
2     root     [kthreadd]
45     root     [kworker/0:2]
$ echo "Welcome to MultiOS Interactive Demo!"
Welcome to MultiOS Interactive Demo!`
    },
    process: {
      title: "Process Management",
      description: "Visual demonstration of process creation, scheduling, and termination.",
      features: [
        "Process creation and termination",
        "Priority-based scheduling",
        "CPU usage visualization",
        "Real-time process monitoring"
      ],
      code: `Process Management Demo
=========================

Process Tree:
init (PID: 1, Priority: 0, CPU: 2%)
├─ bash (PID: 42, Priority: 5, CPU: 1%)
├─ chrome (PID: 100, Priority: 3, CPU: 15%)
│   └─ renderer (PID: 101, Priority: 2, CPU: 8%)
└─ multios-demo (PID: 200, Priority: 4, CPU: 3%)

Scheduler Information:
- Ready Queue: 3 processes
- Running: 2 processes
- Waiting: 5 processes
- Completed: 156 processes

Creating new process...
fork() successful, child PID: 201
Setting priority to 3
Scheduling on CPU 1...`
    },
    memory: {
      title: "Memory Management",
      description: "Interactive visualization of virtual memory, page allocation, and memory protection.",
      features: [
        "Virtual address space mapping",
        "Page table visualization",
        "Memory allocation tracking",
        "Protection violation simulation"
      ],
      code: `Memory Management Demo
==========================

Physical Memory: 8GB (0x00000000 - 0x1FFFFFFF)
Virtual Memory per Process: 4TB

Memory Map:
[0x00000000] Kernel Code       512MB  RW-
[0x20000000] Kernel Data       1GB    RW-
[0x80000000] Heap              2GB    RW-
[0x100000000] Stack            8MB    RW-
[0x200000000] Shared Library   256MB  R-X
[0x400000000] Mapped Files     512MB  RW-

Page Table Entries:
Entry 4K Page @ 0x80020000: PRESENT, RW, USER
Entry 4K Page @ 0x80021000: PRESENT, RW, USER
Entry 4K Page @ 0x80022000: PRESENT, RW, USER

Allocating 128MB of heap memory...
Mapping pages: 0x80080000 - 0x80800000 (32768 pages)
Allocation successful!`
    },
    filesystem: {
      title: "File System Explorer",
      description: "Navigate through MultiOS file system structure and operations.",
      features: [
        "Directory structure navigation",
        "File creation and editing",
        "Permission management",
        "Disk usage visualization"
      ],
      code: `MultiOS File System (MultiFS v1.0)
====================================

/home/user/
├── Documents/
│   ├── project.md          2.4KB
│   ├── thesis.pdf          12.8MB
│   └── code/
│       ├── main.rs         1.2KB
│       └── lib.rs          856B
├── Downloads/
│   ├── multios-1.0.iso     512MB
│   └── rust-toolchain.tar  45.2MB
└── .config/
    ├── bashrc              234B
    └── neovim/
        └── init.lua        156B

Total disk usage: 580.5MB (21% of available space)

Creating new file: hello.txt
Writing "Hello from MultiOS!" to hello.txt
File created successfully!

Setting permissions: rwxr-xr-x (755)
File permissions updated.`
    },
    drivers: {
      title: "Driver Management",
      description: "Real-time display of device drivers and hardware detection.",
      features: [
        "Hardware detection and initialization",
        "Driver status monitoring",
        "Interrupt handling visualization",
        "DMA operation tracking"
      ],
      code: `Driver Management System
===========================

Hardware Detection Results:
- CPU: Intel Core i7-9700K (8 cores, 3.6GHz)
- Memory: 16GB DDR4-3200
- Graphics: Intel UHD Graphics 630
- Storage: NVMe SSD (Samsung 970 EVO Plus 1TB)
- Network: Intel I219-V Ethernet

Loaded Drivers:
✓ VGA Graphics Driver     [OK]   @ 0xFFFF800000020000
✓ AHCI SATA Controller    [OK]   @ 0xFFFF800000024000
✓ Intel I219-V Network    [OK]   @ 0xFFFF800000028000
✓ USB 3.0 Host Controller [OK]   @ 0xFFFF80000002C000
✓ Intel HDA Audio         [OK]   @ 0xFFFF800000030000

Driver Statistics:
- Total interrupts: 1,247,891
- DMA operations: 89,432
- Power management: Active

Initializing driver interface...
Driver framework ready for user requests.`
    }
  };

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Interactive Demos</h1>
          <p className="text-large max-w-3xl mx-auto">
            Experience MultiOS capabilities through interactive demonstrations. 
            Explore kernel features, process management, memory operations, and more in real-time.
          </p>
        </div>
      </section>

      {/* Demo Categories */}
      <section className="section">
        <div className="container">
          <div className="flex flex-wrap justify-center gap-4 mb-12">
            {demoCategories.map((category) => (
              <button
                key={category.id}
                onClick={() => setActiveDemo(category.id)}
                className={`px-6 py-3 text-sm font-bold uppercase tracking-wider transition-colors border-2 ${
                  activeDemo === category.id
                    ? 'border-red-600 text-red-600 bg-white'
                    : 'border-black text-black hover:bg-gray-50'
                }`}
              >
                <span className="mr-2">{category.icon}</span>
                {category.label}
              </button>
            ))}
          </div>

          {/* Demo Content */}
          <div className="max-w-5xl mx-auto">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
              {/* Demo Info */}
              <div className="lg:col-span-1">
                <h2 className="text-h2 mb-4">
                  {demoContent[activeDemo as keyof typeof demoContent].title}
                </h2>
                <p className="text-body mb-6">
                  {demoContent[activeDemo as keyof typeof demoContent].description}
                </p>
                <div className="space-y-3">
                  <h3 className="text-small font-bold uppercase tracking-wider text-gray-600">Key Features</h3>
                  {demoContent[activeDemo as keyof typeof demoContent].features.map((feature, index) => (
                    <div key={index} className="flex items-start">
                      <div className="w-2 h-2 bg-red-600 mt-2 mr-3 flex-shrink-0"></div>
                      <span className="text-body text-gray-600">{feature}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Demo Terminal */}
              <div className="lg:col-span-2">
                <div className="code-block" style={{ minHeight: '400px' }}>
                  <pre className="text-white text-sm leading-relaxed whitespace-pre-wrap">
                    {demoContent[activeDemo as keyof typeof demoContent].code}
                  </pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Code Examples */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <h2 className="text-h2 text-center mb-12">Code Examples</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div className="card">
              <h3 className="text-h3 mb-4">Process Creation</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::process;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a new process
    let child = process::fork()?;
    
    if child.is_child() {
        println!("Child process created");
        exec("/bin/sh")?;
    } else {
        println!("Parent process");
        child.wait()?;
    }
    
    Ok(())
}`}</pre>
              </div>
            </div>

            <div className="card">
              <h3 className="text-h3 mb-4">Memory Allocation</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::memory;

fn allocate_buffer(size: usize) -> Result<*mut u8> {
    // Allocate virtual memory
    let ptr = memory::alloc(size)?;
    
    // Map physical pages
    memory::map_pages(ptr, size)?;
    
    // Set permissions
    memory::set_permissions(ptr, size, 
        Permission::READ | Permission::WRITE)?;
    
    Ok(ptr)
}`}</pre>
              </div>
            </div>

            <div className="card">
              <h3 className="text-h3 mb-4">File Operations</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::fs;

fn read_config() -> Result<String> {
    let file = fs::File::open("/etc/config.conf")?;
    let mut buffer = String::new();
    
    file.read_to_string(&mut buffer)?;
    
    Ok(buffer)
}

fn write_log(message: &str) -> Result<()> {
    let file = fs::File::create("/var/log/app.log")?;
    file.write_all(message.as_bytes())?;
    
    Ok(())
}`}</pre>
              </div>
            </div>

            <div className="card">
              <h3 className="text-h3 mb-4">Driver Registration</h3>
              <div className="code-block">
                <pre className="text-sm text-white">{`use multios::drivers;

#[device_driver]
struct MyDevice {
    base_addr: *mut u8,
}

impl MyDevice {
    fn new(base_addr: usize) -> Self {
        Self {
            base_addr: base_addr as *mut u8,
        }
    }
    
    fn write(&self, data: u8) {
        unsafe {
            self.base_addr.write_volatile(data);
        }
    }
}`}</pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Try Now CTA */}
      <section className="section">
        <div className="container text-center">
          <h2 className="text-h2 mb-6">Ready to Try MultiOS?</h2>
          <p className="text-large mb-8 max-w-2xl mx-auto">
            These demos showcase a fraction of MultiOS capabilities. 
            Download the full system to experience the complete operating system.
          </p>
          <div className="flex flex-col sm:flex-row justify-center gap-4">
            <a href="/download" className="btn btn-primary btn-large">
              Download MultiOS
            </a>
            <a href="/developers" className="btn btn-secondary btn-large">
              View Documentation
            </a>
          </div>
        </div>
      </section>
    </div>
  );
};

export default DemosPage;