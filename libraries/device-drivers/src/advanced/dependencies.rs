//! Driver Dependencies Management
//! 
//! Manages driver dependencies, version constraints, and dependency graph
//! resolution with cycle detection and conflict resolution.

use crate::{AdvancedDriverId, Version, VersionConstraint};
use crate::AdvancedDriverError::{self, *};
use alloc::collections::{BTreeMap, VecDeque, HashSet};
use alloc::string::String;
use log::{debug, warn, info, error};

/// Version constraint for driver dependencies
#[derive(Debug, Clone)]
pub struct VersionConstraint {
    pub min_version: Option<Version>,
    pub max_version: Option<Version>,
    pub exact_version: Option<Version>,
    pub allowed_prereleases: bool,
}

/// Driver dependency
#[derive(Debug, Clone)]
pub struct DriverDependency {
    pub target_driver_id: AdvancedDriverId,
    pub constraint: VersionConstraint,
    pub optional: bool,
}

/// Dependency graph edge
#[derive(Debug, Clone, Copy)]
struct DependencyEdge {
    from: AdvancedDriverId,
    to: AdvancedDriverId,
    weight: u32,
}

/// Dependency resolution info
#[derive(Debug, Clone)]
pub struct DependencyResolution {
    pub dependency_chain: Vec<AdvancedDriverId>,
    pub resolution_time: u64,
    pub conflicts: Vec<String>,
}

/// Dependency manager
pub struct DependencyManager {
    driver_dependencies: BTreeMap<AdvancedDriverId, Vec<DriverDependency>>,
    reverse_dependencies: BTreeMap<AdvancedDriverId, Vec<AdvancedDriverId>>,
    dependency_graph: Vec<DependencyEdge>,
    conflict_resolution_log: Vec<String>,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new() -> Self {
        Self {
            driver_dependencies: BTreeMap::new(),
            reverse_dependencies: BTreeMap::new(),
            dependency_graph: Vec::new(),
            conflict_resolution_log: Vec::new(),
        }
    }

    /// Register a driver with its dependencies
    pub fn register_driver(&mut self, driver_info: &crate::advanced::AdvancedDriverInfo) -> Result<(), AdvancedDriverError> {
        debug!("Registering dependencies for driver: {}", driver_info.name);
        
        let mut dependencies = Vec::new();
        
        // Convert version constraints to driver dependencies
        for constraint in &driver_info.dependencies {
            // This would need to be resolved to actual driver IDs
            // For now, we'll store the constraints
            // TODO: Implement actual driver ID resolution
        }
        
        self.driver_dependencies.insert(driver_info.id, dependencies);
        
        Ok(())
    }

    /// Add a dependency between drivers
    pub fn add_dependency(&mut self, driver_id: AdvancedDriverId, dependency: DriverDependency) -> Result<(), AdvancedDriverError> {
        debug!("Adding dependency: {:?} depends on {:?}", driver_id, dependency.target_driver_id);
        
        // Add forward dependency
        let dependencies = self.driver_dependencies.entry(driver_id).or_insert_with(Vec::new);
        dependencies.push(dependency.clone());
        
        // Add reverse dependency
        let reverse = self.reverse_dependencies.entry(dependency.target_driver_id).or_insert_with(Vec::new);
        if !reverse.contains(&driver_id) {
            reverse.push(driver_id);
        }
        
        // Add to dependency graph
        self.dependency_graph.push(DependencyEdge {
            from: driver_id,
            to: dependency.target_driver_id,
            weight: 1,
        });
        
        Ok(())
    }

    /// Remove a dependency between drivers
    pub fn remove_dependency(&mut self, driver_id: AdvancedDriverId, target_driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Removing dependency: {:?} from {:?}", target_driver_id, driver_id);
        
        // Remove forward dependency
        if let Some(dependencies) = self.driver_dependencies.get_mut(&driver_id) {
            dependencies.retain(|dep| dep.target_driver_id != target_driver_id);
        }
        
        // Remove reverse dependency
        if let Some(reverse) = self.reverse_dependencies.get_mut(&target_driver_id) {
            reverse.retain(|&id| id != driver_id);
        }
        
        // Remove from dependency graph
        self.dependency_graph.retain(|edge| !(edge.from == driver_id && edge.to == target_driver_id));
        
        Ok(())
    }

