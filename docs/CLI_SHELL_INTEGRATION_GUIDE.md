# MultiOS CLI Shell Integration Guide

## Build Configuration

### Adding CLI Shell to MultiOS Kernel

1. **Add to kernel dependencies** in `kernel/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
cli-shell = { path = "../services/cli-service" }
```

2. **Initialize in kernel startup** in `kernel/src/lib.rs`:

```rust
// Add to the kernel_main function after service initialization
crate::services::cli_service::init()?;
crate::services::cli_service::start()?;

// For batch mode
// crate::services::cli_service::start_batch_mode("/path/to/script")?;
```

3. **Add to system services** in `kernel/src/services/mod.rs`:

```rust
pub mod cli_service;
pub mod cli_script_interpreter;
pub mod cli_application;

// Add to init() function:
// cli_service::init()?;
// cli_script_interpreter::init()?;
// cli_application::init()?;
```

## Service Integration

### Registering with Service Manager

The CLI service integrates seamlessly with MultiOS service management:

```rust
use crate::service_manager::{ServiceManager, ServiceDescriptor, ServiceType};

fn register_cli_service() {
    let descriptor = ServiceDescriptor {
        name: "cli-service".to_string(),
        display_name: "MultiOS CLI Service".to_string(),
        description: "Command-line interface and shell".to_string(),
        service_type: ServiceType::SystemService,
        dependencies: vec![], // Depends on other core services
        resource_limits: None,
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 5000,
        max_restarts: 3,
        health_check_interval: 30000,
        tags: vec!["cli".to_string(), "shell".to_string(), "interface".to_string()],
    };
    
    let service_id = ServiceManager::register_service(descriptor);
    
    // Start the service
    ServiceManager::start_service(service_id);
}
```

### Health Check Integration

```rust
fn cli_health_check() -> HealthCheckResult {
    // Check if CLI service is responsive
    let result = CLI_SERVICE.lock()
        .as_mut()
        .ok_or(HealthCheckError::ServiceUnavailable)?
        .get_stats();
    
    if result.total_commands_executed > 0 {
        Ok(HealthStatus::Healthy)
    } else {
        Ok(HealthStatus::Degraded)
    }
}
```

## User Interface Integration

### Terminal Driver Interface

```rust
use crate::drivers::terminal::{TerminalDriver, TerminalEvent};

pub struct CliTerminalInterface {
    terminal: TerminalDriver,
    cli_service: Arc<Mutex<CliService>>,
}

impl CliTerminalInterface {
    pub fn new() -> Self {
        CliTerminalInterface {
            terminal: TerminalDriver::new(),
            cli_service: CLI_SERVICE.lock().unwrap().clone(),
        }
    }
    
    pub fn start_interactive_loop(&mut self) {
        loop {
            // Display prompt
            self.terminal.write_str("MultiOS> ");
            
            // Read user input
            let input = self.read_line();
            
            // Process command
            if let Some(mut service) = self.cli_service.lock().as_mut() {
                let result = service.execute_command(&input);
                
                // Display output
                if let Ok(output) = result {
                    if !output.output.is_empty() {
                        self.terminal.write_str(&output.output);
                    }
                }
            }
        }
    }
    
    fn read_line(&self) -> String {
        // Implementation would read from terminal driver
        String::new() // Placeholder
    }
}
```

### Serial Console Integration

```rust
pub struct SerialConsole {
    port: SerialPort,
    cli_interface: CliTerminalInterface,
}

impl SerialConsole {
    pub fn new(port: SerialPort) -> Self {
        SerialConsole {
            port,
            cli_interface: CliTerminalInterface::new(),
        }
    }
    
    pub fn start(&mut self) {
        // Send welcome message
        self.send_string("MultiOS CLI Shell v1.0.0\n");
        self.send_string("Type 'help' for available commands.\n\n");
        
        // Start CLI loop
        self.cli_interface.start_interactive_loop();
    }
    
    fn send_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.port.write_byte(byte);
        }
    }
}
```

