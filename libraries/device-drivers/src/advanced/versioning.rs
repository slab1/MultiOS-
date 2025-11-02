//! Version Management Module
//! 
//! Provides comprehensive version management for device drivers including
//! version constraints, compatibility checking, and version conflict resolution.

use crate::{AdvancedDriverId, Version, VersionConstraint};
use crate::AdvancedDriverError::{self, *};
use alloc::collections::BTreeMap;
use alloc::string::String;
use log::{debug, warn, info, error};

/// Version compatibility modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityMode {
    Strict,      // Only exact versions
    Backward,    // Backward compatible versions
    Forward,     // Forward compatible versions
    SemVer,      // Semantic versioning rules
    Custom,      // Custom compatibility rules
}

/// Version information for a driver
#[derive(Debug, Clone)]
pub struct DriverVersionInfo {
    pub driver_id: AdvancedDriverId,
    pub name: &'static str,
    pub version: Version,
    pub compatible_versions: Vec<Version>,
    pub required_dependencies: Vec<(String, VersionConstraint)>,
    pub api_version: Option<Version>,
    pub is_compatible: bool,
}

/// Version manager
pub struct VersionManager {
    registered_versions: BTreeMap<String, Vec<DriverVersionInfo>>,
    version_dependencies: BTreeMap<(String, Version), Vec<(String, VersionConstraint)>>,
    compatibility_mode: CompatibilityMode,
    conflict_resolution_policy: ConflictResolutionPolicy,
    version_history: Vec<VersionChange>,
    upgrade_suggestions: BTreeMap<String, Vec<String>>,
    max_versions_per_driver: usize,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Self {
        info!("Initializing Version Manager");
        
        let manager = Self {
            registered_versions: BTreeMap::new(),
            version_dependencies: BTreeMap::new(),
            compatibility_mode: CompatibilityMode::SemVer,
            conflict_resolution_policy: ConflictResolutionPolicy::Latest,
            version_history: Vec::new(),
            upgrade_suggestions: BTreeMap::new(),
            max_versions_per_driver: 10,
        };
        
        info!("Version Manager initialized with {:?} compatibility mode", manager.compatibility_mode);
        manager
    }

    /// Register a version for a driver
    pub fn register_version(&mut self, driver_name: &'static str, version: Version) -> Result<(), AdvancedDriverError> {
        debug!("Registering version {} for driver {}", version, driver_name);
        
        let driver_info = DriverVersionInfo {
            driver_id: AdvancedDriverId(0), // Will be set by caller
            name: driver_name,
            version,
            compatible_versions: Vec::new(),
            required_dependencies: Vec::new(),
            api_version: None,
            is_compatible: true,
        };
        
        let versions = self.registered_versions.entry(driver_name).or_insert_with(Vec::new);
        
        // Check for version conflicts
        for existing_info in versions.iter() {
            if existing_info.version == version {
                warn!("Version {} already registered for driver {}", version, driver_name);
                return Err(VersionConflict);
            }
            
            if self.is_incompatible(&existing_info.version, &version) {
                warn!("Version conflict detected: {} and {} for driver {}", 
                      existing_info.version, version, driver_name);
                return Err(VersionConflict);
            }
        }
        
        // Limit number of versions per driver
        if versions.len() >= self.max_versions_per_driver {
            warn!("Too many versions registered for driver {}, removing oldest", driver_name);
            versions.remove(0);
        }
        
        versions.push(driver_info);
        
        // Record version change
        self.version_history.push(VersionChange {
            driver_name: driver_name.to_string(),
            old_version: None,
            new_version: Some(version),
            timestamp: 0, // TODO: Get actual timestamp
            change_type: VersionChangeType::Registered,
        });
        
        info!("Version {} registered for driver {}", version, driver_name);
        Ok(())
    }

