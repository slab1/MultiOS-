//! Service Discovery and Registry
//! 
//! This module provides service discovery mechanisms including
//! service registration, lookup, and discovery patterns.

use spin::{Mutex, RwLock};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, HashSet};
use core::sync::atomic::{AtomicU64, Ordering};

use super::{ServiceId, ServiceResult, ServiceError, service::{ServiceInstance, ServiceRegistryEntry, ServiceDiscoveryQuery}};
use super::service::ServiceState;

/// Get current system time
fn get_current_time() -> u64 {
    super::get_current_time()
}

/// Service Registry - Central registry for all services
pub struct ServiceRegistry {
    entries: RwLock<BTreeMap<ServiceId, ServiceRegistryEntry>>,
    name_index: RwLock<BTreeMap<String, HashSet<ServiceId>>>,
    tag_index: RwLock<BTreeMap<String, HashSet<ServiceId>>>,
    type_index: RwLock<BTreeMap<super::service::ServiceType, HashSet<ServiceId>>>,
    endpoint_index: RwLock<BTreeMap<String, ServiceId>>,
    next_entry_id: AtomicU64,
}

/// Service Discovery - Query and discover services
pub struct ServiceDiscovery {
    registry: ServiceRegistry,
    discovery_cache: RwLock<BTreeMap<String, DiscoveryCacheEntry>>,
    query_history: RwLock<Vec<DiscoveryQuery>>,
    discovery_stats: DiscoveryStats,
}

/// Discovery Cache Entry
#[derive(Debug, Clone)]
struct DiscoveryCacheEntry {
    service_ids: Vec<ServiceId>,
    timestamp: u64,
    ttl: u64,
}

/// Discovery Query History
#[derive(Debug, Clone)]
struct DiscoveryQuery {
    pattern: String,
    timestamp: u64,
    result_count: usize,
    response_time: u64, // microseconds
}

/// Discovery Statistics
#[derive(Debug, Clone)]
struct DiscoveryStats {
    total_queries: u64,
    cache_hits: u64,
    cache_misses: u64,
    average_response_time: f64,
}

/// Service Discovery Endpoint
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub service_id: ServiceId,
    pub name: String,
    pub address: String,
    pub port: Option<u16>,
    pub protocol: super::config::Protocol,
    pub health_status: super::service::HealthStatus,
    pub metadata: BTreeMap<String, String>,
}

/// Service Query Filter
#[derive(Debug, Clone)]
pub struct ServiceFilter {
    pub name_pattern: Option<String>,
    pub tags: Vec<String>,
    pub service_types: Vec<super::service::ServiceType>,
    pub healthy_only: bool,
    pub available_only: bool,
    pub max_results: Option<usize>,
}

/// Service Subscription
#[derive(Debug, Clone)]
pub struct ServiceSubscription {
    pub id: String,
    pub filter: ServiceFilter,
    pub callback: ServiceDiscoveryCallback,
}

/// Service Discovery Callback
pub trait ServiceDiscoveryCallback: Send + Sync {
    fn on_service_discovered(&self, service: &ServiceEndpoint);
    fn on_service_removed(&self, service_id: ServiceId);
    fn on_service_health_changed(&self, service_id: ServiceId, health: super::service::HealthStatus);
}

/// Default Service Discovery Callback
#[derive(Debug, Clone)]
pub struct DefaultDiscoveryCallback;

impl ServiceDiscoveryCallback for DefaultDiscoveryCallback {
    fn on_service_discovered(&self, service: &ServiceEndpoint) {
        info!("Service discovered: {} at {}:{}", service.name, service.address, 
              service.port.unwrap_or(0));
    }

    fn on_service_removed(&self, service_id: ServiceId) {
        info!("Service removed from discovery: {}", service_id.0);
    }