## File System Integration

### Virtual File System (VFS) Commands

```rust
pub struct VfsCliCommands;

impl VfsCliCommands {
    fn register_vfs_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "mount",
            description: "Mount filesystem",
            usage: "mount <device> <mountpoint> <filesystem_type>",
            min_args: 3,
            max_args: 3,
            handler: Self::mount_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "umount",
            description: "Unmount filesystem",
            usage: "umount <mountpoint>",
            min_args: 1,
            max_args: 1,
            handler: Self::umount_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "df",
            description: "Show filesystem disk space usage",
            usage: "df [filesystem]",
            min_args: 0,
            max_args: 1,
            handler: Self::df_handler,
            builtin: true,
        });
    }
    
    fn mount_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        let device = &args[0];
        let mountpoint = &args[1];
        let fstype = &args[2];
        
        // Mount filesystem using VFS
        let result = crate::filesystem::vfs::mount(device, mountpoint, fstype);
        
        match result {
            Ok(_) => Ok(CommandResult {
                success: true,
                output: format!("Mounted {} on {}", device, mountpoint),
                exit_code: 0,
                duration_ms: 0,
            }),
            Err(e) => Ok(CommandResult {
                success: false,
                output: format!("Mount failed: {:?}", e),
                exit_code: 1,
                duration_ms: 0,
            }),
        }
    }
}
```

### File Operations Commands

```rust
pub struct FileCliCommands;

impl FileCliCommands {
    fn register_file_commands(cli_service: &mut CliService) {
        // Already implemented in cli_service.rs:
        // - ls, cd, pwd, cat, mkdir, touch, rm
    }
}
```

## Process Management Integration

### Scheduler Integration

```rust
pub struct ProcessCliCommands;

impl ProcessCliCommands {
    fn register_process_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "ps",
            description: "Display running processes",
            usage: "ps [options]",
            min_args: 0,
            max_args: 5,
            handler: Self::ps_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "kill",
            description: "Terminate process",
            usage: "kill <signal> <pid>",
            min_args: 1,
            max_args: 2,
            handler: Self::kill_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "jobs",
            description: "Display background jobs",
            usage: "jobs",
            min_args: 0,
            max_args: 0,
            handler: Self::jobs_handler,
            builtin: true,
        });
    }
    
    fn ps_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        // Get process list from scheduler
        let processes = crate::scheduler::get_all_processes();
        
        let mut output = String::new();
        output.push_str("  PID TTY          TIME CMD\n");
        
        for process in processes {
            output.push_str(&format!(
                "{:5} {:<12} {:8} {}\n",
                process.pid,
                process.tty,
                format_time(process.cpu_time),
                process.name
            ));
        }
        
        Ok(CommandResult {
            success: true,
            output,
            exit_code: 0,
            duration_ms: 5,
        })
    }
    
    fn kill_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        let signal = if args.len() > 1 { &args[0] } else { "TERM" };
        let pid = args[if args.len() > 1 { 1 } else { 0 }].parse::<u32>();
        
        match pid {
            Ok(pid) => {
                let result = crate::scheduler::send_signal(pid, signal);
                match result {
                    Ok(_) => Ok(CommandResult {
                        success: true,
                        output: format!("Sent signal {} to process {}", signal, pid),
                        exit_code: 0,
                        duration_ms: 2,
                    }),
                    Err(e) => Ok(CommandResult {
                        success: false,
                        output: format!("Failed to send signal: {:?}", e),
                        exit_code: 1,
                        duration_ms: 2,
                    }),
                }
            }
            Err(_) => Ok(CommandResult {
                success: false,
                output: "Invalid PID".to_string(),
                exit_code: 1,
                duration_ms: 1,
            }),
        }
    }
}
```

## Network Integration

### Network Service Commands

