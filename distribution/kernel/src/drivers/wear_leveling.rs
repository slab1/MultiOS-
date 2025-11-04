//! Wear Leveling Manager for SSDs
//! 
//! Advanced wear leveling algorithms to ensure even distribution
//! of write operations across SSD blocks to maximize lifespan.

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockOperation, BlockDeviceError};

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap, collections::VecDeque};
use core::time::{Duration, Instant};

/// Wear leveling strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WearLevelingStrategy {
    None = 0,
    Static = 1,      // Simple static mapping
    Dynamic = 2,     // Dynamic wear leveling
    Advanced = 3,    // Advanced algorithms with prediction
    Adaptive = 4,    // Adaptive based on usage patterns
}

/// Block information for wear leveling
#[derive(Debug, Clone)]
struct WearBlock {
    block_id: u64,
    logical_block: u64,  // Logical block number
    physical_block: u64, // Physical block number
    erase_count: u32,
    last_erased: Instant,
    last_written: Instant,
    is_free: bool,
    write_intensity: f32, // Writes per time unit
    error_count: u32,
    health_status: BlockHealth,
}

/// Block health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum BlockHealth {
    Excellent = 0,   // >90% lifespan remaining
    Good = 1,        // 70-90% lifespan remaining
    Fair = 2,        // 50-70% lifespan remaining
    Poor = 3,        // 30-50% lifespan remaining
    Critical = 4,    // <30% lifespan remaining
    Failed = 5,      // Block has failed
}

/// Device wear leveling information
#[derive(Debug, Clone)]
struct DeviceWearInfo {
    device_id: BlockDeviceId,
    total_blocks: u64,
    total_logical_blocks: u64,
    free_blocks: u64,
    blocks: BTreeMap<u64, WearBlock>, // physical block ID -> block info
    free_block_queue: VecDeque<u64>,
    wear_level_threshold: u32,
    max_erase_cycles: u32,
    current_erase_cycles: u32,
    trim_pending: Vec<u64>,
    wear_leveling_active: bool,
    last_wear_check: Instant,
    wear_check_interval: Duration,
    statistics: WearStatistics,
}

/// Wear leveling statistics
#[derive(Debug, Clone, Default)]
struct WearStatistics {
    total_writes: u64,
    total_trims: u64,
    total_erasures: u64,
    wear_leveling_operations: u64,
    avg_erase_cycles: f32,
    min_erase_cycles: u32,
    max_erase_cycles: u32,
    blocks_reclaimed: u64,
    blocks_failed: u64,
    health_score: f32, // 0.0 to 1.0
    estimated_lifespan_remaining: Duration,
}

/// Wear level check result
#[derive(Debug, Clone)]
struct WearCheckResult {
    needs_wear_leveling: bool,
    recommended_blocks_to_move: u32,
    target_blocks: Vec<u64>,
    estimated_improvement: f32,
}

/// Global wear leveling manager
pub struct WearLevelingManager {
    device_info: RwLock<BTreeMap<BlockDeviceId, DeviceWearInfo>>,
    strategy: WearLevelingStrategy,
    wear_threshold: u32,        // Threshold for wear leveling trigger
    max_concurrent_moves: u32,  // Maximum simultaneous block moves
    background_wear_leveling: bool,
    prediction_window: Duration, // Time window for usage prediction
    statistics: Arc<RwLock<BTreeMap<BlockDeviceId, WearStatistics>>>,
}

