//! Delta update implementation for efficient incremental updates
//! 
//! This module provides binary diff algorithms, delta compression,
//! and bandwidth optimization for efficient system updates.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::hash::{Hash, Hasher};
use core::fmt;

use crate::log::LogLevel;
use crate::security::{EncryptionManager, CryptographicHash};

/// Binary diff algorithm types for different use cases
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffAlgorithm {
    /// Fast and efficient for similar binary data
    Bsdiff,
    /// Lightweight diff for memory-constrained environments
    Xdelta3,
    /// Custom implementation optimized for kernel updates
    KernelOptimized,
}

/// Delta update result containing compressed patch data
#[derive(Debug)]
pub struct DeltaPatch {
    /// Algorithm used for generating the delta
    pub algorithm: DiffAlgorithm,
    /// Compressed delta data
    pub patch_data: Vec<u8>,
    /// Original file hash for verification
    pub original_hash: CryptographicHash,
    /// Target file hash for verification
    pub target_hash: CryptographicHash,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Metadata about the patch
    pub metadata: PatchMetadata,
}

/// Metadata about the delta patch
#[derive(Debug, Clone)]
pub struct PatchMetadata {
    /// Size of original file in bytes
    pub original_size: usize,
    /// Size of target file in bytes
    pub target_size: usize,
    /// Number of differences found
    pub diff_count: usize,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Performance metrics for delta operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time taken to generate delta in milliseconds
    pub generation_time_ms: u64,
    /// Memory usage peak in bytes
    pub peak_memory_bytes: usize,
    /// Bandwidth savings percentage
    pub bandwidth_savings: f64,
}

/// Binary diff engine with multiple algorithm support
pub struct BinaryDiffEngine {
    /// Configured algorithm
    algorithm: DiffAlgorithm,
    /// Encryption manager for secure operations
    encryption: Option<EncryptionManager>,
    /// Maximum chunk size for processing
    max_chunk_size: usize,
    /// Enable compression
    enable_compression: bool,
}

impl BinaryDiffEngine {
    /// Create a new binary diff engine
    pub fn new(algorithm: DiffAlgorithm) -> Self {
        Self {
            algorithm,
            encryption: None,
            max_chunk_size: 16 * 1024 * 1024, // 16MB chunks
            enable_compression: true,
        }
    }

    /// Set encryption manager for secure operations
    pub fn set_encryption_manager(&mut self, encryption: EncryptionManager) {
        self.encryption = Some(encryption);
    }

    /// Set maximum chunk size for memory-efficient processing
    pub fn set_max_chunk_size(&mut self, size: usize) {
        self.max_chunk_size = min(size, 64 * 1024 * 1024); // Max 64MB
    }

    /// Generate delta patch between two binary data streams
    pub fn generate_delta(
        &self,
        original: &[u8],
        target: &[u8],
        target_hash: CryptographicHash,
    ) -> Result<DeltaPatch, DeltaError> {
        let start_time = crate::hal::timers::get_system_time_ms();
        
        // Hash original data for verification
        let original_hash = self.hash_data(original)?;
        
        match self.algorithm {
            DiffAlgorithm::KernelOptimized => {
                self.generate_kernel_optimized_delta(original, target, original_hash, target_hash, start_time)
            }
            DiffAlgorithm::Bsdiff => {
                self.generate_bsdiff_delta(original, target, original_hash, target_hash, start_time)
            }
            DiffAlgorithm::Xdelta3 => {
                self.generate_xdelta_delta(original, target, original_hash, target_hash, start_time)
            }
        }
    }

    /// Apply delta patch to original data
    pub fn apply_delta(
        &self,
        original: &[u8],
        patch: &DeltaPatch,
    ) -> Result<Vec<u8>, DeltaError> {
        // Verify original data hash
        let current_hash = self.hash_data(original)?;
        if current_hash != patch.original_hash {
            return Err(DeltaError::HashMismatch {
                expected: patch.original_hash,
                actual: current_hash,
            });
        }

        match patch.algorithm {
            DiffAlgorithm::KernelOptimized => {
                self.apply_kernel_optimized_delta(original, patch)
            }
            DiffAlgorithm::Bsdiff => {
                self.apply_bsdiff_delta(original, patch)
            }
            DiffAlgorithm::Xdelta3 => {
                self.apply_xdelta_delta(original, patch)
            }
        }
    }