```rust
pub struct NetworkCliCommands;

impl NetworkCliCommands {
    fn register_network_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "ifconfig",
            description: "Configure network interfaces",
            usage: "ifconfig [interface] [options]",
            min_args: 0,
            max_args: 10,
            handler: Self::ifconfig_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "ping",
            description: "Test network connectivity",
            usage: "ping <host> [count]",
            min_args: 1,
            max_args: 2,
            handler: Self::ping_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "netstat",
            description: "Show network connections",
            usage: "netstat [options]",
            min_args: 0,
            max_args: 5,
            handler: Self::netstat_handler,
            builtin: true,
        });
    }
    
    fn ifconfig_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        // Get network interface information
        let interfaces = crate::services::network_service::get_interfaces();
        
        let mut output = String::new();
        
        if args.is_empty() {
            // Show all interfaces
            for interface in interfaces {
                output.push_str(&format!("{}: flags={} mtu {}\n",
                    interface.name,
                    interface.flags,
                    interface.mtu));
                output.push_str(&format!("        inet {} netmask {}\n",
                    interface.address,
                    interface.netmask));
            }
        } else {
            // Show specific interface
            let interface_name = &args[0];
            // Find and display specific interface
        }
        
        Ok(CommandResult {
            success: true,
            output,
            exit_code: 0,
            duration_ms: 10,
        })
    }
    
    fn ping_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        let host = &args[0];
        let count = args.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(4);
        
        // Test connectivity using network service
        let mut success_count = 0;
        
        for i in 0..count {
            let result = crate::services::network_service::ping(host, 1000);
            if result.is_ok() {
                success_count += 1;
            }
            
            if i < count - 1 {
                sleep(1);
            }
        }
        
        let output = format!("PING {}: {} data bytes\n{} packets transmitted, {} packets received\n",
            host, 64, count, success_count);
        
        Ok(CommandResult {
            success: true,
            output,
            exit_code: if success_count == count { 0 } else { 1 },
            duration_ms: count * 1000,
        })
    }
}
```

## Device Driver Integration

### Hardware Management Commands

```rust
pub struct DeviceCliCommands;

impl DeviceCliCommands {
    fn register_device_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "lsblk",
            description: "List block devices",
            usage: "lsblk",
            min_args: 0,
            max_args: 0,
            handler: Self::lsblk_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "lspci",
            description: "List PCI devices",
            usage: "lspci",
            min_args: 0,
            max_args: 0,
            handler: Self::lspci_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "lsusb",
            description: "List USB devices",
            usage: "lsusb",
            min_args: 0,
            max_args: 0,
            handler: Self::lsusb_handler,
            builtin: true,
        });
    }
    
    fn lsblk_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        // Get block device information from storage drivers
        let devices = crate::drivers::storage::get_block_devices();
        
        let mut output = String::new();
        output.push_str("NAME   MAJ:MIN RM SIZE RO TYPE MOUNTPOINT\n");
        
        for device in devices {
            output.push_str(&format!("{} {}:{} {} {} {} {}\n",
                device.name,
                device.major,
                device.minor,
                if device.removable { "1" } else { "0" },
                device.size,
                if device.readonly { "ro" } else { "rw" },
                device.mountpoint
            ));
        }
        
        Ok(CommandResult {
            success: true,
            output,
            exit_code: 0,
            duration_ms: 5,
        })
    }
}
```

## Memory Management Integration

### Memory Commands

```rust
pub struct MemoryCliCommands;

impl MemoryCliCommands {
    fn register_memory_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "free",
            description: "Display memory usage",
            usage: "free [options]",
            min_args: 0,
            max_args: 3,
            handler: Self::free_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "vmstat",
            description: "Virtual memory statistics",
            usage: "vmstat [delay] [count]",
            min_args: 0,
            max_args: 2,
            handler: Self::vmstat_handler,
            builtin: true,
        });
    }
    
    fn free_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        // Get memory statistics from memory manager
        let mem_stats = crate::memory::get_memory_stats();
        
        let total_kb = mem_stats.total_pages * 4; // Assuming 4KB pages
        let used_kb = mem_stats.used_pages * 4;
        let free_kb = mem_stats.free_pages * 4;
        let available_kb = mem_stats.available_pages * 4;
        
        let output = format!(
            "              total        used        free      shared  buff/cache   available\n\
             Mem:        {:8} {:8} {:8} {:8} {:8} {:8}\n\
             Swap:       {:8} {:8} {:8} {:8} {:8} {:8}\n",
            total_kb, used_kb, free_kb, 0, total_kb - used_kb, available_kb,
            0, 0, 0, 0, 0, 0 // No swap in this example
        );
        
        Ok(CommandResult {
            success: true,
            output,
            exit_code: 0,
            duration_ms: 8,
        })
    }
}
```

