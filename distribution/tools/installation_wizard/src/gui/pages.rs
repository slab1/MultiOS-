//! GUI pages for the MultiOS installation wizard

use eframe::egui;
use crate::core::config::{InstallationConfig, PartitionConfig, UserConfig};
use crate::hardware::HardwareInfo;
use crate::core::progress::ProgressTracker;
use crate::core::state::InstallationState;
use super::components::*;

/// Welcome page
pub struct WelcomePage {
    show_advanced_options: bool,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            show_advanced_options: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            
            // Welcome message
            ui.heading("Welcome to MultiOS");
            ui.add_space(20.0);
            
            ui.label("This wizard will guide you through installing MultiOS on your computer.");
            ui.label("The installation process includes:");
            
            ui.add_space(10.0);
            ui.label("• Automatic hardware detection and driver installation");
            ui.label("• Flexible disk partitioning options");
            ui.label("• User account creation and system configuration");
            ui.label("• Recovery and rollback capabilities");
            
            ui.add_space(30.0);
            
            // Hardware summary
            self.show_hardware_summary(ui, hardware_info);
            
            ui.add_space(30.0);
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Start Installation").clicked() {
                    // Start installation process
                }
                
                ui.add_space(20.0);
                
                if ui.button("Advanced Options").clicked() {
                    self.show_advanced_options = !self.show_advanced_options;
                }
            });
            
            ui.add_space(20.0);
            
            if self.show_advanced_options {
                self.show_advanced_options_panel(ui);
            }
        });
    }
    
    fn show_hardware_summary(&self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        egui::CollapsingHeader::new("Detected Hardware")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("CPU: {} cores", hardware_info.cpu.core_count));
                ui.label(format!("Memory: {:.1} GB", 
                    hardware_info.memory.total_bytes as f64 / 1e9));
                ui.label(format!("Storage: {:.1} GB", 
                    hardware_info.storage.total_capacity as f64 / 1e9));
                ui.label(format!("Graphics: {}", hardware_info.graphics.gpu_vendor));
            });
    }
    
    fn show_advanced_options_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style(), egui::Stroke::none()).show(ui, |ui| {
            ui.heading("Advanced Options");
            
            ui.checkbox(&mut false, "Skip hardware detection");
            ui.checkbox(&mut false, "Enable debug mode");
            ui.checkbox(&mut false, "Install with minimal features");
            ui.checkbox(&mut false, "Preserve existing data");
            
            ui.add_space(10.0);
            
            if ui.button("Load Configuration").clicked() {
                // Load configuration file
            }
            
            if ui.button("Export Configuration").clicked() {
                // Export configuration template
            }
        });
    }
}

/// Hardware detection page
pub struct HardwareDetectionPage {
    detection_complete: bool,
    detection_progress: f32,
}

