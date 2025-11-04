//! GUI components for the MultiOS installation wizard

use eframe::egui;
use crate::hardware::HardwareInfo;
use crate::core::progress::ProgressTracker;
use crate::core::state::InstallationState;

/// Progress bar with enhanced styling for installation progress
pub struct EnhancedProgressBar {
    pub progress: f32,
    pub label: String,
    pub color: egui::Color32,
    pub height: f32,
    pub animated: bool,
}

impl EnhancedProgressBar {
    pub fn new(progress: f32, label: String) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label,
            color: egui::Color32::from_rgb(100, 150, 200),
            height: 24.0,
            animated: true,
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Progress label
            ui.label(&self.label);
            
            // Progress bar
            let mut progress_bar = egui::ProgressBar::new(self.progress)
                .desired_width(300.0)
                .show_percentage()
                .animate(self.animated);
            
            if self.animated {
                progress_bar = progress_bar.filled_color(self.color);
            }
            
            progress_bar.ui(ui);
            
            // Percentage label
            ui.label(format!("{:.1}%", self.progress * 100.0));
        });
    }
}

/// Hardware information display component
pub struct HardwareInfoWidget {
    hardware_info: HardwareInfo,
}

impl HardwareInfoWidget {
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self { hardware_info }
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Hardware Information")
            .default_open(true)
            .show(ui, |ui| {
                self.show_cpu_info(ui);
                self.show_memory_info(ui);
                self.show_storage_info(ui);
                self.show_graphics_info(ui);
                self.show_network_info(ui);
            });
    }
    
    fn show_cpu_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("CPU Information")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(format!("Architecture: {}", self.hardware_info.cpu.architecture));
                ui.label(format!("Vendor: {}", self.hardware_info.cpu.vendor));
                ui.label(format!("Model: {}", self.hardware_info.cpu.model));
                ui.label(format!("Cores: {} ({} threads)", 
                    self.hardware_info.cpu.core_count, 
                    self.hardware_info.cpu.thread_count));
                ui.label(format!("Frequency: {} MHz", self.hardware_info.cpu.frequency_mhz));
            });
    }
    
    fn show_memory_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Memory Information")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(format!("Total Memory: {:.1} GB", 
                    self.hardware_info.memory.total_bytes as f64 / 1e9));
                ui.label(format!("Available Memory: {:.1} GB", 
                    self.hardware_info.memory.available_bytes as f64 / 1e9));
                ui.label(format!("Memory Modules: {}", self.hardware_info.memory.module_count));
                ui.label(format!("Memory Type: {}", self.hardware_info.memory.memory_type));
                ui.label(format!("Memory Speed: {} MHz", self.hardware_info.memory.speed_mhz));
                ui.label(format!("ECC Enabled: {}", 
                    if self.hardware_info.memory.ecc_enabled { "Yes" } else { "No" }));
            });
    }
    
    fn show_storage_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Storage Information")
            .default_open(false)
            .show(ui, |ui| {
                for (index, device) in self.hardware_info.storage.devices.iter().enumerate() {
                    ui.collapsing(format!("Device {}: {}", index + 1, device.device_name), |ui| {
                        ui.label(format!("Type: {}", device.device_type));
                        ui.label(format!("Capacity: {:.1} GB", device.capacity as f64 / 1e9));
                        ui.label(format!("Interface: {}", device.interface));
                        ui.label(format!("Model: {}", device.model));
                        ui.label(format!("Rotational: {}", 
                            if device.is_rotational { "Yes" } else { "No (SSD)" }));
                        ui.label(format!("Removable: {}", 
                            if device.is_removable { "Yes" } else { "No" }));
                    });
                }
            });
    }
    
    fn show_graphics_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Graphics Information")
            .default_open(false)
            .show(ui, |ui| {
                for (index, device) in self.hardware_info.graphics.devices.iter().enumerate() {
                    ui.collapsing(format!("GPU {}: {}", index + 1, device.device_name), |ui| {
                        ui.label(format!("Vendor: {}", device.vendor));
                        ui.label(format!("Model: {}", device.model));
                        ui.label(format!("Driver: {}", device.driver));
                        ui.label(format!("Memory: {} MB", device.memory_mb));
                        ui.label(format!("Max Resolution: {}x{}", 
                            device.max_resolution.0, device.max_resolution.1));
                    });
                }
            });
    }
    
    fn show_network_info(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Network Information")
            .default_open(false)
            .show(ui, |ui| {
                for interface in &self.hardware_info.network.devices {
                    ui.collapsing(interface.interface_name.clone(), |ui| {
                        ui.label(format!("Type: {}", interface.device_type));
                        ui.label(format!("MAC Address: {}", interface.mac_address));
                        ui.label(format!("Speed: {} Mbps", interface.speed_mbps));
                        ui.label(format!("Duplex: {}", interface.duplex));
                        ui.label(format!("MTU: {}", interface.mtu));
                        ui.label(format!("State: {}", interface.state));
                        ui.label(format!("Driver: {}", interface.driver));
                    });
                }
            });
    }
}

