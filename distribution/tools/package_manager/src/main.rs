//! MultiOS Package Manager - CLI Entry Point
//! 
//! This file provides the main entry point for the command-line interface
//! to the MultiOS Package Manager.

use clap::{Parser, Subcommand, ArgEnum};

/// MultiOS Package Manager CLI
#[derive(Parser, Debug)]
#[clap(name = "multios-pm")]
#[clap(about = "MultiOS Package Manager - Comprehensive package and update management system")]
#[clap(version = "0.1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    
    /// Package manager data directory
    #[clap(short, long, default_value = "/var/lib/multios-package-manager")]
    data_dir: std::path::PathBuf,
    
    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
    
    /// Configuration file path
    #[clap(short, long, default_value = "/etc/multios-package-manager/config.toml")]
    config: std::path::PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install packages
    Install {
        #[clap(required = true)]
        packages: Vec<String>,
        
        /// Package versions (optional)
        #[clap(short, long)]
        version: Option<Vec<String>>,
        
        /// Force installation without dependency checks
        #[clap(short, long)]
        force: bool,
        
        /// Install from specific repository
        #[clap(short, long)]
        repository: Option<String>,
    },
    
    /// Uninstall packages
    Uninstall {
        #[clap(required = true)]
        packages: Vec<String>,
        
        /// Remove configuration files
        #[clap(short, long)]
        purge: bool,
        
        /// Force removal even if dependencies exist
        #[clap(short, long)]
        force: bool,
    },
    
    /// Update packages
    Update {
        /// Specific packages to update (all if not specified)
        packages: Vec<String>,
        
        /// Include development packages
        #[clap(short, long)]
        development: bool,
        
        /// Update only security fixes
        #[clap(short, long)]
        security_only: bool,
    },
    
    /// Search for packages
    Search {
        #[clap(required = true)]
        query: String,
        
        /// Search in descriptions
        #[clap(short, long)]
        description: bool,
        
        /// Search in tags
        #[clap(short, long)]
        tags: bool,
        
        /// Limit results
        #[clap(short, long, default_value = "50")]
        limit: usize,
    },
    
    /// List installed packages
    List {
        /// Filter by architecture
        #[clap(arg_enum)]
        architecture: Option<Architecture>,
        
        /// Show package details
        #[clap(short, long)]
        detailed: bool,
        
        /// Sort by name
        #[clap(short, long)]
        sort_by_name: bool,
    },
    
    /// Show package information
    Info {
        #[clap(required = true)]
        package: String,
        
        /// Show dependencies
        #[clap(short, long)]
        dependencies: bool,
        
        /// Show files
        #[clap(short, long)]
        files: bool,
        
        /// Show security information
        #[clap(short, long)]
        security: bool,
    },
    
    /// Verify installed packages
    Verify {
        /// Specific packages to verify (all if not specified)
        packages: Vec<String>,
        
        /// Fix detected issues
        #[clap(short, long)]
        fix: bool,
    },
    
    /// Check for available updates
    CheckUpdates {
        /// Include development packages
        #[clap(short, long)]
        development: bool,
        
        /// Show only security updates
        #[clap(short, long)]
        security_only: bool,
        
        /// JSON output
        #[clap(short, long)]
        json: bool,
    },
    
    /// Synchronize repositories
    Sync {
        /// Force refresh even if recently updated
        #[clap(short, long)]
        force: bool,
        
        /// Specific repositories to sync
        repository: Option<String>,
    },
    
    /// Rollback package to previous version
    Rollback {
        #[clap(required = true)]
        package: String,
        
        #[clap(required = true)]
        version: String,
        
        /// Create backup before rollback
        #[clap(short, long)]
        backup: bool,
    },
    
    /// Show package manager status
    Status {
        /// Show detailed statistics
        #[clap(short, long)]
        detailed: bool,
        
        /// JSON output
        #[clap(short, long)]
        json: bool,
    },
    
    /// Clean up old packages and cache
    Cleanup {
        /// Remove old package versions
        #[clap(short, long)]
        old_versions: bool,
        
        /// Clear package cache
        #[clap(short, long)]
        clear_cache: bool,
        
        /// Remove orphaned dependencies
        #[clap(short, long)]
        orphaned: bool,
        
        /// Preview changes without applying
        #[clap(short, long)]
        dry_run: bool,
    },
    
    /// Add or manage repositories
    Repository {
        #[clap(subcommand)]
        command: RepositoryCommands,
    },
    
    /// Configure automatic updates
    Configure {
        /// Enable automatic update checking
        #[clap(short, long)]
        auto_check: bool,
        
        /// Enable automatic installation
        #[clap(short, long)]
        auto_install: bool,
        
        /// Set check interval (in hours)
        #[clap(short, long)]
        check_interval: Option<u32>,
        
        /// Set maintenance day
        #[clap(short, long)]
        maintenance_day: Option<String>,
        
        /// Set maintenance time
        #[clap(short, long)]
        maintenance_time: Option<String>,
        
        /// Show current configuration
        #[clap(short, long)]
        show: bool,
    },
    
    /// Package signing and verification
    Sign {
        #[clap(subcommand)]
        command: SignCommands,
    },
    
    /// Export package information
    Export {
        /// Output format
        #[clap(arg_enum, short, long, default_value = "json")]
        format: ExportFormat,
        
        /// Include file lists
        #[clap(short, long)]
        include_files: bool,
        
        /// Output file (stdout if not specified)
        #[clap(short, long)]
        output: Option<std::path::PathBuf>,
    },
    
    /// Show help and version information
    Help,
}