    fn on_service_health_changed(&self, service_id: ServiceId, health: super::service::HealthStatus) {
        info!("Service health changed: {} -> {:?}", service_id.0, health);
    }
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        ServiceRegistry {
            entries: RwLock::new(BTreeMap::new()),
            name_index: RwLock::new(BTreeMap::new()),
            tag_index: RwLock::new(BTreeMap::new()),
            type_index: RwLock::new(BTreeMap::new()),
            endpoint_index: RwLock::new(BTreeMap::new()),
            next_entry_id: AtomicU64::new(1),
        }
    }

    /// Initialize the registry
    pub fn init(&self) -> ServiceResult<()> {
        // Perform any necessary initialization
        info!("Service registry initialized");
        Ok(())
    }

    /// Register a service instance
    pub fn register_service(&self, service_handle: super::service::ServiceHandle) -> ServiceResult<()> {
        let service_id = service_handle.get_service_id();
        let service = service_handle.get_service()
            .ok_or(ServiceError::ServiceNotFound)?;

        let entry = ServiceRegistryEntry {
            service_id,
            name: service.descriptor.name.clone(),
            instance: ServiceInstance {
                service_id,
                instance_id: service.instance_id.clone(),
                host: "localhost".to_string(), // Would be determined dynamically
                port: None,
                endpoint: format!("{}://{}", service.descriptor.name, service_id.0),
                weight: 100,
                status: service.state,
                last_health_check: service.last_health_check.unwrap_or(0),
            },
            registered_at: get_current_time(),
            last_updated: get_current_time(),
        };

        // Add to registry
        let mut entries = self.entries.write();
        entries.insert(service_id, entry);

        // Update indexes
        self.update_indexes(service_id, &service.descriptor.name, &service.descriptor.tags, service.descriptor.service_type)?;

        info!("Service registered in discovery: {} ({})", service.descriptor.name, service_id.0);
        Ok(())
    }

    /// Unregister a service instance
    pub fn unregister_service(&self, service_id: ServiceId) -> ServiceResult<()> {
        let service = {
            let entries = self.entries.read();
            entries.get(&service_id).cloned()
        }.ok_or(ServiceError::ServiceNotFound)?;

        // Remove from registry
        let mut entries = self.entries.write();
        entries.remove(&service_id);

        // Update indexes
        self.remove_from_indexes(service_id, &service.name)?;

        info!("Service unregistered from discovery: {}", service_id.0);
        Ok(())
    }

    /// Update service information
    pub fn update_service(&self, service_id: ServiceId) -> ServiceResult<()> {
        // Implementation would update service information in registry
        info!("Service updated in discovery: {}", service_id.0);
        Ok(())
    }

    /// Lookup service by ID
    pub fn lookup_service(&self, service_id: ServiceId) -> ServiceResult<ServiceEndpoint> {
        let entries = self.entries.read();
        let entry = entries.get(&service_id).ok_or(ServiceError::ServiceNotFound)?;

        let endpoint = ServiceEndpoint {
            service_id,
            name: entry.name.clone(),
            address: entry.instance.host.clone(),
            port: entry.instance.port,
            protocol: super::config::Protocol::Http,
            health_status: super::service::HealthStatus::Unknown, // Would be determined from monitoring
            metadata: BTreeMap::new(),
        };

        Ok(endpoint)
    }

    /// Lookup services by name
    pub fn lookup_by_name(&self, name: &str) -> ServiceResult<Vec<ServiceEndpoint>> {
        let name_index = self.name_index.read();
        let service_ids = name_index.get(name).ok_or(ServiceError::ServiceNotFound)?;

        let mut endpoints = Vec::new();
        let entries = self.entries.read();

        for service_id in service_ids {
            if let Some(entry) = entries.get(service_id) {
                let endpoint = ServiceEndpoint {
                    service_id: *service_id,
                    name: entry.name.clone(),
                    address: entry.instance.host.clone(),
                    port: entry.instance.port,
                    protocol: super::config::Protocol::Http,
                    health_status: super::service::HealthStatus::Unknown,
                    metadata: BTreeMap::new(),
                };
                endpoints.push(endpoint);
            }
        }

        Ok(endpoints)
    }

    /// Get all registered services
    pub fn get_all_services(&self) -> Vec<ServiceEndpoint> {
        let entries = self.entries.read();
        
        entries.values().map(|entry| {
            ServiceEndpoint {
                service_id: entry.service_id,
                name: entry.name.clone(),
                address: entry.instance.host.clone(),
                port: entry.instance.port,
                protocol: super::config::Protocol::Http,
                health_status: super::service::HealthStatus::Unknown,
                metadata: BTreeMap::new(),
            }
        }).collect()
    }

    /// Get services by type
    pub fn get_services_by_type(&self, service_type: super::service::ServiceType) -> Vec<ServiceEndpoint> {
        let type_index = self.type_index.read();
        let service_ids = type_index.get(&service_type).cloned().unwrap_or_default();

        let mut endpoints = Vec::new();
        let entries = self.entries.read();

        for service_id in service_ids {
            if let Some(entry) = entries.get(&service_id) {
                let endpoint = ServiceEndpoint {
                    service_id,
                    name: entry.name.clone(),
                    address: entry.instance.host.clone(),
                    port: entry.instance.port,
                    protocol: super::config::Protocol::Http,
                    health_status: super::service::HealthStatus::Unknown,
                    metadata: BTreeMap::new(),
                };
                endpoints.push(endpoint);
            }
        }

        endpoints
    }

    /// Internal methods
    fn update_indexes(&self, service_id: ServiceId, name: &str, tags: &Vec<String>, service_type: super::service::ServiceType) -> ServiceResult<()> {
        // Update name index
        let mut name_index = self.name_index.write();
        let service_set = name_index.entry(name.to_string()).or_insert_with(HashSet::new);
        service_set.insert(service_id);

        // Update tag index
        let mut tag_index = self.tag_index.write();
        for tag in tags {
            let tag_set = tag_index.entry(tag.clone()).or_insert_with(HashSet::new);
            tag_set.insert(service_id);
        }

        // Update type index
        let mut type_index = self.type_index.write();
        let type_set = type_index.entry(service_type).or_insert_with(HashSet::new);
        type_set.insert(service_id);

        Ok(())
    }

    fn remove_from_indexes(&self, service_id: ServiceId, name: &str) -> ServiceResult<()> {
        // Remove from name index
        let mut name_index = self.name_index.write();
        if let Some(service_set) = name_index.get_mut(name) {
            service_set.remove(&service_id);
            if service_set.is_empty() {
                name_index.remove(name);
            }
        }

        // Would need to implement similar logic for tags and types
        // This is a simplified implementation

        Ok(())
    }
}

