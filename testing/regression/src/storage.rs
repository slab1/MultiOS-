//! Storage Module
//!
//! Provides efficient storage and retrieval capabilities for performance baselines,
//! measurement data, and test results with caching, compression, and optimized
//! querying for regression testing workflows.

use anyhow::{Result};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    PerformanceBaseline, PerformanceMeasurement, TestEnvironment, Uuid,
    database::DatabaseManager,
};

/// Performance baseline storage manager
#[derive(Debug, Clone)]
pub struct BaselineStore {
    /// In-memory cache of baselines
    cache: BaselineCache,
    /// Storage configuration
    config: BaselineStorageConfig,
    /// Compression settings
    compression: CompressionConfig,
}

/// Configuration for baseline storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStorageConfig {
    pub enable_caching: bool,
    pub cache_size_limit: usize,
    pub compression_enabled: bool,
    pub validation_required: bool,
    pub auto_cleanup_days: u32,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: CompressionLevel,
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompressionAlgorithm {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Compression levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompressionLevel {
    Fast,
    Balanced,
    Maximum,
}

/// In-memory cache for baselines
#[derive(Debug)]
struct BaselineCache {
    /// Cache entries with metadata
    entries: BTreeMap<String, CacheEntry>,
    /// LRU tracking for eviction
    lru_order: Vec<String>,
    /// Current cache size
    current_size: usize,
    /// Maximum cache size
    max_size: usize,
}

/// Cache entry for baseline
#[derive(Debug, Clone)]
struct CacheEntry {
    pub baseline: PerformanceBaseline,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
    pub compressed: bool,
}

/// Performance measurement storage manager
#[derive(Debug, Clone)]
pub struct MeasurementStore {
    /// In-memory cache of recent measurements
    cache: MeasurementCache,
    /// Storage configuration
    config: MeasurementStorageConfig,
    /// Retention policy
    retention_policy: RetentionPolicy,
}

/// Configuration for measurement storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementStorageConfig {
    pub enable_caching: bool,
    pub cache_size_limit: usize,
    pub batch_insert_enabled: bool,
    pub batch_size: usize,
    pub auto_aggregation: bool,
}

/// Retention policy for measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetentionPolicy {
    pub raw_data_retention_days: u32,
    pub aggregated_data_retention_days: u32,
    pub cleanup_interval_hours: u32,
}

/// In-memory cache for measurements
#[derive(Debug)]
struct MeasurementCache {
    /// Recent measurements cache
    recent_measurements: BTreeMap<String, Vec<CachedMeasurement>>,
    /// Aggregation cache
    aggregated_data: HashMap<String, AggregatedData>,
    /// Cache statistics
    stats: CacheStats,
}

/// Individual cached measurement
#[derive(Debug, Clone)]
struct CachedMeasurement {
    pub measurement: PerformanceMeasurement,
    pub cache_timestamp: DateTime<Utc>,
}

/// Aggregated measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedData {
    pub metric_name: String,
    pub component: String,
    pub time_window: TimeWindow,
    pub statistics: AggregatedStatistics,
    pub raw_count: u32,
    pub aggregation_timestamp: DateTime<Utc>,
}

/// Time window for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeWindow {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub resolution: WindowResolution,
}

/// Aggregation resolution levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WindowResolution {
    Minute,
    Hour,
    Day,
    Week,
}

/// Aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStatistics {
    pub count: u32,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub standard_deviation: f64,
    pub percentile_95: f64,
    pub percentile_99: f64,
    pub outlier_count: u32,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size_bytes: usize,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub total_baselines: usize,
    pub total_measurements: usize,
    pub cached_baselines: usize,
    pub cached_measurements: usize,
    pub cache_hit_rate: f64,
    pub storage_efficiency: f64,
    pub compression_ratio: f64,
}

impl BaselineStore {
    /// Create new baseline store
    pub fn new() -> Self {
        Self {
            cache: BaselineCache {
                entries: BTreeMap::new(),
                lru_order: Vec::new(),
                current_size: 0,
                max_size: 1000, // Default cache size
            },
            config: BaselineStorageConfig {
                enable_caching: true,
                cache_size_limit: 1000,
                compression_enabled: false,
                validation_required: true,
                auto_cleanup_days: 90,
            },
            compression: CompressionConfig {
                algorithm: CompressionAlgorithm::None,
                level: CompressionLevel::Balanced,
            },
        }
    }

