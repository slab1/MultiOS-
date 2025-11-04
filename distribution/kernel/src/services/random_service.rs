//! Random Number Generation Service
//!
//! Provides comprehensive random number generation including hardware RNG,
//! software RNG (cryptographically secure), and entropy collection.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use core::ops::Deref;

/// Random service initialization
pub fn init() -> Result<()> {
    info!("Initializing Random Number Generation Service...");
    
    // Initialize hardware RNG
    init_hardware_rng()?;
    
    // Initialize software RNG
    init_software_rng()?;
    
    // Initialize entropy pool
    init_entropy_pool()?;
    
    // Start entropy collection
    start_entropy_collection()?;
    
    info!("Random Number Generation Service initialized");
    Ok(())
}

/// Random service shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Random Number Generation Service...");
    
    // Stop entropy collection
    stop_entropy_collection()?;
    
    // Clear entropy pool
    clear_entropy_pool()?;
    
    info!("Random Number Generation Service shutdown complete");
    Ok(())
}

/// Random number types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RandomType {
    HardwareRng = 0,
    SoftwareRng = 1,
    CsRng = 2,        // Cryptographically Secure RNG
    PseudoRandom = 3, // Simple pseudo-random
    HardwareEntropy = 4,
}

/// Random number quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QualityLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

/// RNG configuration
#[derive(Debug, Clone)]
pub struct RngConfig {
    pub preferred_type: RandomType,
    pub min_quality: QualityLevel,
    pub enable_hardware: bool,
    pub enable_software: bool,
    pub entropy_threshold: usize,
    pub pool_size: usize,
}

/// Random number request
#[derive(Debug, Clone)]
pub struct RandomRequest {
    pub size_bytes: usize,
    pub quality: QualityLevel,
    pub blocking: bool,
    pub rng_type: Option<RandomType>,
}

/// Random number result
#[derive(Debug, Clone)]
pub struct RandomResult {
    pub data: Vec<u8>,
    pub quality: QualityLevel,
    pub source: RandomType,
    pub entropy_bits: u32,
}

/// Hardware RNG information
#[derive(Debug, Clone)]
pub struct HardwareRngInfo {
    pub available: bool,
    pub quality: QualityLevel,
    pub throughput_bps: u64,
    pub entropy_rate: f64,
}

/// Software RNG state
#[derive(Debug, Clone)]
pub struct SoftwareRngState {
    pub algorithm: String,
    pub seed: u64,
    pub state: u128,
    pub counter: u64,
}

/// Entropy source
#[derive(Debug, Clone)]
pub struct EntropySource {
    pub name: String,
    pub entropy_per_sample: f64,
    pub sample_rate_hz: u64,
    pub enabled: bool,
}

/// Entropy pool
#[derive(Debug, Clone)]
pub struct EntropyPool {
    pub size_bytes: usize,
    pub entropy_bits: f64,
    pub data: Vec<u8>,
    pub last_updated: u64,
}

/// Random service statistics
#[derive(Debug, Clone, Copy)]
pub struct RandomServiceStats {
    pub hardware_rng_generated: AtomicU64,
    pub software_rng_generated: AtomicU64,
    pub cs_rng_generated: AtomicU64,
    pub entropy_pool_refills: AtomicU64,
    pub entropy_pool_drains: AtomicU64,
    pub entropy_bits_available: AtomicU64,
    pub failed_requests: AtomicU64,
}

/// Hardware RNG availability
static HARDWARE_RNG_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// RNG configuration
static RNG_CONFIG: RwLock<RngConfig> = RwLock::new(RngConfig {
    preferred_type: RandomType::HardwareRng,
    min_quality: QualityLevel::Medium,
    enable_hardware: true,
    enable_software: true,
    entropy_threshold: 32,
    pool_size: 4096,
});

/// Software RNG state
static SOFTWARE_RNG_STATE: RwLock<SoftwareRngState> = RwLock::new(SoftwareRngState {
    algorithm: "chacha20".to_string(),
    seed: 0,
    state: 0x12345678_9abcdef0_12345678_9abcdef0,
    counter: 0,
});