    /// Kernel-optimized delta algorithm (custom implementation)
    fn generate_kernel_optimized_delta(
        &self,
        original: &[u8],
        target: &[u8],
        original_hash: CryptographicHash,
        target_hash: CryptographicHash,
        start_time: u64,
    ) -> Result<DeltaPatch, DeltaError> {
        let mut patch_data = Vec::new();
        let mut diff_count = 0;

        // Process data in chunks for memory efficiency
        let chunk_size = self.max_chunk_size;
        let mut pos = 0;

        while pos < original.len() || pos < target.len() {
            let orig_end = min(pos + chunk_size, original.len());
            let target_end = min(pos + chunk_size, target.len());
            
            let orig_chunk = &original[pos..orig_end];
            let target_chunk = &target[pos..target_end];

            // Generate delta for this chunk
            let chunk_delta = self.diff_chunks(orig_chunk, target_chunk)?;
            patch_data.extend(chunk_delta);
            
            diff_count += 1;
            pos += chunk_size;
        }

        let patch_data = if self.enable_compression {
            self.compress_patch_data(&patch_data)?
        } else {
            patch_data
        };

        let generation_time = crate::hal::timers::get_system_time_ms() - start_time;
        let compression_ratio = if patch_data.len() > 0 {
            1.0 - (patch_data.len() as f64) / (target.len() as f64)
        } else {
            0.0
        };

        Ok(DeltaPatch {
            algorithm: DiffAlgorithm::KernelOptimized,
            patch_data,
            original_hash,
            target_hash,
            compression_ratio,
            metadata: PatchMetadata {
                original_size: original.len(),
                target_size: target.len(),
                diff_count,
                performance: PerformanceMetrics {
                    generation_time_ms: generation_time,
                    peak_memory_bytes: self.max_chunk_size * 2,
                    bandwidth_savings: compression_ratio * 100.0,
                },
            },
        })
    }

    /// Generate BSDiff-compatible delta
    fn generate_bsdiff_delta(
        &self,
        original: &[u8],
        target: &[u8],
        original_hash: CryptographicHash,
        target_hash: CryptographicHash,
        start_time: u64,
    ) -> Result<DeltaPatch, DeltaError> {
        // Simplified BSDiff implementation
        // In a real implementation, this would use a proper BSDiff algorithm
        
        let mut patch_data = Vec::new();
        let diff_count = self.generate_simple_diff(original, target, &mut patch_data)?;
        
        let patch_data = if self.enable_compression {
            self.compress_patch_data(&patch_data)?
        } else {
            patch_data
        };

        let generation_time = crate::hal::timers::get_system_time_ms() - start_time;
        let compression_ratio = (1.0 - (patch_data.len() as f64) / (target.len() as f64)).max(0.0);

        Ok(DeltaPatch {
            algorithm: DiffAlgorithm::Bsdiff,
            patch_data,
            original_hash,
            target_hash,
            compression_ratio,
            metadata: PatchMetadata {
                original_size: original.len(),
                target_size: target.len(),
                diff_count,
                performance: PerformanceMetrics {
                    generation_time_ms: generation_time,
                    peak_memory_bytes: original.len() + target.len(),
                    bandwidth_savings: compression_ratio * 100.0,
                },
            },
        })
    }

    /// Generate xdelta3-compatible delta
    fn generate_xdelta_delta(
        &self,
        original: &[u8],
        target: &[u8],
        original_hash: CryptographicHash,
        target_hash: CryptographicHash,
        start_time: u64,
    ) -> Result<DeltaPatch, DeltaError> {
        // Simplified xdelta3 implementation
        let mut patch_data = Vec::new();
        let diff_count = self.generate_simple_diff(original, target, &mut patch_data)?;
        
        let patch_data = if self.enable_compression {
            self.compress_patch_data(&patch_data)?
        } else {
            patch_data
        };

        let generation_time = crate::hal::timers::get_system_time_ms() - start_time;
        let compression_ratio = (1.0 - (patch_data.len() as f64) / (target.len() as f64)).max(0.0);

        Ok(DeltaPatch {
            algorithm: DiffAlgorithm::Xdelta3,
            patch_data,
            original_hash,
            target_hash,
            compression_ratio,
            metadata: PatchMetadata {
                original_size: original.len(),
                target_size: target.len(),
                diff_count,
                performance: PerformanceMetrics {
                    generation_time_ms: generation_time,
                    peak_memory_bytes: original.len() + target.len(),
                    bandwidth_savings: compression_ratio * 100.0,
                },
            },
        })
    }

