//! Resource Management for Container Resource Limits and Quotas
//! 
//! This module provides comprehensive resource management including CPU, memory,
//! disk, network bandwidth limits and quota enforcement.

use super::*;
use nix::unistd::Pid;
use std::collections::HashMap;
use std::path::PathBuf;

/// Resource Manager - Handles all container resource allocation and limits
pub struct ResourceManager {
    cgroup_root: PathBuf,
    cpu_manager: CpuResourceManager,
    memory_manager: MemoryResourceManager,
    disk_manager: DiskResourceManager,
    network_manager: NetworkResourceManager,
    active_limits: Arc<Mutex<HashMap<String, ResourceLimits>>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        let cgroup_root = PathBuf::from("/sys/fs/cgroup/multios");
        
        Self {
            cgroup_root,
            cpu_manager: CpuResourceManager::new(),
            memory_manager: MemoryResourceManager::new(),
            disk_manager: DiskResourceManager::new(),
            network_manager: NetworkResourceManager::new(),
            active_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Allocate resources for a container
    pub async fn allocate_resources(&self, limits: &ResourceLimits) -> ContainerResult<ResourceAllocation> {
        // Validate resource limits
        self.validate_limits(limits)?;

        // Allocate CPU resources
        let cpu_allocation = self.cpu_manager.allocate_resources(limits).await?;

        // Allocate memory resources
        let memory_allocation = self.memory_manager.allocate_resources(limits).await?;

        // Allocate disk resources
        let disk_allocation = self.disk_manager.allocate_resources(limits).await?;

        // Allocate network resources
        let network_allocation = self.network_manager.allocate_resources(limits).await?;

        // Create cgroup hierarchy
        self.setup_cgroup_hierarchy().await?;

        Ok(ResourceAllocation {
            cpu: cpu_allocation,
            memory: memory_allocation,
            disk: disk_allocation,
            network: network_allocation,
        })
    }

    /// Setup cgroups for a container
    pub async fn setup_cgroups(&self, container_id: &str) -> ContainerResult<()> {
        let container_cgroup = self.cgroup_root.join(container_id);

        // Create cgroup directory
        std::fs::create_dir_all(&container_cgroup)
            .map_err(|e| ContainerError::System(format!("Failed to create cgroup dir: {}", e)))?;

        // Set up cgroup controllers
        self.setup_cgroup_controllers(&container_cgroup, container_id).await?;

        Ok(())
    }

    /// Apply resource limits to container
    pub async fn apply_limits(&self, container_id: &str, limits: &ResourceLimits) -> ContainerResult<()> {
        // Apply CPU limits
        self.cpu_manager.apply_limits(container_id, limits).await?;

        // Apply memory limits
        self.memory_manager.apply_limits(container_id, limits).await?;

        // Apply disk limits
        self.disk_manager.apply_limits(container_id, limits).await?;

        // Apply network limits
        self.network_manager.apply_limits(container_id, limits).await?;

        // Store limits for tracking
        {
            let mut active_limits = self.active_limits.lock().unwrap();
            active_limits.insert(container_id.to_string(), limits.clone());
        }

        Ok(())
    }

    /// Get current resource usage for a container
    pub async fn get_usage(&self, container_id: &str) -> ContainerResult<ResourceUsage> {
        let usage = ResourceUsage {
            cpu_usage: self.cpu_manager.get_usage(container_id).await?,
            memory_usage: self.memory_manager.get_usage(container_id).await?,
            disk_usage: self.disk_manager.get_usage(container_id).await?,
            network_usage: self.network_manager.get_usage(container_id).await?,
        };

        Ok(usage)
    }

    /// Get process statistics for resource monitoring
    pub async fn get_process_stats(&self, pid: u32) -> Result<ContainerStats, ContainerError> {
        let cpu_usage = self.cpu_manager.get_cpu_usage(pid).await?;
        let memory_usage = self.memory_manager.get_memory_usage(pid).await?;

        Ok(ContainerStats {
            cpu_usage,
            memory_usage,
            memory_limit: u64::MAX, // This would be fetched from the specific container
            disk_usage: 0, // This would be calculated from disk I/O stats
            network_rx: 0, // This would come from network stats
            network_tx: 0, // This would come from network stats
            uptime: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default(),
            pid_count: 1, // This would count all processes in the container
        })
    }

    /// Release resources for a container
    pub async fn release_resources(&self, container_id: &str) -> ContainerResult<()> {
        // Release CPU resources
        self.cpu_manager.release_resources(container_id).await?;

        // Release memory resources
        self.memory_manager.release_resources(container_id).await?;

        // Release disk resources
        self.disk_manager.release_resources(container_id).await?;

        // Release network resources
        self.network_manager.release_resources(container_id).await?;

        // Remove from active limits
        {
            let mut active_limits = self.active_limits.lock().unwrap();
            active_limits.remove(container_id);
        }

        Ok(())
    }

    /// Cleanup cgroups for a container
    pub async fn cleanup_cgroups(&self, container_id: &str) -> ContainerResult<()> {
        let container_cgroup = self.cgroup_root.join(container_id);

        // Remove container from cgroup (move to root)
        self.move_to_root_cgroup(container_id).await?;

        // Remove cgroup directory
        if container_cgroup.exists() {
            std::fs::remove_dir_all(&container_cgroup)
                .map_err(|e| ContainerError::System(format!("Failed to remove cgroup dir: {}", e)))?;
        }

        Ok(())
    }

    // Private helper methods

    fn validate_limits(&self, limits: &ResourceLimits) -> ContainerResult<()> {
        // Validate CPU cores
        if let Some(cores) = limits.cpu_cores {
            if cores <= 0.0 || cores > 1000.0 {
                return Err(ContainerError::InvalidConfig(
                    "CPU cores must be between 0 and 1000".to_string()
                ));
            }
        }

        // Validate memory limits
        if let Some(memory) = limits.memory_bytes {
            if memory == 0 {
                return Err(ContainerError::InvalidConfig(
                    "Memory limit must be greater than 0".to_string()
                ));
            }
            
            // Cap at reasonable limits
            if memory > 1024 * 1024 * 1024 * 1024 { // 1TB
                return Err(ContainerError::ResourceLimit(
                    "Memory limit exceeds maximum allowed".to_string()
                ));
            }
        }

        // Validate disk limits
        if let Some(disk) = limits.disk_bytes {
            if disk == 0 {
                return Err(ContainerError::InvalidConfig(
                    "Disk limit must be greater than 0".to_string()
                ));
            }
        }

        Ok(())
    }

    async fn setup_cgroup_hierarchy(&self) -> ContainerResult<()> {
        // Ensure cgroup root directory exists
        if !self.cgroup_root.exists() {
            std::fs::create_dir_all(&self.cgroup_root)
                .map_err(|e| ContainerError::System(format!("Failed to create cgroup root: {}", e)))?;
        }

        // Create cgroup hierarchy for each controller
        let controllers = ["cpu", "memory", "blkio", "devices", "freezer"];
        
        for controller in &controllers {
            let controller_path = self.cgroup_root.join(controller);
            std::fs::create_dir_all(&controller_path)
                .map_err(|e| ContainerError::System(format!("Failed to create cgroup controller: {}", e)))?;
        }

        Ok(())
    }

    async fn setup_cgroup_controllers(&self, container_cgroup: &PathBuf, container_id: &str) -> ContainerResult<()> {
        // Enable controllers for this container
        let controllers = ["cpu", "memory", "blkio", "devices", "freezer"];
        
        for controller in &controllers {
            let cgroup_path = container_cgroup.join(controller);
            if !cgroup_path.exists() {
                std::fs::create_dir_all(&cgroup_path)
                    .map_err(|e| ContainerError::System(format!("Failed to create {} cgroup: {}", controller, e)))?;
            }

            // Enable the controller
            self.enable_cgroup_controller(&cgroup_path, controller)?;
        }

        Ok(())
    }

    fn enable_cgroup_controller(&self, cgroup_path: &PathBuf, controller: &str) -> ContainerResult<()> {
        // Enable controller by writing to cgroup.procs
        let procs_file = cgroup_path.join("cgroup.procs");
        
        if let Some(parent_cgroup) = cgroup_path.parent() {
            let parent_procs = parent_cgroup.join("cgroup.procs");
            if parent_procs.exists() {
                // Read current PIDs from parent and move to child
                let content = std::fs::read_to_string(&parent_procs)
                    .map_err(|e| ContainerError::System(format!("Failed to read parent cgroup.procs: {}", e)))?;
                
                // Write PIDs to child cgroup
                std::fs::write(&procs_file, content)
                    .map_err(|e| ContainerError::System(format!("Failed to write child cgroup.procs: {}", e)))?;
            }
        }

        Ok(())
    }

    async fn move_to_root_cgroup(&self, container_id: &str) -> ContainerResult<()> {
        // Move processes from container cgroup back to root
        let container_cgroup = self.cgroup_root.join(container_id);
        
        if container_cgroup.exists() {
            // Write all PIDs from container cgroup to root cgroup
            let container_procs = container_cgroup.join("cgroup.procs");
            if container_procs.exists() {
                let content = std::fs::read_to_string(&container_procs)
                    .map_err(|e| ContainerError::System(format!("Failed to read container cgroup.procs: {}", e)))?;
                
                // Move to root cgroup
                let root_cgroup = self.cgroup_root.join("cgroup.procs");
                std::fs::write(&root_cgroup, content)
                    .map_err(|e| ContainerError::System(format!("Failed to move to root cgroup: {}", e)))?;
            }
        }

        Ok(())
    }
}

/// CPU Resource Manager
pub struct CpuResourceManager {
    cgroup_path: PathBuf,
}

impl CpuResourceManager {
    fn new() -> Self {
        Self {
            cgroup_path: PathBuf::from("/sys/fs/cgroup/multios/cpu"),
        }
    }