    /// Find compatible driver version
    pub fn find_compatible_driver(&self, constraint: &VersionConstraint) -> Result<AdvancedDriverId, AdvancedDriverError> {
        debug!("Finding compatible driver for constraint: {}", constraint.to_string());
        
        let mut best_match: Option<(AdvancedDriverId, &DriverVersionInfo)> = None;
        
        for (driver_name, versions) in &self.registered_versions {
            for version_info in versions {
                if version_info.version.satisfies(constraint) {
                    // Check if this is a better match
                    if let Some((_, best_info)) = &best_match {
                        if self.is_better_match(&version_info.version, &best_info.version, constraint) {
                            best_match = Some((version_info.driver_id, version_info));
                        }
                    } else {
                        best_match = Some((version_info.driver_id, version_info));
                    }
                }
            }
        }
        
        match best_match {
            Some((driver_id, _)) => {
                debug!("Found compatible driver: {:?}", driver_id);
                Ok(driver_id)
            }
            None => {
                debug!("No compatible driver found for constraint: {}", constraint.to_string());
                Err(VersionMismatch)
            }
        }
    }

    /// Check version compatibility
    pub fn is_compatible(&self, version1: &Version, version2: &Version) -> bool {
        match self.compatibility_mode {
            CompatibilityMode::Strict => *version1 == *version2,
            CompatibilityMode::Backward => {
                // Version2 should be >= version1
                *version2 >= *version1
            }
            CompatibilityMode::Forward => {
                // Version1 should be >= version2  
                *version1 >= *version2
            }
            CompatibilityMode::SemVer => {
                // Semantic versioning: compatible if major version matches and minor/patch are >=
                version1.major == version2.major &&
                (version1.minor < version2.minor || 
                 (version1.minor == version2.minor && version1.patch <= version2.patch))
            }
            CompatibilityMode::Custom => {
                // Custom logic would go here
                *version1 == *version2
            }
        }
    }

    /// Check if two versions are incompatible
    fn is_incompatible(&self, version1: &Version, version2: &Version) -> bool {
        match self.compatibility_mode {
            CompatibilityMode::Strict | CompatibilityMode::Custom => {
                version1.major == version2.major && 
                (version1.minor != version2.minor || version1.patch != version2.patch)
            }
            CompatibilityMode::SemVer => {
                // Major version mismatch means incompatibility
                version1.major != version2.major
            }
            _ => false,
        }
    }

    /// Check if a version is a better match than another
    fn is_better_match(&self, candidate: &Version, current: &Version, constraint: &VersionConstraint) -> bool {
        // For exact matches, prefer exact over range
        if constraint.exact_version.is_some() {
            return *candidate == constraint.exact_version.unwrap();
        }
        
        // For range constraints, prefer versions closer to the middle
        let (min, max) = if let (Some(min), Some(max)) = (constraint.min_version, constraint.max_version) {
            (min, max)
        } else {
            return *candidate > *current; // Default to newer version
        };
        
        let candidate_score = self.calculate_version_score(candidate, &min, &max);
        let current_score = self.calculate_version_score(current, &min, &max);
        
        candidate_score > current_score
    }

    /// Calculate version score for selection
    fn calculate_version_score(&self, version: &Version, min: &Version, max: &Version) -> f64 {
        // Simple scoring: closer to middle of range gets higher score
        let min_val = (min.major as f64 * 1000.0) + (min.minor as f64 * 100.0) + min.patch as f64;
        let max_val = (max.major as f64 * 1000.0) + (max.minor as f64 * 100.0) + max.patch as f64;
        let version_val = (version.major as f64 * 1000.0) + (version.minor as f64 * 100.0) + version.patch as f64;
        
        let range = max_val - min_val;
        let distance = (version_val - min_val).abs();
        
        if range > 0.0 {
            1.0 - (distance / range)
        } else {
            1.0
        }
    }

    /// Get all registered versions for a driver
    pub fn get_driver_versions(&self, driver_name: &str) -> Vec<&DriverVersionInfo> {
        self.registered_versions.get(driver_name)
            .map(|versions| versions.iter().collect())
            .unwrap_or_default()
    }

