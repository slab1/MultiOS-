//! Block I/O Scheduler
//! 
//! Advanced I/O scheduling algorithms for block devices including
//! elevator (deadline), CFQ (Complete Fair Queuing), and deadline scheduling.

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockIoRequest, BlockIoResult, BlockOperation, RequestPriority, RequestFlags, BlockDeviceError, BlockDeviceInfo};
use crate::drivers::block::{BlockDeviceError as SuperBlockDeviceError};

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::VecDeque, collections::BTreeMap, collections::HashMap};
use alloc::sync::Arc;
use core::time::Duration;

/// Scheduler types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SchedulerType {
    None = 0,
    Elevator = 1,    // Deadline/elevator algorithm
    Cfq = 2,         // Complete Fair Queuing
    Deadline = 3,    // Deadline scheduler
    MqDeadline = 4,  // Multi-queue deadline (for NVMe)
    NoneOps = 5,     // No-op for fast devices like RAM disks
}

/// I/O request state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RequestState {
    Pending = 0,
    Executing = 1,
    Completed = 2,
    Failed = 3,
}

/// I/O request with scheduler metadata
#[derive(Debug, Clone)]
struct SchedulerRequest {
    request: BlockIoRequest,
    state: RequestState,
    arrival_time: u64,
    start_time: Option<u64>,
    deadline: Option<u64>,
    sector_start: u64,
    sector_end: u64,
    byte_size: usize,
}

/// Device queue information
#[derive(Debug, Clone)]
struct DeviceQueue {
    device_id: BlockDeviceId,
    queue_depth: u32,
    current_depth: u32,
    pending_reads: VecDeque<SchedulerRequest>,
    pending_writes: VecDeque<SchedulerRequest>,
    executing_request: Option<SchedulerRequest>,
    last_sector: u64,
    elevator_direction: ElevatorDirection,
    cfq_time_slice: Duration,
    cfq_current_time: u64,
    cfq_group_queues: HashMap<u64, CfqGroup>, // Process group ID -> CFQ group
}

/// CFQ group for fair queuing
#[derive(Debug, Clone)]
struct CfqGroup {
    group_id: u64,
    time_used: u64,
    dispatch_queue: VecDeque<SchedulerRequest>,
    wait_queue: VecDeque<SchedulerRequest>,
}

/// Elevator direction for elevator algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ElevatorDirection {
    Up = 0,
    Down = 1,
}

/// Block I/O Scheduler
pub struct BlockIoScheduler {
    scheduler_type: SchedulerType,
    devices: RwLock<HashMap<BlockDeviceId, DeviceQueue>>,
    global_queue: VecDeque<SchedulerRequest>,
    time_slice_ns: u64,
    fifo_expire_read: Duration,
    fifo_expire_write: Duration,
    max_batch_size: u32,
    cfq_quantum: Duration,
    cfq_group_isolation: bool,
}

impl BlockIoScheduler {
    /// Create new I/O scheduler
    pub fn new(scheduler_type: SchedulerType) -> Self {
        info!("Initializing Block I/O Scheduler: {:?}", scheduler_type);
        
        Self {
            scheduler_type,
            devices: RwLock::new(HashMap::new()),
            global_queue: VecDeque::new(),
            time_slice_ns: 100_000_000, // 100ms default time slice
            fifo_expire_read: Duration::from_millis(500),
            fifo_expire_write: Duration::from_millis(5000),
            max_batch_size: 128,
            cfq_quantum: Duration::from_millis(100),
            cfq_group_isolation: true,
        }
    }

    /// Initialize the scheduler
    pub fn init(&mut self) -> Result<(), BlockDeviceError> {
        info!("Initializing I/O scheduler: {:?}", self.scheduler_type);
        
        match self.scheduler_type {
            SchedulerType::Elevator | SchedulerType::Deadline => {
                info!("Using elevator/deadline scheduler");
            }
            SchedulerType::Cfq => {
                info!("Using CFQ scheduler");
            }
            SchedulerType::NoneOps => {
                info!("Using no-op scheduler");
            }
            SchedulerType::MqDeadline => {
                info!("Using multi-queue deadline scheduler");
            }
            SchedulerType::None => {
                warn!("No I/O scheduler specified");
            }
        }
        
        Ok(())
    }