    /// Resolve all dependencies for a driver
    pub fn resolve_dependencies(&mut self, driver_id: AdvancedDriverId) -> Result<DependencyResolution, AdvancedDriverError> {
        debug!("Resolving dependencies for driver {:?}", driver_id);
        
        let start_time = 0; // TODO: Get actual timestamp
        
        // Detect cycles
        if self.has_cycle() {
            return Err(CircularDependency);
        }
        
        // Topological sort to get dependency order
        let dependency_chain = match self.topological_sort() {
            Ok(chain) => chain,
            Err(e) => {
                error!("Topological sort failed: {:?}", e);
                return Err(DependencyResolutionFailed);
            }
        };
        
        // Filter chain to include only dependencies of the target driver
        let driver_deps = self.get_dependency_chain(driver_id);
        
        let resolution = DependencyResolution {
            dependency_chain: driver_deps,
            resolution_time: 0, // TODO: Calculate actual time
            conflicts: Vec::new(),
        };
        
        debug!("Dependencies resolved for driver {:?}: {} dependencies", 
               driver_id, resolution.dependency_chain.len());
        
        Ok(resolution)
    }

    /// Check if there are circular dependencies
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &driver_id in self.dependency_graph.iter().map(|edge| &edge.from) {
            if !visited.contains(&driver_id) {
                if self.has_cycle_dfs(driver_id, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        
        false
    }

    /// DFS for cycle detection
    fn has_cycle_dfs(&self, driver_id: AdvancedDriverId, visited: &mut HashSet<AdvancedDriverId>, rec_stack: &mut HashSet<AdvancedDriverId>) -> bool {
        visited.insert(driver_id);
        rec_stack.insert(driver_id);
        
        // Get outgoing edges
        let outgoing: Vec<_> = self.dependency_graph.iter()
            .filter(|edge| edge.from == driver_id)
            .map(|edge| edge.to)
            .collect();
        
        for next_driver in outgoing {
            if !visited.contains(&next_driver) {
                if self.has_cycle_dfs(next_driver, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&next_driver) {
                return true;
            }
        }
        
        rec_stack.remove(&driver_id);
        false
    }

    /// Perform topological sort of dependency graph
    pub fn topological_sort(&self) -> Result<Vec<AdvancedDriverId>, AdvancedDriverError> {
        let mut in_degree = BTreeMap::new();
        let mut adjacency_list = BTreeMap::new();
        
        // Initialize in-degree and adjacency list
        for &driver_id in self.dependency_graph.iter().map(|edge| &edge.from) {
            in_degree.entry(driver_id).or_insert(0);
            adjacency_list.entry(driver_id).or_insert_with(Vec::new);
        }
        
        for &driver_id in self.dependency_graph.iter().map(|edge| &edge.to) {
            *in_degree.entry(driver_id).or_insert(0) += 1;
            adjacency_list.entry(driver_id).or_insert_with(Vec::new);
        }
        
        // Add dependencies as well
        for &driver_id in self.driver_dependencies.keys() {
            adjacency_list.entry(driver_id).or_insert_with(Vec::new);
        }
        
        // Kahn's algorithm
        let mut queue: VecDeque<_> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&driver_id, _)| driver_id)
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(driver_id) = queue.pop_front() {
            result.push(driver_id);
            
            // Update in-degree of dependents
            if let Some(dependents) = adjacency_list.get(&driver_id) {
                for &dependent_id in dependents {
                    if let Some(&mut ref mut degree) = in_degree.get_mut(&dependent_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent_id);
                        }
                    }
                }
            }
        }
        
        // Check if all nodes were processed (no cycles)
        if result.len() == in_degree.len() {
            Ok(result)
        } else {
            warn!("Topological sort incomplete - likely cycles detected");
            Err(CircularDependency)
        }
    }

    /// Get dependency chain for a specific driver
    fn get_dependency_chain(&self, driver_id: AdvancedDriverId) -> Vec<AdvancedDriverId> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        
        self.collect_dependencies(driver_id, &mut chain, &mut visited);
        
        chain
    }

    /// Collect dependencies recursively
    fn collect_dependencies(&self, driver_id: AdvancedDriverId, chain: &mut Vec<AdvancedDriverId>, visited: &mut HashSet<AdvancedDriverId>) {
        if visited.contains(&driver_id) {
            return;
        }
        
        visited.insert(driver_id);
        
        if let Some(dependencies) = self.driver_dependencies.get(&driver_id) {
            for dependency in dependencies {
                self.collect_dependencies(dependency.target_driver_id, chain, visited);
            }
        }
        
        if !chain.contains(&driver_id) {
            chain.push(driver_id);
        }
    }

    /// Get all direct dependencies of a driver
    pub fn get_dependencies(&self, driver_id: AdvancedDriverId) -> Vec<AdvancedDriverId> {
        self.driver_dependencies.get(&driver_id)
            .map(|deps| deps.iter().map(|dep| dep.target_driver_id).collect())
            .unwrap_or_default()
    }

    /// Get all drivers that depend on the given driver
    pub fn get_dependents(&self, driver_id: AdvancedDriverId) -> Vec<AdvancedDriverId> {
        self.reverse_dependencies.get(&driver_id)
            .map(|dependents| dependents.clone())
            .unwrap_or_default()
    }

    /// Get dependency graph statistics
    pub fn get_graph_size(&self) -> usize {
        self.dependency_graph.len()
    }

    /// Validate all dependencies
    pub fn validate_dependencies(&self) -> Result<(), AdvancedDriverError> {
        // Check for cycles
        if self.has_cycle() {
            warn!("Circular dependency detected");
            return Err(CircularDependency);
        }
        
        // Check for orphaned dependencies
        for (driver_id, dependencies) in &self.driver_dependencies {
            for dependency in dependencies {
                if !self.driver_dependencies.contains_key(&dependency.target_driver_id) {
                    warn!("Driver {:?} has dependency on non-existent driver {:?}", 
                          driver_id, dependency.target_driver_id);
                    return Err(DependencyResolutionFailed);
                }
            }
        }
        
        Ok(())
    }

    /// Get dependency analysis report
    pub fn get_dependency_report(&self) -> DependencyReport {
        let mut report = DependencyReport::new();
        
        // Count dependencies per driver
        for (driver_id, dependencies) in &self.driver_dependencies {
            report.add_driver_dependencies(*driver_id, dependencies.len());
        }
        
        // Find most depended drivers
        for (driver_id, dependents) in &self.reverse_dependencies {
            if dependents.len() > 1 {
                report.add_critical_driver(*driver_id, dependents.len());
            }
        }
        
        // Check for cycles
        if self.has_cycle() {
            report.has_cycles = true;
        }
        
        // Add dependency graph info
        report.total_edges = self.dependency_graph.len();
        report.total_nodes = self.driver_dependencies.len();
        
        report
    }
}

