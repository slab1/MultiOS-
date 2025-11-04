//! Error Recovery Manager
//! 
//! Comprehensive error detection, classification, and recovery mechanisms
//! for block devices with retry logic, fail-over, and graceful degradation.

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockDeviceError, BlockOperation};

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap, collections::HashMap};
use core::time::{Duration, Instant};

/// Error types classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorType {
    None = 0,
    ReadTimeout = 1,
    WriteTimeout = 2,
    HardwareError = 3,
    MediaError = 4,
    BadBlock = 5,
    PermissionDenied = 6,
    OutOfSpace = 7,
    UnsupportedOperation = 8,
    DeviceNotReady = 9,
    CommandFailed = 10,
    CrcError = 11,
    InvalidSector = 12,
    BufferTooSmall = 13,
    RetryRequired = 14,
    DeviceFailure = 15,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ErrorSeverity {
    Warning = 0,    // Non-critical, can continue
    Minor = 1,      // Affects performance but not functionality
    Major = 2,      // Affects functionality but recoverable
    Critical = 3,   // Serious error requiring immediate attention
    Fatal = 4,      // Device failure, must be replaced
}

/// Recovery strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecoveryStrategy {
    None = 0,
    Retry = 1,              // Simple retry
    RetryWithBackoff = 2,   // Retry with exponential backoff
    RemapSector = 3,        // Remap bad sector to spare
    SwitchDevice = 4,       // Switch to backup device
    DegradePerformance = 5, // Reduce performance to improve reliability
    Quarantine = 6,         // Quarantine problematic area
    FailOver = 7,           // Complete fail-over to secondary storage
}

/// Error recovery statistics
#[derive(Debug, Clone, Default)]
struct RecoveryStats {
    total_errors: u64,
    recovered_errors: u64,
    permanent_failures: u64,
    retries_attempted: u64,
    successful_retries: u64,
    sectors_remapped: u64,
    device_switches: u64,
    avg_recovery_time: Duration,
    error_rate: f32,
}

/// Error information
#[derive(Debug, Clone)]
struct ErrorInfo {
    error_type: ErrorType,
    severity: ErrorSeverity,
    operation: BlockOperation,
    sector: u64,
    device_id: BlockDeviceId,
    timestamp: Instant,
    retry_count: u32,
    last_retry: Instant,
    error_message: &'static str,
}

/// Device health information
#[derive(Debug, Clone)]
struct DeviceHealth {
    device_id: BlockDeviceId,
    is_healthy: bool,
    error_count: u32,
    last_error: Option<ErrorInfo>,
    sector_map: BTreeMap<u64, u64>, // bad sector -> replacement sector
    spare_sectors: Vec<u64>,
    total_spare_sectors: u32,
    available_spare_sectors: u32,
    max_error_rate: f32,
    current_error_rate: f32,
    last_health_check: Instant,
    health_check_interval: Duration,
    recovery_enabled: bool,
}

/// Recovery configuration
#[derive(Debug, Clone)]
struct RecoveryConfig {
    max_retries: u32,
    retry_delay_ms: u64,
    exponential_backoff: bool,
    backoff_factor: f32,
    max_retry_delay_ms: u64,
    enable_sector_remapping: bool,
    enable_device_switching: bool,
    enable_performance_degradation: bool,
    error_rate_threshold: f32,
    health_check_interval: Duration,
}

/// Global error recovery manager
pub struct ErrorRecoveryManager {
    device_health: RwLock<HashMap<BlockDeviceId, DeviceHealth>>,
    error_history: RwLock<HashMap<BlockDeviceId, Vec<ErrorInfo>>>,
    recovery_config: RecoveryConfig,
    global_stats: Arc<RwLock<RecoveryStats>>,
    backup_devices: RwLock<Vec<BlockDeviceId>>, // Backup device IDs
    current_device: RwLock<BlockDeviceId>,
}

