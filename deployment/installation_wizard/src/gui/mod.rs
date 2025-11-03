pub mod components;
pub mod pages;

use crate::core::{InstallationWizard, WizardState, InstallationConfig};
use crate::hardware::HardwareInfo;
use crate::core::progress::{ProgressEvent, ProgressTracker};

use eframe::egui;
use tokio::sync::mpsc;

/// GUI manager for the installation wizard
pub struct GuiManager {
    current_page: GuiPage,
    installation_wizard: Option<InstallationWizard>,
}

impl GuiManager {
    pub fn new() -> Self {
        Self {
            current_page: GuiPage::Welcome,
            installation_wizard: None,
        }
    }

    /// Run the GUI
    pub async fn run(&mut self, wizard: &mut InstallationWizard) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.installation_wizard = Some(wizard);
        
        // Create the native window
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1200, 800)),
            min_window_size: Some(egui::vec2(800, 600)),
            resizable: true,
            ..Default::default()
        };

        // Run the GUI
        eframe::run_native(
            "MultiOS Installation Wizard",
            options,
            Box::new(|_cc| Box::new(GuiApp::new(self))),
        );

        Ok(())
    }

    /// Set the current page
    pub fn set_current_page(&mut self, page: GuiPage) {
        self.current_page = page;
    }

    /// Get the current page
    pub fn get_current_page(&self) -> GuiPage {
        self.current_page.clone()
    }
}

/// Main GUI application
pub struct GuiApp {
    gui_manager: GuiManager,
    installation_config: InstallationConfig,
    hardware_info: HardwareInfo,
    progress_tracker: ProgressTracker,
    event_receiver: Option<mpsc::UnboundedReceiver<ProgressEvent>>,
}

impl GuiApp {
    fn new(gui_manager: &mut GuiManager) -> Self {
        let wizard = gui_manager.installation_wizard.as_ref().unwrap();
        
        Self {
            gui_manager: gui_manager.clone(),
            installation_config: wizard.get_config().clone(),
            hardware_info: wizard.get_hardware_info().clone(),
            progress_tracker: wizard.get_progress().clone(),
            event_receiver: Some(wizard.get_progress().get_event_receiver()),
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_style(ctx);
        
        // Handle progress events
        if let Some(ref mut receiver) = self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                self.handle_progress_event(event);
            }
        }
        
        // Main window layout
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_header(ui);
            
            match self.gui_manager.get_current_page() {
                GuiPage::Welcome => self.show_welcome_page(ui),
                GuiPage::HardwareDetection => self.show_hardware_detection_page(ui),
                GuiPage::NetworkConfig => self.show_network_config_page(ui),
                GuiPage::Partitioning => self.show_partitioning_page(ui),
                GuiPage::UserAccounts => self.show_user_accounts_page(ui),
                GuiPage::DriverSelection => self.show_driver_selection_page(ui),
                GuiPage::Installation => self.show_installation_page(ui),
                GuiPage::Complete => self.show_complete_page(ui),
                GuiPage::Error => self.show_error_page(ui),
            }
        });
    }
}

impl GuiApp {
    /// Setup the GUI style
    fn setup_style(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Custom styling for MultiOS
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(240, 240, 240);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(100, 150, 200);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(200, 220, 240);
        
        ctx.set_style(style);
    }

    /// Show the header section
    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            // MultiOS logo and title
            ui.add_space(10.0);
            ui.heading("MultiOS Installation Wizard");
            ui.add_space(10.0);
            
