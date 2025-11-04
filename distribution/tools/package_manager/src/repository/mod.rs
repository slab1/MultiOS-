//! MultiOS Package Manager - Repository Management
//! 
//! This module provides comprehensive repository management capabilities,
//! including package discovery, metadata synchronization, and repository security.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::{Package, Repository, PackageError, types::{Architecture, Version, Dependency}, security::SecurityValidator};
use super::storage::{PackageStorage, MetadataCache};

/// Repository manager for handling multiple repositories
#[derive(Debug)]
pub struct RepositoryManager {
    repositories: HashMap<String, Arc<RwLock<RepositoryInfo>>>,
    storage: Arc<PackageStorage>,
    cache: Arc<MetadataCache>,
    security_validator: Arc<SecurityValidator>,
    concurrency_limit: Arc<Semaphore>,
    config: RepositoryManagerConfig,
}

impl RepositoryManager {
    pub fn new(storage: Arc<PackageStorage>, cache: Arc<MetadataCache>) -> Self {
        Self {
            repositories: HashMap::new(),
            storage,
            cache,
            security_validator: Arc::new(SecurityValidator::new()),
            concurrency_limit: Arc::new(Semaphore::new(10)), // Limit concurrent downloads
            config: RepositoryManagerConfig::default(),
        }
    }
    
    /// Add a new repository to the manager
    pub async fn add_repository(&mut self, repository: Repository) -> Result<()> {
        let repo_info = RepositoryInfo::new(repository)?;
        self.repositories.insert(repo_info.name.clone(), Arc::new(RwLock::new(repo_info)));
        log::info!("Added repository: {}", repository.name);
        Ok(())
    }
    
    /// Remove a repository from the manager
    pub async fn remove_repository(&mut self, name: &str) -> Result<()> {
        if let Some(repo) = self.repositories.remove(name) {
            // Clean up repository data
            let repo_lock = repo.read().await;
            self.storage.cleanup_repository(&repo_lock.id).await?;
            log::info!("Removed repository: {}", name);
        }
        Ok(())
    }
    
    /// List all repositories
    pub async fn list_repositories(&self) -> Vec<String> {
        self.repositories.keys().cloned().collect()
    }
    
    /// Search for packages across all repositories
    pub async fn search_packages(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let mut tasks = Vec::new();
        
        for repo_info in self.repositories.values() {
            let repo_lock = repo_info.read().await;
            let repository = &repo_lock.repository;
            
            if !repository.enabled {
                continue;
            }
            
            // Search repository metadata cache first
            if let Some(cached_results) = self.cache.search_metadata(repository.id, query).await? {
                results.extend(cached_results.into_iter().map(|pkg| SearchResult {
                    package: pkg,
                    repository: repository.name.clone(),
                    match_score: 0.0,
                    match_reason: "cached".to_string(),
                }));
            } else {
                // Schedule async search if not in cache
                let task = self.search_repository_package(repository.clone(), query.clone());
                tasks.push(task);
            }
        }
        
        // Execute all search tasks
        let search_results = futures::future::join_all(tasks).await;
        for result in search_results {
            if let Ok(repo_results) = result {
                results.extend(repo_results);
            }
        }
        
        // Sort results by score
        results.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }
    
    /// Get package information by name and version
    pub async fn get_package(&self, name: &str, version: Option<&Version>) -> Result<Option<Package>> {
        for repo_info in self.repositories.values() {
            let repo_lock = repo_info.read().await;
            let repository = &repo_lock.repository;
            
            if !repository.enabled {
                continue;
            }
            
            if let Some(package) = self.cache.get_package_metadata(repository.id, name, version).await? {
                return Ok(Some(package));
            }
        }
        Ok(None)
    }
    
    /// Get all available versions of a package
    pub async fn get_package_versions(&self, name: &str) -> Result<Vec<Version>> {
        let mut versions = HashSet::new();
        
        for repo_info in self.repositories.values() {
            let repo_lock = repo_info.read().await;
            let repository = &repo_lock.repository;
            
            if !repository.enabled {
                continue;
            }
            
            let repo_versions = self.cache.get_package_versions(repository.id, name).await?;
            versions.extend(repo_versions);
        }
        
        Ok(versions.into_iter().collect())
    }
    
    /// Check for package updates
    pub async fn check_for_updates(&self) -> Result<Vec<UpdateInfo>> {
        let mut updates = Vec::new();
        
        for repo_info in self.repositories.values() {
            let repo_lock = repo_info.read().await;
            let repository = &repo_lock.repository;
            
            if !repository.enabled {
                continue;
            }
            
            let repo_updates = self.check_repository_updates(repository).await?;
            updates.extend(repo_updates);
        }
        
        Ok(updates)
    }
    
    /// Synchronize repository metadata
    pub async fn sync_repository(&self, name: &str) -> Result<()> {
        if let Some(repo_info) = self.repositories.get(name) {
            let mut repo_lock = repo_info.write().await;
            self.update_repository_metadata(&mut repo_lock).await?;
            log::info!("Synchronized repository: {}", name);
        }
        Ok(())
    }
    