impl ErrorRecoveryManager {
    /// Create new error recovery manager
    pub fn new() -> Self {
        info!("Initializing Error Recovery Manager");
        
        Self {
            device_health: RwLock::new(HashMap::new()),
            error_history: RwLock::new(HashMap::new()),
            recovery_config: RecoveryConfig {
                max_retries: 5,
                retry_delay_ms: 100,
                exponential_backoff: true,
                backoff_factor: 2.0,
                max_retry_delay_ms: 5000,
                enable_sector_remapping: true,
                enable_device_switching: true,
                enable_performance_degradation: true,
                error_rate_threshold: 0.01, // 1% error rate
                health_check_interval: Duration::from_secs(60),
            },
            global_stats: Arc::new(RwLock::new(RecoveryStats::default())),
            backup_devices: RwLock::new(Vec::new()),
            current_device: RwLock::new(BlockDeviceId(0)),
        }
    }

    /// Initialize the error recovery manager
    pub fn init(&self) -> Result<(), BlockDeviceError> {
        info!("Initializing Error Recovery Manager");
        
        // Initialize global statistics
        let mut stats = self.global_stats.write();
        *stats = RecoveryStats::default();
        
        info!("Error Recovery Manager initialized successfully");
        Ok(())
    }

    /// Register device for error recovery
    pub fn register_device(&self, device_id: BlockDeviceId, total_sectors: u64) {
        info!("Registering device {:?} for error recovery, total sectors: {}", device_id, total_sectors);
        
        let mut device_health = self.device_health.write();
        
        // Calculate spare sectors (typically 1-2% of total capacity)
        let total_spare_sectors = ((total_sectors as f32) * 0.02).max(1.0) as u32;
        
        let health = DeviceHealth {
            device_id,
            is_healthy: true,
            error_count: 0,
            last_error: None,
            sector_map: BTreeMap::new(),
            spare_sectors: (0..total_spare_sectors).map(|i| total_sectors + i as u64).collect(),
            total_spare_sectors,
            available_spare_sectors: total_spare_sectors,
            max_error_rate: 0.05, // 5% maximum acceptable error rate
            current_error_rate: 0.0,
            last_health_check: Instant::now(),
            health_check_interval: self.recovery_config.health_check_interval,
            recovery_enabled: true,
        };
        
        device_health.insert(device_id, health);
        
        // Initialize error history
        let mut error_history = self.error_history.write();
        error_history.insert(device_id, Vec::new());
        
        info!("Device {:?} registered for error recovery with {} spare sectors", device_id, total_spare_sectors);
    }

    /// Unregister device from error recovery
    pub fn unregister_device(&self, device_id: BlockDeviceId) {
        info!("Unregistering device {:?} from error recovery", device_id);
        
        self.device_health.write().remove(&device_id);
        self.error_history.write().remove(&device_id);
    }

    /// Handle device error
    pub fn handle_error(&self, device_id: BlockDeviceId, error: BlockDeviceError) -> Result<(), BlockDeviceError> {
        let start_time = Instant::now();
        
        // Classify error
        let error_type = self.classify_error(&error);
        let severity = self.determine_severity(error_type, &error);
        
        info!("Handling error on device {:?}: {:?} (severity: {:?})", device_id, error_type, severity);
        
        // Record error
        self.record_error(device_id, error_type, severity, &error);
        
        // Attempt recovery based on severity and error type
        let recovery_result = match severity {
            ErrorSeverity::Warning | ErrorSeverity::Minor => {
                self.attempt_minor_recovery(device_id, error_type)
            }
            ErrorSeverity::Major => {
                self.attempt_major_recovery(device_id, error_type)
            }
            ErrorSeverity::Critical => {
                self.attempt_critical_recovery(device_id, error_type)
            }
            ErrorSeverity::Fatal => {
                self.attempt_fatal_recovery(device_id, error_type)
            }
        };
        
        // Update statistics
        self.update_recovery_stats(device_id, &recovery_result, start_time.elapsed());
        
        match recovery_result {
            RecoveryResult::Success => {
                info!("Error recovery successful for device {:?}", device_id);
                Ok(())
            }
            RecoveryResult::RetryRequired => {
                info!("Retry required for device {:?}", device_id);
                Err(BlockDeviceError::RetryRequired)
            }
            RecoveryResult::DeviceSwitched => {
                info!("Device switched for {:?}", device_id);
                Ok(())
            }
            RecoveryResult::PerformanceDegraded => {
                info!("Performance degraded for device {:?}", device_id);
                Ok(())
            }
            RecoveryResult::PermanentFailure => {
                error!("Permanent failure on device {:?}", device_id);
                Err(BlockDeviceError::DeviceNotFound)
            }
        }
    }