    /// Create baseline store with custom configuration
    pub fn with_config(config: BaselineStorageConfig, compression: CompressionConfig) -> Self {
        Self {
            cache: BaselineCache {
                entries: BTreeMap::new(),
                lru_order: Vec::new(),
                current_size: 0,
                max_size: config.cache_size_limit,
            },
            config,
            compression,
        }
    }

    /// Load baselines from database
    pub async fn load_from_database(&mut self, db: &DatabaseManager) -> Result<()> {
        info!("Loading baselines from database");
        
        let components = vec!["kernel", "filesystem", "network", "memory"];
        
        for component in components {
            for metric_type in &["latency", "throughput", "cpu_usage", "memory_usage"] {
                match db.get_performance_baselines(component, metric_type).await {
                    Ok(baselines) => {
                        for baseline in baselines {
                            self.cache_baseline(baseline)?;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to load baselines for {}/{}: {}", component, metric_type, e);
                    }
                }
            }
        }
        
        info!("Loaded {} baselines into cache", self.cache.entries.len());
        Ok(())
    }

    /// Store baseline in cache and database
    pub async fn store_baseline(&mut self, db: &DatabaseManager, baseline: PerformanceBaseline) -> Result<()> {
        debug!("Storing baseline for {}/{}", baseline.component, baseline.metric_type);
        
        // Store in database
        db.store_performance_baseline(&baseline).await
            .context("Failed to store baseline in database")?;
        
        // Cache the baseline
        self.cache_baseline(baseline)?;
        
        Ok(())
    }

    /// Get baseline from cache or database
    pub async fn get_baseline(
        &mut self,
        db: &DatabaseManager,
        component: &str,
        metric_type: &str,
    ) -> Result<Option<PerformanceBaseline>> {
        let cache_key = self.generate_cache_key(component, metric_type);
        
        // Check cache first
        if let Some(entry) = self.cache.entries.get(&cache_key) {
            self.update_access_stats(&cache_key);
            debug!("Baseline cache hit for {}/{}", component, metric_type);
            return Ok(Some(entry.baseline.clone()));
        }
        
        debug!("Baseline cache miss for {}/{}, loading from database", component, metric_type);
        
        // Load from database
        let baselines = db.get_performance_baselines(component, metric_type).await?;
        
        if let Some(baseline) = baselines.first() {
            // Cache the loaded baseline
            self.cache_baseline(baseline.clone())?;
            Ok(Some(baseline.clone()))
        } else {
            // Record cache miss
            self.cache.stats.misses += 1;
            Ok(None)
        }
    }

    /// Get all cached baselines
    pub fn get_baselines(&self) -> Vec<PerformanceBaseline> {
        self.cache.entries.values().map(|entry| entry.baseline.clone()).collect()
    }

    /// Cache baseline entry
    fn cache_baseline(&mut self, baseline: PerformanceBaseline) -> Result<()> {
        if !self.config.enable_caching {
            return Ok(());
        }
        
        let cache_key = self.generate_cache_key(&baseline.component, &baseline.metric_type);
        
        // Check cache size limit
        if self.cache.entries.len() >= self.config.cache_size_limit {
            self.evict_lru_entry();
        }
        
        // Create cache entry
        let entry = CacheEntry {
            baseline,
            access_count: 1,
            last_accessed: Utc::now(),
            compressed: false, // Compression not implemented in this version
        };
        
        // Update cache
        self.cache.entries.insert(cache_key.clone(), entry);
        self.cache.lru_order.push(cache_key);
        self.cache.current_size += 1;
        
        Ok(())
    }

    /// Generate cache key for baseline
    fn generate_cache_key(&self, component: &str, metric_type: &str) -> String {
        format!("{}:{}", component, metric_type)
    }

    /// Update access statistics for cache entry
    fn update_access_stats(&mut self, cache_key: &str) {
        if let Some(entry) = self.cache.entries.get_mut(cache_key) {
            entry.access_count += 1;
            entry.last_accessed = Utc::now();
            self.cache.stats.hits += 1;
            
            // Update LRU order
            if let Some(pos) = self.cache.lru_order.iter().position(|key| key == cache_key) {
                let key = self.cache.lru_order.remove(pos);
                self.cache.lru_order.push(key);
            }
        }
    }

    /// Evict least recently used cache entry
    fn evict_lru_entry(&mut self) {
        if let Some(lru_key) = self.cache.lru_order.first().cloned() {
            if let Some(entry) = self.cache.entries.remove(&lru_key) {
                // Remove from LRU order
                self.cache.lru_order.retain(|key| key != &lru_key);
                self.cache.current_size -= 1;
                self.cache.stats.evictions += 1;
                
                debug!("Evicted baseline from cache: {}", lru_key);
                
                // In a real implementation, we might persist the evicted baseline
                // to slower storage before removal
            }
        }
    }

    /// Invalidate cache entry
    pub fn invalidate_cache(&mut self, component: &str, metric_type: &str) {
        let cache_key = self.generate_cache_key(component, metric_type);
        self.cache.entries.remove(&cache_key);
        self.cache.lru_order.retain(|key| key != &cache_key);
        debug!("Invalidated baseline cache for {}", cache_key);
    }

    /// Clear all cached baselines
    pub fn clear_cache(&mut self) {
        self.cache.entries.clear();
        self.cache.lru_order.clear();
        self.cache.current_size = 0;
        info!("Baseline cache cleared");
    }

    /// Get storage statistics
    pub fn get_statistics(&self) -> StorageStatistics {
        let total_baselines = self.cache.entries.len();
        let cache_hit_rate = if self.cache.stats.hits + self.cache.stats.misses > 0 {
            self.cache.stats.hits as f64 / (self.cache.stats.hits + self.cache.stats.misses) as f64
        } else {
            0.0
        };
        
        StorageStatistics {
            total_baselines,
            total_measurements: 0, // Measurements are handled by MeasurementStore
            cached_baselines: total_baselines,
            cached_measurements: 0,
            cache_hit_rate,
            storage_efficiency: (total_baselines as f64 / self.cache.current_size as f64) * 100.0,
            compression_ratio: 1.0, // Compression not implemented
        }
    }

    /// Clean up old baselines
    pub async fn cleanup_old_baselines(&mut self, db: &DatabaseManager, retention_days: u32) -> Result<u64> {
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut cleaned_count = 0;
        
        // This would query the database for old baselines and delete them
        // For now, just return a mock count
        debug!("Cleaning up baselines older than {} days", retention_days);
        
        cleaned_count = 0; // Mock cleanup
        
        Ok(cleaned_count)
    }
}

impl MeasurementStore {
    /// Create new measurement store
    pub fn new() -> Self {
        Self {
            cache: MeasurementCache {
                recent_measurements: BTreeMap::new(),
                aggregated_data: HashMap::new(),
                stats: CacheStats::default(),
            },
            config: MeasurementStorageConfig {
                enable_caching: true,
                cache_size_limit: 10000,
                batch_insert_enabled: true,
                batch_size: 100,
                auto_aggregation: true,
            },
            retention_policy: RetentionPolicy {
                raw_data_retention_days: 30,
                aggregated_data_retention_days: 365,
                cleanup_interval_hours: 24,
            },
        }
    }