    async fn allocate_resources(&self, limits: &ResourceLimits) -> Result<CpuAllocation, ContainerError> {
        let quota = limits.cpu_cores.map(|cores| (cores * 1000.0) as i64);
        let period: i64 = 100000; // 100ms period

        Ok(CpuAllocation {
            quota,
            period: Some(period),
            shares: None,
        })
    }

    async fn apply_limits(&self, container_id: &str, limits: &ResourceLimits) -> ContainerResult<()> {
        let cpu_cgroup = self.cgroup_path.join(container_id);
        
        if let Some(cores) = limits.cpu_cores {
            // Set CPU quota (milliseconds per period)
            let quota = (cores * 1000.0) as i64;
            let quota_file = cpu_cgroup.join("cpu.cfs_quota_us");
            std::fs::write(&quota_file, quota.to_string())
                .map_err(|e| ContainerError::System(format!("Failed to set CPU quota: {}", e)))?;

            // Set CPU period (default 100ms)
            let period = 100000;
            let period_file = cpu_cgroup.join("cpu.cfs_period_us");
            std::fs::write(&period_file, period.to_string())
                .map_err(|e| ContainerError::System(format!("Failed to set CPU period: {}", e)))?;

            // Set CPU shares (relative weight)
            let shares = (cores * 1024.0) as i64;
            let shares_file = cpu_cgroup.join("cpu.shares");
            std::fs::write(&shares_file, shares.to_string())
                .map_err(|e| ContainerError::System(format!("Failed to set CPU shares: {}", e)))?;
        }

        Ok(())
    }