            // Progress indicator
            ui.separator();
            self.show_progress_indicator(ui);
        });
        
        ui.separator();
    }

    /// Show progress indicator
    fn show_progress_indicator(&self, ui: &mut egui::Ui) {
        let state = futures::executor::block_on(self.progress_tracker.get_state());
        let progress = state.overall_progress;
        
        ui.label(format!("Progress: {:.0}%", progress * 100.0));
        
        // Progress bar
        egui::ProgressBar::new(progress)
            .animate(true)
            .desired_width(200.0)
            .show_percentage()
            .ui(ui);
    }

    /// Show welcome page
    fn show_welcome_page(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.heading("Welcome to MultiOS");
            ui.add_space(20.0);
            
            ui.label("This wizard will guide you through the installation process.");
            ui.label("The installation will detect your hardware and configure MultiOS optimally.");
            
            ui.add_space(30.0);
            
            if ui.button("Start Installation").clicked() {
                // Move to next page
            }
            
            ui.add_space(20.0);
            
            if ui.button("Advanced Options").clicked() {
                // Show advanced options
            }
            
            ui.add_space(50.0);
            self.show_system_info(ui);
        });
    }

    /// Show hardware detection page
    fn show_hardware_detection_page(&self, ui: &mut egui::Ui) {
        ui.heading("Hardware Detection");
        ui.add_space(20.0);
        
        ui.label("Detecting your system hardware...");
        
        ui.add_space(20.0);
        
        // Hardware summary
        egui::CollapsingHeader::new("System Information").default_open(true).show(ui, |ui| {
            ui.label(format!("CPU: {} {} ({} cores)", 
                self.hardware_info.cpu.vendor,
                self.hardware_info.cpu.model,
                self.hardware_info.cpu.core_count));
                
            ui.label(format!("Memory: {:.1} GB", 
                self.hardware_info.memory.total_bytes as f64 / 1e9));
                
            ui.label(format!("Storage: {} devices", 
                self.hardware_info.storage.devices.len()));
                
            ui.label(format!("Graphics: {} {}", 
                self.hardware_info.graphics.gpu_vendor,
                self.hardware_info.graphics.gpu_model));
        });
        
        ui.add_space(20.0);
        
        if ui.button("Continue").clicked() {
            // Move to next page
        }
    }

    /// Show network configuration page
    fn show_network_config_page(&self, ui: &mut egui::Ui) {
        ui.heading("Network Configuration");
        ui.add_space(20.0);
        
        ui.label("Configure your network settings:");
        
        ui.add_space(10.0);
        
        egui::CollapsingHeader::new("Current Network Interfaces").default_open(true).show(ui, |ui| {
            for interface in &self.hardware_info.network.devices {
                ui.label(format!("{}: {} ({} Mbps)", 
                    interface.interface_name,
                    interface.device_type,
                    interface.speed_mbps));
            }
        });
        
        ui.add_space(20.0);
        
        // Network configuration options
        ui.horizontal(|ui| {
            ui.label("Configuration:");
            ui.radio_value(&mut true, "DHCP", "Use DHCP");
            ui.radio_value(&mut false, "Static IP", "Use Static IP");
        });
        
        ui.add_space(20.0);
        
        if ui.button("Test Connection").clicked() {
            // Test network connectivity
        }
        
        ui.add_space(20.0);
        
        if ui.button("Continue").clicked() {
            // Move to next page
        }
    }

    /// Show partitioning page
    fn show_partitioning_page(&self, ui: &mut egui::Ui) {
        ui.heading("Disk Partitioning");
        ui.add_space(20.0);
        
        ui.label("Choose how to partition your disk:");
        
        ui.add_space(10.0);
        
        // Partitioning options
        egui::CollapsingHeader::new("Partitioning Options").default_open(true).show(ui, |ui| {
            ui.radio_value(&mut true, "Guided", "Guided - Use entire disk");
            ui.radio_value(&mut false, "Manual", "Manual - Configure partitions yourself");
        });
        
        ui.add_space(20.0);
        
        // Partition summary
        egui::CollapsingHeader::new("Partition Layout").default_open(true).show(ui, |ui| {
            ui.label("Root partition: 20 GB (ext4)");
            ui.label("Home partition: 50 GB (ext4)");
            ui.label("Swap partition: 4 GB (swap)");
            ui.label("Boot partition: 512 MB (FAT32)");
        });
        
        ui.add_space(20.0);
        
        if ui.button("Apply Changes").clicked() {
            // Apply partitioning changes
        }
        
        ui.add_space(10.0);
        
        if ui.button("Continue").clicked() {
            // Move to next page
        }
    }

    /// Show user accounts page
    fn show_user_accounts_page(&self, ui: &mut egui::Ui) {
        ui.heading("User Accounts");
        ui.add_space(20.0);
        
        ui.label("Create user accounts for this system:");
        
        ui.add_space(10.0);
        
        // User creation form
        ui.group(|ui| {
            ui.heading("Administrator Account");
            
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut self.installation_config.username);
            });
            
            ui.horizontal(|ui| {
                ui.label("Full Name:");
                ui.text_edit_singleline(&mut self.installation_config.full_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.text_edit_singleline(&mut self.installation_config.password);
            });
            
            ui.checkbox(&mut self.installation_config.auto_login, "Auto-login");
            ui.checkbox(&mut self.installation_config.password.is_some(), "Use password");
        });
        
        ui.add_space(20.0);
        
        // System settings
        ui.group(|ui| {
            ui.heading("System Settings");
            
            ui.horizontal(|ui| {
                ui.label("Hostname:");
                ui.text_edit_singleline(&mut self.installation_config.hostname);
            });
            
            ui.horizontal(|ui| {
                ui.label("Timezone:");
                ui.text_edit_singleline(&mut self.installation_config.timezone);
            });
            
            ui.horizontal(|ui| {
                ui.label("Keyboard Layout:");
                ui.text_edit_singleline(&mut self.installation_config.keyboard_layout);
            });
        });
        
        ui.add_space(20.0);
        
        if ui.button("Continue").clicked() {
            // Move to next page
        }
    }

    /// Show driver selection page
    fn show_driver_selection_page(&self, ui: &mut egui::Ui) {
        ui.heading("Driver Selection");
        ui.add_space(20.0);
        
        ui.label("Choose drivers for your hardware:");
        
        ui.add_space(10.0);
        
        // Graphics drivers
        egui::CollapsingHeader::new("Graphics Drivers").default_open(true).show(ui, |ui| {
            ui.label(format!("Detected Graphics: {} {}", 
                self.hardware_info.graphics.gpu_vendor,
                self.hardware_info.graphics.gpu_model));
            
            ui.radio_value(&mut true, "Automatic", "Use recommended driver");
            ui.radio_value(&mut false, "Manual", "Select specific driver");
        });
        
        ui.add_space(10.0);
        
        // Network drivers
        egui::CollapsingHeader::new("Network Drivers").default_open(true).show(ui, |ui| {
            for interface in &self.hardware_info.network.devices {
                ui.label(format!("{}: {} ({})", 
                    interface.interface_name,
                    interface.device_type,
                    interface.driver));
            }
        });
        
        ui.add_space(20.0);
        
        if ui.button("Install Drivers").clicked() {
            // Install selected drivers
        }
        
        ui.add_space(10.0);
        
        if ui.button("Continue").clicked() {
            // Move to next page
        }
    }

    /// Show installation page
    fn show_installation_page(&self, ui: &mut egui::Ui) {
        ui.heading("Installing MultiOS");
        ui.add_space(20.0);
        
        let state = futures::executor::block_on(self.progress_tracker.get_state());
        
        // Installation progress
        ui.group(|ui| {
            ui.heading("Installation Progress");
            
            // Overall progress
            ui.horizontal(|ui| {
                ui.label("Overall Progress:");
                egui::ProgressBar::new(state.overall_progress)
                    .desired_width(300.0)
                    .show_percentage()
                    .ui(ui);
                ui.label(format!("{:.1}%", state.overall_progress * 100.0));
            });
            
            ui.add_space(10.0);
            
            // Current step progress
            ui.horizontal(|ui| {
                ui.label("Current Step:");
                egui::ProgressBar::new(state.step_progress)
                    .desired_width(300.0)
                    .show_percentage()
                    .ui(ui);
                ui.label(format!("{:.1}%", state.step_progress * 100.0));
            });
            
            ui.add_space(10.0);
            
            // Current step info
            ui.label(&state.current_step_name);
            
            ui.add_space(10.0);
            
            // Estimated time remaining
            if let Some(time_remaining) = state.estimated_time_remaining {
                ui.label(format!("Estimated time remaining: {:?}", time_remaining));
            }
        });
        
        ui.add_space(20.0);
        
        // Installation log
        egui::CollapsingHeader::new("Installation Log").default_open(true).show(ui, |ui| {
            let recent_logs = state.get_recent_logs();
            
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for log_entry in recent_logs {
                    ui.label(format!("[{}] {}: {}", 
                        log_entry.timestamp.format("%H:%M:%S"),
                        log_entry.level,
                        log_entry.message));
                }
            });
        });
        
        ui.add_space(20.0);
        
        // Installation controls
        if ui.button("Pause Installation").clicked() {
            // Pause installation
        }
        
        if ui.button("Cancel Installation").clicked() {
            // Cancel installation
        }
    }

    /// Show completion page
    fn show_complete_page(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.heading("Installation Complete!");
            ui.add_space(20.0);
            
            ui.label("MultiOS has been successfully installed on your system.");
            ui.label("Your computer will restart in a few seconds.");
            
            ui.add_space(30.0);
            
            if ui.button("Restart Now").clicked() {
                // Restart system
            }
            
            if ui.button("Restart Later").clicked() {
                // Don't restart
            }
            
            ui.add_space(50.0);
            
            ui.label("Thank you for choosing MultiOS!");
        });
    }

    /// Show error page
    fn show_error_page(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.heading("Installation Failed");
            ui.add_space(20.0);
            
            ui.label("An error occurred during installation.");
            ui.label("Please check the error details below and try again.");
            
            ui.add_space(30.0);
            
            if ui.button("Retry Installation").clicked() {
                // Retry installation
            }
            
            if ui.button("Rollback Changes").clicked() {
                // Rollback changes
            }
            
            ui.add_space(20.0);
            
            if ui.button("Quit").clicked() {
                // Quit installer
            }
        });
    }

    /// Show system information summary
    fn show_system_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Detected System Information").show(ui, |ui| {
            ui.label(format!("CPU: {} cores", self.hardware_info.cpu.core_count));
            ui.label(format!("Memory: {:.1} GB", 
                self.hardware_info.memory.total_bytes as f64 / 1e9));
            ui.label(format!("Architecture: {}", self.hardware_info.cpu.architecture));
        });
    }

    /// Handle progress events
    fn handle_progress_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::Started { total_steps, .. } => {
                // Handle installation start
            }
            ProgressEvent::StepStarted { step_index, step_name } => {
                // Handle step start
            }
            ProgressEvent::StepProgress { step_index, progress, message } => {
                // Handle step progress
            }
            ProgressEvent::StepCompleted { step_index, step_name, .. } => {
                // Handle step completion
            }
            ProgressEvent::Failed { error, .. } => {
                // Handle failure
            }
            ProgressEvent::Completed { .. } => {
                // Handle completion
            }
            _ => {
                // Handle other events
            }
        }
    }
}

/// GUI pages enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum GuiPage {
    Welcome,
    HardwareDetection,
    NetworkConfig,
    Partitioning,
    UserAccounts,
    DriverSelection,
    Installation,
    Complete,
    Error,
}

impl Default for GuiPage {
    fn default() -> Self {
        GuiPage::Welcome
    }
}