/// Installation log viewer component
pub struct LogViewerWidget {
    max_lines: usize,
    show_timestamps: bool,
    filter_level: LogLevelFilter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevelFilter {
    All,
    ErrorsAndWarnings,
    ErrorsOnly,
}

impl LogViewerWidget {
    pub fn new() -> Self {
        Self {
            max_lines: 1000,
            show_timestamps: true,
            filter_level: LogLevelFilter::All,
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui, state: &InstallationState) {
        egui::CollapsingHeader::new("Installation Log")
            .default_open(true)
            .show(ui, |ui| {
                self.show_log_controls(ui);
                self.show_log_entries(ui, state);
            });
    }
    
    fn show_log_controls(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            
            let filter_options = [("All", LogLevelFilter::All), 
                                ("Errors & Warnings", LogLevelFilter::ErrorsAndWarnings),
                                ("Errors Only", LogLevelFilter::ErrorsOnly)];
            
            for (label, filter) in &filter_options {
                ui.radio_value(&mut self.filter_level.clone(), filter.clone(), label);
            }
            
            ui.add_space(20.0);
            
            if ui.button("Clear Log").clicked() {
                // Clear the log
            }
            
            if ui.button("Save Log").clicked() {
                // Save log to file
            }
        });
    }
    
    fn show_log_entries(&self, ui: &mut egui::Ui, state: &InstallationState) {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let recent_logs = state.get_recent_logs();
                let filtered_logs = self.filter_logs(recent_logs);
                
                for log_entry in filtered_logs.iter().rev().take(self.max_lines) {
                    self.show_log_entry(ui, log_entry);
                }
            });
    }
    
    fn filter_logs(&self, logs: &[crate::core::state::LogEntry]) -> Vec<&crate::core::state::LogEntry> {
        match self.filter_level {
            LogLevelFilter::All => logs.to_vec(),
            LogLevelFilter::ErrorsAndWarnings => logs.iter()
                .filter(|log| matches!(log.level, 
                    crate::core::state::LogLevel::Error | 
                    crate::core::state::LogLevel::Warning |
                    crate::core::state::LogLevel::Critical))
                .collect(),
            LogLevelFilter::ErrorsOnly => logs.iter()
                .filter(|log| matches!(log.level, 
                    crate::core::state::LogLevel::Error | 
                    crate::core::state::LogLevel::Critical))
                .collect(),
        }
    }
    
    fn show_log_entry(&self, ui: &mut egui::Ui, log_entry: &crate::core::state::LogEntry) {
        let color = match log_entry.level {
            crate::core::state::LogLevel::Debug => egui::Color32::from_gray(128),
            crate::core::state::LogLevel::Info => egui::Color32::from_rgb(0, 128, 0),
            crate::core::state::LogLevel::Warning => egui::Color32::from_rgb(255, 165, 0),
            crate::core::state::LogLevel::Error => egui::Color32::from_rgb(255, 0, 0),
            crate::core::state::LogLevel::Critical => egui::Color32::from_rgb(128, 0, 128),
        };
        
        let text = if self.show_timestamps {
            format!("[{}] {}: {}", 
                log_entry.timestamp.format("%H:%M:%S"),
                log_entry.level,
                log_entry.message)
        } else {
            format!("{}: {}", log_entry.level, log_entry.message)
        };
        
        ui.colored_label(color, text);
    }
}

/// Warning and error display component
pub struct AlertWidget {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl AlertWidget {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        if !self.errors.is_empty() {
            self.show_error_panel(ui);
        }
        
        if !self.warnings.is_empty() {
            self.show_warning_panel(ui);
        }
    }
    
    fn show_error_panel(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Errors")
            .default_open(true)
            .show(ui, |ui| {
                for error in &self.errors {
                    ui.colored_label(egui::Color32::from_rgb(255, 0, 0), 
                        format!("❌ {}", error));
                }
            });
    }
    
    fn show_warning_panel(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Warnings")
            .default_open(true)
            .show(ui, |ui| {
                for warning in &self.warnings {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), 
                        format!("⚠️ {}", warning));
                }
            });
    }
}

/// Multi-step wizard navigation component
pub struct WizardNavigation {
    current_step: usize,
    total_steps: usize,
    step_names: Vec<String>,
    completed_steps: Vec<bool>,
}