    /// Get version statistics
    pub fn get_version_statistics(&self) -> VersionStatistics {
        let mut total_drivers = self.registered_versions.len();
        let mut total_versions = 0;
        let mut conflicts_detected = 0;
        
        for versions in self.registered_versions.values() {
            total_versions += versions.len();
            
            // Check for conflicts
            for i in 0..versions.len() {
                for j in (i + 1)..versions.len() {
                    if self.is_incompatible(&versions[i].version, &versions[j].version) {
                        conflicts_detected += 1;
                    }
                }
            }
        }
        
        VersionStatistics {
            total_drivers,
            total_versions,
            conflicts_detected,
            compatibility_mode: self.compatibility_mode,
            resolution_policy: self.conflict_resolution_policy,
            version_history_size: self.version_history.len(),
        }
    }

    /// Set compatibility mode
    pub fn set_compatibility_mode(&mut self, mode: CompatibilityMode) -> Result<(), AdvancedDriverError> {
        debug!("Setting compatibility mode to {:?}", mode);
        self.compatibility_mode = mode;
        Ok(())
    }

    /// Set conflict resolution policy
    pub fn set_conflict_resolution_policy(&mut self, policy: ConflictResolutionPolicy) -> Result<(), AdvancedDriverError> {
        debug!("Setting conflict resolution policy to {:?}", policy);
        self.conflict_resolution_policy = policy;
        Ok(())
    }