impl WearLevelingManager {
    /// Create new wear leveling manager
    pub fn new(strategy: WearLevelingStrategy) -> Self {
        info!("Initializing Wear Leveling Manager with strategy: {:?}", strategy);
        
        Self {
            device_info: RwLock::new(BTreeMap::new()),
            strategy,
            wear_threshold: 20, // Trigger when blocks differ by 20 erase cycles
            max_concurrent_moves: 4,
            background_wear_leveling: true,
            prediction_window: Duration::from_secs(300), // 5 minutes
            statistics: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Initialize the wear leveling manager
    pub fn init(&self) -> Result<(), BlockDeviceError> {
        info!("Initializing Wear Leveling Manager");
        
        if self.strategy == WearLevelingStrategy::None {
            warn!("Wear leveling disabled");
            return Ok(());
        }
        
        info!("Wear Leveling Manager initialized with strategy {:?}", self.strategy);
        Ok(())
    }

    /// Add SSD device for wear leveling
    pub fn add_ssd_device(&self, device_id: BlockDeviceId, total_sectors: u64) {
        info!("Adding SSD device {:?} to wear leveling, total sectors: {}", device_id, total_sectors);
        
        let sectors_per_block = 64; // Assuming 32KB blocks (64 sectors * 512 bytes)
        let total_blocks = total_sectors / sectors_per_block;
        
        let mut device_info = self.device_info.write();
        
        let wear_info = DeviceWearInfo {
            device_id,
            total_blocks,
            total_logical_blocks: total_sectors,
            free_blocks: total_blocks,
            blocks: BTreeMap::new(),
            free_block_queue: VecDeque::new(),
            wear_level_threshold: self.wear_threshold,
            max_erase_cycles: 3000, // Typical TLC SSD cycle limit
            current_erase_cycles: 0,
            trim_pending: Vec::new(),
            wear_leveling_active: false,
            last_wear_check: Instant::now(),
            wear_check_interval: Duration::from_secs(60), // Check every minute
            statistics: WearStatistics::default(),
        };
        
        device_info.insert(device_id, wear_info);
        
        // Initialize statistics
        let mut stats = self.statistics.write();
        stats.insert(device_id, WearStatistics::default());
        
        info!("SSD device {:?} added to wear leveling with {} blocks", device_id, total_blocks);
    }

    /// Remove device from wear leveling
    pub fn remove_device(&self, device_id: BlockDeviceId) {
        info!("Removing device {:?} from wear leveling", device_id);
        
        self.device_info.write().remove(&device_id);
        self.statistics.write().remove(&device_id);
    }

    /// Record write operation for wear leveling
    pub fn record_write(&self, device_id: BlockDeviceId, sector: u64, sector_count: u32) {
        let mut device_info = self.device_info.write();
        let wear_info = match device_info.get_mut(&device_id) {
            Some(info) => info,
            None => return,
        };
        
        if self.strategy == WearLevelingStrategy::None {
            return;
        }
        
        let block_size = 64; // sectors per block
        let start_block = sector / block_size;
        let end_block = (sector + sector_count as u64 - 1) / block_size;
        
        // Update block write information
        for block_id in start_block..=end_block {
            if let Some(block) = wear_info.blocks.get_mut(&block_id) {
                block.last_written = Instant::now();
                block.write_intensity = self.calculate_write_intensity(block);
            }
        }
        
        // Update statistics
        wear_info.statistics.total_writes += 1;
        
        info!("Recorded write on device {:?}, blocks {}-{}", device_id, start_block, end_block);
        
        // Check if wear leveling is needed
        self.check_wear_leveling(device_id, &mut device_info);
    }

    /// Record TRIM operation for wear leveling
    pub fn trim_sectors(&self, device_id: BlockDeviceId, sector: u64, sector_count: u32) {
        let mut device_info = self.device_info.write();
        let wear_info = match device_info.get_mut(&device_id) {
            Some(info) => info,
            None => return,
        };
        
        if self.strategy == WearLevelingStrategy::None {
            return;
        }
        
        let block_size = 64; // sectors per block
        let start_block = sector / block_size;
        let end_block = (sector + sector_count as u64 - 1) / block_size;
        
        // Add blocks to trim pending list
        for block_id in start_block..=end_block {
            if !wear_info.trim_pending.contains(&block_id) {
                wear_info.trim_pending.push(block_id);
            }
        }
        
        // Update statistics
        wear_info.statistics.total_trims += 1;
        
        info!("Recorded TRIM on device {:?}, blocks {}-{}", device_id, start_block, end_block);
    }

    /// Perform wear leveling operation
    pub fn perform_wear_leveling(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        let mut device_info = self.device_info.write();
        let wear_info = match device_info.get_mut(&device_id) {
            Some(info) => info,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        if self.strategy == WearLevelingStrategy::None {
            return Ok(());
        }
        
        if wear_info.wear_leveling_active {
            info!("Wear leveling already active for device {:?}", device_id);
            return Ok(());
        }
        
        wear_info.wear_leveling_active = true;
        
        info!("Starting wear leveling for device {:?}", device_id);
        
        // Analyze current wear distribution
        let wear_analysis = self.analyze_wear_distribution(device_id, &wear_info)?;
        
        if !wear_analysis.needs_wear_leveling {
            info!("Wear leveling not needed for device {:?}", device_id);
            wear_info.wear_leveling_active = false;
            return Ok(());
        }
        
        // Perform wear leveling based on strategy
        let result = match self.strategy {
            WearLevelingStrategy::Static => self.perform_static_wear_leveling(device_id, &wear_analysis),
            WearLevelingStrategy::Dynamic => self.perform_dynamic_wear_leveling(device_id, &wear_analysis),
            WearLevelingStrategy::Advanced => self.perform_advanced_wear_leveling(device_id, &wear_analysis),
            WearLevelingStrategy::Adaptive => self.perform_adaptive_wear_leveling(device_id, &wear_analysis),
            WearLevelingStrategy::None => Ok(()),
        };
        
        match result {
            Ok(_) => {
                wear_info.statistics.wear_leveling_operations += 1;
                wear_info.last_wear_check = Instant::now();
                info!("Wear leveling completed for device {:?}", device_id);
            }
            Err(e) => {
                error!("Wear leveling failed for device {:?}: {:?}", device_id, e);
            }
        }
        
        wear_info.wear_leveling_active = false;
        result
    }

    /// Analyze wear distribution across blocks
    fn analyze_wear_distribution(&self, device_id: BlockDeviceId, wear_info: &DeviceWearInfo) -> Result<WearCheckResult, BlockDeviceError> {
        let mut min_erase_count = u32::MAX;
        let mut max_erase_count = u32::MIN;
        let mut total_erase_count = 0u64;
        let mut block_count = 0;
        
        // Find min/max erase counts
        for block in wear_info.blocks.values() {
            if !block.is_free {
                min_erase_count = min_erase_count.min(block.erase_count);
                max_erase_count = max_erase_count.max(block.erase_count);
                total_erase_count += block.erase_count as u64;
                block_count += 1;
            }
        }
        
        let avg_erase_count = if block_count > 0 {
            total_erase_count / block_count
        } else {
            0
        };
        
        // Check if wear leveling is needed
        let wear_difference = max_erase_count.saturating_sub(min_erase_count);
        let needs_wear_leveling = wear_difference > wear_info.wear_level_threshold;
        
        // Identify target blocks for wear leveling
        let mut target_blocks = Vec::new();
        if needs_wear_leveling {
            for block in wear_info.blocks.values() {
                if block.erase_count > avg_erase_count as u32 + wear_info.wear_level_threshold / 2 {
                    target_blocks.push(block.block_id);
                }
            }
        }
        
        // Calculate estimated improvement
        let estimated_improvement = if needs_wear_leveling {
            (wear_difference as f32 / max_erase_count.max(1) as f32) * 100.0
        } else {
            0.0
        };
        
        let result = WearCheckResult {
            needs_wear_leveling,
            recommended_blocks_to_move: target_blocks.len() as u32,
            target_blocks,
            estimated_improvement,
        };
        
        info!("Wear analysis for device {:?}: wear diff = {}, avg = {}, needs leveling = {}", 
              device_id, wear_difference, avg_erase_count, needs_wear_leveling);
        
        Ok(result)
    }

    /// Perform static wear leveling
    fn perform_static_wear_leveling(&self, device_id: BlockDeviceId, analysis: &WearCheckResult) -> Result<(), BlockDeviceError> {
        info!("Performing static wear leveling for device {:?}", device_id);
        
        // Static wear leveling: move blocks from high-wear to low-wear areas
        let moves_performed = core::cmp::min(analysis.target_blocks.len() as u32, self.max_concurrent_moves) as usize;
        
        for i in 0..moves_performed {
            if i >= analysis.target_blocks.len() {
                break;
            }
            
            let block_id = analysis.target_blocks[i];
            info!("Moving high-wear block {} for device {:?}", block_id, device_id);
            
            // In real implementation, this would:
            // 1. Copy data from source block to free block
            // 2. Update mapping tables
            // 3. Erase source block
            // 4. Mark source block as free
            
            // Simulate the operation
            self.simulate_block_move(device_id, block_id)?;
        }
        
        Ok(())
    }

    /// Perform dynamic wear leveling
    fn perform_dynamic_wear_leveling(&self, device_id: BlockDeviceId, analysis: &WearCheckResult) -> Result<(), BlockDeviceError> {
        info!("Performing dynamic wear leveling for device {:?}", device_id);
        
        // Dynamic wear leveling considers write patterns and block health
        let mut device_info = self.device_info.write();
        let wear_info = device_info.get_mut(&device_id).unwrap();
        
        let moves_performed = core::cmp::min(analysis.target_blocks.len() as u32, self.max_concurrent_moves) as usize;
        
        for i in 0..moves_performed {
            if i >= analysis.target_blocks.len() {
                break;
            }
            
            let block_id = analysis.target_blocks[i];
            
            // Get block information
            let block = wear_info.blocks.get(&block_id).unwrap();
            
            // Find best target block based on criteria
            let target_block = self.find_best_target_block(device_id, block, &wear_info)?;
            
            if let Some(target) = target_block {
                info!("Dynamic move: block {} -> block {} for device {:?}", block_id, target, device_id);
                self.simulate_block_move(device_id, block_id)?;
            }
        }
        
        Ok(())
    }

    /// Perform advanced wear leveling
    fn perform_advanced_wear_leveling(&self, device_id: BlockDeviceId, analysis: &WearCheckResult) -> Result<(), BlockDeviceError> {
        info!("Performing advanced wear leveling for device {:?}", device_id);
        
        // Advanced wear leveling uses predictive algorithms
        let mut device_info = self.device_info.write();
        let wear_info = device_info.get_mut(&device_id).unwrap();
        
        // Predict future write patterns
        let predicted_hot_blocks = self.predict_hot_blocks(wear_info);
        
        // Consider predicted patterns in wear leveling decisions
        let moves_performed = core::cmp::min(analysis.target_blocks.len() as u32, self.max_concurrent_moves) as usize;
        
        for i in 0..moves_performed {
            if i >= analysis.target_blocks.len() {
                break;
            }
            
            let block_id = analysis.target_blocks[i];
            
            // Find optimal target considering predictions
            let target_block = self.find_optimal_target(device_id, block_id, &predicted_hot_blocks)?;
            
            if let Some(target) = target_block {
                info!("Advanced move: block {} -> block {} for device {:?}", block_id, target, device_id);
                self.simulate_block_move(device_id, block_id)?;
            }
        }
        
        Ok(())
    }

    /// Perform adaptive wear leveling
    fn perform_adaptive_wear_leveling(&self, device_id: BlockDeviceId, analysis:WearCheckResult) -> Result<(), BlockDeviceError> {
        info!("Performing adaptive wear leveling for device {:?}", device_id);
        
        // Adaptive wear leveling adjusts parameters based on usage patterns
        let mut device_info = self.device_info.write();
        let wear_info = device_info.get_mut(&device_id).unwrap();
        
        // Adapt wear threshold based on device age and usage
        let adaptation_factor = self.calculate_adaptation_factor(wear_info);
        wear_info.wear_level_threshold = (self.wear_threshold as f32 * adaptation_factor) as u32;
        
        // Perform wear leveling with adapted parameters
        self.perform_dynamic_wear_leveling(device_id, &analysis)
    }

    /// Find best target block for wear leveling
    fn find_best_target_block(&self, device_id: BlockDeviceId, source_block: &WearBlock, wear_info: &DeviceWearInfo) -> Result<Option<u64>, BlockDeviceError> {
        let mut best_block = None;
        let mut best_score = f32::MIN;
        
        for (&physical_block, block) in &wear_info.blocks {
            if block.is_free && block.erase_count < source_block.erase_count {
                // Calculate score based on multiple factors
                let score = self.calculate_block_score(block, source_block);
                
                if score > best_score {
                    best_score = score;
                    best_block = Some(physical_block);
                }
            }
        }
        
        Ok(best_block)
    }

    /// Find optimal target block considering predictions
    fn find_optimal_target(&self, device_id: BlockDeviceId, source_block_id: u64, predicted_hot_blocks: &BTreeMap<u64, f32>) -> Result<Option<u64>, BlockDeviceError> {
        let device_info = self.device_info.read();
        let wear_info = device_info.get(&device_id).unwrap();
        
        let source_block = wear_info.blocks.get(&source_block_id).unwrap();
        
        let mut best_block = None;
        let mut best_score = f32::MIN;
        
        for (&physical_block, block) in &wear_info.blocks {
            if block.is_free {
                // Calculate score considering predicted hot blocks
                let base_score = self.calculate_block_score(block, source_block);
                let hot_penalty = predicted_hot_blocks.get(&physical_block).unwrap_or(&0.0);
                let final_score = base_score - hot_penalty * 0.1; // Avoid moving hot data
                
                if final_score > best_score {
                    best_score = final_score;
                    best_block = Some(physical_block);
                }
            }
        }
        
        Ok(best_block)
    }

    /// Calculate block score for wear leveling decisions
    fn calculate_block_score(&self, target_block: &WearBlock, source_block: &WearBlock) -> f32 {
        let mut score = 0.0;
        
        // Lower erase count is better
        score += (source_block.erase_count - target_block.erase_count) as f32 * 2.0;
        
        // Recent writes penalty
        let time_since_last_write = target_block.last_written.elapsed().as_secs_f32();
        score += time_since_last_write * 0.1;
        
        // Error count penalty
        score -= target_block.error_count as f32 * 5.0;
        
        // Health bonus
        match target_block.health_status {
            BlockHealth::Excellent => score += 10.0,
            BlockHealth::Good => score += 5.0,
            BlockHealth::Fair => score += 0.0,
            BlockHealth::Poor => score -= 5.0,
            BlockHealth::Critical => score -= 10.0,
            BlockHealth::Failed => score -= 100.0,
        }
        
        score
    }

    /// Predict hot blocks based on write patterns
    fn predict_hot_blocks(&self, wear_info: &DeviceWearInfo) -> BTreeMap<u64, f32> {
        let mut predictions = BTreeMap::new();
        
        for block in wear_info.blocks.values() {
            if !block.is_free {
                // Predict based on recent write patterns
                let time_window = self.prediction_window.as_secs_f32();
                let write_rate = block.write_intensity;
                
                // Higher write rate = hotter block
                let hotness = write_rate * time_window / 3600.0; // Convert to per-hour rate
                
                if hotness > 1.0 {
                    predictions.insert(block.block_id, hotness);
                }
            }
        }
        
        predictions
    }

    /// Calculate adaptation factor for adaptive wear leveling
    fn calculate_adaptation_factor(&self, wear_info: &DeviceWearInfo) -> f32 {
        let usage_intensity = wear_info.statistics.total_writes as f32 / wear_info.total_blocks as f32;
        let age_factor = wear_info.current_erase_cycles as f32 / wear_info.max_erase_cycles as f32;
        
        // Adapt based on usage and age
        let adaptation = 1.0 + usage_intensity * 0.5 + age_factor * 0.3;
        
        // Cap adaptation
        adaptation.min(2.0).max(0.5)
    }

    /// Calculate write intensity for a block
    fn calculate_write_intensity(&self, block: &WearBlock) -> f32 {
        let time_elapsed = block.last_written.elapsed().as_secs_f32();
        if time_elapsed > 0.0 {
            block.access_count as f32 / time_elapsed
        } else {
            0.0
        }
    }

    /// Check if wear leveling is needed
    fn check_wear_leveling(&self, device_id: BlockDeviceId, device_info: &mut BTreeMap<BlockDeviceId, DeviceWearInfo>) {
        let wear_info = device_info.get_mut(&device_id).unwrap();
        
        // Check if it's time to check wear levels
        if wear_info.last_wear_check.elapsed() < wear_info.wear_check_interval {
            return;
        }
        
        // Trigger wear leveling check
        let _ = self.analyze_wear_distribution(device_id, wear_info);
        
        wear_info.last_wear_check = Instant::now();
    }

    /// Simulate block move operation
    fn simulate_block_move(&self, device_id: BlockDeviceId, block_id: u64) -> Result<(), BlockDeviceError> {
        info!("Simulating block move for device {:?}, block {}", device_id, block_id);
        
        // In real implementation, this would perform actual block movement
        // For now, just update statistics
        
        let mut device_info = self.device_info.write();
        let wear_info = device_info.get_mut(&device_id).unwrap();
        
        if let Some(block) = wear_info.blocks.get_mut(&block_id) {
            // Simulate erase cycle
            block.erase_count += 1;
            block.last_erased = Instant::now();
            
            // Update device statistics
            wear_info.statistics.total_erasures += 1;
            wear_info.current_erase_cycles = wear_info.current_erase_cycles.max(block.erase_count);
            
            // Update health status
            block.health_status = self.calculate_block_health(block.erase_count, wear_info.max_erase_cycles);
        }
        
        Ok(())
    }

    /// Calculate block health status
    fn calculate_block_health(&self, erase_count: u32, max_cycles: u32) -> BlockHealth {
        let remaining_ratio = 1.0 - (erase_count as f32 / max_cycles as f32);
        
        if remaining_ratio >= 0.9 {
            BlockHealth::Excellent
        } else if remaining_ratio >= 0.7 {
            BlockHealth::Good
        } else if remaining_ratio >= 0.5 {
            BlockHealth::Fair
        } else if remaining_ratio >= 0.3 {
            BlockHealth::Poor
        } else if remaining_ratio > 0.0 {
            BlockHealth::Critical
        } else {
            BlockHealth::Failed
        }
    }

    /// Get wear leveling statistics for a device
    pub fn get_wear_statistics(&self, device_id: BlockDeviceId) -> Result<WearStatistics, BlockDeviceError> {
        let stats = self.statistics.read();
        match stats.get(&device_id) {
            Some(device_stats) => Ok(device_stats.clone()),
            None => Err(BlockDeviceError::DeviceNotFound),
        }
    }

    /// Get overall wear health for a device
    pub fn get_device_health(&self, device_id: BlockDeviceId) -> Result<DeviceHealth, BlockDeviceError> {
        let device_info = self.device_info.read();
        let wear_info = device_info.get(&device_id).ok_or(BlockDeviceError::DeviceNotFound)?;
        
        let mut total_blocks = 0;
        let mut healthy_blocks = 0;
        let mut failed_blocks = 0;
        
        for block in wear_info.blocks.values() {
            total_blocks += 1;
            match block.health_status {
                BlockHealth::Failed => failed_blocks += 1,
                BlockHealth::Excellent | BlockHealth::Good | BlockHealth::Fair | BlockHealth::Poor | BlockHealth::Critical => {
                    healthy_blocks += 1;
                }
            }
        }
        
        let health_ratio = if total_blocks > 0 {
            healthy_blocks as f32 / total_blocks as f32
        } else {
            1.0
        };
        
        Ok(DeviceHealth {
            overall_health: health_ratio,
            total_blocks,
            healthy_blocks,
            failed_blocks,
            avg_erase_cycles: wear_info.current_erase_cycles as f32,
            estimated_lifespan_remaining: self.estimate_lifespan_remaining(wear_info),
        })
    }

    /// Estimate remaining device lifespan
    fn estimate_lifespan_remaining(&self, wear_info: &DeviceWearInfo) -> Duration {
        if wear_info.current_erase_cycles >= wear_info.max_erase_cycles {
            return Duration::from_secs(0);
        }
        
        let remaining_cycles = wear_info.max_erase_cycles - wear_info.current_erase_cycles;
        let usage_rate = wear_info.statistics.total_writes as f32 / wear_info.total_blocks as f32;
        
        if usage_rate > 0.0 {
            let remaining_days = remaining_cycles as f32 / usage_rate / 86400.0; // Convert to days
            Duration::from_secs_f64(remaining_days * 86400.0)
        } else {
            Duration::from_secs(365 * 24 * 60 * 60) // Assume 1 year if no usage
        }
    }
}

/// Device health information
#[derive(Debug, Clone)]
pub struct DeviceHealth {
    pub overall_health: f32,           // 0.0 to 1.0
    pub total_blocks: u64,
    pub healthy_blocks: u64,
    pub failed_blocks: u64,
    pub avg_erase_cycles: f32,
    pub estimated_lifespan_remaining: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wear_leveling_manager_creation() {
        let manager = WearLevelingManager::new(WearLevelingStrategy::Dynamic);
        assert_eq!(manager.strategy, WearLevelingStrategy::Dynamic);
    }

    #[test]
    fn test_block_health_calculation() {
        let manager = WearLevelingManager::new(WearLevelingStrategy::None);
        let health = manager.calculate_block_health(500, 3000);
        assert_eq!(health, BlockHealth::Good);
    }

    #[test]
    fn test_device_health() {
        let health = DeviceHealth {
            overall_health: 0.95,
            total_blocks: 1000,
            healthy_blocks: 950,
            failed_blocks: 50,
            avg_erase_cycles: 1500.0,
            estimated_lifespan_remaining: Duration::from_secs(86400 * 30), // 30 days
        };
        
        assert_eq!(health.overall_health, 0.95);
    }
}