    /// Generate diff between two chunks
    fn diff_chunks(&self, original: &[u8], target: &[u8]) -> Result<Vec<u8>, DeltaError> {
        let mut patch = Vec::new();
        
        // Simple byte-level diff for kernel implementation
        let min_len = min(original.len(), target.len());
        let mut i = 0;
        
        while i < min_len {
            if original[i] == target[i] {
                // Same byte, skip
                i += 1;
            } else {
                // Different byte, record the change
                // Format: [command][position][old_byte][new_byte]
                patch.push(0x01); // Replace command
                patch.extend_from_slice(&i.to_le_bytes()[..4]);
                patch.push(original[i]);
                patch.push(target[i]);
                i += 1;
            }
        }
        
        // Handle remaining bytes in target (insertions)
        if target.len() > original.len() {
            for &byte in &target[original.len()..] {
                patch.push(0x02); // Insert command
                patch.extend_from_slice(&i.to_le_bytes()[..4]);
                patch.push(byte);
                i += 1;
            }
        }
        
        Ok(patch)
    }

    /// Generate simple diff for basic algorithms
    fn generate_simple_diff(&self, original: &[u8], target: &[u8], patch: &mut Vec<u8>) -> Result<usize, DeltaError> {
        let mut diff_count = 0;
        patch.extend_from_slice(b"XD3"); // Signature
        
        // Write sizes
        patch.extend_from_slice(&original.len().to_le_bytes()[..8]);
        patch.extend_from_slice(&target.len().to_le_bytes()[..8]);
        
        // Simple diff approach
        for i in 0..min(original.len(), target.len()) {
            if original[i] != target[i] {
                patch.extend_from_slice(&i.to_le_bytes()[..4]);
                patch.push(original[i]);
                patch.push(target[i]);
                diff_count += 1;
            }
        }
        
        // Handle insertions
        if target.len() > original.len() {
            for i in original.len()..target.len() {
                patch.extend_from_slice(&i.to_le_bytes()[..4]);
                patch.push(0x00);
                patch.push(target[i]);
                diff_count += 1;
            }
        }
        
        Ok(diff_count)
    }

    /// Compress patch data for bandwidth optimization
    fn compress_patch_data(&self, data: &[u8]) -> Result<Vec<u8>, DeltaError> {
        // Simple compression - in production, use proper algorithms like zlib or lz4
        let mut compressed = Vec::new();
        compressed.push(0x01); // Compression flag
        
        // Run-length encoding for simple compression
        let mut i = 0;
        while i < data.len() {
            let byte = data[i];
            let mut run_length = 1;
            
            // Count consecutive identical bytes
            while i + run_length < data.len() && data[i + run_length] == byte && run_length < 255 {
                run_length += 1;
            }
            
            if run_length > 3 {
                // Use run-length encoding
                compressed.push(0xFF);
                compressed.push(byte);
                compressed.push(run_length as u8);
                i += run_length;
            } else {
                // Copy bytes as-is
                compressed.extend_from_slice(&data[i..i + run_length]);
                i += run_length;
            }
        }
        
        Ok(compressed)
    }