    /// Resolve version conflicts
    pub fn resolve_conflicts(&mut self) -> Result<(), AdvancedDriverError> {
        debug!("Resolving version conflicts");
        
        let mut conflicts_resolved = 0;
        
        for (driver_name, versions) in self.registered_versions.iter_mut() {
            let mut i = 0;
            while i < versions.len() {
                let mut j = i + 1;
                while j < versions.len() {
                    if self.is_incompatible(&versions[i].version, &versions[j].version) {
                        debug!("Resolving conflict between {} v{} and v{} for driver {}", 
                               driver_name, versions[i].version, versions[j].version, driver_name);
                        
                        match self.conflict_resolution_policy {
                            ConflictResolutionPolicy::Latest => {
                                // Keep newer version
                                if versions[i].version > versions[j].version {
                                    versions.remove(j);
                                } else {
                                    versions.remove(i);
                                    i -= 1; // Adjust index after removal
                                    break;
                                }
                            }
                            ConflictResolutionPolicy::Earliest => {
                                // Keep older version
                                if versions[i].version < versions[j].version {
                                    versions.remove(j);
                                } else {
                                    versions.remove(i);
                                    i -= 1; // Adjust index after removal
                                    break;
                                }
                            }
                            ConflictResolutionPolicy::Manual => {
                                // Mark for manual resolution
                                versions[i].is_compatible = false;
                                versions[j].is_compatible = false;
                                j += 1;
                            }
                        }
                        
                        conflicts_resolved += 1;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
        
        info!("Resolved {} version conflicts", conflicts_resolved);
        Ok(())
    }

    /// Get version history
    pub fn get_version_history(&self) -> &[VersionChange] {
        &self.version_history
    }

    /// Clear version history
    pub fn clear_version_history(&mut self) -> Result<(), AdvancedDriverError> {
        debug!("Clearing version history");
        self.version_history.clear();
        Ok(())
    }

    /// Generate version report
    pub fn generate_version_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Version Management Report\n");
        report.push_str("=========================\n\n");
        
        if self.registered_versions.is_empty() {
            report.push_str("No versions registered.\n");
            return report;
        }
        
        for (driver_name, versions) in &self.registered_versions {
            report.push_str(&format!("Driver: {}\n", driver_name));
            
            for version_info in versions {
                report.push_str(&format!("  Version: {}\n", version_info.version));
                if !version_info.is_compatible {
                    report.push_str("    Status: INCOMPATIBLE\n");
                }
            }
            
            report.push_str("\n");
        }
        
        let stats = self.get_version_statistics();
        report.push_str(&format!("Statistics:\n"));
        report.push_str(&format!("  Total Drivers: {}\n", stats.total_drivers));
        report.push_str(&format!("  Total Versions: {}\n", stats.total_versions));
        report.push_str(&format!("  Conflicts: {}\n", stats.conflicts_detected));
        report.push_str(&format!("  Compatibility Mode: {:?}\n", stats.compatibility_mode));
        
        report
    }
}

/// Conflict resolution policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionPolicy {
    Latest,      // Keep latest version
    Earliest,    // Keep earliest version
    Manual,      // Manual resolution required
}

/// Version change types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionChangeType {
    Registered,
    Updated,
    Removed,
    Conflict,
}

/// Version change record
#[derive(Debug, Clone)]
pub struct VersionChange {
    pub driver_name: String,
    pub old_version: Option<Version>,
    pub new_version: Option<Version>,
    pub timestamp: u64,
    pub change_type: VersionChangeType,
}

/// Version management statistics
#[derive(Debug, Clone)]
pub struct VersionStatistics {
    pub total_drivers: usize,
    pub total_versions: usize,
    pub conflicts_detected: usize,
    pub compatibility_mode: CompatibilityMode,
    pub resolution_policy: ConflictResolutionPolicy,
    pub version_history_size: usize,
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.prerelease, None);
    }

    #[test]
    fn test_version_constraint_satisfaction() {
        let version = Version::new(2, 0, 0);
        let constraint = VersionConstraint::minimum(Version::new(1, 0, 0));
        
        assert!(version.satisfies(&constraint));
        
        let constraint = VersionConstraint::exact(Version::new(2, 0, 0));
        assert!(version.satisfies(&constraint));
        
        let constraint = VersionConstraint::maximum(Version::new(1, 0, 0));
        assert!(!version.satisfies(&constraint));
    }

    #[test]
    fn test_version_registration() {
        let mut manager = VersionManager::new();
        
        assert!(manager.register_version("test_driver", Version::new(1, 0, 0)).is_ok());
        
        let versions = manager.get_driver_versions("test_driver");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_version_conflict() {
        let mut manager = VersionManager::new();
        
        assert!(manager.register_version("test_driver", Version::new(1, 0, 0)).is_ok());
        assert!(manager.register_version("test_driver", Version::new(1, 0, 0)).is_err());
    }

    #[test]
    fn test_compatibility_checking() {
        let mut manager = VersionManager::new();
        manager.set_compatibility_mode(CompatibilityMode::SemVer).unwrap();
        
        assert!(manager.is_compatible(&Version::new(1, 0, 0), &Version::new(1, 0, 1)));
        assert!(manager.is_compatible(&Version::new(1, 0, 0), &Version::new(1, 1, 0)));
        assert!(!manager.is_compatible(&Version::new(1, 0, 0), &Version::new(2, 0, 0)));
    }

    #[test]
    fn test_constraint_matching() {
        let mut manager = VersionManager::new();
        
        manager.register_version("test_driver", Version::new(1, 0, 0)).unwrap();
        manager.register_version("test_driver", Version::new(1, 1, 0)).unwrap();
        manager.register_version("test_driver", Version::new(2, 0, 0)).unwrap();
        
        let constraint = VersionConstraint::minimum(Version::new(1, 0, 0));
        let driver_id = manager.find_compatible_driver(&constraint).unwrap();
        
        assert!(driver_id.0 > 0);
    }

    #[test]
    fn test_version_statistics() {
        let mut manager = VersionManager::new();
        
        manager.register_version("driver1", Version::new(1, 0, 0)).unwrap();
        manager.register_version("driver2", Version::new(1, 0, 0)).unwrap();
        manager.register_version("driver2", Version::new(1, 1, 0)).unwrap();
        
        let stats = manager.get_version_statistics();
        assert_eq!(stats.total_drivers, 2);
        assert_eq!(stats.total_versions, 3);
    }
}