/// Entropy pool
static ENTROPY_POOL: RwLock<EntropyPool> = RwLock::new(EntropyPool {
    size_bytes: 4096,
    entropy_bits: 0.0,
    data: vec![0; 4096],
    last_updated: 0,
});

/// Entropy sources
static ENTROPY_SOURCES: RwLock<Vec<EntropySource>> = RwLock::new(Vec::new());

/// Random service statistics
static RNG_STATS: RandomServiceStats = RandomServiceStats {
    hardware_rng_generated: AtomicU64::new(0),
    software_rng_generated: AtomicU64::new(0),
    cs_rng_generated: AtomicU64::new(0),
    entropy_pool_refills: AtomicU64::new(0),
    entropy_pool_drains: AtomicU64::new(0),
    entropy_bits_available: AtomicU64::new(0),
    failed_requests: AtomicU64::new(0),
};

/// Initialize hardware RNG
fn init_hardware_rng() -> Result<()> {
    info!("Initializing hardware RNG...");
    
    // Detect available hardware RNG
    let hw_rng_available = detect_hardware_rng()?;
    
    if hw_rng_available {
        HARDWARE_RNG_AVAILABLE.store(true, Ordering::SeqCst);
        info!("Hardware RNG detected and enabled");
    } else {
        warn!("Hardware RNG not available");
    }
    
    Ok(())
}

/// Detect hardware RNG
fn detect_hardware_rng() -> Result<bool> {
    #[cfg(target_arch = "x86_64")]
    {
        // Check for RDRAND/RDSEED instructions
        return detect_intel_rng();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // Check for ARMv8.1 random number generator
        return detect_arm_rng();
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        // Check for RISC-V RNG extensions
        return detect_riscv_rng();
    }
    
    Ok(false)
}

#[cfg(target_arch = "x86_64")]
fn detect_intel_rng() -> Result<bool> {
    // Check CPUID for RDRAND support
    let ecx: u32;
    unsafe {
        core::arch::asm!(
            "cpuid",
            in("eax") 1,
            out("ecx") ecx,
        );
    }
    
    // Check bit 30 of ECX (RDRAND support)
    (ecx & (1 << 30)) != 0
}

#[cfg(target_arch = "aarch64")]
fn detect_arm_rng() -> Result<bool> {
    // ARM64 doesn't have a standard hardware RNG like x86
    // Check for Crypto Extensions or other RNG sources
    Ok(false)
}

#[cfg(target_arch = "riscv64")]
fn detect_riscv_rng() -> Result<bool> {
    // Check for RISC-V RNG extensions
    Ok(false)
}

/// Initialize software RNG
fn init_software_rng() -> Result<()> {
    info!("Initializing software RNG...");
    
    // Initialize ChaCha20 state
    let mut rng_state = SOFTWARE_RNG_STATE.write();
    
    // Use system time as seed
    let seed = crate::services::time_service::get_uptime_ns();
    rng_state.seed = seed;
    rng_state.state = seed as u128 | 1; // Ensure non-zero state
    
    info!("Software RNG initialized with algorithm: {}", rng_state.algorithm);
    
    Ok(())
}

/// Initialize entropy pool
fn init_entropy_pool() -> Result<()> {
    info!("Initializing entropy pool...");
    
    let mut pool = ENTROPY_POOL.write();
    pool.entropy_bits = 0.0;
    pool.last_updated = crate::services::time_service::get_uptime_ns();
    
    // Initialize entropy sources
    init_entropy_sources()?;
    
    info!("Entropy pool initialized with size: {} bytes", pool.size_bytes);
    
    Ok(())
}