impl ServiceDiscovery {
    /// Create a new service discovery instance
    pub fn new() -> Self {
        ServiceDiscovery {
            registry: ServiceRegistry::new(),
            discovery_cache: RwLock::new(BTreeMap::new()),
            query_history: RwLock::new(Vec::new()),
            discovery_stats: DiscoveryStats {
                total_queries: 0,
                cache_hits: 0,
                cache_misses: 0,
                average_response_time: 0.0,
            },
        }
    }

    /// Initialize service discovery
    pub fn init(&self) -> ServiceResult<()> {
        self.registry.init()?;
        
        // Initialize discovery cache with reasonable TTL
        let mut cache = self.discovery_cache.write();
        cache.clear();
        
        info!("Service discovery initialized");
        Ok(())
    }

    /// Discover services by name pattern
    pub fn discover_by_pattern(&self, pattern: &str) -> ServiceResult<Vec<ServiceId>> {
        let start_time = get_current_time();

        // Check cache first
        let cache_key = format!("pattern:{}", pattern);
        if let Some(cached_entry) = self.discovery_cache.read().get(&cache_key) {
            if cached_entry.timestamp + cached_entry.ttl > get_current_time() {
                // Cache hit
                self.discovery_stats.cache_hits += 1;
                return Ok(cached_entry.service_ids.clone());
            }
        }

        // Cache miss - perform discovery
        self.discovery_stats.cache_misses += 1;
        
        let query = ServiceDiscoveryQuery {
            name_pattern: Some(pattern.to_string()),
            service_type: None,
            tags: Vec::new(),
            healthy_only: false,
            include_metadata: false,
        };

        let service_ids = self.registry.entries.read().values()
            .filter(|entry| self.matches_pattern(&entry.name, pattern))
            .map(|entry| entry.service_id)
            .collect();

        // Cache the result
        let mut cache = self.discovery_cache.write();
        cache.insert(cache_key, DiscoveryCacheEntry {
            service_ids: service_ids.clone(),
            timestamp: get_current_time(),
            ttl: 30000, // 30 seconds TTL
        });

        // Record query
        let response_time = get_current_time() - start_time;
        let mut history = self.query_history.write();
        history.push(DiscoveryQuery {
            pattern: pattern.to_string(),
            timestamp: get_current_time(),
            result_count: service_ids.len(),
            response_time,
        });

        // Update stats
        self.discovery_stats.total_queries += 1;
        self.update_average_response_time(response_time);

        info!("Service discovery by pattern '{}': {} services found", pattern, service_ids.len());
        Ok(service_ids)
    }