#[derive(ArgEnum, Debug, Clone)]
enum Architecture {
    X86_64,
    ARM64,
    RISCV64,
    Universal,
}

#[derive(Subcommand, Debug)]
enum RepositoryCommands {
    /// Add a new repository
    Add {
        /// Repository name
        #[clap(required = true)]
        name: String,
        
        /// Repository URL
        #[clap(required = true)]
        url: String,
        
        /// Repository description
        #[clap(short, long)]
        description: Option<String>,
        
        /// Repository priority
        #[clap(short, long, default_value = "0")]
        priority: i32,
    },
    
    /// Remove a repository
    Remove {
        #[clap(required = true)]
        name: String,
    },
    
    /// List repositories
    List,
    
    /// Update repository
    Update {
        #[clap(required = true)]
        name: String,
        
        /// New URL
        url: Option<String>,
        
        /// New description
        description: Option<String>,
        
        /// New priority
        priority: Option<i32>,
    },
}

#[derive(Subcommand, Debug)]
enum SignCommands {
    /// Generate signing key
    GenerateKey {
        /// Key identifier
        #[clap(required = true)]
        key_id: String,
        
        /// Key algorithm
        #[clap(arg_enum, short, long, default_value = "ed25519")]
        algorithm: SignAlgorithm,
    },
    
    /// Sign a package
    Sign {
        /// Package file
        #[clap(required = true)]
        package: std::path::PathBuf,
        
        /// Key ID
        #[clap(required = true)]
        key_id: String,
    },
    
    /// Verify package signature
    Verify {
        /// Package file
        #[clap(required = true)]
        package: std::path::PathBuf,
        
        /// Public key ID
        #[clap(required = true)]
        key_id: String,
    },
    
    /// Trust a public key
    Trust {
        /// Public key ID
        #[clap(required = true)]
        key_id: String,
    },
    
    /// Revoke a public key
    Revoke {
        /// Public key ID
        #[clap(required = true)]
        key_id: String,
    },
}

#[derive(ArgEnum, Debug, Clone)]
enum SignAlgorithm {
    Ed25519,
    RSA2048,
    RSA4096,
    ECDSAP256,
    ECDSAP384,
}

