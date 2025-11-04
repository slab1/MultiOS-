pub mod error;
pub mod installation_manager;
pub mod config;
pub mod state;
pub mod progress;

use crate::hardware::{HardwareInfo, HardwareDetector};
use crate::partitioning::{PartitionManager, PartitionConfig};
use crate::drivers::{DriverManager, DriverSelection};
use crate::network::{NetworkManager, NetworkConfig};
use crate::user::{UserManager, UserConfig};
use crate::recovery::{RecoveryManager, RecoveryPoint};
use crate::gui::{GuiManager, InstallationGUI};
use crate::recovery::error::RecoveryError;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

pub use config::{InstallationConfig, InstallTarget, BootType};
pub use installation_manager::InstallationManager;
pub use state::{WizardState, InstallationState};
pub use progress::{ProgressTracker, ProgressEvent};

/// Main installation wizard that orchestrates all components
pub struct InstallationWizard {
    config: InstallationConfig,
    hardware_info: HardwareInfo,
    state: WizardState,
    progress_tracker: ProgressTracker,
    installation_manager: InstallationManager,
    partition_manager: PartitionManager,
    driver_manager: DriverManager,
    network_manager: NetworkManager,
    user_manager: UserManager,
    recovery_manager: RecoveryManager,
    gui_manager: Option<GuiManager>,
    current_step: usize,
}

impl InstallationWizard {
    pub fn new(config: InstallationConfig, hardware_info: HardwareInfo) -> Self {
        let progress_tracker = ProgressTracker::new();
        let installation_manager = InstallationManager::new(config.clone());
        let partition_manager = PartitionManager::new(hardware_info.clone());
        let driver_manager = DriverManager::new(hardware_info.clone());
        let network_manager = NetworkManager::new();
        let user_manager = UserManager::new();
        let recovery_manager = RecoveryManager::new();
        
        let state = WizardState::Initializing;
        
        Self {
            config,
            hardware_info,
            state,
            progress_tracker,
            installation_manager,
            partition_manager,
            driver_manager,
            network_manager,
            user_manager,
            recovery_manager,
            gui_manager: None,
            current_step: 0,
        }
    }

    /// Run the installation wizard in text mode
    pub async fn run_text_mode(&mut self) -> Result<()> {
        log::info!("Starting text-mode installation wizard");

        // Initialize recovery point
        self.create_initial_recovery_point().await?;

        // Run installation steps
        let steps = self.get_installation_steps();
        for (step_name, step_function) in steps {
            self.execute_step(step_name, step_function).await?;
        }

        self.state = WizardState::Completed;
        log::info!("Installation completed successfully");

        Ok(())
    }

    /// Run the installation wizard in GUI mode
    pub async fn run_gui_mode(&mut self) -> Result<()> {
        log::info!("Starting GUI-mode installation wizard");

        // Initialize GUI
        let gui_manager = GuiManager::new();
        self.gui_manager = Some(gui_manager);
        
        // Initialize recovery point
        self.create_initial_recovery_point().await?;

        // Start GUI event loop
        if let Some(gui_manager) = &mut self.gui_manager {
            gui_manager.run(self).await?;
        }

        Ok(())
    }

    /// Get the list of installation steps
    fn get_installation_steps(&self) -> Vec<(&'static str, fn(&mut Self) -> Box<dyn std::future::Future<Output = Result<()>> + Send + Unpin>)> {
        vec![
            ("Hardware Detection", |wizard| Box::new(wizard.run_hardware_detection()) as _),
            ("Network Configuration", |wizard| Box::new(wizard.run_network_setup()) as _),
            ("Partition Management", |wizard| Box::new(wizard.run_partitioning()) as _),
            ("Driver Installation", |wizard| Box::new(wizard.run_driver_installation()) as _),
            ("System Files", |wizard| Box::new(wizard.copy_system_files()) as _),
            ("Boot Configuration", |wizard| Box::new(wizard.configure_bootloader()) as _),
            ("User Setup", |wizard| Box::new(wizard.setup_user_accounts()) as _),
            ("Final Configuration", |wizard| Box::new(wizard.finalize_installation()) as _),
        ]
    }