impl HardwareDetectionPage {
    pub fn new() -> Self {
        Self {
            detection_complete: false,
            detection_progress: 0.0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, hardware_info: &HardwareInfo, progress_tracker: &ProgressTracker) {
        ui.heading("Hardware Detection");
        ui.add_space(20.0);
        
        if !self.detection_complete {
            self.show_detection_progress(ui, progress_tracker);
        } else {
            self.show_detection_results(ui, hardware_info);
        }
    }
    
    fn show_detection_progress(&self, ui: &mut egui::Ui, progress_tracker: &ProgressTracker) {
        let state = futures::executor::block_on(progress_tracker.get_state());
        
        ui.label("Detecting your system hardware...");
        
        ui.add_space(20.0);
        
        EnhancedProgressBar::new(self.detection_progress, "Detection Progress".to_string()).show(ui);
        
        ui.add_space(20.0);
        
        // Detection stages
        let stages = ["CPU", "Memory", "Storage", "Network", "Graphics", "Audio", "Input"];
        
        for (index, stage) in stages.iter().enumerate() {
            let stage_complete = self.detection_progress >= ((index + 1) as f32) / (stages.len() as f32);
            
            ui.horizontal(|ui| {
                let icon = if stage_complete { "✓" } else { "○" };
                ui.label(icon);
                ui.label(stage);
                
                if stage_complete {
                    ui.colored_label(egui::Color32::from_rgb(0, 128, 0), "Complete");
                } else {
                    ui.colored_label(egui::Color32::from_gray(128), "Pending");
                }
            });
        }
        
        ui.add_space(30.0);
        
        if self.detection_progress >= 1.0 {
            self.detection_complete = true;
        }
    }
    
    fn show_detection_results(&self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        ui.label("Hardware detection completed successfully!");
        
        ui.add_space(20.0);
        
        // Show detailed hardware information
        HardwareInfoWidget::new(hardware_info.clone()).show(ui);
        
        ui.add_space(20.0);
        
        // Compatibility check
        CompatibilityChecker::new(hardware_info.clone()).show(ui);
        
        ui.add_space(30.0);
        
        ui.horizontal(|ui| {
            if ui.button("Redetect Hardware").clicked() {
                // Redetect hardware
            }
            
            ui.add_space(20.0);
            
            if ui.button("Continue").clicked() {
                // Continue to next step
            }
        });
    }
}

/// Network configuration page
pub struct NetworkConfigPage {
    config_method: ConfigMethod,
    static_ip: String,
    netmask: String,
    gateway: String,
    dns_servers: String,
    hostname: String,
    test_result: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigMethod {
    Dhcp,
    Static,
}

impl NetworkConfigPage {
    pub fn new() -> Self {
        Self {
            config_method: ConfigMethod::Dhcp,
            static_ip: "192.168.1.100".to_string(),
            netmask: "255.255.255.0".to_string(),
            gateway: "192.168.1.1".to_string(),
            dns_servers: "8.8.8.8, 8.8.4.4".to_string(),
            hostname: "multios".to_string(),
            test_result: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        ui.heading("Network Configuration");
        ui.add_space(20.0);
        
        ui.label("Configure your network settings:");
        
        ui.add_space(20.0);
        
        // Configuration method
        ui.group(|ui| {
            ui.heading("Configuration Method");
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.config_method, ConfigMethod::Dhcp, "Use DHCP (Recommended)");
                ui.radio_value(&mut self.config_method, ConfigMethod::Static, "Use Static IP");
            });
        });
        
        ui.add_space(20.0);
        
        // Network interfaces
        egui::CollapsingHeader::new("Detected Network Interfaces")
            .default_open(true)
            .show(ui, |ui| {
                for interface in &hardware_info.network.devices {
                    ui.group(|ui| {
                        ui.label(&interface.interface_name);
                        ui.label(format!("Type: {}", interface.device_type));
                        ui.label(format!("Speed: {} Mbps", interface.speed_mbps));
                        ui.label(format!("Driver: {}", interface.driver));
                        ui.label(format!("State: {}", interface.state));
                    });
                    ui.add_space(10.0);
                }
            });
        
        ui.add_space(20.0);
        
        // Configuration form
        match self.config_method {
            ConfigMethod::Dhcp => self.show_dhcp_config(ui),
            ConfigMethod::Static => self.show_static_config(ui),
        }
        
        ui.add_space(20.0);
        
        // Network testing
        if ui.button("Test Connection").clicked() {
            // Test network connectivity
            self.test_result = Some("Testing network connectivity...".to_string());
        }
        
        if let Some(result) = &self.test_result {
            ui.colored_label(
                if result.contains("success") { 
                    egui::Color32::from_rgb(0, 128, 0) 
                } else { 
                    egui::Color32::from_rgb(255, 0, 0) 
                }, 
                result
            );
        }
        
        ui.add_space(30.0);
        
        ui.horizontal(|ui| {
            if ui.button("Continue").clicked() {
                // Save configuration and continue
            }
        });
    }
    
    fn show_dhcp_config(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("DHCP Configuration")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Hostname:");
                    ui.text_edit_singleline(&mut self.hostname);
                });
                
                ui.label("Your network will be configured automatically using DHCP.");
                ui.label("Network settings will be obtained from your router.");
            });
    }
    
    fn show_static_config(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Static IP Configuration")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("IP Address:");
                    ui.text_edit_singleline(&mut self.static_ip);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Netmask:");
                    ui.text_edit_singleline(&mut self.netmask);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Gateway:");
                    ui.text_edit_singleline(&mut self.gateway);
                });
                
                ui.horizontal(|ui| {
                    ui.label("DNS Servers:");
                    ui.text_edit_singleline(&mut self.dns_servers);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Hostname:");
                    ui.text_edit_singleline(&mut self.hostname);
                });
            });
    }
}