    /// Add device to scheduler
    pub fn add_device(&self, device_id: BlockDeviceId, queue_depth: u32) {
        info!("Adding device {:?} to scheduler with queue depth {}", device_id, queue_depth);
        
        let mut devices = self.devices.write();
        
        let device_queue = DeviceQueue {
            device_id,
            queue_depth,
            current_depth: 0,
            pending_reads: VecDeque::new(),
            pending_writes: VecDeque::new(),
            executing_request: None,
            last_sector: 0,
            elevator_direction: ElevatorDirection::Up,
            cfq_time_slice: Duration::from_millis(100),
            cfq_current_time: crate::arch::get_time_ns(),
            cfq_group_queues: HashMap::new(),
        };
        
        devices.insert(device_id, device_queue);
    }

    /// Remove device from scheduler
    pub fn remove_device(&self, device_id: BlockDeviceId) {
        info!("Removing device {:?} from scheduler", device_id);
        self.devices.write().remove(&device_id);
    }

    /// Submit I/O request
    pub fn submit_request(&mut self, request: BlockIoRequest) -> Result<usize, BlockDeviceError> {
        let current_time = crate::arch::get_time_ns();
        
        // Create scheduler request
        let sector_start = request.sector;
        let sector_end = request.sector + request.sector_count as u64 - 1;
        let byte_size = (request.sector_count as usize) * 512; // Assume 512-byte sectors
        
        let scheduler_request = SchedulerRequest {
            request,
            state: RequestState::Pending,
            arrival_time: current_time,
            start_time: None,
            deadline: request.deadline,
            sector_start,
            sector_end,
            byte_size,
        };
        
        // Add to appropriate queue based on scheduler type
        match self.scheduler_type {
            SchedulerType::Elevator | SchedulerType::Deadline => {
                self.submit_elevator_request(scheduler_request)
            }
            SchedulerType::Cfq => {
                self.submit_cfq_request(scheduler_request)
            }
            SchedulerType::NoneOps => {
                self.submit_noop_request(scheduler_request)
            }
            SchedulerType::MqDeadline => {
                self.submit_mq_deadline_request(scheduler_request)
            }
            SchedulerType::None => {
                self.submit_none_request(scheduler_request)
            }
        }
    }

    /// Submit request using elevator/deadline algorithm
    fn submit_elevator_request(&mut self, mut request: SchedulerRequest) -> Result<usize, BlockDeviceError> {
        let device_id = request.request.device_id;
        let mut devices = self.devices.write();
        
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Determine deadline for deadline scheduler
        if self.scheduler_type == SchedulerType::Deadline {
            let expire_time = crate::arch::get_time_ns() + 
                match request.request.operation {
                    BlockOperation::Read => self.fifo_expire_read.as_nanos() as u64,
                    BlockOperation::Write => self.fifo_expire_write.as_nanos() as u64,
                    _ => self.fifo_expire_read.as_nanos() as u64,
                };
            request.deadline = Some(expire_time);
        }
        
        // Choose appropriate queue based on operation type
        match request.request.operation {
            BlockOperation::Read => {
                device_queue.pending_reads.push_back(request);
            }
            BlockOperation::Write | BlockOperation::Trim => {
                device_queue.pending_writes.push_back(request);
            }
            _ => {
                // Synchronous operations - execute immediately
                device_queue.pending_writes.push_back(request);
            }
        }
        
        device_queue.current_depth += 1;
        
        // Try to dispatch requests immediately
        self.try_dispatch_requests(device_id, &mut devices)?;
        
        Ok(request.byte_size)
    }