    /// Create measurement store with custom configuration
    pub fn with_config(config: MeasurementStorageConfig, retention_policy: RetentionPolicy) -> Self {
        Self {
            cache: MeasurementCache {
                recent_measurements: BTreeMap::new(),
                aggregated_data: HashMap::new(),
                stats: CacheStats::default(),
            },
            config,
            retention_policy,
        }
    }

    /// Store measurement in cache and database
    pub async fn store_measurement(&mut self, db: &DatabaseManager, measurement: PerformanceMeasurement) -> Result<()> {
        debug!("Storing measurement for {}/{}", measurement.component, measurement.metric_type);
        
        // Store in database
        db.store_performance_measurement(&measurement).await
            .context("Failed to store measurement in database")?;
        
        // Cache the measurement if caching is enabled
        if self.config.enable_caching {
            self.cache_measurement(measurement)?;
        }
        
        // Trigger aggregation if auto-aggregation is enabled
        if self.config.auto_aggregation {
            self.trigger_aggregation(&measurement).await?;
        }
        
        Ok(())
    }

    /// Store multiple measurements in batch
    pub async fn store_measurements_batch(&mut self, db: &DatabaseManager, measurements: Vec<PerformanceMeasurement>) -> Result<()> {
        debug!("Storing {} measurements in batch", measurements.len());
        
        if self.config.batch_insert_enabled {
            // Use batch insert for better performance
            for measurement in &measurements {
                db.store_performance_measurement(measurement).await?;
            }
        } else {
            // Individual inserts
            for measurement in measurements {
                db.store_performance_measurement(&measurement).await?;
            }
        }
        
        // Cache measurements if enabled
        if self.config.enable_caching {
            for measurement in measurements {
                self.cache_measurement(measurement)?;
            }
        }
        
        Ok(())
    }