/// Partitioning page
pub struct PartitioningPage {
    partitioning_mode: PartitioningMode,
    custom_config: PartitionConfig,
    detected_layouts: Vec<DetectedLayout>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartitioningMode {
    Guided,
    Manual,
    UseFreeSpace,
}

#[derive(Debug, Clone)]
pub struct DetectedLayout {
    device_name: String,
    total_size: u64,
    partitions: Vec<DetectedPartition>,
    has_os: bool,
    os_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedPartition {
    size: u64,
    filesystem: String,
    mount_point: Option<String>,
}

impl PartitioningPage {
    pub fn new() -> Self {
        Self {
            partitioning_mode: PartitioningMode::Guided,
            custom_config: PartitionConfig {
                root_size: 20 * 1024 * 1024 * 1024,
                home_size: 0,
                swap_size: 4 * 1024 * 1024 * 1024,
                boot_size: 512 * 1024 * 1024,
                use_lvm: false,
                encryption: false,
                encryption_password: None,
            },
            detected_layouts: Vec::new(),
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        ui.heading("Disk Partitioning");
        ui.add_space(20.0);
        
        ui.label("Choose how to partition your disk:");
        
        ui.add_space(20.0);
        
        // Partitioning mode selection
        self.show_partitioning_mode_selection(ui);
        
        ui.add_space(20.0);
        
        // Partition layout display
        self.show_partition_layout(ui, hardware_info);
        
        ui.add_space(20.0);
        
        // Custom configuration
        if self.partitioning_mode == PartitioningMode::Manual {
            self.show_custom_configuration(ui);
        }
        
        ui.add_space(30.0);
        
        // Actions
        ui.horizontal(|ui| {
            if ui.button("Apply Changes").clicked() {
                // Apply partitioning changes
            }
            
            if ui.button("Continue").clicked() {
                // Continue to next step
            }
        });
    }
    
    fn show_partitioning_mode_selection(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Partitioning Method")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.partitioning_mode, PartitioningMode::Guided, "Guided - Use entire disk");
                    ui.radio_value(&mut self.partitioning_mode, PartitioningMode::UseFreeSpace, "Guided - Use free space");
                    ui.radio_value(&mut self.partitioning_mode, PartitioningMode::Manual, "Manual - Configure partitions yourself");
                });
            });
    }
    
    fn show_partition_layout(&self, ui: &mut egui::Ui, hardware_info: &HardwareInfo) {
        egui::CollapsingHeader::new("Current Disk Layout")
            .default_open(true)
            .show(ui, |ui| {
                for device in &hardware_info.storage.devices {
                    ui.group(|ui| {
                        ui.heading(&device.device_name);
                        ui.label(format!("Type: {}", device.device_type));
                        ui.label(format!("Size: {:.1} GB", device.capacity as f64 / 1e9));
                        ui.label(format!("Interface: {}", device.interface));
                    });
                    ui.add_space(10.0);
                }
            });
        
        // Show recommended layout
        egui::CollapsingHeader::new("Recommended Partition Layout")
            .default_open(true)
            .show(ui, |ui| {
                let layout = self.get_recommended_layout();
                
                for (index, partition) in layout.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Partition {}:", index + 1));
                        ui.label(&partition.0);
                        ui.label(format!("{} GB", partition.1 / (1024 * 1024 * 1024)));
                    });
                }
            });
    }
    
    fn get_recommended_layout(&self) -> Vec<(String, u64)> {
        match self.partitioning_mode {
            PartitioningMode::Guided | PartitioningMode::UseFreeSpace => {
                vec![
                    ("Boot (FAT32)".to_string(), 512 * 1024 * 1024),
                    ("Root (ext4)".to_string(), self.custom_config.root_size),
                    ("Home (ext4)".to_string(), self.custom_config.home_size),
                    ("Swap".to_string(), self.custom_config.swap_size),
                ]
            }
            PartitioningMode::Manual => {
                vec![
                    ("Boot (FAT32)".to_string(), self.custom_config.boot_size),
                    ("Root (ext4)".to_string(), self.custom_config.root_size),
                ]
            }
        }
    }
    
    fn show_custom_configuration(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Custom Partition Configuration")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Root partition size (GB):");
                    ui.add(egui::DragValue::new(&mut self.custom_config.root_size)
                        .speed(1.0)
                        .clamp_range(5..=1000));
                });
                
                if self.custom_config.home_size > 0 {
                    ui.horizontal(|ui| {
                        ui.label("Home partition size (GB):");
                        ui.add(egui::DragValue::new(&mut self.custom_config.home_size)
                            .speed(1.0)
                            .clamp_range(0..=10000));
                    });
                }
                
                ui.horizontal(|ui| {
                    ui.label("Swap size (GB):");
                    ui.add(egui::DragValue::new(&mut self.custom_config.swap_size)
                        .speed(0.5)
                        .clamp_range(0..=64));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Boot partition size (MB):");
                    ui.add(egui::DragValue::new(&mut self.custom_config.boot_size)
                        .speed(64.0)
                        .clamp_range(256..=2048));
                });
                
                ui.add_space(10.0);
                
                ui.checkbox(&mut self.custom_config.use_lvm, "Use LVM");
                ui.checkbox(&mut self.custom_config.encryption, "Encrypt partitions");
                
                if self.custom_config.encryption {
                    ui.horizontal(|ui| {
                        ui.label("Encryption password:");
                        ui.text_edit_singleline(&mut self.custom_config.encryption_password);
                    });
                }
            });
    }
}