    /// Submit request using CFQ algorithm
    fn submit_cfq_request(&mut self, request: SchedulerRequest) -> Result<usize, BlockDeviceError> {
        let device_id = request.request.device_id;
        let mut devices = self.devices.write();
        
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Get or create CFQ group for this request (using PID as group ID)
        let group_id = request.request.request_id as u64; // Simplified - would use actual process group ID
        let cfq_group = device_queue.cfq_group_queues
            .entry(group_id)
            .or_insert_with(|| CfqGroup {
                group_id,
                time_used: 0,
                dispatch_queue: VecDeque::new(),
                wait_queue: VecDeque::new(),
            });
        
        // Add request to CFQ group
        if cfq_group.time_used < self.cfq_quantum.as_nanos() as u64 {
            cfq_group.dispatch_queue.push_back(request);
        } else {
            cfq_group.wait_queue.push_back(request);
        }
        
        device_queue.current_depth += 1;
        
        // Try to dispatch requests immediately
        self.try_dispatch_requests(device_id, &mut devices)?;
        
        Ok(request.byte_size)
    }

    /// Submit request using no-op algorithm (simple FIFO)
    fn submit_noop_request(&mut self, request: SchedulerRequest) -> Result<usize, BlockDeviceError> {
        let device_id = request.request.device_id;
        let mut devices = self.devices.write();
        
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Add to the end of writes queue (no distinction for no-op)
        device_queue.pending_writes.push_back(request);
        device_queue.current_depth += 1;
        
        // Try to dispatch requests immediately
        self.try_dispatch_requests(device_id, &mut devices)?;
        
        Ok(request.byte_size)
    }

    /// Submit request using multi-queue deadline algorithm
    fn submit_mq_deadline_request(&mut self, request: SchedulerRequest) -> Result<usize, BlockDeviceError> {
        // MQ Deadline is similar to regular deadline but optimized for multi-queue devices like NVMe
        self.submit_elevator_request(request)
    }

    /// Submit request without scheduling (immediate execution)
    fn submit_none_request(&mut self, request: SchedulerRequest) -> Result<usize, BlockDeviceError> {
        // For "none" scheduler, execute immediately without queuing
        let device_id = request.request.device_id;
        
        // This would execute the request directly on the device
        // For now, just simulate successful completion
        info!("Executing request immediately on device {:?}", device_id);
        
        Ok(request.byte_size)
    }

    /// Try to dispatch requests from queues
    fn try_dispatch_requests(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Check if device can accept more requests
        if device_queue.current_depth >= device_queue.queue_depth {
            return Ok(());
        }
        
        // Dispatch requests based on scheduler type
        match self.scheduler_type {
            SchedulerType::Elevator => self.dispatch_elevator_request(device_id, devices)?,
            SchedulerType::Deadline => self.dispatch_deadline_request(device_id, devices)?,
            SchedulerType::Cfq => self.dispatch_cfq_request(device_id, devices)?,
            SchedulerType::NoneOps => self.dispatch_noop_request(device_id, devices)?,
            SchedulerType::MqDeadline => self.dispatch_mq_deadline_request(device_id, devices)?,
            SchedulerType::None => self.dispatch_none_request(device_id, devices)?,
        }
        
        Ok(())
    }