    /// Decompress patch data
    fn decompress_patch_data(&self, data: &[u8]) -> Result<Vec<u8>, DeltaError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        
        if data[0] == 0x01 {
            // Compressed data
            let mut decompressed = Vec::new();
            let mut i = 1;
            
            while i < data.len() {
                if i + 1 < data.len() && data[i] == 0xFF {
                    // Run-length encoded
                    let byte = data[i + 1];
                    let length = data[i + 2] as usize;
                    decompressed.extend_from_slice(&vec![byte; length]);
                    i += 3;
                } else {
                    // Regular byte
                    decompressed.push(data[i]);
                    i += 1;
                }
            }
            
            Ok(decompressed)
        } else {
            // Uncompressed data
            Ok(data.to_vec())
        }
    }

    /// Apply kernel-optimized delta
    fn apply_kernel_optimized_delta(&self, original: &[u8], patch: &DeltaPatch) -> Result<Vec<u8>, DeltaError> {
        let patch_data = self.decompress_patch_data(&patch.patch_data)?;
        
        let mut result = original.to_vec();
        let mut pos = 0;
        
        while pos < patch_data.len() {
            if pos + 5 >= patch_data.len() {
                break;
            }
            
            let command = patch_data[pos];
            match command {
                0x01 => { // Replace
                    if pos + 6 >= patch_data.len() {
                        return Err(DeltaError::InvalidPatchData);
                    }
                    let offset = u32::from_le_bytes([
                        patch_data[pos + 1],
                        patch_data[pos + 2],
                        patch_data[pos + 3],
                        patch_data[pos + 4],
                    ]) as usize;
                    
                    if offset < result.len() {
                        result[offset] = patch_data[pos + 5];
                    }
                    pos += 6;
                }
                0x02 => { // Insert
                    if pos + 6 >= patch_data.len() {
                        return Err(DeltaError::InvalidPatchData);
                    }
                    let offset = u32::from_le_bytes([
                        patch_data[pos + 1],
                        patch_data[pos + 2],
                        patch_data[pos + 3],
                        patch_data[pos + 4],
                    ]) as usize;
                    
                    if offset >= result.len() {
                        result.push(patch_data[pos + 5]);
                    }
                    pos += 6;
                }
                _ => {
                    return Err(DeltaError::InvalidPatchData);
                }
            }
        }
        
        Ok(result)
    }

    /// Apply BSDiff delta
    fn apply_bsdiff_delta(&self, original: &[u8], patch: &DeltaPatch) -> Result<Vec<u8>, DeltaError> {
        // Simplified implementation - in production, use proper BSDiff algorithm
        let patch_data = self.decompress_patch_data(&patch.patch_data)?;
        
        if patch_data.len() < 8 {
            return Err(DeltaError::InvalidPatchData);
        }
        
        // Check signature
        if &patch_data[..3] != b"XD3" {
            return Err(DeltaError::InvalidPatchData);
        }
        
        let orig_len = usize::from_le_bytes([
            patch_data[3], patch_data[4], patch_data[5], patch_data[6],
            patch_data[7], patch_data[8], patch_data[9], patch_data[10],
        ]);
        
        if orig_len != original.len() {
            return Err(DeltaError::SizeMismatch);
        }
        
        let mut result = original.to_vec();
        let mut pos = 11;
        
        while pos + 4 < patch_data.len() {
            let offset = u32::from_le_bytes([
                patch_data[pos],
                patch_data[pos + 1],
                patch_data[pos + 2],
                patch_data[pos + 3],
            ]) as usize;
            
            if pos + 6 >= patch_data.len() {
                break;
            }
            
            let old_byte = patch_data[pos + 4];
            let new_byte = patch_data[pos + 5];
            
            if offset < result.len() && result[offset] == old_byte {
                result[offset] = new_byte;
            }
            
            pos += 6;
        }
        
        Ok(result)
    }

    /// Apply xdelta3 delta
    fn apply_xdelta_delta(&self, original: &[u8], patch: &DeltaPatch) -> Result<Vec<u8>, DeltaError> {
        // Simplified xdelta3 implementation
        self.apply_bsdiff_delta(original, patch) // Reuse BSDiff logic for now
    }

    /// Hash data for verification
    fn hash_data(&self, data: &[u8]) -> Result<CryptographicHash, DeltaError> {
        // Use built-in hashing for now
        use core::hash::{Hash, Hasher};
        use core::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        
        let hash_value = hasher.finish();
        Ok(CryptographicHash::from_raw_bytes(&hash_value.to_ne_bytes()))
    }
}

/// Errors that can occur during delta operations
#[derive(Debug)]
pub enum DeltaError {
    /// Invalid patch data format
    InvalidPatchData,
    /// Hash mismatch during verification
    HashMismatch {
        expected: CryptographicHash,
        actual: CryptographicHash,
    },
    /// Size mismatch
    SizeMismatch,
    /// Compression/decompression error
    CompressionError(String),
    /// Memory allocation error
    MemoryAllocationError,
    /// Algorithm not supported
    AlgorithmNotSupported,
}

impl fmt::Display for DeltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeltaError::InvalidPatchData => write!(f, "Invalid patch data format"),
            DeltaError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {:?}, got {:?}", expected, actual)
            }
            DeltaError::SizeMismatch => write!(f, "Size mismatch in delta operation"),
            DeltaError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            DeltaError::MemoryAllocationError => write!(f, "Memory allocation error"),
            DeltaError::AlgorithmNotSupported => write!(f, "Algorithm not supported"),
        }
    }
}