## Configuration Management

### System Configuration

```rust
pub struct ConfigCliCommands;

impl ConfigCliCommands {
    fn register_config_commands(cli_service: &mut CliService) {
        cli_service.register_builtin_command(CliCommand {
            name: "systemctl",
            description: "Control system services",
            usage: "systemctl <command> [service]",
            min_args: 1,
            max_args: 2,
            handler: Self::systemctl_handler,
            builtin: true,
        });
        
        cli_service.register_builtin_command(CliCommand {
            name: "config",
            description: "System configuration management",
            usage: "config <command> [key] [value]",
            min_args: 1,
            max_args: 3,
            handler: Self::config_handler,
            builtin: true,
        });
    }
    
    fn systemctl_handler(args: &[String], context: &CliContext) -> CliResult<CommandResult> {
        let command = &args[0];
        let service = args.get(1);
        
        match command.as_str() {
            "start" => {
                if let Some(service_name) = service {
                    let result = crate::service_manager::start_service_by_name(service_name);
                    match result {
                        Ok(_) => Ok(CommandResult {
                            success: true,
                            output: format!("Service {} started", service_name),
                            exit_code: 0,
                            duration_ms: 100,
                        }),
                        Err(e) => Ok(CommandResult {
                            success: false,
                            output: format!("Failed to start service: {:?}", e),
                            exit_code: 1,
                            duration_ms: 100,
                        }),
                    }
                } else {
                    Ok(CommandResult {
                        success: false,
                        output: "Service name required".to_string(),
                        exit_code: 1,
                        duration_ms: 1,
                    })
                }
            }
            "stop" => {
                // Similar implementation for stop
                Ok(CommandResult {
                    success: true,
                    output: "Service stop command processed".to_string(),
                    exit_code: 0,
                    duration_ms: 50,
                })
            }
            "status" => {
                // Get service status
                let services = crate::service_manager::get_all_services();
                Ok(CommandResult {
                    success: true,
                    output: "Service status retrieved".to_string(),
                    exit_code: 0,
                    duration_ms: 10,
                })
            }
            _ => Ok(CommandResult {
                success: false,
                output: format!("Unknown systemctl command: {}", command),
                exit_code: 1,
                duration_ms: 1,
            }),
        }
    }
}
```

## Boot Integration

### Early Boot Shell

```rust
pub struct BootShell;

impl BootShell {
    pub fn start_early_shell() {
        // Start minimal CLI for early boot
        let config = CliApplicationConfig {
            enable_interactive_mode: true,
            enable_batch_mode: false,
            enable_scripting: false, // Disable scripting for early boot
            max_concurrent_sessions: 1,
            default_session_timeout: 300000, // 5 minutes
            enable_debug_mode: true,
            auto_save_history: false,
            history_file_path: "/tmp/boot_history".to_string(),
            config_file_path: "/etc/boot.conf".to_string(),
            log_level: "debug".to_string(),
            prompt_format: "boot> ".to_string(),
            completion_style: "minimal".to_string(),
        };
        
        // Start with boot-specific commands only
        start(config).expect("Failed to start boot shell");
    }
}
```

This integration guide provides a comprehensive overview of how to integrate the MultiOS CLI Shell with various system components. The CLI system is designed to be modular and extensible, allowing for easy integration with new services and drivers as the MultiOS ecosystem grows.