    /// Dispatch request using elevator algorithm
    fn dispatch_elevator_request(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Choose queue based on elevator direction and sector position
        let (queue_to_use, direction) = match device_queue.elevator_direction {
            ElevatorDirection::Up => {
                if let Some(read_req) = device_queue.pending_reads.front() {
                    if read_req.sector_start >= device_queue.last_sector {
                        (&mut device_queue.pending_reads, ElevatorDirection::Up)
                    } else {
                        (&mut device_queue.pending_writes, ElevatorDirection::Down)
                    }
                } else {
                    (&mut device_queue.pending_writes, ElevatorDirection::Down)
                }
            }
            ElevatorDirection::Down => {
                if let Some(write_req) = device_queue.pending_writes.back() {
                    if write_req.sector_start <= device_queue.last_sector {
                        (&mut device_queue.pending_writes, ElevatorDirection::Down)
                    } else {
                        (&mut device_queue.pending_reads, ElevatorDirection::Up)
                    }
                } else {
                    (&mut device_queue.pending_reads, ElevatorDirection::Up)
                }
            }
        };
        
        // Update direction
        device_queue.elevator_direction = direction;
        
        // Execute the request
        if let Some(mut request) = queue_to_use.pop_front() {
            request.state = RequestState::Executing;
            request.start_time = Some(crate::arch::get_time_ns());
            device_queue.executing_request = Some(request.clone());
            
            info!("Dispatching elevator request: sector {} ({} sectors)", request.sector_start, request.request.sector_count);
            
            // Simulate execution - in real implementation this would call the device driver
            device_queue.current_depth = device_queue.current_depth.saturating_sub(1);
            device_queue.last_sector = request.sector_end;
            
            // Mark as completed
            device_queue.executing_request = None;
            request.state = RequestState::Completed;
        }
        
        Ok(())
    }

    /// Dispatch request using deadline algorithm
    fn dispatch_deadline_request(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        let current_time = crate::arch::get_time_ns();
        let deadline_expired = Duration::from_nanos(current_time);
        
        // Check for expired requests first
        let expired_read = device_queue.pending_reads.iter()
            .find(|req| req.deadline.map_or(false, |d| current_time > d));
            
        let expired_write = device_queue.pending_writes.iter()
            .find(|req| req.deadline.map_or(false, |d| current_time > d));
        
        // Choose request to dispatch
        let request_to_dispatch = if let Some(expired) = expired_read {
            Some(expired.clone())
        } else if let Some(expired) = expired_write {
            Some(expired.clone())
        } else {
            // No expired requests, use elevator algorithm
            device_queue.pending_reads.front().or_else(|| device_queue.pending_writes.front()).cloned()
        };
        
        if let Some(mut request) = request_to_dispatch {
            // Remove from queue
            match request.request.operation {
                BlockOperation::Read => {
                    if let Some(pos) = device_queue.pending_reads.iter().position(|r| r.request.request_id == request.request.request_id) {
                        device_queue.pending_reads.remove(pos);
                    }
                }
                _ => {
                    if let Some(pos) = device_queue.pending_writes.iter().position(|r| r.request.request_id == request.request.request_id) {
                        device_queue.pending_writes.remove(pos);
                    }
                }
            }
            
            // Execute request
            request.state = RequestState::Executing;
            request.start_time = Some(current_time);
            device_queue.executing_request = Some(request.clone());
            
            info!("Dispatching deadline request: sector {} ({} sectors)", request.sector_start, request.request.sector_count);
            
            // Simulate execution
            device_queue.current_depth = device_queue.current_depth.saturating_sub(1);
            device_queue.last_sector = request.sector_end;
            
            // Mark as completed
            device_queue.executing_request = None;
            request.state = RequestState::Completed;
        }
        
        Ok(())
    }