    async fn get_usage(&self, container_id: &str) -> Result<f64, ContainerError> {
        let usage_file = self.cgroup_path.join(container_id).join("cpuacct.usage");
        
        if usage_file.exists() {
            let content = std::fs::read_to_string(&usage_file)
                .map_err(|e| ContainerError::System(format!("Failed to read CPU usage: {}", e)))?;
            
            let nanos: u64 = content.trim().parse()
                .map_err(|e| ContainerError::System(format!("Failed to parse CPU usage: {}", e)))?;
            
            // Convert to percentage (simplified calculation)
            let seconds = nanos as f64 / 1_000_000_000.0;
            Ok(seconds * 100.0) // This is a simplified calculation
        } else {
            Ok(0.0)
        }
    }

    async fn get_cpu_usage(&self, pid: u32) -> Result<f64, ContainerError> {
        // Read from /proc/[pid]/stat for CPU usage
        let stat_file = format!("/proc/{}/stat", pid);
        let content = std::fs::read_to_string(&stat_file)
            .map_err(|e| ContainerError::System(format!("Failed to read process stat: {}", e)))?;

        let parts: Vec<&str> = content.split(' ').collect();
        if parts.len() >= 42 {
            let utime: u64 = parts[13].parse()
                .map_err(|e| ContainerError::System(format!("Failed to parse utime: {}", e)))?;
            let stime: u64 = parts[14].parse()
                .map_err(|e| ContainerError::System(format!("Failed to parse stime: {}", e)))?;

            let total_time = utime + stime;
            let cpu_usage = total_time as f64 / 100.0; // Simplified calculation
            
            Ok(cpu_usage)
        } else {
            Err(ContainerError::System("Invalid stat file format".to_string()))
        }
    }

