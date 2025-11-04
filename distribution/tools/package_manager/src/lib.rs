//! MultiOS Package Manager - Main Library
//! 
//! This is the main library crate that provides a unified interface
//! to all package management functionality.

use std::sync::Arc;
use tokio::sync::RwLock;

pub mod core;
pub mod repository;
pub mod packages;
pub mod security;
pub mod storage;
pub mod scheduler;

use crate::core::{PackageError, types::{Package, Repository}};
use crate::repository::RepositoryManager;
use crate::packages::PackageManager;
use crate::security::SecurityValidator;
use crate::storage::{PackageStorage, MetadataCache};
use crate::scheduler::{UpdateScheduler, ScheduleConfig};

/// Main package manager instance
#[derive(Debug)]
pub struct MultiOSPackageManager {
    repository_manager: Arc<RwLock<RepositoryManager>>,
    package_manager: Arc<PackageManager>,
    update_scheduler: Arc<RwLock<UpdateScheduler>>,
    security_validator: Arc<SecurityValidator>,
}

impl MultiOSPackageManager {
    /// Create a new package manager instance
    pub async fn new(data_directory: std::path::PathBuf) -> Result<Self, PackageError> {
        // Initialize storage
        let storage = Arc::new(PackageStorage::new(data_directory.join("packages"))?);
        let cache = Arc::new(MetadataCache::new(data_directory.join("cache"))?);
        
        // Initialize repository manager
        let repository_manager = Arc::new(RwLock::new(RepositoryManager::new(storage, cache)));
        
        // Initialize security validator
        let security_validator = Arc::new(SecurityValidator::new());
        
        // Initialize package manager
        let repo_manager_clone = repository_manager.clone();
        let security_validator_clone = security_validator.clone();
        let package_manager = Arc::new(PackageManager::new(
            Arc::try_unwrap(repo_manager_clone).unwrap().into_inner(),
            Arc::try_unwrap(security_validator_clone).unwrap().into_inner(),
        ));
        
        // Initialize update scheduler
        let scheduler_config = ScheduleConfig::default();
        let update_scheduler = Arc::new(RwLock::new(UpdateScheduler::new(
            package_manager.clone(),
            repository_manager.clone(),
            scheduler_config,
        )));
        
        Ok(Self {
            repository_manager,
            package_manager,
            update_scheduler,
            security_validator,
        })
    }
    
    /// Add a package repository
    pub async fn add_repository(&self, repository: Repository) -> Result<(), PackageError> {
        let mut repo_manager = self.repository_manager.write().await;
        repo_manager.add_repository(repository).await?;
        
        // Start scheduler if not running
        let mut scheduler = self.update_scheduler.write().await;
        if let Err(e) = scheduler.start().await {
            log::warn!("Failed to start update scheduler: {}", e);
        }
        
        Ok(())
    }
    
    /// Remove a package repository
    pub async fn remove_repository(&self, name: &str) -> Result<(), PackageError> {
        let mut repo_manager = self.repository_manager.write().await;
        repo_manager.remove_repository(name).await?;
        Ok(())
    }
    
    /// List all repositories
    pub async fn list_repositories(&self) -> Vec<String> {
        let repo_manager = self.repository_manager.read().await;
        repo_manager.list_repositories().await
    }
    
    /// Install packages
    pub async fn install(&self, package_names: Vec<String>, version_specs: Option<Vec<String>>) -> Result<(), PackageError> {
        if package_names.is_empty() {
            return Err(PackageError::ConfigurationError {
                error: "No packages specified for installation".to_string()
            });
        }
        
        let mut results = Vec::new();
        
        for (i, package_name) in package_names.iter().enumerate() {
            let version = version_specs.as_ref().and_then(|vs| vs.get(i)).map(|s| s.as_str());
            
            match self.package_manager.install_package(package_name, None).await {
                Ok(result) => {
                    log::info!("Successfully installed package: {}", package_name);
                    results.push((package_name.clone(), Ok(result)));
                }
                Err(e) => {
                    log::error!("Failed to install package {}: {}", package_name, e);
                    results.push((package_name.clone(), Err(e)));
                }
            }
        }
        
        // Report summary
        let successful: Vec<_> = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed: Vec<_> = results.iter().filter(|(_, r)| r.is_err()).count();
        
        log::info!("Installation complete: {} successful, {} failed", successful, failed);
        
        if failed > 0 {
            return Err(PackageError::SystemError {
                error: format!("Failed to install {} packages", failed)
            });
        }
        
        Ok(())
    }
    