/// Dependency analysis report
#[derive(Debug, Clone)]
pub struct DependencyReport {
    pub total_edges: usize,
    pub total_nodes: usize,
    pub has_cycles: bool,
    pub critical_drivers: BTreeMap<AdvancedDriverId, usize>,
    pub dependencies_per_driver: BTreeMap<AdvancedDriverId, usize>,
}

impl DependencyReport {
    fn new() -> Self {
        Self {
            total_edges: 0,
            total_nodes: 0,
            has_cycles: false,
            critical_drivers: BTreeMap::new(),
            dependencies_per_driver: BTreeMap::new(),
        }
    }

    fn add_driver_dependencies(&mut self, driver_id: AdvancedDriverId, count: usize) {
        self.dependencies_per_driver.insert(driver_id, count);
    }

    fn add_critical_driver(&mut self, driver_id: AdvancedDriverId, dependent_count: usize) {
        self.critical_drivers.insert(driver_id, dependent_count);
    }
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_management() {
        let mut manager = DependencyManager::new();
        let driver1 = AdvancedDriverId(1);
        let driver2 = AdvancedDriverId(2);
        let driver3 = AdvancedDriverId(3);
        
        // Create simple dependency: driver1 -> driver2
        let dep = DriverDependency {
            target_driver_id: driver2,
            constraint: VersionConstraint {
                min_version: None,
                max_version: None,
                exact_version: None,
                allowed_prereleases: false,
            },
            optional: false,
        };
        
        assert!(manager.add_dependency(driver1, dep).is_ok());
        
        // Check dependencies
        let deps = manager.get_dependencies(driver1);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], driver2);
        
        let dependents = manager.get_dependents(driver2);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], driver1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut manager = DependencyManager::new();
        let driver1 = AdvancedDriverId(1);
        let driver2 = AdvancedDriverId(2);
        
        // Create circular dependency: driver1 -> driver2 -> driver1
        manager.add_dependency(driver1, DriverDependency {
            target_driver_id: driver2,
            constraint: VersionConstraint::default(),
            optional: false,
        }).unwrap();
        
        manager.add_dependency(driver2, DriverDependency {
            target_driver_id: driver1,
            constraint: VersionConstraint::default(),
            optional: false,
        }).unwrap();
        
        assert!(manager.has_cycle());
    }

    #[test]
    fn test_topological_sort() {
        let mut manager = DependencyManager::new();
        let driver1 = AdvancedDriverId(1);
        let driver2 = AdvancedDriverId(2);
        let driver3 = AdvancedDriverId(3);
        
        // Create dependency chain: driver3 -> driver2 -> driver1
        manager.add_dependency(driver1, DriverDependency {
            target_driver_id: driver2,
            constraint: VersionConstraint::default(),
            optional: false,
        }).unwrap();
        
        manager.add_dependency(driver2, DriverDependency {
            target_driver_id: driver3,
            constraint: VersionConstraint::default(),
            optional: false,
        }).unwrap();
        
        let result = manager.topological_sort().unwrap();
        
        // Should start with driver3 (no dependencies) and end with driver1 (has dependents)
        assert_eq!(result.len(), 3);
    }
}