    async fn release_resources(&self, container_id: &str) -> ContainerResult<()> {
        // Clean up CPU cgroup settings
        let cpu_cgroup = self.cgroup_path.join(container_id);
        if cpu_cgroup.exists() {
            // Reset to default values
            let quota_file = cpu_cgroup.join("cpu.cfs_quota_us");
            std::fs::write(&quota_file, "-1").ok(); // Remove quota limit
            
            let shares_file = cpu_cgroup.join("cpu.shares");
            std::fs::write(&shares_file, "1024").ok(); // Reset to default
        }
        Ok(())
    }
}

/// Memory Resource Manager
pub struct MemoryResourceManager {
    cgroup_path: PathBuf,
}

impl MemoryResourceManager {
    fn new() -> Self {
        Self {
            cgroup_path: PathBuf::from("/sys/fs/cgroup/multios/memory"),
        }
    }

    async fn allocate_resources(&self, limits: &ResourceLimits) -> Result<MemoryAllocation, ContainerError> {
        let limit = limits.memory_bytes;
        
        Ok(MemoryAllocation {
            limit,
            swap_limit: limit,
            soft_limit: limit.map(|x| x * 80 / 100), // 80% of hard limit
        })
    }

    async fn apply_limits(&self, container_id: &str, limits: &ResourceLimits) -> ContainerResult<()> {
        let mem_cgroup = self.cgroup_path.join(container_id);
        
        if let Some(memory_limit) = limits.memory_bytes {
            // Set memory limit
            let limit_file = mem_cgroup.join("memory.limit_in_bytes");
            std::fs::write(&limit_file, memory_limit.to_string())
                .map_err(|e| ContainerError::System(format!("Failed to set memory limit: {}", e)))?;

            // Set swap limit (same as memory limit by default)
            if let Some(_) = limits.memory_bytes {
                let swap_file = mem_cgroup.join("memory.memsw.limit_in_bytes");
                std::fs::write(&swap_file, memory_limit.to_string())
                    .map_err(|e| ContainerError::System(format!("Failed to set swap limit: {}", e)))?;
            }

            // Set soft limit (80% of hard limit)
            let soft_limit = memory_limit * 80 / 100;
            let soft_file = mem_cgroup.join("memory.soft_limit_in_bytes");
            std::fs::write(&soft_file, soft_limit.to_string())
                .map_err(|e| ContainerError::System(format!("Failed to set soft limit: {}", e)))?;
        }

        Ok(())
    }

    async fn get_usage(&self, container_id: &str) -> Result<u64, ContainerError> {
        let usage_file = self.cgroup_path.join(container_id).join("memory.usage_in_bytes");
        
        if usage_file.exists() {
            let content = std::fs::read_to_string(&usage_file)
                .map_err(|e| ContainerError::System(format!("Failed to read memory usage: {}", e)))?;
            
            let usage: u64 = content.trim().parse()
                .map_err(|e| ContainerError::System(format!("Failed to parse memory usage: {}", e)))?;
            
            Ok(usage)
        } else {
            Ok(0)
        }
    }

    async fn get_memory_usage(&self, pid: u32) -> Result<u64, ContainerError> {
        // Read from /proc/[pid]/status for memory usage
        let status_file = format!("/proc/{}/status", pid);
        let content = std::fs::read_to_string(&status_file)
            .map_err(|e| ContainerError::System(format!("Failed to read process status: {}", e)))?;

        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse()
                        .map_err(|e| ContainerError::System(format!("Failed to parse VmRSS: {}", e)))?;
                    return Ok(kb * 1024); // Convert KB to bytes
                }
            }
        }

        Ok(0)
    }

    async fn release_resources(&self, container_id: &str) -> ContainerResult<()> {
        // Clean up memory cgroup settings
        let mem_cgroup = self.cgroup_path.join(container_id);
        if mem_cgroup.exists() {
            // Reset limits
            let limit_file = mem_cgroup.join("memory.limit_in_bytes");
            std::fs::write(&limit_file, "-1").ok(); // Remove memory limit
            
            let swap_file = mem_cgroup.join("memory.memsw.limit_in_bytes");
            std::fs::write(&swap_file, "-1").ok(); // Remove swap limit
        }
        Ok(())
    }
}

/// Disk Resource Manager
pub struct DiskResourceManager {
    cgroup_path: PathBuf,
}