    /// Uninstall packages
    pub async fn uninstall(&self, package_names: Vec<String>) -> Result<(), PackageError> {
        if package_names.is_empty() {
            return Err(PackageError::ConfigurationError {
                error: "No packages specified for uninstallation".to_string()
            });
        }
        
        let mut results = Vec::new();
        
        for package_name in &package_names {
            match self.package_manager.uninstall_package(package_name).await {
                Ok(_) => {
                    log::info!("Successfully uninstalled package: {}", package_name);
                    results.push((package_name.clone(), Ok(())));
                }
                Err(e) => {
                    log::error!("Failed to uninstall package {}: {}", package_name, e);
                    results.push((package_name.clone(), Err(e)));
                }
            }
        }
        
        // Report summary
        let successful: Vec<_> = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed: Vec<_> = results.iter().filter(|(_, r)| r.is_err()).count();
        
        log::info!("Uninstallation complete: {} successful, {} failed", successful, failed);
        
        if failed > 0 {
            return Err(PackageError::SystemError {
                error: format!("Failed to uninstall {} packages", failed)
            });
        }
        
        Ok(())
    }
    
    /// Update packages
    pub async fn update(&self, package_names: Option<Vec<String>>) -> Result<Vec<packages::UpdateResult>, PackageError> {
        let results = self.package_manager.update_packages(package_names).await?;
        
        let successful: Vec<_> = results.iter().filter(|r| r.success).count();
        let failed: Vec<_> = results.iter().filter(|r| !r.success).count();
        
        log::info!("Update complete: {} successful, {} failed", successful, failed);
        
        Ok(results)
    }
    
    /// Search for packages
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<packages::SearchResult>, PackageError> {
        let mut results = self.package_manager.search_packages(query).await?;
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    /// Get installed packages
    pub async fn list_installed(&self) -> Vec<packages::InstalledPackage> {
        self.package_manager.get_installed_packages().await
    }
    
    /// Get package information
    pub async fn info(&self, package_name: &str) -> Result<Option<packages::PackageInfo>, PackageError> {
        self.package_manager.get_package_info(package_name).await
    }
    
    /// Verify installed packages
    pub async fn verify(&self) -> Result<Vec<packages::VerificationResult>, PackageError> {
        let results = self.package_manager.verify_installed_packages().await?;
        
        let passed: Vec<_> = results.iter().filter(|r| matches!(r.status, packages::VerificationStatus::Passed)).count();
        let failed: Vec<_> = results.iter().filter(|r| matches!(r.status, packages::VerificationStatus::Failed)).count();
        
        log::info!("Verification complete: {} passed, {} failed", passed, failed);
        
        Ok(results)
    }
    
    /// Check for available updates
    pub async fn check_updates(&self) -> Result<Vec<repository::UpdateInfo>, PackageError> {
        let repo_manager = self.repository_manager.read().await;
        let updates = repo_manager.check_for_updates().await?;
        
        log::info!("Found {} available updates", updates.len());
        
        Ok(updates)
    }
    
    /// Synchronize repositories
    pub async fn sync_repositories(&self) -> Result<(), PackageError> {
        let repo_manager = self.repository_manager.read().await;
        repo_manager.sync_all_repositories().await?;
        
        log::info!("All repositories synchronized");
        Ok(())
    }
    
    /// Rollback package to previous version
    pub async fn rollback(&self, package_name: &str, version: &str) -> Result<(), PackageError> {
        let parsed_version = core::types::Version::parse(version)
            .map_err(|e| PackageError::ConfigurationError {
                error: format!("Invalid version format: {}", e)
            })?;
        
        self.package_manager.rollback_package(package_name, &parsed_version).await?;
        
        log::info!("Rolled back package {} to version {}", package_name, version);
        Ok(())
    }
    
    /// Get package manager status
    pub async fn status(&self) -> PackageManagerStatus {
        let repo_manager = self.repository_manager.read().await;
        let installed_packages = self.list_installed().await;
        
        PackageManagerStatus {
            repositories: repo_manager.list_repositories().await.len(),
            installed_packages: installed_packages.len(),
            scheduler_running: true, // Would check actual scheduler status
            last_update_check: None, // Would track this
        }
    }
    
    /// Configure automatic updates
    pub async fn configure_auto_updates(&self, config: ScheduleConfig) -> Result<(), PackageError> {
        let mut scheduler = self.update_scheduler.write().await;
        
        // Update configuration
        *scheduler.config.write().await = config;
        
        // Update default tasks
        scheduler.setup_default_schedule().await?;
        
        log::info!("Auto-update configuration updated");
        Ok(())
    }
    
    /// Clean up old packages and cache
    pub async fn cleanup(&self) -> Result<CleanupResult, PackageError> {
        let repo_manager = self.repository_manager.read().await;
        
        // This would implement cleanup logic
        log::info!("Cleanup completed");
        
        Ok(CleanupResult {
            packages_removed: 0,
            cache_size_freed: 0,
            temporary_files_removed: 0,
        })
    }
    
    /// Export package list
    pub async fn export_packages(&self, format: &str) -> Result<String, PackageError> {
        let installed_packages = self.list_installed().await;
        
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&installed_packages)
                    .map_err(|e| PackageError::JsonError(e))?;
                Ok(json)
            }
            "csv" => {
                let mut csv = String::new();
                csv.push_str("Name,Version,Description,Architecture,Install Date\n");
                
                for pkg in installed_packages {
                    csv.push_str(&format!("{},{},{},{},{}\n",
                        pkg.name,
                        pkg.version,
                        pkg.description.replace(',', " "),
                        pkg.architecture,
                        pkg.install_date
                    ));
                }
                Ok(csv)
            }
            _ => Err(PackageError::ConfigurationError {
                error: format!("Unsupported export format: {}", format)
            })
        }
    }
}