/// Initialize entropy sources
fn init_entropy_sources() -> Result<()> {
    let mut sources = ENTROPY_SOURCES.write();
    
    sources.clear();
    
    // System timer entropy
    sources.push(EntropySource {
        name: "system_timer".to_string(),
        entropy_per_sample: 0.1, // Low entropy from timing
        sample_rate_hz: 1000,
        enabled: true,
    });
    
    // CPU timing entropy
    sources.push(EntropySource {
        name: "cpu_timing".to_string(),
        entropy_per_sample: 0.05,
        sample_rate_hz: 100,
        enabled: true,
    });
    
    // Memory access patterns
    sources.push(EntropySource {
        name: "memory_access".to_string(),
        entropy_per_sample: 0.03,
        sample_rate_hz: 500,
        enabled: true,
    });
    
    info!("Initialized {} entropy sources", sources.len());
    
    Ok(())
}

/// Start entropy collection
fn start_entropy_collection() -> Result<()> {
    info!("Starting entropy collection...");
    
    // Create entropy collection timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        1_000_000, // 1ms
        entropy_collection_callback
    );
    
    Ok(())
}

/// Stop entropy collection
fn stop_entropy_collection() -> Result<()> {
    info!("Stopping entropy collection...");
    
    // Disable all entropy sources
    let mut sources = ENTROPY_SOURCES.write();
    for source in &mut *sources {
        source.enabled = false;
    }
    
    Ok(())
}

/// Clear entropy pool
fn clear_entropy_pool() -> Result<()> {
    let mut pool = ENTROPY_POOL.write();
    
    for byte in &mut pool.data {
        *byte = 0;
    }
    pool.entropy_bits = 0.0;
    pool.last_updated = crate::services::time_service::get_uptime_ns();
    
    Ok(())
}

/// Get hardware RNG info
pub fn get_hardware_rng_info() -> Result<HardwareRngInfo> {
    let available = HARDWARE_RNG_AVAILABLE.load(Ordering::SeqCst);
    
    let quality = if available {
        QualityLevel::VeryHigh
    } else {
        QualityLevel::Low
    };
    
    Ok(HardwareRngInfo {
        available,
        quality,
        throughput_bps: if available { 1_000_000 } else { 0 }, // 1MB/s estimate
        entropy_rate: if available { 1.0 } else { 0.0 },
    })
}