    /// Discover services by filter
    pub fn discover_by_filter(&self, filter: &ServiceFilter) -> ServiceResult<Vec<ServiceEndpoint>> {
        let start_time = get_current_time();

        let mut endpoints = Vec::new();
        let entries = self.registry.entries.read();

        for entry in entries.values() {
            if self.matches_filter(entry, filter) {
                let endpoint = ServiceEndpoint {
                    service_id: entry.service_id,
                    name: entry.name.clone(),
                    address: entry.instance.host.clone(),
                    port: entry.instance.port,
                    protocol: super::config::Protocol::Http,
                    health_status: super::service::HealthStatus::Unknown,
                    metadata: BTreeMap::new(),
                };
                endpoints.push(endpoint);
            }
        }

        // Apply max results limit
        if let Some(max_results) = filter.max_results {
            endpoints.truncate(max_results);
        }

        // Record query
        let response_time = get_current_time() - start_time;
        let mut history = self.query_history.write();
        history.push(DiscoveryQuery {
            pattern: format!("filter:{:?}", filter),
            timestamp: get_current_time(),
            result_count: endpoints.len(),
            response_time,
        });

        info!("Service discovery by filter: {} services found", endpoints.len());
        Ok(endpoints)
    }

    /// Subscribe to service discovery events
    pub fn subscribe(&self, subscription: ServiceSubscription) -> ServiceResult<()> {
        // Implementation would handle service discovery subscriptions
        info!("Service discovery subscription created: {}", subscription.id);
        Ok(())
    }

    /// Unsubscribe from service discovery events
    pub fn unsubscribe(&self, subscription_id: &str) -> ServiceResult<()> {
        // Implementation would handle service discovery unsubscriptions
        info!("Service discovery subscription removed: {}", subscription_id);
        Ok(())
    }

    /// Get discovery statistics
    pub fn get_stats(&self) -> &DiscoveryStats {
        &self.discovery_stats
    }

    /// Get discovery count (for overall service manager stats)
    pub fn get_discovery_count(&self) -> u64 {
        self.discovery_stats.total_queries
    }

    /// Clear discovery cache
    pub fn clear_cache(&self) -> ServiceResult<()> {
        let mut cache = self.discovery_cache.write();
        cache.clear();
        
        info!("Service discovery cache cleared");
        Ok(())
    }

    /// Internal methods
    fn matches_pattern(&self, name: &str, pattern: &str) -> bool {
        // Enhanced pattern matching with wildcards and regex support
        if pattern == "*" || pattern.is_empty() {
            return true;
        }
        
        // Handle wildcards
        let regex_pattern = pattern
            .replace("*", ".*")    // * matches any sequence
            .replace("?", ".");    // ? matches any single character
        
        // Simple regex implementation - would use proper regex crate in full implementation
        if regex_pattern.starts_with(".*") && regex_pattern.ends_with(".*") {
            // Contains pattern anywhere
            let search = &regex_pattern[2..regex_pattern.len()-2];
            name.contains(search)
        } else if regex_pattern.starts_with(".*") {
            // Ends with pattern
            let search = &regex_pattern[2..];
            name.ends_with(search)
        } else if regex_pattern.ends_with(".*") {
            // Starts with pattern
            let search = &regex_pattern[..regex_pattern.len()-2];
            name.starts_with(search)
        } else {
            // Exact match
            name == regex_pattern
        }
    }

    /// Advanced pattern matching with regex support
    pub fn matches_advanced_pattern(&self, name: &str, pattern: &str) -> bool {
        // Support for more complex patterns
        if pattern.starts_with("regex:") {
            let regex_pattern = &pattern[6..]; // Remove "regex:" prefix
            // Would implement proper regex matching here
            self.matches_pattern(name, regex_pattern)
        } else if pattern.starts_with("glob:") {
            let glob_pattern = &pattern[5..]; // Remove "glob:" prefix
            self.matches_glob_pattern(name, glob_pattern)
        } else {
            self.matches_pattern(name, pattern)
        }
    }