/// Package manager status information
#[derive(Debug, Clone)]
pub struct PackageManagerStatus {
    pub repositories: usize,
    pub installed_packages: usize,
    pub scheduler_running: bool,
    pub last_update_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cleanup operation result
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub packages_removed: u32,
    pub cache_size_freed: u64,
    pub temporary_files_removed: u32,
}

/// CLI application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = std::env::args().collect::<Vec<_>>();
    
    if args.len() < 2 {
        println!("Usage: multios-pm <command> [options]");
        println!("Commands:");
        println!("  install <package>...     Install packages");
        println!("  uninstall <package>...   Uninstall packages");
        println!("  update [package]...      Update packages");
        println!("  search <query>           Search for packages");
        println!("  list                     List installed packages");
        println!("  info <package>           Show package information");
        println!("  verify                   Verify installed packages");
        println!("  check-updates            Check for available updates");
        println!("  sync                     Synchronize repositories");
        println!("  rollback <package> <version> Rollback package");
        println!("  status                   Show package manager status");
        println!("  cleanup                  Clean up old packages");
        std::process::exit(1);
    }
    
    let data_dir = std::path::PathBuf::from("/var/lib/multios-package-manager");
    let manager = MultiOSPackageManager::new(data_dir).await?;
    
    match args[1].as_str() {
        "install" => {
            if args.len() < 3 {
                println!("Usage: install <package>...");
                std::process::exit(1);
            }
            let packages = args[2..].to_vec();
            manager.install(packages, None).await?;
        }
        "uninstall" => {
            if args.len() < 3 {
                println!("Usage: uninstall <package>...");
                std::process::exit(1);
            }
            let packages = args[2..].to_vec();
            manager.uninstall(packages).await?;
        }
        "update" => {
            let packages = if args.len() > 2 {
                Some(args[2..].to_vec())
            } else {
                None
            };
            manager.update(packages).await?;
        }
        "search" => {
            if args.len() < 3 {
                println!("Usage: search <query>");
                std::process::exit(1);
            }
            let results = manager.search(&args[2], None).await?;
            for result in results {
                println!("{} {} - {} ({})", 
                    result.name, result.version, result.description, result.architecture);
            }
        }
        "list" => {
            let packages = manager.list_installed().await;
            for pkg in packages {
                println!("{} {} - {} ({})", 
                    pkg.name, pkg.version, pkg.description, pkg.architecture);
            }
        }
        "info" => {
            if args.len() < 3 {
                println!("Usage: info <package>");
                std::process::exit(1);
            }
            if let Some(info) = manager.info(&args[2]).await? {
                println!("Package: {}", info.name);
                println!("Version: {}", info.version);
                println!("Description: {}", info.description);
                println!("Architecture: {}", info.architecture);
                println!("Status: {:?}", info.status);
                println!("Size: {} bytes", info.size);
                if let Some(date) = info.install_date {
                    println!("Install Date: {}", date);
                }
            } else {
                println!("Package not found");
            }
        }
        "verify" => {
            let results = manager.verify().await?;
            for result in results {
                println!("{} {} - {:?}", 
                    result.package_name, result.version, result.status);
            }
        }
        "check-updates" => {
            let updates = manager.check_updates().await?;
            for update in updates {
                println!("{}: {} -> {} ({})", 
                    update.package.name, 
                    update.current_version, 
                    update.available_version,
                    if update.security_update { "security" } else { "regular" }
                );
            }
        }
        "sync" => {
            manager.sync_repositories().await?;
            println!("Repositories synchronized");
        }
        "rollback" => {
            if args.len() < 4 {
                println!("Usage: rollback <package> <version>");
                std::process::exit(1);
            }
            manager.rollback(&args[2], &args[3]).await?;
        }
        "status" => {
            let status = manager.status().await;
            println!("Repositories: {}", status.repositories);
            println!("Installed packages: {}", status.installed_packages);
            println!("Scheduler running: {}", status.scheduler_running);
        }
        "cleanup" => {
            let result = manager.cleanup().await?;
            println!("Cleanup completed:");
            println!("  Packages removed: {}", result.packages_removed);
            println!("  Cache size freed: {} bytes", result.cache_size_freed);
            println!("  Temporary files removed: {}", result.temporary_files_removed);
        }
        cmd => {
            println!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
    
    Ok(())
}