    /// Synchronize all repositories
    pub async fn sync_all_repositories(&self) -> Result<()> {
        let mut tasks = Vec::new();
        
        for repo_name in self.repositories.keys() {
            let repo_name = repo_name.clone();
            let task = self.sync_repository(&repo_name);
            tasks.push(tokio::spawn(task));
        }
        
        futures::future::join_all(tasks).await;
        log::info!("Synchronized all repositories");
        Ok(())
    }
    
    /// Download package from repository
    pub async fn download_package(&self, repository_name: &str, package: &Package) -> Result<PathBuf> {
        let repo_info = self.repositories.get(repository_name)
            .ok_or_else(|| PackageError::RepositoryError {
                error: format!("Repository not found: {}", repository_name)
            })?;
        
        let repo_lock = repo_info.read().await;
        let repository = &repo_lock.repository;
        
        let download_permit = self.concurrency_limit.acquire().await?;
        
        // Check if package already exists in storage
        let package_path = self.storage.get_package_path(repository.id, package).await?;
        if package_path.exists() {
            return Ok(package_path);
        }
        
        // Download package
        let download_url = format!("{}/packages/{}-{}.tar.xz", 
            repository.base_url, package.name, package.version);
        
        log::info!("Downloading package: {} from {}", package.name, download_url);
        
        let package_data = self.download_package_data(&download_url).await?;
        
        // Validate package signature
        if let Some(ref signature) = package.signature {
            self.security_validator.verify_package_signature(&package_data, signature)?;
        }
        
        // Store package
        let stored_path = self.storage.store_package(repository.id, package, &package_data).await?;
        
        log::info!("Downloaded and stored package: {}", package.name);
        
        Ok(stored_path)
    }
    
    async fn search_repository_package(&self, repository: Repository, query: SearchQuery) -> Result<Vec<SearchResult>> {
        // Implementation would query the repository's package database
        // This is a placeholder for the actual implementation
        Ok(Vec::new())
    }
    
    async fn check_repository_updates(&self, repository: &Repository) -> Result<Vec<UpdateInfo>> {
        // Implementation would check for package updates in the repository
        // This is a placeholder for the actual implementation
        Ok(Vec::new())
    }
    
    async fn update_repository_metadata(&self, repo_info: &mut RepositoryInfo) -> Result<()> {
        // Implementation would update repository metadata from the remote server
        // This is a placeholder for the actual implementation
        Ok(())
    }
    
    async fn download_package_data(&self, url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(url).await
            .map_err(|e| PackageError::DownloadFailed {
                url: url.to_string(),
                error: e.to_string()
            })?;
        
        if !response.status().is_success() {
            return Err(PackageError::DownloadFailed {
                url: url.to_string(),
                error: format!("HTTP {}", response.status())
            }.into());
        }
        
        let bytes = response.bytes().await
            .map_err(|e| PackageError::NetworkError {
                error: e.to_string()
            })?;
        
        Ok(bytes.to_vec())
    }
}

/// Repository information wrapper
#[derive(Debug)]
struct RepositoryInfo {
    repository: Repository,
    metadata_cache: MetadataCacheInfo,
    sync_status: SyncStatus,
    error_count: u32,
}

impl RepositoryInfo {
    fn new(repository: Repository) -> Result<Self> {
        let metadata_cache = MetadataCacheInfo::new(&repository)?;
        
        Ok(Self {
            repository,
            metadata_cache,
            sync_status: SyncStatus::NeverSynced,
            error_count: 0,
        })
    }
}

/// Metadata cache for repository
#[derive(Debug)]
struct MetadataCacheInfo {
    packages: HashMap<String, Package>,
    last_sync: Option<DateTime<Utc>>,
    cache_valid: bool,
}

impl MetadataCacheInfo {
    fn new(_repository: &Repository) -> Result<Self> {
        Ok(Self {
            packages: HashMap::new(),
            last_sync: None,
            cache_valid: false,
        })
    }
}

/// Repository sync status
#[derive(Debug, Clone)]
enum SyncStatus {
    NeverSynced,
    Syncing,
    Synced(DateTime<Utc>),
    Error(String),
}

/// Package search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub architecture: Option<Architecture>,
    pub version_constraint: Option<String>,
    pub include_development: bool,
    pub include_deprecated: bool,
    pub limit: Option<usize>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub package: Package,
    pub repository: String,
    pub match_score: f32,
    pub match_reason: String,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub package: Package,
    pub current_version: Version,
    pub available_version: Version,
    pub update_type: UpdateType,
    pub security_update: bool,
    pub delta_available: bool,
    pub repository: String,
    pub description: String,
}

/// Type of update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    Minor,
    Major,
    Security,
    BugFix,
    Feature,
}

/// Repository manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryManagerConfig {
    pub max_concurrent_downloads: usize,
    pub download_timeout: std::time::Duration,
    pub cache_ttl: std::time::Duration,
    pub auto_sync_interval: std::time::Duration,
    pub retry_count: u32,
    pub retry_delay: std::time::Duration,
}

impl Default for RepositoryManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 10,
            download_timeout: std::time::Duration::from_secs(300),
            cache_ttl: std::time::Duration::from_secs(3600),
            auto_sync_interval: std::time::Duration::from_secs(86400), // 24 hours
            retry_count: 3,
            retry_delay: std::time::Duration::from_secs(5),
        }
    }
}