#[derive(ArgEnum, Debug, Clone)]
enum ExportFormat {
    Json,
    Csv,
    Xml,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }
    
    // Load configuration
    let config = if cli.config.exists() {
        std::fs::read_to_string(&cli.config)
            .map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        String::new()
    };
    
    // Initialize package manager
    let manager = multios_package_manager::MultiOSPackageManager::new(cli.data_dir).await?;
    
    // Execute command
    match cli.command {
        Commands::Install { packages, version, force, repository } => {
            let versions = version.map(|v| v.into_iter().map(|s| s.as_str().to_string()).collect());
            manager.install(packages, versions).await?;
            println!("Successfully installed packages");
        }
        
        Commands::Uninstall { packages, purge, force } => {
            manager.uninstall(packages).await?;
            println!("Successfully uninstalled packages");
        }
        
        Commands::Update { packages, development, security_only } => {
            let results = manager.update(Some(packages)).await?;
            for result in results {
                if result.success {
                    println!("Updated {}: {} -> {}", result.name, result.from_version, result.to_version);
                } else {
                    eprintln!("Failed to update {}", result.name);
                }
            }
        }
        
        Commands::Search { query, description, tags, limit } => {
            let results = manager.search(&query, Some(limit)).await?;
            for result in results {
                println!("{} {} - {} ({})", 
                    result.name, result.version, result.description, result.architecture);
            }
            println!("Found {} results", results.len());
        }
        
        Commands::List { architecture, detailed, sort_by_name } => {
            let mut packages = manager.list_installed().await;
            
            if let Some(arch) = architecture {
                packages.retain(|pkg| {
                    match arch {
                        Architecture::X86_64 => matches!(pkg.architecture, multios_package_manager::types::Architecture::X86_64),
                        Architecture::ARM64 => matches!(pkg.architecture, multios_package_manager::types::Architecture::ARM64),
                        Architecture::RISCV64 => matches!(pkg.architecture, multios_package_manager::types::Architecture::RISCV64),
                        Architecture::Universal => matches!(pkg.architecture, multios_package_manager::types::Architecture::Universal),
                    }
                });
            }
            
            if sort_by_name {
                packages.sort_by(|a, b| a.name.cmp(&b.name));
            }
            
            for pkg in packages {
                if detailed {
                    println!("{} {} - {} ({})\n  Installed: {}  Size: {} bytes", 
                        pkg.name, pkg.version, pkg.description, pkg.architecture, 
                        pkg.install_date, pkg.size);
                } else {
                    println!("{} {}", pkg.name, pkg.version);
                }
            }
        }
        
        Commands::Info { package, dependencies, files, security } => {
            if let Some(info) = manager.info(&package).await? {
                println!("Package: {}", info.name);
                println!("Version: {}", info.version);
                println!("Description: {}", info.description);
                println!("Architecture: {}", info.architecture);
                println!("Status: {:?}", info.status);
                println!("Size: {} bytes", info.size);
                
                if let Some(date) = info.install_date {
                    println!("Install Date: {}", date);
                }
                
                if dependencies && !info.dependencies.is_empty() {
                    println!("\nDependencies:");
                    for dep in info.dependencies {
                        println!("  - {} ({})", dep.name, dep.version_constraint);
                    }
                }
            } else {
                eprintln!("Package not found: {}", package);
                std::process::exit(1);
            }
        }
        
        Commands::Verify { packages, fix } => {
            let results = manager.verify().await?;
            let mut issues_found = false;
            
            for result in results {
                match result.status {
                    multios_package_manager::packages::VerificationStatus::Passed => {
                        println!("✓ {} {} - PASS", result.package_name, result.version);
                    }
                    multios_package_manager::packages::VerificationStatus::Failed => {
                        println!("✗ {} {} - FAIL", result.package_name, result.version);
                        issues_found = true;
                        for issue in result.issues {
                            println!("  - {}", issue);
                        }
                    }
                    multios_package_manager::packages::VerificationStatus::Warning => {
                        println!("⚠ {} {} - WARNING", result.package_name, result.version);
                        issues_found = true;
                    }
                }
            }
            
            if issues_found && fix {
                println!("Auto-fix not yet implemented");
            }
            
            std::process::exit(if issues_found { 1 } else { 0 });
        }
        
        Commands::CheckUpdates { development, security_only, json } => {
            let updates = manager.check_updates().await?;
            
            if json {
                let json_output = serde_json::to_string_pretty(&updates)?;
                println!("{}", json_output);
            } else {
                if updates.is_empty() {
                    println!("No updates available");
                } else {
                    println!("Available updates:");
                    for update in updates {
                        let update_type = if update.security_update {
                            "[SECURITY]"
                        } else {
                            "[UPDATE]"
                        };
                        println!("  {} {}: {} -> {}", update_type, update.package.name, 
                            update.current_version, update.available_version);
                    }
                }
            }
        }
        
        Commands::Sync { force, repository } => {
            if let Some(repo_name) = repository {
                // Sync specific repository
                println!("Syncing repository: {}", repo_name);
            } else {
                manager.sync_repositories().await?;
                println!("All repositories synchronized");
            }
        }
        
        Commands::Rollback { package, version, backup } => {
            manager.rollback(&package, &version).await?;
            println!("Rolled back {} to version {}", package, version);
        }
        
        Commands::Status { detailed, json } => {
            let status = manager.status().await;
            
            if json {
                let json_output = serde_json::to_string_pretty(&status)?;
                println!("{}", json_output);
            } else {
                println!("Package Manager Status:");
                println!("  Repositories: {}", status.repositories);
                println!("  Installed packages: {}", status.installed_packages);
                println!("  Scheduler running: {}", status.scheduler_running);
            }
        }
        
        Commands::Cleanup { old_versions, clear_cache, orphaned, dry_run } => {
            if dry_run {
                println!("Dry run mode - would perform cleanup");
            } else {
                let result = manager.cleanup().await?;
                println!("Cleanup completed:");
                println!("  Packages removed: {}", result.packages_removed);
                println!("  Cache size freed: {} bytes", result.cache_size_freed);
                println!("  Temporary files removed: {}", result.temporary_files_removed);
            }
        }
        
        Commands::Repository { command } => {
            match command {
                RepositoryCommands::Add { name, url, description, priority } => {
                    let repository = multios_package_manager::types::Repository::new(name, url);
                    if let Some(desc) = description {
                        // This would need to be added to the Repository struct
                    }
                    repository.priority = priority;
                    
                    manager.add_repository(repository).await?;
                    println!("Repository added successfully");
                }
                
                RepositoryCommands::Remove { name } => {
                    manager.remove_repository(&name).await?;
                    println!("Repository removed successfully");
                }
                
                RepositoryCommands::List => {
                    let repos = manager.list_repositories().await;
                    for repo in repos {
                        println!("{}", repo);
                    }
                }
                
                RepositoryCommands::Update { name, url, description, priority } => {
                    // This would need implementation
                    println!("Repository update not yet implemented");
                }
            }
        }
        
        Commands::Configure { auto_check, auto_install, check_interval, maintenance_day, maintenance_time, show } => {
            if show {
                println!("Configuration display not yet implemented");
            } else {
                let mut config = multios_package_manager::scheduler::ScheduleConfig::default();
                
                if auto_check {
                    config.auto_check_updates = true;
                }
                if auto_install {
                    config.auto_install_updates = true;
                }
                if let Some(interval) = check_interval {
                    config.check_interval = std::time::Duration::from_secs(interval as u64 * 3600);
                }
                if let Some(day) = maintenance_day {
                    config.maintenance_day = day;
                }
                if let Some(time) = maintenance_time {
                    config.maintenance_time = time;
                }
                
                manager.configure_auto_updates(config).await?;
                println!("Configuration updated successfully");
            }
        }
        
        Commands::Sign { command } => {
            match command {
                SignCommands::GenerateKey { key_id, algorithm } => {
                    println!("Key generation not yet implemented");
                }
                
                SignCommands::Sign { package, key_id } => {
                    println!("Package signing not yet implemented");
                }
                
                SignCommands::Verify { package, key_id } => {
                    println!("Signature verification not yet implemented");
                }
                
                SignCommands::Trust { key_id } => {
                    println!("Key trust management not yet implemented");
                }
                
                SignCommands::Revoke { key_id } => {
                    println!("Key revocation not yet implemented");
                }
            }
        }
        
        Commands::Export { format, include_files, output } => {
            let export_data = manager.export_packages(&format.to_string()).await?;
            
            if let Some(output_path) = output {
                std::fs::write(&output_path, export_data)?;
                println!("Export written to {}", output_path.display());
            } else {
                println!("{}", export_data);
            }
        }
        
        Commands::Help => {
            print_help();
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("MultiOS Package Manager CLI");
    println!();
    println!("Usage: multios-pm <command> [options]");
    println!();
    println!("Commands:");
    println!("  install <packages>     Install packages");
    println!("  uninstall <packages>   Uninstall packages");
    println!("  update [packages]      Update packages");
    println!("  search <query>         Search for packages");
    println!("  list                   List installed packages");
    println!("  info <package>         Show package information");
    println!("  verify [packages]      Verify installed packages");
    println!("  check-updates          Check for available updates");
    println!("  sync [repository]      Synchronize repositories");
    println!("  rollback <pkg> <ver>   Rollback package to version");
    println!("  status                 Show package manager status");
    println!("  cleanup                Clean up old packages");
    println!("  repository <command>   Manage repositories");
    println!("  configure              Configure automatic updates");
    println!("  sign <command>         Package signing operations");
    println!("  export                 Export package information");
    println!();
    println!("For more information on specific commands, run:");
    println!("  multios-pm <command> --help");
}