    /// Get measurements from cache or database
    pub async fn get_measurements(
        &mut self,
        db: &DatabaseManager,
        component: &str,
        metric_type: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<PerformanceMeasurement>> {
        debug!("Getting measurements for {}/{} from {} to {}", component, metric_type, start_time, end_time);
        
        // Check cache for recent measurements
        let cache_key = self.generate_cache_key(component, metric_type);
        
        // For simplicity, we'll load from database directly
        // In a real implementation, we'd check cache first and then database
        
        let measurements = db.get_performance_measurements(component, metric_type, start_time, end_time).await?;
        
        // Cache recent measurements
        for measurement in &measurements {
            if (Utc::now() - measurement.timestamp).num_hours() < 24 {
                self.cache_measurement(measurement.clone())?;
            }
        }
        
        Ok(measurements)
    }

    /// Cache measurement entry
    fn cache_measurement(&mut self, measurement: PerformanceMeasurement) -> Result<()> {
        let cache_key = self.generate_cache_key(&measurement.component, &measurement.metric_type);
        
        // Check cache size limit
        let current_count = self.cache.recent_measurements.get(&cache_key)
            .map(|v| v.len())
            .unwrap_or(0);
            
        if current_count >= 1000 { // Limit per metric/component
            self.evict_old_measurements(&cache_key);
        }
        
        let cached_measurement = CachedMeasurement {
            measurement,
            cache_timestamp: Utc::now(),
        };
        
        self.cache.recent_measurements
            .entry(cache_key)
            .or_insert_with(Vec::new)
            .push(cached_measurement);
            
        Ok(())
    }

    /// Generate cache key for measurements
    fn generate_cache_key(&self, component: &str, metric_type: &str) -> String {
        format!("{}:{}", component, metric_type)
    }

    /// Evict old measurements from cache
    fn evict_old_measurements(&mut self, cache_key: &str) {
        if let Some(measurements) = self.cache.recent_measurements.get_mut(cache_key) {
            // Remove oldest measurements (keep most recent 500)
            if measurements.len() > 500 {
                measurements.drain(0..measurements.len() - 500);
                debug!("Evicted old measurements from cache for {}", cache_key);
            }
        }
    }

    /// Trigger aggregation for measurement
    async fn trigger_aggregation(&mut self, measurement: &PerformanceMeasurement) -> Result<()> {
        // This would trigger aggregation based on measurement
        // For now, just log the intent
        
        debug!("Triggering aggregation for measurement: {}/{}", 
               measurement.component, measurement.metric_type);
        
        Ok(())
    }

    /// Get aggregated data
    pub fn get_aggregated_data(&self, component: &str, metric_type: &str, time_window: TimeWindow) -> Option<&AggregatedData> {
        let key = format!("{}:{}:{:?}", component, metric_type, time_window.resolution);
        self.cache.aggregated_data.get(&key)
    }

    /// Create aggregated data from measurements
    pub fn create_aggregated_data(
        &mut self,
        component: &str,
        metric_type: &str,
        measurements: &[PerformanceMeasurement],
        resolution: WindowResolution,
    ) -> Result<AggregatedData> {
        if measurements.is_empty() {
            return Err(anyhow::anyhow!("No measurements provided for aggregation"));
        }
        
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        
        let statistics = self.calculate_statistics(&values)?;
        
        let aggregated = AggregatedData {
            metric_name: metric_type.to_string(),
            component: component.to_string(),
            time_window: TimeWindow {
                start_time: measurements.first().unwrap().timestamp,
                end_time: measurements.last().unwrap().timestamp,
                resolution,
            },
            statistics,
            raw_count: measurements.len() as u32,
            aggregation_timestamp: Utc::now(),
        };
        
        // Cache the aggregated data
        let cache_key = format!("{}:{}:{:?}", component, metric_type, resolution);
        self.cache.aggregated_data.insert(cache_key, aggregated.clone());
        
        Ok(aggregated)
    }

    /// Calculate statistics from values
    fn calculate_statistics(&self, values: &[f64]) -> Result<AggregatedStatistics> {
        if values.is_empty() {
            return Err(anyhow::anyhow!("No values provided for statistics calculation"));
        }
        
        let count = values.len() as u32;
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median = if sorted_values.len() % 2 == 0 {
            let mid = sorted_values.len() / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };
        
        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];
        
        // Calculate standard deviation
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let standard_deviation = variance.sqrt();
        
        // Calculate percentiles
        let percentile_95 = self.calculate_percentile(&sorted_values, 95.0)?;
        let percentile_99 = self.calculate_percentile(&sorted_values, 99.0)?;
        
        // Count outliers (values beyond 2 standard deviations)
        let outlier_count = values.iter()
            .filter(|&&x| (x - mean).abs() > 2.0 * standard_deviation)
            .count() as u32;
        
        Ok(AggregatedStatistics {
            count,
            mean,
            median,
            min,
            max,
            standard_deviation,
            percentile_95,
            percentile_99,
            outlier_count,
        })
    }