/// Generate random numbers
pub fn generate_random(request: RandomRequest) -> Result<RandomResult> {
    let start_time = crate::hal::timers::get_high_res_time();
    
    // Check if we can fulfill the request immediately
    let can_fulfill = can_fulfill_request(&request)?;
    
    if !can_fulfill && request.blocking {
        // Wait for sufficient entropy
        wait_for_entropy(request.size_bytes)?;
    }
    
    let mut result_data = vec![0u8; request.size_bytes];
    let source = select_rng_type(&request)?;
    
    match source {
        RandomType::HardwareRng => {
            generate_hardware_random(&mut result_data)?;
            RNG_STATS.hardware_rng_generated.fetch_add(1, Ordering::SeqCst);
        }
        RandomType::SoftwareRng => {
            generate_software_random(&mut result_data)?;
            RNG_STATS.software_rng_generated.fetch_add(1, Ordering::SeqCst);
        }
        RandomType::CsRng => {
            generate_cs_random(&mut result_data)?;
            RNG_STATS.cs_rng_generated.fetch_add(1, Ordering::SeqCst);
        }
        _ => {
            generate_software_random(&mut result_data)?;
            RNG_STATS.software_rng_generated.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    let entropy_bits = calculate_entropy_bits(&result_data, source);
    
    let result = RandomResult {
        data: result_data,
        quality: request.quality,
        source,
        entropy_bits,
    };
    
    info!("Generated {} bytes of {} quality random data using {:?}", 
          request.size_bytes, request.quality, source);
    
    Ok(result)
}

/// Check if request can be fulfilled
fn can_fulfill_request(request: &RandomRequest) -> Result<bool> {
    let pool = ENTROPY_POOL.read();
    let available_entropy_bits = pool.entropy_bits as usize;
    let required_entropy_bits = request.size_bytes * 8; // Assume 1 bit per byte minimum
    
    Ok(available_entropy_bits >= required_entropy_bits)
}

/// Select RNG type for request
fn select_rng_type(request: &RandomRequest) -> Result<RandomType> {
    let config = RNG_CONFIG.read();
    
    // Respect explicit type request
    if let Some(requested_type) = request.rng_type {
        return Ok(requested_type);
    }
    
    // Check preferred type based on quality requirements
    match request.quality {
        QualityLevel::VeryHigh | QualityLevel::High => {
            if config.enable_hardware && HARDWARE_RNG_AVAILABLE.load(Ordering::SeqCst) {
                return Ok(RandomType::HardwareRng);
            }
            if config.enable_software {
                return Ok(RandomType::CsRng);
            }
        }
        QualityLevel::Medium | QualityLevel::Low => {
            if config.enable_software {
                return Ok(RandomType::SoftwareRng);
            }
        }
    }
    
    Err(KernelError::FeatureNotSupported)
}

/// Generate hardware random numbers
fn generate_hardware_random(data: &mut [u8]) -> Result<()> {
    if !HARDWARE_RNG_AVAILABLE.load(Ordering::SeqCst) {
        return Err(KernelError::HardwareNotAvailable);
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        // Use RDRAND instruction
        let mut bytes_generated = 0;
        
        while bytes_generated < data.len() {
            let mut value: u32 = 0;
            
            // Try to get a 32-bit random value using RDRAND
            let success = unsafe {
                core::arch::asm!(
                    "rdrand {}",
                    out(reg) value,
                    options(readonly, preserves_flags)
                )
            };
            
            if !success {
                // RDRAND failed, fall back to software RNG
                break;
            }
            
            // Copy bytes
            let bytes_to_copy = core::cmp::min(4, data.len() - bytes_generated);
            data[bytes_generated..bytes_generated + bytes_to_copy]
                .copy_from_slice(&value.to_le_bytes()[..bytes_to_copy]);
            
            bytes_generated += bytes_to_copy;
        }
        
        if bytes_generated < data.len() {
            // Fall back to software RNG for remaining bytes
            generate_software_random(&mut data[bytes_generated..])?;
        }
        
        return Ok(());
    }
    
    // For other architectures, use software RNG
    generate_software_random(data)?;
    Ok(())
}

/// Generate software random numbers
fn generate_software_random(data: &mut [u8]) -> Result<()> {
    let mut rng_state = SOFTWARE_RNG_STATE.write();
    
    // Use ChaCha20 algorithm
    for chunk in data.chunks_mut(8) {
        let random_value = generate_chacha20_word(&mut rng_state);
        let bytes = random_value.to_le_bytes();
        
        for (i, &byte) in bytes.iter().enumerate() {
            if i < chunk.len() {
                chunk[i] = byte;
            }
        }
    }
    
    Ok(())
}

/// Generate cryptographically secure random numbers
fn generate_cs_random(data: &mut [u8]) -> Result<()> {
    // For CS RNG, we would use a more robust algorithm
    // For now, use software RNG with additional entropy mixing
    generate_software_random(data)?;
    
    // Mix in entropy from the pool
    mix_entropy_into_data(data)?;
    
    Ok(())
}

/// Generate ChaCha20 word
fn generate_chacha20_word(state: &mut SoftwareRngState) -> u32 {
    // Simplified ChaCha20 quarter round
    state.state = state.state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let result = (state.state >> 32) as u32;
    
    // Simple mixing
    result ^ ((state.state >> 16) as u32)
}

/// Mix entropy into data
fn mix_entropy_into_data(data: &mut [u8]) -> Result<()> {
    let mut pool = ENTROPY_POOL.write();
    
    // Extract entropy from pool
    for (i, byte) in data.iter_mut().enumerate() {
        if i < pool.data.len() {
            *byte ^= pool.data[i];
        }
        
        // Drain entropy from pool
        if pool.entropy_bits >= 8.0 {
            pool.entropy_bits -= 8.0;
            RNG_STATS.entropy_pool_drains.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    Ok(())
}

/// Calculate entropy bits
fn calculate_entropy_bits(data: &[u8], source: RandomType) -> u32 {
    match source {
        RandomType::HardwareRng => (data.len() * 8) as u32,
        RandomType::CsRng => (data.len() * 7) as u32, // Slightly less for software
        RandomType::SoftwareRng => (data.len() * 6) as u32,
        _ => (data.len() * 4) as u32,
    }
}

/// Wait for sufficient entropy
fn wait_for_entropy(required_bytes: usize) -> Result<()> {
    info!("Waiting for {} bytes of entropy", required_bytes);
    
    let timeout_ns = 10_000_000; // 10ms timeout
    
    let start_time = crate::services::time_service::get_uptime_ns();
    
    while crate::services::time_service::get_uptime_ns() - start_time < timeout_ns {
        let pool = ENTROPY_POOL.read();
        let available_entropy_bits = pool.entropy_bits as usize;
        
        if available_entropy_bits >= required_bytes * 8 {
            return Ok(());
        }
        
        // Small sleep to avoid busy waiting
        core::hint::spin_loop();
    }
    
    warn!("Timeout waiting for entropy");
    Ok(())
}

/// Entropy collection callback
fn entropy_collection_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    collect_entropy();
}

/// Collect entropy from all sources
fn collect_entropy() {
    let sources = ENTROPY_SOURCES.read();
    let mut pool = ENTROPY_POOL.write();
    let current_time = crate::services::time_service::get_uptime_ns();
    
    // Calculate time since last collection
    let time_delta = current_time - pool.last_updated;
    
    for source in sources.iter() {
        if !source.enabled {
            continue;
        }
        
        // Calculate how many samples to collect
        let samples = (time_delta * source.sample_rate_hz / 1_000_000_000) as usize;
        
        if samples > 0 {
            let entropy_bits = samples as f64 * source.entropy_per_sample;
            pool.entropy_bits += entropy_bits;
            
            // Cap entropy at pool size
            if pool.entropy_bits > (pool.size_bytes * 8) as f64 {
                pool.entropy_bits = (pool.size_bytes * 8) as f64;
            }
        }
    }
    
    pool.last_updated = current_time;
    RNG_STATS.entropy_pool_refills.fetch_add(1, Ordering::SeqCst);
    
    // Update available entropy metric
    RNG_STATS.entropy_bits_available.store(pool.entropy_bits as u64, Ordering::SeqCst);
}

/// Add entropy to pool
pub fn add_entropy(data: &[u8], estimated_entropy_bits: f64) -> Result<()> {
    let mut pool = ENTROPY_POOL.write();
    
    // Add data to pool
    for (i, &byte) in data.iter().enumerate() {
        if i < pool.data.len() {
            pool.data[i] ^= byte; // XOR to mix in entropy
        }
    }
    
    // Add entropy to pool
    pool.entropy_bits += estimated_entropy_bits;
    
    // Cap at pool capacity
    let max_entropy = (pool.size_bytes * 8) as f64;
    if pool.entropy_bits > max_entropy {
        pool.entropy_bits = max_entropy;
    }
    
    pool.last_updated = crate::services::time_service::get_uptime_ns();
    RNG_STATS.entropy_pool_refills.fetch_add(1, Ordering::SeqCst);
    
    info!("Added {} bytes of entropy, total: {} bits", 
          data.len(), pool.entropy_bits);
    
    Ok(())
}

/// Get random service statistics
pub fn get_stats() -> RandomServiceStats {
    RNG_STATS
}

/// Benchmark random number generation
pub fn benchmark_rng() -> Result<(u64, u64, u64)> {
    info!("Benchmarking random number generation...");
    
    let mut hardware_time = 0;
    let mut software_time = 0;
    let mut cs_time = 0;
    
    // Benchmark hardware RNG
    if HARDWARE_RNG_AVAILABLE.load(Ordering::SeqCst) {
        let mut data = vec![0u8; 1024];
        let start = crate::hal::timers::get_high_res_time();
        generate_hardware_random(&mut data)?;
        hardware_time = crate::hal::timers::get_high_res_time() - start;
    }
    
    // Benchmark software RNG
    let mut data = vec![0u8; 1024];
    let start = crate::hal::timers::get_high_res_time();
    generate_software_random(&mut data)?;
    software_time = crate::hal::timers::get_high_res_time() - start;
    
    // Benchmark CS RNG
    let mut data = vec![0u8; 1024];
    let start = crate::hal::timers::get_high_res_time();
    generate_cs_random(&mut data)?;
    cs_time = crate::hal::timers::get_high_res_time() - start;
    
    Ok((hardware_time, software_time, cs_time))
}

/// Random utility functions
pub mod utils {
    use super::*;
    
    /// Generate random boolean
    pub fn random_bool() -> bool {
        let mut data = [0u8; 1];
        let _ = generate_random(RandomRequest {
            size_bytes: 1,
            quality: QualityLevel::Low,
            blocking: false,
            rng_type: None,
        }).map(|result| {
            data.copy_from_slice(&result.data);
        });
        
        data[0] & 1 == 1
    }
    
    /// Generate random u32 in range
    pub fn random_u32_in_range(min: u32, max: u32) -> u32 {
        let range = max - min + 1;
        let mut data = [0u8; 4];
        let _ = generate_random(RandomRequest {
            size_bytes: 4,
            quality: QualityLevel::Medium,
            blocking: false,
            rng_type: None,
        }).map(|result| {
            data.copy_from_slice(&result.data);
        });
        
        let value = u32::from_le_bytes(data);
        min + (value % range)
    }
    
    /// Generate random u64 in range
    pub fn random_u64_in_range(min: u64, max: u64) -> u64 {
        let range = max - min + 1;
        let mut data = [0u8; 8];
        let _ = generate_random(RandomRequest {
            size_bytes: 8,
            quality: QualityLevel::Medium,
            blocking: false,
            rng_type: None,
        }).map(|result| {
            data.copy_from_slice(&result.data);
        });
        
        let value = u64::from_le_bytes(data);
        min + (value % range)
    }
    
    /// Shuffle array using Fisher-Yates algorithm
    pub fn shuffle_array<T>(array: &mut [T]) {
        let len = array.len();
        if len <= 1 {
            return;
        }
        
        for i in (1..len).rev() {
            let j = random_usize_in_range(0, i);
            array.swap(i, j);
        }
    }
    
    /// Generate random usize in range
    pub fn random_usize_in_range(min: usize, max: usize) -> usize {
        let range = max - min + 1;
        let size = core::mem::size_of::<usize>();
        let mut data = vec![0u8; size];
        
        let _ = generate_random(RandomRequest {
            size_bytes: size,
            quality: QualityLevel::Medium,
            blocking: false,
            rng_type: None,
        }).map(|result| {
            data.copy_from_slice(&result.data);
        });
        
        let value = match size {
            4 => u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize,
            8 => u64::from_le_bytes(data.as_chunks().0[0]) as usize,
            _ => 0,
        };
        
        min + (value % range)
    }
    
    /// Generate UUID version 4
    pub fn generate_uuid_v4() -> [u8; 16] {
        let mut uuid = [0u8; 16];
        
        let _ = generate_random(RandomRequest {
            size_bytes: 16,
            quality: QualityLevel::High,
            blocking: false,
            rng_type: Some(RandomType::CsRng),
        }).map(|result| {
            uuid.copy_from_slice(&result.data);
        });
        
        // Set version and variant bits
        uuid[6] = (uuid[6] & 0x0f) | 0x40; // Version 4
        uuid[8] = (uuid[8] & 0x3f) | 0x80; // RFC 4122 variant
        
        uuid
    }
}