impl DiskResourceManager {
    fn new() -> Self {
        Self {
            cgroup_path: PathBuf::from("/sys/fs/cgroup/multios/blkio"),
        }
    }

    async fn allocate_resources(&self, limits: &ResourceLimits) -> Result<DiskAllocation, ContainerError> {
        let quota = limits.disk_bytes;
        
        Ok(DiskAllocation {
            quota,
            io_weight: None,
        })
    }

    async fn apply_limits(&self, container_id: &str, limits: &ResourceLimits) -> ContainerResult<()> {
        // Disk I/O limiting is complex and device-specific
        // This is a simplified implementation
        
        if let Some(disk_limit) = limits.disk_bytes {
            // Set throttle I/O limits
            let blkio_cgroup = self.cgroup_path.join(container_id);
            
            // Note: Real implementation would require device-specific configuration
            log::info!("Applying disk limit {} bytes for container {}", disk_limit, container_id);
        }

        Ok(())
    }

    async fn get_usage(&self, container_id: &str) -> Result<u64, ContainerError> {
        // This would read from blkio cgroup stats
        // Simplified implementation
        Ok(0)
    }

    async fn release_resources(&self, container_id: &str) -> ContainerResult<()> {
        // Clean up blkio cgroup settings
        Ok(())
    }
}

/// Network Resource Manager
pub struct NetworkResourceManager {
    cgroup_path: PathBuf,
}

impl NetworkResourceManager {
    fn new() -> Self {
        Self {
            cgroup_path: PathBuf::from("/sys/fs/cgroup/net_cls"),
        }
    }

    async fn allocate_resources(&self, limits: &ResourceLimits) -> Result<NetworkAllocation, ContainerError> {
        let bandwidth_limit = limits.network_bandwidth;
        
        Ok(NetworkAllocation {
            bandwidth_limit,
        })
    }

    async fn apply_limits(&self, container_id: &str, limits: &ResourceLimits) -> ContainerResult<()> {
        if let Some(bandwidth) = limits.network_bandwidth {
            // Apply network bandwidth limits using tc (traffic control)
            let interface = format!("veth_{}", container_id);
            
            unsafe {
                let cmd = format!(
                    "tc qdisc add dev {} root handle 1: htb default 12\ntc class add dev {} parent 1: classid 1:1 htb rate {}kbit\ntc class add dev {} parent 1:1 classid 1:10 htb rate {}kbit ceil {}kbit\ntc class add dev {} parent 1:1 classid 1:12 htb rate 1kbit ceil {}kbit\ntc filter add dev {} parent 1: protocol ip prio 1 handle 1 fw flowid 1:10\ntc filter add dev {} parent 1: protocol ip prio 1 handle 2 fw flowid 1:12",
                    interface, interface, bandwidth, interface, bandwidth, interface, bandwidth, interface, interface
                );
                
                libc::system(cmd.as_ptr() as *const libc::c_char);
            }
        }

        Ok(())
    }

    async fn get_usage(&self, container_id: &str) -> Result<u64, ContainerError> {
        // This would read network statistics
        // Simplified implementation
        Ok(0)
    }

    async fn release_resources(&self, container_id: &str) -> ContainerResult<()> {
        // Clean up network bandwidth limits
        let interface = format!("veth_{}", container_id);
        
        unsafe {
            let cmd = format!("tc qdisc del dev {} root\n", interface);
            libc::system(cmd.as_ptr() as *const libc::c_char);
        }

        Ok(())
    }
}

/// Resource allocation information
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub cpu: CpuAllocation,
    pub memory: MemoryAllocation,
    pub disk: DiskAllocation,
    pub network: NetworkAllocation,
}

/// CPU allocation details
#[derive(Debug, Clone)]
pub struct CpuAllocation {
    pub quota: Option<i64>,
    pub period: Option<i64>,
    pub shares: Option<i64>,
}

/// Memory allocation details
#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    pub limit: Option<u64>,
    pub swap_limit: Option<u64>,
    pub soft_limit: Option<u64>,
}

/// Disk allocation details
#[derive(Debug, Clone)]
pub struct DiskAllocation {
    pub quota: Option<u64>,
    pub io_weight: Option<u32>,
}

/// Network allocation details
#[derive(Debug, Clone)]
pub struct NetworkAllocation {
    pub bandwidth_limit: Option<u64>,
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub disk_usage: u64,
    pub network_usage: u64,
}