    /// Glob pattern matching
    fn matches_glob_pattern(&self, name: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let name_chars: Vec<char> = name.chars().collect();
        
        let mut pattern_idx = 0;
        let mut name_idx = 0;
        let mut star_idx = usize::MAX;
        let mut temp_idx = usize::MAX;
        
        while name_idx < name_chars.len() {
            if pattern_idx < pattern_chars.len() && 
               (pattern_chars[pattern_idx] == name_chars[name_idx] || pattern_chars[pattern_idx] == '?') {
                name_idx += 1;
                pattern_idx += 1;
            } else if pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
                star_idx = pattern_idx;
                temp_idx = name_idx;
                pattern_idx += 1;
            } else if star_idx != usize::MAX {
                pattern_idx = star_idx + 1;
                name_idx = temp_idx + 1;
                temp_idx = name_idx;
            } else {
                return false;
            }
        }
        
        // Check remaining pattern chars are all '*'
        while pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
            pattern_idx += 1;
        }
        
        pattern_idx == pattern_chars.len()
    }

    fn matches_filter(&self, entry: &ServiceRegistryEntry, filter: &ServiceFilter) -> bool {
        // Check name pattern
        if let Some(name_pattern) = &filter.name_pattern {
            if !self.matches_pattern(&entry.name, name_pattern) {
                return false;
            }
        }

        // Check service types
        if !filter.service_types.is_empty() {
            // Would need to map service ID to service type
            // This is a simplified implementation
        }

        // Check tags
        for required_tag in &filter.tags {
            // Would need to get service tags from registry
            // This is a simplified implementation
        }

        // Check availability and health
        if filter.healthy_only && entry.instance.status != ServiceState::Running {
            return false;
        }

        if filter.available_only && entry.instance.status != ServiceState::Running {
            return false;
        }

        true
    }

    fn update_average_response_time(&self, new_response_time: u64) {
        // Calculate running average
        let total_queries = self.discovery_stats.total_queries as f64;
        let current_avg = self.discovery_stats.average_response_time;
        let new_avg = (current_avg * (total_queries - 1.0) + new_response_time as f64) / total_queries;
        self.discovery_stats.average_response_time = new_avg;
    }
}

/// Utility functions for service discovery
pub mod utils {
    use super::*;

    /// Parse service name from endpoint
    pub fn parse_service_name(endpoint: &ServiceEndpoint) -> &str {
        &endpoint.name
    }

    /// Check if service is available
    pub fn is_service_available(endpoint: &ServiceEndpoint) -> bool {
        matches!(endpoint.health_status, 
                super::super::service::HealthStatus::Healthy | 
                super::super::service::HealthStatus::Degraded)
    }

    /// Get service endpoint string
    pub fn get_endpoint_url(endpoint: &ServiceEndpoint) -> String {
        if let Some(port) = endpoint.port {
            format!("{}:{}", endpoint.address, port)
        } else {
            endpoint.address.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ServiceRegistry::new();
        let services = registry.get_all_services();
        assert_eq!(services.len(), 0);
    }

    #[test]
    fn test_discovery_creation() {
        let discovery = ServiceDiscovery::new();
        assert_eq!(discovery.get_stats().total_queries, 0);
        assert_eq!(discovery.get_stats().cache_hits, 0);
    }

    #[test]
    fn test_pattern_matching() {
        let discovery = ServiceDiscovery::new();
        
        assert!(discovery.matches_pattern("my-service", "my-"));
        assert!(discovery.matches_pattern("my-service", "service"));
        assert!(discovery.matches_pattern("my-service", "*"));
        assert!(!discovery.matches_pattern("my-service", "other"));
    }

    #[test]
    fn test_service_endpoint_creation() {
        let endpoint = ServiceEndpoint {
            service_id: ServiceId(1),
            name: "test-service".to_string(),
            address: "localhost".to_string(),
            port: Some(8080),
            protocol: super::super::config::Protocol::Http,
            health_status: super::super::service::HealthStatus::Healthy,
            metadata: BTreeMap::new(),
        };

        assert_eq!(endpoint.name, "test-service");
        assert_eq!(endpoint.port, Some(8080));
    }
}