/// User accounts page
pub struct UserAccountsPage {
    admin_user: UserConfig,
    additional_users: Vec<UserConfig>,
    system_settings: SystemSettings,
    show_add_user: bool,
}

#[derive(Debug, Clone)]
pub struct SystemSettings {
    hostname: String,
    timezone: String,
    keyboard_layout: String,
    locale: String,
    enable_auto_login: bool,
    enable_home_encryption: bool,
}

impl UserAccountsPage {
    pub fn new() -> Self {
        Self {
            admin_user: UserConfig {
                username: "user".to_string(),
                full_name: Some("User".to_string()),
                password: None,
                is_admin: true,
                auto_login: false,
            },
            additional_users: Vec::new(),
            system_settings: SystemSettings {
                hostname: "multios".to_string(),
                timezone: "UTC".to_string(),
                keyboard_layout: "us".to_string(),
                locale: "en_US.UTF-8".to_string(),
                enable_auto_login: false,
                enable_home_encryption: false,
            },
            show_add_user: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("User Accounts and System Settings");
        ui.add_space(20.0);
        
        // Administrator account
        self.show_admin_account(ui);
        
        ui.add_space(20.0);
        
        // Additional users
        self.show_additional_users(ui);
        
        ui.add_space(20.0);
        
        // System settings
        self.show_system_settings(ui);
        
        ui.add_space(30.0);
        
        if ui.button("Continue").clicked() {
            // Validate and continue
        }
    }
    
    fn show_admin_account(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Administrator Account")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.admin_user.username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Full Name:");
                    ui.text_edit_singleline(&mut self.admin_user.full_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut self.admin_user.password);
                });
                
                ui.checkbox(&mut self.admin_user.auto_login, "Enable auto-login for this user");
            });
    }
    
    fn show_additional_users(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Additional Users")
            .default_open(false)
            .show(ui, |ui| {
                if ui.button("Add User").clicked() {
                    self.show_add_user = true;
                }
                
                if self.show_add_user {
                    self.show_add_user_dialog(ui);
                }
                
                for (index, user) in self.additional_users.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&user.username);
                            if user.full_name.is_some() {
                                ui.label(format!("({})", user.full_name.as_ref().unwrap()));
                            }
                        });
                        
                        if ui.button("Remove").clicked() {
                            self.additional_users.remove(index);
                        }
                    });
                    ui.add_space(5.0);
                }
            });
    }
    
    fn show_add_user_dialog(&mut self, ui: &mut egui::Ui) {
        let mut temp_user = UserConfig {
            username: "".to_string(),
            full_name: None,
            password: None,
            is_admin: false,
            auto_login: false,
        };
        
        egui::Window::new("Add User")
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut temp_user.username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Full Name:");
                    ui.text_edit_singleline(&mut temp_user.full_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut temp_user.password);
                });
                
                ui.checkbox(&mut temp_user.is_admin, "Administrator privileges");
                ui.checkbox(&mut temp_user.auto_login, "Auto-login");
                
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        if !temp_user.username.is_empty() {
                            self.additional_users.push(temp_user.clone());
                            self.show_add_user = false;
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_add_user = false;
                    }
                });
            });
    }
    
    fn show_system_settings(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("System Settings")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Computer name:");
                    ui.text_edit_singleline(&mut self.system_settings.hostname);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Timezone:");
                    ui.text_edit_singleline(&mut self.system_settings.timezone);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Keyboard layout:");
                    ui.text_edit_singleline(&mut self.system_settings.keyboard_layout);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Locale:");
                    ui.text_edit_singleline(&mut self.system_settings.locale);
                });
                
                ui.add_space(10.0);
                
                ui.checkbox(&mut self.system_settings.enable_auto_login, "Enable auto-login");
                ui.checkbox(&mut self.system_settings.enable_home_encryption, "Encrypt home directories");
            });
    }
}