    /// Dispatch request using CFQ algorithm
    fn dispatch_cfq_request(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        let current_time = crate::arch::get_time_ns();
        
        // Check if current CFQ time slice has expired
        if current_time - device_queue.cfq_current_time >= device_queue.cfq_time_slice.as_nanos() as u64 {
            // Time slice expired, reset and select next group
            device_queue.cfq_current_time = current_time;
            
            // Simple round-robin selection of CFQ groups
            for cfq_group in device_queue.cfq_group_queues.values_mut() {
                cfq_group.time_used = 0;
            }
        }
        
        // Find CFQ group with remaining time slice
        let mut selected_group = None;
        let mut selected_request = None;
        
        for cfq_group in device_queue.cfq_group_queues.values_mut() {
            if cfq_group.time_used < self.cfq_quantum.as_nanos() as u64 {
                if let Some(request) = cfq_group.dispatch_queue.pop_front() {
                    selected_group = Some(cfq_group.group_id);
                    selected_request = Some(request);
                    break;
                }
            }
        }
        
        if let Some(request) = selected_request {
            if let Some(group_id) = selected_group {
                if let Some(cfq_group) = device_queue.cfq_group_queues.get_mut(&group_id) {
                    cfq_group.time_used += request.byte_size as u64;
                }
            }
            
            // Execute request
            let mut request = request;
            request.state = RequestState::Executing;
            request.start_time = Some(current_time);
            device_queue.executing_request = Some(request.clone());
            
            info!("Dispatching CFQ request: sector {} ({} sectors)", request.sector_start, request.request.sector_count);
            
            // Simulate execution
            device_queue.current_depth = device_queue.current_depth.saturating_sub(1);
            device_queue.last_sector = request.sector_end;
            
            // Mark as completed
            device_queue.executing_request = None;
            request.state = RequestState::Completed;
        }
        
        Ok(())
    }

    /// Dispatch request using no-op algorithm
    fn dispatch_noop_request(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        let device_queue = match devices.get_mut(&device_id) {
            Some(queue) => queue,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Simple FIFO dispatch
        if let Some(mut request) = device_queue.pending_writes.pop_front() {
            request.state = RequestState::Executing;
            request.start_time = Some(crate::arch::get_time_ns());
            device_queue.executing_request = Some(request.clone());
            
            info!("Dispatching noop request: sector {} ({} sectors)", request.sector_start, request.request.sector_count);
            
            // Simulate execution
            device_queue.current_depth = device_queue.current_depth.saturating_sub(1);
            device_queue.last_sector = request.sector_end;
            
            // Mark as completed
            device_queue.executing_request = None;
            request.state = RequestState::Completed;
        }
        
        Ok(())
    }

    /// Dispatch request using multi-queue deadline algorithm
    fn dispatch_mq_deadline_request(&mut self, device_id: BlockDeviceId, devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        // MQ Deadline is similar to regular deadline but optimized for multi-queue devices
        self.dispatch_deadline_request(device_id, devices)
    }

    /// Dispatch request with no scheduling
    fn dispatch_none_request(&mut self, _device_id: BlockDeviceId, _devices: &mut HashMap<BlockDeviceId, DeviceQueue>) -> Result<(), BlockDeviceError> {
        // No scheduling - requests are executed immediately upon submission
        Ok(())
    }

    /// Get scheduler statistics
    pub fn get_statistics(&self) -> SchedulerStats {
        let mut stats = SchedulerStats::default();
        
        let devices = self.devices.read();
        for (device_id, queue) in devices.iter() {
            stats.total_devices += 1;
            stats.total_queue_depth += queue.queue_depth as u32;
            stats.current_queue_depth += queue.current_depth;
            
            stats.pending_reads += queue.pending_reads.len() as u32;
            stats.pending_writes += queue.pending_writes.len() as u32;
            
            if queue.executing_request.is_some() {
                stats.active_requests += 1;
            }
        }
        
        stats.scheduler_type = self.scheduler_type;
        stats
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub scheduler_type: SchedulerType,
    pub total_devices: u32,
    pub total_queue_depth: u32,
    pub current_queue_depth: u32,
    pub pending_reads: u32,
    pub pending_writes: u32,
    pub active_requests: u32,
    pub dispatched_requests: u64,
    pub avg_wait_time: u64, // nanoseconds
    pub avg_completion_time: u64, // nanoseconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = BlockIoScheduler::new(SchedulerType::Elevator);
        assert_eq!(scheduler.scheduler_type, SchedulerType::Elevator);
    }

    #[test]
    fn test_scheduler_stats() {
        let stats = SchedulerStats::default();
        assert_eq!(stats.total_devices, 0);
        assert_eq!(stats.pending_reads, 0);
    }
}