/// Configuration for delta update operations
#[derive(Debug, Clone)]
pub struct DeltaConfig {
    /// Preferred diff algorithm
    pub algorithm: DiffAlgorithm,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (0-9)
    pub compression_level: u8,
    /// Chunk size for processing
    pub chunk_size: usize,
    /// Bandwidth optimization level
    pub bandwidth_optimization: BandwidthOptimization,
}

/// Bandwidth optimization levels
#[derive(Debug, Clone, Copy)]
pub enum BandwidthOptimization {
    /// Minimal optimization
    Minimal,
    /// Moderate optimization
    Moderate,
    /// Maximum optimization
    Maximum,
}

impl Default for DeltaConfig {
    fn default() -> Self {
        Self {
            algorithm: DiffAlgorithm::KernelOptimized,
            max_memory_bytes: 64 * 1024 * 1024,
            enable_compression: true,
            compression_level: 6,
            chunk_size: 1024 * 1024,
            bandwidth_optimization: BandwidthOptimization::Maximum,
        }
    }
}

/// Bandwidth monitor for tracking optimization
pub struct BandwidthMonitor {
    /// Total bytes transferred
    total_bytes: usize,
    /// Delta bytes transferred
    delta_bytes: usize,
    /// Original bytes that would have been transferred
    original_bytes: usize,
    /// Transfer history
    transfer_history: Vec<TransferRecord>,
}

/// Record of a transfer operation
#[derive(Debug, Clone)]
struct TransferRecord {
    timestamp_ms: u64,
    bytes_transferred: usize,
    compression_ratio: f64,
    algorithm: DiffAlgorithm,
}

impl BandwidthMonitor {
    /// Create a new bandwidth monitor
    pub fn new() -> Self {
        Self {
            total_bytes: 0,
            delta_bytes: 0,
            original_bytes: 0,
            transfer_history: Vec::new(),
        }
    }

    /// Record a transfer operation
    pub fn record_transfer(&mut self, delta_bytes: usize, original_bytes: usize, algorithm: DiffAlgorithm) {
        let timestamp = crate::hal::timers::get_system_time_ms();
        let compression_ratio = if original_bytes > 0 {
            1.0 - (delta_bytes as f64) / (original_bytes as f64)
        } else {
            0.0
        };

        self.total_bytes += delta_bytes;
        self.delta_bytes += delta_bytes;
        self.original_bytes += original_bytes;

        self.transfer_history.push(TransferRecord {
            timestamp_ms: timestamp,
            bytes_transferred: delta_bytes,
            compression_ratio,
            algorithm,
        });

        // Keep only recent history (last 100 records)
        if self.transfer_history.len() > 100 {
            self.transfer_history.remove(0);
        }
    }

    /// Get bandwidth savings statistics
    pub fn get_statistics(&self) -> BandwidthStatistics {
        let total_savings = if self.original_bytes > 0 {
            ((self.original_bytes - self.total_bytes) as f64 / self.original_bytes as f64) * 100.0
        } else {
            0.0
        };

        BandwidthStatistics {
            total_bytes_transferred: self.total_bytes,
            original_bytes_would_transfer: self.original_bytes,
            bandwidth_savings_percentage: total_savings,
            average_compression_ratio: self.transfer_history
                .iter()
                .map(|r| r.compression_ratio)
                .sum::<f64>() / self.transfer_history.len().max(1) as f64,
            total_transfers: self.transfer_history.len(),
        }
    }

    /// Get transfer history
    pub fn get_history(&self) -> &[TransferRecord] {
        &self.transfer_history
    }
}

/// Bandwidth statistics
#[derive(Debug, Clone)]
pub struct BandwidthStatistics {
    pub total_bytes_transferred: usize,
    pub original_bytes_would_transfer: usize,
    pub bandwidth_savings_percentage: f64,
    pub average_compression_ratio: f64,
    pub total_transfers: usize,
}

/// Delta update manager for coordinating operations
pub struct DeltaUpdateManager {
    engine: BinaryDiffEngine,
    config: DeltaConfig,
    bandwidth_monitor: BandwidthMonitor,
    active_transfers: Vec<ActiveTransfer>,
}