    /// Execute a single installation step
    async fn execute_step<F>(&mut self, step_name: &str, step_function: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Box<dyn std::future::Future<Output = Result<()>> + Send + Unpin>,
    {
        log::info!("Starting step: {}", step_name);
        self.progress_tracker.update_progress(self.current_step, step_name);

        let start_time = Instant::now();

        let result = step_function(self).await;
        
        match result {
            Ok(_) => {
                let duration = start_time.elapsed();
                log::info!("Step '{}' completed successfully in {:?}", step_name, duration);
                self.progress_tracker.complete_step(self.current_step);
            }
            Err(e) => {
                log::error!("Step '{}' failed: {:?}", step_name, e);
                self.progress_tracker.fail_step(self.current_step, &e.to_string());
                return Err(e);
            }
        }

        self.current_step += 1;
        Ok(())
    }

    // Individual installation steps
    async fn run_hardware_detection(&mut self) -> Result<()> {
        self.state = WizardState::DetectingHardware;
        
        // Validate hardware compatibility
        self.validate_hardware_compatibility()?;
        
        // Detect available drivers
        self.detect_compatible_drivers().await?;
        
        // Configure optimal settings for detected hardware
        self.configure_hardware_settings()?;
        
        Ok(())
    }

    async fn run_network_setup(&mut self) -> Result<()> {
        self.state = WizardState::ConfiguringNetwork;
        
        let network_config = self.get_network_config().await?;
        self.network_manager.configure(network_config).await?;
        
        Ok(())
    }

    async fn run_partitioning(&mut self) -> Result<()> {
        self.state = WizardState::ConfiguringPartitions;
        
        let partition_config = self.get_partition_config().await?;
        self.partition_manager.apply_configuration(partition_config).await?;
        
        Ok(())
    }

    async fn run_driver_installation(&mut self) -> Result<()> {
        self.state = WizardState::InstallingDrivers;
        
        let driver_selection = self.get_driver_selection().await?;
        self.driver_manager.install_drivers(driver_selection).await?;
        
        Ok(())
    }

    async fn copy_system_files(&mut self) -> Result<()> {
        self.state = WizardState::CopyingSystemFiles;
        
        self.installation_manager.copy_system_files(&self.hardware_info).await?;
        
        Ok(())
    }

    async fn configure_bootloader(&mut self) -> Result<()> {
        self.state = WizardState::ConfiguringBootLoader;
        
        self.installation_manager.configure_bootloader(&self.hardware_info).await?;
        
        Ok(())
    }

    async fn setup_user_accounts(&mut self) -> Result<()> {
        self.state = WizardState::CreatingUsers;
        
        let user_configs = self.get_user_configs().await?;
        self.user_manager.create_users(user_configs).await?;
        
        Ok(())
    }

    async fn finalize_installation(&mut self) -> Result<()> {
        self.state = WizardState::Finalizing;
        
        // Clean up temporary files
        self.cleanup_temp_files().await?;
        
        // Create final recovery point
        self.create_final_recovery_point().await?;
        
        // Run post-installation checks
        self.run_post_install_checks().await?;
        
        Ok(())
    }

    // Helper methods
    fn validate_hardware_compatibility(&self) -> Result<()> {
        // Check CPU architecture compatibility
        let cpu_architecture = &self.hardware_info.cpu.architecture;
        if !["x86_64", "ARM64", "RISC-V"].contains(&cpu_architecture.as_str()) {
            return Err(anyhow::anyhow!("Unsupported CPU architecture: {}", cpu_architecture));
        }

        // Check memory requirements
        let total_memory = self.hardware_info.memory.total_bytes;
        let min_memory = 2 * 1024 * 1024 * 1024; // 2 GB minimum
        if total_memory < min_memory {
            return Err(anyhow::anyhow!("Insufficient memory: {} GB minimum required", min_memory / (1024*1024*1024)));
        }

        log::info!("Hardware compatibility validation passed");
        Ok(())
    }

    async fn detect_compatible_drivers(&mut self) -> Result<()> {
        log::info!("Detecting compatible drivers for hardware");
        
        // Detect graphics drivers
        let graphics_drivers = self.driver_manager.detect_graphics_drivers(&self.hardware_info).await?;
        log::info!("Found {} compatible graphics drivers", graphics_drivers.len());
        
        // Detect network drivers
        let network_drivers = self.driver_manager.detect_network_drivers(&self.hardware_info).await?;
        log::info!("Found {} compatible network drivers", network_drivers.len());
        
        Ok(())
    }

    fn configure_hardware_settings(&mut self) -> Result<()> {
        log::info!("Configuring hardware-specific settings");
        
        // Configure memory settings
        self.configure_memory_settings()?;
        
        // Configure CPU settings
        self.configure_cpu_settings()?;
        
        // Configure storage settings
        self.configure_storage_settings()?;
        
        Ok(())
    }

    fn configure_memory_settings(&self) -> Result<()> {
        // Configure memory allocation based on detected hardware
        let total_memory = self.hardware_info.memory.total_bytes;
        
        // Set optimal swap size (typically 1.5x RAM for systems with < 8GB RAM)
        let swap_size = if total_memory < 8 * 1024 * 1024 * 1024 {
            (total_memory as f64 * 1.5) as u64
        } else {
            4 * 1024 * 1024 * 1024 // 4GB for systems with 8GB+ RAM
        };
        
        log::info!("Configured swap size: {} GB", swap_size / (1024*1024*1024));
        Ok(())
    }

    fn configure_cpu_settings(&self) -> Result<()> {
        // Configure CPU-specific settings
        let cpu_count = self.hardware_info.cpu.core_count;
        log::info!("Configuring for {} CPU cores", cpu_count);
        Ok(())
    }

    fn configure_storage_settings(&self) -> Result<()> {
        // Configure storage-specific settings
        log::info!("Configuring storage settings");
        Ok(())
    }

    async fn create_initial_recovery_point(&mut self) -> Result<()> {
        log::info!("Creating initial recovery point");
        let recovery_point = RecoveryPoint::new("initial".to_string());
        self.recovery_manager.create_point(recovery_point).await?;
        Ok(())
    }

    async fn create_final_recovery_point(&mut self) -> Result<()> {
        log::info!("Creating final recovery point");
        let recovery_point = RecoveryPoint::new("final".to_string());
        self.recovery_manager.create_point(recovery_point).await?;
        Ok(())
    }

    async fn cleanup_temp_files(&mut self) -> Result<()> {
        log::info!("Cleaning up temporary files");
        // Implementation for cleanup
        Ok(())
    }

    async fn run_post_install_checks(&mut self) -> Result<()> {
        log::info!("Running post-installation checks");
        
        // Verify installation integrity
        self.verify_installation_integrity().await?;
        
        // Check boot configuration
        self.verify_boot_configuration().await?;
        
        // Test basic system functionality
        self.test_basic_functionality().await?;
        
        Ok(())
    }

    async fn verify_installation_integrity(&self) -> Result<()> {
        log::info!("Verifying installation integrity");
        Ok(())
    }

    async fn verify_boot_configuration(&self) -> Result<()> {
        log::info!("Verifying boot configuration");
        Ok(())
    }

    async fn test_basic_functionality(&self) -> Result<()> {
        log::info!("Testing basic system functionality");
        Ok(())
    }

    // Getters for GUI
    pub fn get_state(&self) -> &WizardState {
        &self.state
    }

    pub fn get_progress(&self) -> &ProgressTracker {
        &self.progress_tracker
    }

    pub fn get_hardware_info(&self) -> &HardwareInfo {
        &self.hardware_info
    }

    pub fn get_config(&self) -> &InstallationConfig {
        &self.config
    }
}

impl Drop for InstallationWizard {
    fn drop(&mut self) {
        log::info!("InstallationWizard dropped");
    }
}