impl WizardNavigation {
    pub fn new(step_names: Vec<String>) -> Self {
        let total_steps = step_names.len();
        
        Self {
            current_step: 0,
            total_steps,
            step_names,
            completed_steps: vec![false; total_steps],
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Installation Steps");
            
            for (index, step_name) in self.step_names.iter().enumerate() {
                self.show_step_indicator(ui, index, step_name);
            }
        });
    }
    
    fn show_step_indicator(&self, ui: &mut egui::Ui, index: usize, step_name: &str) {
        let (icon, color) = if index < self.current_step {
            ("✓", egui::Color32::from_rgb(0, 128, 0)) // Completed
        } else if index == self.current_step {
            ("●", egui::Color32::from_rgb(100, 150, 200)) // Current
        } else {
            ("○", egui::Color32::from_gray(128)) // Upcoming
        };
        
        ui.horizontal(|ui| {
            ui.colored_label(color, icon);
            ui.label(step_name);
        });
    }
    
    pub fn next_step(&mut self) {
        if self.current_step < self.total_steps - 1 {
            self.completed_steps[self.current_step] = true;
            self.current_step += 1;
        }
    }
    
    pub fn previous_step(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }
    
    pub fn go_to_step(&mut self, step_index: usize) {
        if step_index < self.total_steps {
            self.current_step = step_index;
        }
    }
}

/// Hardware compatibility checker component
pub struct CompatibilityChecker {
    hardware_info: HardwareInfo,
    issues: Vec<CompatibilityIssue>,
}

#[derive(Debug, Clone)]
pub struct CompatibilityIssue {
    pub severity: IssueSeverity,
    pub component: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl CompatibilityChecker {
    pub fn new(hardware_info: HardwareInfo) -> Self {
        let mut checker = Self {
            hardware_info,
            issues: Vec::new(),
        };
        checker.check_compatibility();
        checker
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Compatibility Check")
            .default_open(true)
            .show(ui, |ui| {
                if self.issues.is_empty() {
                    ui.colored_label(egui::Color32::from_rgb(0, 128, 0), 
                        "✓ No compatibility issues detected");
                } else {
                    self.show_issues(ui);
                }
            });
    }
    
    fn check_compatibility(&mut self) {
        // Check minimum memory requirements
        let min_memory = 2 * 1024 * 1024 * 1024; // 2GB
        if self.hardware_info.memory.total_bytes < min_memory {
            self.issues.push(CompatibilityIssue {
                severity: IssueSeverity::Critical,
                component: "Memory".to_string(),
                message: format!("Insufficient memory: {:.1} GB minimum required", 
                    min_memory as f64 / 1e9),
                recommendation: "Install more RAM before proceeding".to_string(),
            });
        }
        
        // Check CPU architecture compatibility
        let compatible_architectures = ["x86_64", "ARM64", "RISC-V"];
        if !compatible_architectures.contains(&self.hardware_info.cpu.architecture.as_str()) {
            self.issues.push(CompatibilityIssue {
                severity: IssueSeverity::Critical,
                component: "CPU".to_string(),
                message: format!("Unsupported CPU architecture: {}", 
                    self.hardware_info.cpu.architecture),
                recommendation: "MultiOS may not run properly on this architecture".to_string(),
            });
        }
        
        // Check storage space
        let min_storage = 10 * 1024 * 1024 * 1024; // 10GB
        let total_storage = self.hardware_info.storage.total_capacity;
        if total_storage < min_storage {
            self.issues.push(CompatibilityIssue {
                severity: IssueSeverity::Warning,
                component: "Storage".to_string(),
                message: format!("Insufficient storage: {:.1} GB minimum required", 
                    min_storage as f64 / 1e9),
                recommendation: "Free up disk space before proceeding".to_string(),
            });
        }
        
        // Check for graphics driver compatibility
        let gpu_vendor = &self.hardware_info.graphics.gpu_vendor;
        if !gpu_vendor.contains("NVIDIA") && 
           !gpu_vendor.contains("AMD") && 
           !gpu_vendor.contains("Intel") {
            self.issues.push(CompatibilityIssue {
                severity: IssueSeverity::Info,
                component: "Graphics".to_string(),
                message: format!("Unknown graphics vendor: {}", gpu_vendor),
                recommendation: "Use generic drivers or install vendor-specific drivers".to_string(),
            });
        }
    }
    
    fn show_issues(&self, ui: &mut egui::Ui) {
        for issue in &self.issues {
            let color = match issue.severity {
                IssueSeverity::Critical => egui::Color32::from_rgb(255, 0, 0),
                IssueSeverity::Warning => egui::Color32::from_rgb(255, 165, 0),
                IssueSeverity::Info => egui::Color32::from_rgb(0, 0, 255),
            };
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let icon = match issue.severity {
                        IssueSeverity::Critical => "🚫",
                        IssueSeverity::Warning => "⚠️",
                        IssueSeverity::Info => "ℹ️",
                    };
                    ui.label(icon);
                    ui.heading(&issue.component);
                });
                
                ui.colored_label(color, &issue.message);
                ui.label(&issue.recommendation);
            });
            
            ui.add_space(10.0);
        }
    }
}