/// Installation progress page
pub struct InstallationPage {
    state: InstallationState,
}

impl InstallationPage {
    pub fn new(progress_tracker: &ProgressTracker) -> Self {
        let state = futures::executor::block_on(progress_tracker.get_state());
        Self { state }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Installing MultiOS");
        ui.add_space(20.0);
        
        // Progress indicators
        self.show_progress(ui);
        
        ui.add_space(20.0);
        
        // Log viewer
        LogViewerWidget::new().show(ui, &self.state);
        
        ui.add_space(20.0);
        
        // Controls
        self.show_controls(ui);
    }
    
    fn show_progress(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Installation Progress");
            
            ui.add_space(10.0);
            
            EnhancedProgressBar::new(
                self.state.overall_progress, 
                "Overall Progress".to_string()
            ).show(ui);
            
            ui.add_space(10.0);
            
            ui.label(format!("Current Step: {}", self.state.current_step_name));
            
            if let Some(time_remaining) = self.state.estimated_time_remaining {
                ui.label(format!("Time Remaining: {:?}", time_remaining));
            }
            
            ui.add_space(20.0);
            
            // Step-by-step progress
            self.show_step_progress(ui);
        });
    }
    
    fn show_step_progress(&self, ui: &mut egui::Ui) {
        ui.label("Step Progress:");
        
        let steps = self.get_installation_steps();
        for (index, step) in steps.iter().enumerate() {
            let is_current = index == self.state.current_step;
            let is_completed = index < self.state.current_step;
            
            ui.horizontal(|ui| {
                let icon = if is_completed { "✓" } else if is_current { "●" } else { "○" };
                let color = if is_completed {
                    egui::Color32::from_rgb(0, 128, 0)
                } else if is_current {
                    egui::Color32::from_rgb(100, 150, 200)
                } else {
                    egui::Color32::from_gray(128)
                };
                
                ui.colored_label(color, icon);
                ui.label(step);
            });
        }
    }
    
    fn show_controls(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Pause").clicked() {
                // Pause installation
            }
            
            if ui.button("Rollback").clicked() {
                // Rollback changes
            }
            
            if ui.button("Cancel").clicked() {
                // Cancel installation
            }
        });
    }
    
    fn get_installation_steps(&self) -> Vec<&'static str> {
        vec![
            "Hardware Detection",
            "Network Configuration", 
            "Disk Partitioning",
            "Driver Installation",
            "System Files Copy",
            "Bootloader Configuration",
            "User Account Creation",
            "Final Configuration"
        ]
    }
}

/// Completion page
pub struct CompletionPage {
    restart_countdown: Option<u32>,
}

impl CompletionPage {
    pub fn new() -> Self {
        Self {
            restart_countdown: Some(30), // 30 seconds countdown
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.heading("🎉 Installation Complete!");
            ui.add_space(20.0);
            
            ui.label("MultiOS has been successfully installed on your computer.");
            ui.label("The system is ready to use.");
            
            ui.add_space(30.0);
            
            // Installation summary
            egui::CollapsingHeader::new("Installation Summary")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label("✓ System files installed");
                    ui.label("✓ Bootloader configured");
                    ui.label("✓ User accounts created");
                    ui.label("✓ Drivers installed");
                    ui.label("✓ Network configured");
                });
            
            ui.add_space(30.0);
            
            // Restart options
            if let Some(countdown) = self.restart_countdown {
                ui.label(format!("Your computer will restart in {} seconds...", countdown));
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Restart Now").clicked() {
                        self.restart_system();
                    }
                    
                    if ui.button("Restart Later").clicked() {
                        self.restart_countdown = None;
                    }
                });
            } else {
                if ui.button("Restart Computer").clicked() {
                    self.restart_system();
                }
            }
            
            ui.add_space(50.0);
            
            ui.label("Thank you for choosing MultiOS!");
            ui.label("Enjoy your new operating system!");
        });
    }
    
    fn restart_system(&self) {
        // Implement system restart
        log::info!("Restarting system...");
    }
}