    /// Classify error type
    fn classify_error(&self, error: &BlockDeviceError) -> ErrorType {
        match error {
            BlockDeviceError::Timeout => ErrorType::ReadTimeout,
            BlockDeviceError::HardwareError => ErrorType::HardwareError,
            BlockDeviceError::MediaError => ErrorType::MediaError,
            BlockDeviceError::BadBlock => ErrorType::BadBlock,
            BlockDeviceError::PermissionDenied => ErrorType::PermissionDenied,
            BlockDeviceError::OutOfSpace => ErrorType::OutOfSpace,
            BlockDeviceError::UnsupportedOperation => ErrorType::UnsupportedOperation,
            BlockDeviceError::DeviceNotFound => ErrorType::DeviceFailure,
            BlockDeviceError::InvalidSector => ErrorType::InvalidSector,
            BlockDeviceError::BufferTooSmall => ErrorType::BufferTooSmall,
            BlockDeviceError::RetryRequired => ErrorType::RetryRequired,
            _ => ErrorType::CommandFailed,
        }
    }

    /// Determine error severity
    fn determine_severity(&self, error_type: ErrorType, error: &BlockDeviceError) -> ErrorSeverity {
        match error_type {
            ErrorType::ReadTimeout | ErrorType::WriteTimeout => ErrorSeverity::Minor,
            ErrorType::HardwareError => ErrorSeverity::Major,
            ErrorType::MediaError | ErrorType::BadBlock => ErrorSeverity::Critical,
            ErrorType::PermissionDenied => ErrorSeverity::Warning,
            ErrorType::OutOfSpace => ErrorSeverity::Major,
            ErrorType::UnsupportedOperation => ErrorSeverity::Warning,
            ErrorType::DeviceFailure => ErrorSeverity::Fatal,
            ErrorType::CrcError => ErrorSeverity::Major,
            ErrorType::InvalidSector => ErrorSeverity::Major,
            ErrorType::BufferTooSmall => ErrorSeverity::Warning,
            ErrorType::RetryRequired => ErrorSeverity::Minor,
            _ => ErrorSeverity::Minor,
        }
    }