/// Information about an active transfer
#[derive(Debug)]
struct ActiveTransfer {
    id: String,
    source_path: String,
    target_path: String,
    bytes_transferred: usize,
    start_time_ms: u64,
    estimated_total: usize,
}

impl DeltaUpdateManager {
    /// Create a new delta update manager
    pub fn new(config: DeltaConfig) -> Self {
        let engine = BinaryDiffEngine::new(config.algorithm);
        
        Self {
            engine,
            config,
            bandwidth_monitor: BandwidthMonitor::new(),
            active_transfers: Vec::new(),
        }
    }

    /// Set encryption manager
    pub fn set_encryption_manager(&mut self, encryption: EncryptionManager) {
        self.engine.set_encryption_manager(encryption);
    }

    /// Update file using delta patches
    pub fn update_with_delta(
        &mut self,
        source_path: &str,
        target_path: &str,
    ) -> Result<DeltaPatch, DeltaError> {
        crate::log::log(LogLevel::Info, &format!(
            "Starting delta update: {} -> {}",
            source_path, target_path
        ));

        // Read source and target files (simplified - would need actual file I/O)
        let source_data = vec![0u8; 1024]; // Placeholder
        let target_data = vec![0u8; 2048]; // Placeholder
        
        let target_hash = self.engine.hash_data(&target_data)?;
        
        let patch = self.engine.generate_delta(&source_data, &target_data, target_hash)?;
        
        // Record bandwidth usage
        self.bandwidth_monitor.record_transfer(
            patch.patch_data.len(),
            target_data.len(),
            patch.algorithm,
        );

        Ok(patch)
    }

    /// Apply delta patch to file
    pub fn apply_delta_patch(
        &mut self,
        file_path: &str,
        patch: &DeltaPatch,
    ) -> Result<Vec<u8>, DeltaError> {
        crate::log::log(LogLevel::Info, &format!(
            "Applying delta patch to: {}",
            file_path
        ));

        // Read original file (simplified - would need actual file I/O)
        let original_data = vec![0u8; 1024]; // Placeholder
        
        self.engine.apply_delta(&original_data, patch)
    }

    /// Get bandwidth statistics
    pub fn get_bandwidth_statistics(&self) -> BandwidthStatistics {
        self.bandwidth_monitor.get_statistics()
    }

    /// Get active transfers
    pub fn get_active_transfers(&self) -> &[ActiveTransfer] {
        &self.active_transfers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<u8>, Vec<u8>) {
        let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let target = vec![1, 2, 3, 4, 5, 11, 12, 13, 14, 15]; // Changed last 5 bytes
        (original, target)
    }

    #[test]
    fn test_kernel_optimized_delta_generation() {
        let config = DeltaConfig::default();
        let mut manager = DeltaUpdateManager::new(config);
        let engine = BinaryDiffEngine::new(DiffAlgorithm::KernelOptimized);
        
        let (original, target) = create_test_data();
        let target_hash = engine.hash_data(&target).unwrap();
        
        let patch = engine.generate_delta(&original, &target, target_hash).unwrap();
        
        assert_eq!(patch.algorithm, DiffAlgorithm::KernelOptimized);
        assert!(!patch.patch_data.is_empty());
        assert!(patch.compression_ratio > 0.0);
    }

    #[test]
    fn test_delta_application() {
        let engine = BinaryDiffEngine::new(DiffAlgorithm::KernelOptimized);
        let (original, target) = create_test_data();
        let target_hash = engine.hash_data(&target).unwrap();
        
        let patch = engine.generate_delta(&original, &target, target_hash).unwrap();
        let result = engine.apply_delta(&original, &patch).unwrap();
        
        assert_eq!(result, target);
    }

    #[test]
    fn test_bandwidth_monitoring() {
        let mut monitor = BandwidthMonitor::new();
        
        // Record some transfers
        monitor.record_transfer(1000, 10000, DiffAlgorithm::KernelOptimized);
        monitor.record_transfer(500, 5000, DiffAlgorithm::Bsdiff);
        
        let stats = monitor.get_statistics();
        
        assert_eq!(stats.total_bytes_transferred, 1500);
        assert_eq!(stats.original_bytes_would_transfer, 15000);
        assert!(stats.bandwidth_savings_percentage > 80.0);
        assert_eq!(stats.total_transfers, 2);
    }
}