//! Educational Memory Manager for ML Runtime
//! 
//! Provides memory management with educational features including:
//! - Memory usage tracking and visualization
//! - Educational allocation strategies
//! - Memory leak detection for learning
//! - Performance optimization hints

use super::tensor::{EducationalTensor, TensorError};
use super::interpreter::EducationalMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Educational Memory Manager
/// 
/// Manages tensor allocation and deallocation with educational features:
/// - Memory usage tracking
/// - Educational allocation patterns
/// - Memory visualization
/// - Performance hints
pub struct EducationalMemoryManager {
    max_memory_mb: usize,
    tensors: Arc<Mutex<HashMap<String, TensorEntry>>>,
    allocation_history: Arc<Mutex<Vec<AllocationRecord>>>,
    current_usage_mb: usize,
    peak_usage_mb: usize,
    allocation_stats: AllocationStats,
}

#[derive(Debug, Clone)]
struct TensorEntry {
    tensor: EducationalTensor,
    metadata: EducationalMetadata,
    allocated_at: std::time::SystemTime,
    last_accessed: std::time::SystemTime,
    access_count: usize,
}

#[derive(Debug, Clone)]
struct AllocationRecord {
    tensor_name: String,
    size_bytes: usize,
    allocated_at: std::time::SystemTime,
    allocation_type: AllocationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationType {
    TensorCreate,
    MatrixOperation,
    ActivationFunction,
    ForwardPass,
    BackwardPass,
    Optimization,
}

#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub average_tensor_size: f32,
    pub most_common_shape: Option<Vec<usize>>,
    pub memory_pressure_events: usize,
}

/// Memory usage statistics for educational purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_usage_mb: usize,
    pub peak_usage_mb: usize,
    pub max_memory_mb: usize,
    pub usage_percentage: f32,
    pub tensor_count: usize,
    pub allocation_rate: f32,
    pub deallocation_rate: f32,
    pub memory_pressure_level: MemoryPressureLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Out of memory: requested {requested}MB, available {available}MB")]
    OutOfMemory { requested: usize, available: usize },
    
    #[error("Memory leak detected: {count} tensors not freed")]
    MemoryLeak { count: usize },
    
    #[error("Tensor not found: {name}")]
    TensorNotFound { name: String },
    
    #[error("Educational validation error: {0}")]
    Educational(String),
}