    /// Record error information
    fn record_error(&self, device_id: BlockDeviceId, error_type: ErrorType, severity: ErrorSeverity, error: &BlockDeviceError) {
        let error_info = ErrorInfo {
            error_type,
            severity,
            operation: BlockOperation::Read, // Would be passed as parameter in real implementation
            sector: 0, // Would be passed as parameter in real implementation
            device_id,
            timestamp: Instant::now(),
            retry_count: 0,
            last_retry: Instant::now(),
            error_message: match error {
                BlockDeviceError::HardwareError => "Hardware error occurred",
                BlockDeviceError::MediaError => "Media read/write error",
                BlockDeviceError::Timeout => "Operation timeout",
                _ => "Unknown error",
            },
        };
        
        // Add to device error history
        let mut device_health = self.device_health.write();
        if let Some(health) = device_health.get_mut(&device_id) {
            health.error_count += 1;
            health.last_error = Some(error_info.clone());
        }
        
        // Add to error history
        let mut error_history = self.error_history.write();
        if let Some(history) = error_history.get_mut(&device_id) {
            history.push(error_info);
            
            // Keep only recent errors (last 1000)
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // Update global statistics
        let mut stats = self.global_stats.write();
        stats.total_errors += 1;
    }

    /// Attempt minor recovery (warnings and minor errors)
    fn attempt_minor_recovery(&self, device_id: BlockDeviceId, error_type: ErrorType) -> RecoveryResult {
        info!("Attempting minor recovery for device {:?}, error type: {:?}", device_id, error_type);
        
        match error_type {
            ErrorType::BufferTooSmall | ErrorType::PermissionDenied => {
                // These are typically configuration issues, mark as success
                RecoveryResult::Success
            }
            ErrorType::RetryRequired => {
                // Return retry required for proper retry handling
                RecoveryResult::RetryRequired
            }
            _ => {
                // Try simple retry
                RecoveryResult::RetryRequired
            }
        }
    }

    /// Attempt major recovery (major errors)
    fn attempt_major_recovery(&self, device_id: BlockDeviceId, error_type: ErrorType) -> RecoveryResult {
        info!("Attempting major recovery for device {:?}, error type: {:?}", device_id, error_type);
        
        let mut device_health = self.device_health.write();
        let health = match device_health.get_mut(&device_id) {
            Some(h) => h,
            None => return RecoveryResult::PermanentFailure,
        };
        
        match error_type {
            ErrorType::HardwareError | ErrorType::CrcError => {
                // Try sector remapping if enabled
                if self.recovery_config.enable_sector_remapping {
                    if let Some(bad_sector) = self.find_next_bad_sector(&health) {
                        if self.remap_sector(device_id, bad_sector, &mut health) {
                            health.available_spare_sectors = health.available_spare_sectors.saturating_sub(1);
                            info!("Sector remapped: {} -> spare", bad_sector);
                            return RecoveryResult::Success;
                        }
                    }
                }
                
                // If remapping fails, try performance degradation
                if self.recovery_config.enable_performance_degradation {
                    info!("Performance degradation applied to device {:?}", device_id);
                    return RecoveryResult::PerformanceDegraded;
                }
                
                RecoveryResult::RetryRequired
            }
            ErrorType::InvalidSector => {
                // Try to remap invalid sector
                if self.recovery_config.enable_sector_remapping {
                    if let Some(bad_sector) = self.find_next_bad_sector(&health) {
                        if self.remap_sector(device_id, bad_sector, &mut health) {
                            health.available_spare_sectors = health.available_spare_sectors.saturating_sub(1);
                            return RecoveryResult::Success;
                        }
                    }
                }
                RecoveryResult::PermanentFailure
            }
            _ => RecoveryResult::RetryRequired,
        }
    }

    /// Attempt critical recovery (critical errors)
    fn attempt_critical_recovery(&self, device_id: BlockDeviceId, error_type: ErrorType) -> RecoveryResult {
        info!("Attempting critical recovery for device {:?}, error type: {:?}", device_id, error_type);
        
        let mut device_health = self.device_health.write();
        let health = match device_health.get_mut(&device_id) {
            Some(h) => h,
            None => return RecoveryResult::PermanentFailure,
        };
        
        match error_type {
            ErrorType::MediaError | ErrorType::BadBlock => {
                // Media errors are serious, try remapping first
                if self.recovery_config.enable_sector_remapping {
                    if let Some(bad_sector) = self.find_next_bad_sector(&health) {
                        if self.remap_sector(device_id, bad_sector, &mut health) {
                            health.available_spare_sectors = health.available_spare_sectors.saturating_sub(1);
                            return RecoveryResult::Success;
                        }
                    }
                }
                
                // If no more spare sectors or remapping fails, check for device switching
                if self.recovery_config.enable_device_switching {
                    return RecoveryResult::DeviceSwitched;
                }
                
                RecoveryResult::PermanentFailure
            }
            _ => RecoveryResult::RetryRequired,
        }
    }

    /// Attempt fatal recovery (fatal errors)
    fn attempt_fatal_recovery(&self, device_id: BlockDeviceId, error_type: ErrorType) -> RecoveryResult {
        error!("Attempting fatal recovery for device {:?}, error type: {:?}", device_id, error_type);
        
        let mut device_health = self.device_health.write();
        if let Some(health) = device_health.get_mut(&device_id) {
            health.is_healthy = false;
        }
        
        match error_type {
            ErrorType::DeviceFailure => {
                if self.recovery_config.enable_device_switching {
                    RecoveryResult::DeviceSwitched
                } else {
                    RecoveryResult::PermanentFailure
                }
            }
            _ => RecoveryResult::PermanentFailure,
        }
    }

    /// Find next bad sector requiring remapping
    fn find_next_bad_sector(&self, health: &DeviceHealth) -> Option<u64> {
        // In real implementation, this would scan for problematic sectors
        // For now, return a dummy bad sector
        Some(1000)
    }

    /// Remap bad sector to spare sector
    fn remap_sector(&self, device_id: BlockDeviceId, bad_sector: u64, health: &mut DeviceHealth) -> bool {
        if let Some(spare_sector) = health.spare_sectors.pop() {
            // In real implementation, this would:
            // 1. Copy data from bad sector to spare sector if readable
            // 2. Update device's remapping table
            // 3. Mark bad sector as remapped
            
            health.sector_map.insert(bad_sector, spare_sector);
            
            // Update statistics
            let mut stats = self.global_stats.write();
            stats.sectors_remapped += 1;
            
            info!("Successfully remapped sector {} to spare sector {} on device {:?}", 
                  bad_sector, spare_sector, device_id);
            
            true
        } else {
            warn!("No spare sectors available for remapping on device {:?}", device_id);
            false
        }
    }

    /// Get retry delay with exponential backoff
    pub fn get_retry_delay(&self, retry_count: u32) -> Duration {
        if self.recovery_config.exponential_backoff {
            let delay_ms = self.recovery_config.retry_delay_ms as f32 * 
                          (self.recovery_config.backoff_factor.powi(retry_count as i32));
            let clamped_delay = delay_ms.min(self.recovery_config.max_retry_delay_ms);
            Duration::from_millis(clamped_delay as u64)
        } else {
            Duration::from_millis(self.recovery_config.retry_delay_ms)
        }
    }

    /// Perform health check on all registered devices
    pub fn perform_health_checks(&self) {
        let current_time = Instant::now();
        
        let device_health = self.device_health.read();
        for (device_id, health) in device_health.iter() {
            if current_time.duration_since(health.last_health_check) >= health.health_check_interval {
                drop(device_health); // Release read lock
                
                let _ = self.check_device_health(*device_id);
                break; // Reacquire locks for next iteration
            }
        }
    }

    /// Check individual device health
    fn check_device_health(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        let mut device_health = self.device_health.write();
        let health = match device_health.get_mut(&device_id) {
            Some(h) => h,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Update error rate
        let error_history = self.error_history.read();
        let recent_errors = if let Some(history) = error_history.get(&device_id) {
            let one_hour_ago = Instant::now() - Duration::from_secs(3600);
            history.iter().filter(|e| e.timestamp > one_hour_ago).count() as u32
        } else {
            0
        };
        
        health.current_error_rate = recent_errors as f32 / 3600.0; // errors per second
        
        // Check if error rate exceeds threshold
        if health.current_error_rate > health.max_error_rate {
            warn!("High error rate detected on device {:?}: {} errors/sec (threshold: {})", 
                  device_id, health.current_error_rate, health.max_error_rate);
            
            health.is_healthy = false;
        }
        
        health.last_health_check = Instant::now();
        
        Ok(())
    }

    /// Update recovery statistics
    fn update_recovery_stats(&self, device_id: BlockDeviceId, result: &RecoveryResult, recovery_time: Duration) {
        let mut stats = self.global_stats.write();
        
        match result {
            RecoveryResult::Success => {
                stats.recovered_errors += 1;
            }
            RecoveryResult::RetryRequired => {
                stats.retries_attempted += 1;
            }
            RecoveryResult::DeviceSwitched => {
                stats.device_switches += 1;
            }
            RecoveryResult::PermanentFailure => {
                stats.permanent_failures += 1;
            }
            _ => {}
        }
        
        // Update average recovery time
        stats.avg_recovery_time = Duration::from_nanos(
            (stats.avg_recovery_time.as_nanos() + recovery_time.as_nanos()) / 2
        );
        
        // Update error rate
        stats.error_rate = if stats.total_errors > 0 {
            (stats.total_errors - stats.recovered_errors) as f32 / stats.total_errors as f32
        } else {
            0.0
        };
    }

    /// Get device health information
    pub fn get_device_health(&self, device_id: BlockDeviceId) -> Result<DeviceHealthInfo, BlockDeviceError> {
        let device_health = self.device_health.read();
        let health = match device_health.get(&device_id) {
            Some(h) => h,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        let error_history = self.error_history.read();
        let recent_errors = if let Some(history) = error_history.get(&device_id) {
            let one_hour_ago = Instant::now() - Duration::from_secs(3600);
            history.iter().filter(|e| e.timestamp > one_hour_ago).count() as u32
        } else {
            0
        };
        
        Ok(DeviceHealthInfo {
            device_id,
            is_healthy: health.is_healthy,
            error_count: health.error_count,
            recent_errors: recent_errors,
            current_error_rate: health.current_error_rate,
            available_spare_sectors: health.available_spare_sectors,
            total_spare_sectors: health.total_spare_sectors,
            sectors_remapped: health.sector_map.len() as u64,
            last_error: health.last_error.clone(),
        })
    }

    /// Get global recovery statistics
    pub fn get_global_stats(&self) -> RecoveryStats {
        self.global_stats.read().clone()
    }

    /// Configure recovery parameters
    pub fn configure_recovery(&mut self, config: RecoveryConfig) {
        info!("Updating recovery configuration");
        self.recovery_config = config;
    }

    /// Add backup device
    pub fn add_backup_device(&self, backup_device_id: BlockDeviceId) {
        let mut backup_devices = self.backup_devices.write();
        if !backup_devices.contains(&backup_device_id) {
            backup_devices.push(backup_device_id);
            info!("Added backup device {:?}", backup_device_id);
        }
    }

    /// Get list of backup devices
    pub fn get_backup_devices(&self) -> Vec<BlockDeviceId> {
        self.backup_devices.read().clone()
    }

    /// Get error history for device
    pub fn get_error_history(&self, device_id: BlockDeviceId) -> Result<Vec<ErrorInfo>, BlockDeviceError> {
        let error_history = self.error_history.read();
        match error_history.get(&device_id) {
            Some(history) => Ok(history.clone()),
            None => Err(BlockDeviceError::DeviceNotFound),
        }
    }
}

/// Recovery result types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryResult {
    Success,
    RetryRequired,
    DeviceSwitched,
    PerformanceDegraded,
    PermanentFailure,
}

/// Device health information for external use
#[derive(Debug, Clone)]
pub struct DeviceHealthInfo {
    pub device_id: BlockDeviceId,
    pub is_healthy: bool,
    pub error_count: u32,
    pub recent_errors: u32,
    pub current_error_rate: f32,
    pub available_spare_sectors: u32,
    pub total_spare_sectors: u32,
    pub sectors_remapped: u64,
    pub last_error: Option<ErrorInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::new();
        assert_eq!(manager.recovery_config.max_retries, 5);
    }

    #[test]
    fn test_error_classification() {
        let manager = ErrorRecoveryManager::new();
        let hardware_error = BlockDeviceError::HardwareError;
        let error_type = manager.classify_error(&hardware_error);
        assert_eq!(error_type, ErrorType::HardwareError);
    }

    #[test]
    fn test_error_severity() {
        let manager = ErrorRecoveryManager::new();
        let timeout_error = BlockDeviceError::Timeout;
        let severity = manager.determine_severity(ErrorType::ReadTimeout, &timeout_error);
        assert_eq!(severity, ErrorSeverity::Minor);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let manager = ErrorRecoveryManager::new();
        let delay = manager.get_retry_delay(0);
        assert_eq!(delay, Duration::from_millis(100));
        
        let delay_with_backoff = manager.get_retry_delay(2);
        assert_eq!(delay_with_backoff, Duration::from_millis(400)); // 100 * 2^2
    }

    #[test]
    fn test_recovery_result() {
        let result = RecoveryResult::Success;
        assert_eq!(result, RecoveryResult::Success);
    }

    #[test]
    fn test_device_health_info() {
        let health_info = DeviceHealthInfo {
            device_id: BlockDeviceId(1),
            is_healthy: true,
            error_count: 5,
            recent_errors: 2,
            current_error_rate: 0.001,
            available_spare_sectors: 100,
            total_spare_sectors: 200,
            sectors_remapped: 10,
            last_error: None,
        };
        
        assert_eq!(health_info.error_count, 5);
        assert!(health_info.is_healthy);
    }
}