    /// Calculate percentile from sorted values
    fn calculate_percentile(&self, sorted_values: &[f64], percentile: f64) -> Result<f64> {
        if sorted_values.is_empty() {
            return Err(anyhow::anyhow!("No values for percentile calculation"));
        }
        
        let rank = (percentile / 100.0) * (sorted_values.len() - 1) as f64;
        let lower = rank.floor() as usize;
        let upper = rank.ceil() as usize;
        
        if lower == upper {
            Ok(sorted_values[lower])
        } else {
            let weight = rank - lower as f64;
            Ok(sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight)
        }
    }

    /// Clear measurement cache
    pub fn clear_cache(&mut self) {
        self.cache.recent_measurements.clear();
        self.cache.aggregated_data.clear();
        info!("Measurement cache cleared");
    }

    /// Get storage statistics
    pub fn get_statistics(&self) -> StorageStatistics {
        let total_measurements: usize = self.cache.recent_measurements
            .values()
            .map(|v| v.len())
            .sum();
        
        let cache_hit_rate = if self.cache.stats.hits + self.cache.stats.misses > 0 {
            self.cache.stats.hits as f64 / (self.cache.stats.hits + self.cache.stats.misses) as f64
        } else {
            0.0
        };
        
        StorageStatistics {
            total_baselines: 0, // Baselines are handled by BaselineStore
            total_measurements,
            cached_baselines: 0,
            cached_measurements: total_measurements,
            cache_hit_rate,
            storage_efficiency: (total_measurements as f64 / 1000.0).min(100.0),
            compression_ratio: 1.0, // Compression not implemented
        }
    }

    /// Clean up old measurements
    pub async fn cleanup_old_measurements(&mut self, db: &DatabaseManager, retention_days: u32) -> Result<u64> {
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut cleaned_count = 0;
        
        // This would query the database for old measurements and delete them
        debug!("Cleaning up measurements older than {} days", retention_days);
        
        cleaned_count = 0; // Mock cleanup
        
        // Clean up cache as well
        let mut keys_to_remove = Vec::new();
        for (key, measurements) in &self.cache.recent_measurements {
            let recent_count = measurements.iter()
                .filter(|m| m.cache_timestamp >= cutoff_time)
                .count();
            
            if recent_count < measurements.len() {
                let recent_measurements: Vec<CachedMeasurement> = measurements.iter()
                    .filter(|m| m.cache_timestamp >= cutoff_time)
                    .cloned()
                    .collect();
                
                if recent_measurements.is_empty() {
                    keys_to_remove.push(key.clone());
                } else {
                    self.cache.recent_measurements.insert(key.clone(), recent_measurements);
                }
            }
        }
        
        for key in keys_to_remove {
            self.cache.recent_measurements.remove(&key);
        }
        
        info!("Cleaned up old measurements");
        Ok(cleaned_count)
    }
}