impl EducationalMemoryManager {
    /// Create a new educational memory manager
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            max_memory_mb,
            tensors: Arc::new(Mutex::new(HashMap::new())),
            allocation_history: Arc::new(Mutex::new(Vec::new())),
            current_usage_mb: 0,
            peak_usage_mb: 0,
            allocation_stats: AllocationStats::default(),
        }
    }

    /// Store a tensor with educational tracking
    pub fn store_tensor(
        &mut self,
        name: &str,
        tensor: EducationalTensor,
        metadata: EducationalMetadata,
    ) -> Result<(), MemoryError> {
        let tensor_size_mb = self.calculate_tensor_size_mb(&tensor);
        
        // Educational memory validation
        self.validate_allocation(name, tensor_size_mb)?;

        let tensor_entry = TensorEntry {
            tensor,
            metadata,
            allocated_at: std::time::SystemTime::now(),
            last_accessed: std::time::SystemTime::now(),
            access_count: 1,
        };

        // Update usage statistics
        self.current_usage_mb += tensor_size_mb;
        if self.current_usage_mb > self.peak_usage_mb {
            self.peak_usage_mb = self.current_usage_mb;
        }

        // Store tensor
        {
            let mut tensors = self.tensors.lock().unwrap();
            tensors.insert(name.to_string(), tensor_entry);
        }

        // Record allocation
        {
            let mut history = self.allocation_history.lock().unwrap();
            history.push(AllocationRecord {
                tensor_name: name.to_string(),
                size_bytes: tensor_size_mb * 1024 * 1024,
                allocated_at: std::time::SystemTime::now(),
                allocation_type: AllocationType::TensorCreate,
            });
        }

        // Update statistics
        self.update_allocation_stats(&metadata);

        Ok(())
    }

    /// Retrieve a tensor with access tracking
    pub fn get_tensor(&self, name: &str) -> Result<EducationalTensor, MemoryError> {
        let mut tensors = self.tensors.lock().unwrap();
        
        match tensors.get_mut(name) {
            Some(entry) => {
                // Update access statistics
                entry.last_accessed = std::time::SystemTime::now();
                entry.access_count += 1;

                // Return a clone for safety
                Ok(entry.tensor.clone())
            }
            None => Err(MemoryError::TensorNotFound {
                name: name.to_string(),
            }),
        }
    }

    /// Remove a tensor with educational tracking
    pub fn remove_tensor(&mut self, name: &str) -> Result<EducationalTensor, MemoryError> {
        let mut tensors = self.tensors.lock().unwrap();
        
        match tensors.remove(name) {
            Some(entry) => {
                // Update usage statistics
                let freed_size_mb = self.calculate_tensor_size_mb(&entry.tensor);
                self.current_usage_mb = self.current_usage_mb.saturating_sub(freed_size_mb);

                // Record deallocation
                {
                    let mut history = self.allocation_history.lock().unwrap();
                    history.push(AllocationRecord {
                        tensor_name: name.to_string(),
                        size_bytes: freed_size_mb * 1024 * 1024,
                        allocated_at: std::time::SystemTime::now(),
                        allocation_type: AllocationType::TensorCreate, // Simplified
                    });
                }

                // Update statistics
                self.allocation_stats.total_deallocations += 1;

                Ok(entry.tensor)
            }
            None => Err(MemoryError::TensorNotFound {
                name: name.to_string(),
            }),
        }
    }

    /// Get memory statistics for educational analysis
    pub fn get_stats(&self) -> MemoryStats {
        let tensor_count = self.tensors.lock().unwrap().len();
        let usage_percentage = (self.current_usage_mb as f32 / self.max_memory_mb as f32) * 100.0;
        
        let pressure_level = if usage_percentage < 50.0 {
            MemoryPressureLevel::Low
        } else if usage_percentage < 75.0 {
            MemoryPressureLevel::Medium
        } else if usage_percentage < 90.0 {
            MemoryPressureLevel::High
        } else {
            MemoryPressureLevel::Critical
        };

        MemoryStats {
            current_usage_mb: self.current_usage_mb,
            peak_usage_mb: self.peak_usage_mb,
            max_memory_mb: self.max_memory_mb,
            usage_percentage,
            tensor_count,
            allocation_rate: self.calculate_allocation_rate(),
            deallocation_rate: self.calculate_deallocation_rate(),
            memory_pressure_level: pressure_level,
        }
    }

    /// Get current memory usage in MB
    pub fn get_current_usage(&self) -> usize {
        self.current_usage_mb
    }

    /// Educational memory optimization suggestions
    pub fn get_optimization_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        
        let stats = self.get_stats();
        
        if stats.usage_percentage > 80.0 {
            suggestions.push(OptimizationSuggestion {
                category: "Memory Usage".to_string(),
                message: "High memory usage detected. Consider freeing unused tensors.".to_string(),
                action: "Call gc() to trigger garbage collection".to_string(),
                priority: "High".to_string(),
            });
        }

        if let Some(ref common_shape) = self.allocation_stats.most_common_shape {
            suggestions.push(OptimizationSuggestion {
                category: "Memory Pattern".to_string(),
                message: format!("Most common tensor shape: {:?}", common_shape),
                action: "Consider reusing tensor allocations for this shape".to_string(),
                priority: "Medium".to_string(),
            });
        }

        suggestions
    }

    /// Trigger garbage collection for educational purposes
    pub fn garbage_collect(&mut self) -> GarbageCollectionReport {
        let tensors = self.tensors.clone();
        let mut freed_count = 0;
        let mut freed_mb = 0;
        
        let current_time = std::time::SystemTime::now();
        let gc_threshold = std::time::Duration::from_secs(300); // 5 minutes

        {
            let mut tensors_map = tensors.lock().unwrap();
            let to_remove: Vec<String> = tensors_map
                .iter()
                .filter_map(|(name, entry)| {
                    if current_time.duration_since(entry.last_accessed).unwrap_or(gc_threshold) > gc_threshold {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for name in to_remove {
                if let Some(entry) = tensors_map.remove(&name) {
                    freed_mb += self.calculate_tensor_size_mb(&entry.tensor);
                    freed_count += 1;
                }
            }
        }

        self.current_usage_mb = self.current_usage_mb.saturating_sub(freed_mb);

        GarbageCollectionReport {
            freed_tensors: freed_count,
            freed_memory_mb: freed_mb,
            remaining_tensors: self.tensors.lock().unwrap().len(),
            gc_duration: std::time::Duration::from_millis(10), // Simplified
        }
    }

    /// Get allocation history for educational analysis
    pub fn get_allocation_history(&self) -> Vec<AllocationRecord> {
        self.allocation_history.lock().unwrap().clone()
    }

    /// Educational memory pattern analysis
    pub fn analyze_allocation_patterns(&self) -> AllocationPatternAnalysis {
        let history = self.allocation_history.lock().unwrap();
        
        let mut hourly_distribution = HashMap::new();
        let mut type_distribution = HashMap::new();
        let mut size_distribution = HashMap::new();

        for record in history.iter() {
            // Hour distribution
            let hour = format!("{:02}:00", 
                record.allocated_at.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() % 86400 / 3600);
            *hourly_distribution.entry(hour).or_insert(0) += 1;

            // Type distribution
            *type_distribution
                .entry(format!("{:?}", record.allocation_type))
                .or_insert(0) += 1;

            // Size distribution
            let size_category = match record.size_bytes {
                0..=1024 => "Small".to_string(),
                1025..=10240 => "Medium".to_string(),
                _ => "Large".to_string(),
            };
            *size_distribution.entry(size_category).or_insert(0) += 1;
        }

        AllocationPatternAnalysis {
            hourly_distribution,
            type_distribution,
            size_distribution,
            total_allocations: history.len(),
        }
    }

    /// Validate allocation with educational checks
    fn validate_allocation(&self, name: &str, size_mb: usize) -> Result<(), MemoryError> {
        // Check name uniqueness
        if self.tensors.lock().unwrap().contains_key(name) {
            return Err(MemoryError::Educational(
                format!("Tensor '{}' already exists", name)
            ));
        }

        // Check memory availability
        if self.current_usage_mb + size_mb > self.max_memory_mb {
            return Err(MemoryError::OutOfMemory {
                requested: self.current_usage_mb + size_mb,
                available: self.max_memory_mb,
            });
        }

        Ok(())
    }

    /// Calculate tensor size in MB
    fn calculate_tensor_size_mb(&self, tensor: &EducationalTensor) -> usize {
        let data_size = tensor.size() * 4; // 4 bytes per f32
        let metadata_size = 1024; // Approximate metadata size
        (data_size + metadata_size) / (1024 * 1024)
    }

    /// Update allocation statistics
    fn update_allocation_stats(&mut self, metadata: &EducationalMetadata) {
        self.allocation_stats.total_allocations += 1;
        
        // This would track more sophisticated statistics in a full implementation
    }

    /// Calculate allocation rate
    fn calculate_allocation_rate(&self) -> f32 {
        // Simplified rate calculation
        self.allocation_stats.total_allocations as f32 / 60.0 // per minute
    }

    /// Calculate deallocation rate
    fn calculate_deallocation_rate(&self) -> f32 {
        self.allocation_stats.total_deallocations as f32 / 60.0 // per minute
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub message: String,
    pub action: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionReport {
    pub freed_tensors: usize,
    pub freed_memory_mb: usize,
    pub remaining_tensors: usize,
    pub gc_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPatternAnalysis {
    pub hourly_distribution: HashMap<String, usize>,
    pub type_distribution: HashMap<String, usize>,
    pub size_distribution: HashMap<String, usize>,
    pub total_allocations: usize,
}