use serde::{Deserialize, Serialize};
use std::fmt;

/// Current state of the installation wizard
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardState {
    /// Initial state - wizard just started
    Initializing,
    
    /// Hardware detection phase
    DetectingHardware,
    
    /// Network configuration phase
    ConfiguringNetwork,
    
    /// Partition configuration phase
    ConfiguringPartitions,
    
    /// Driver installation phase
    InstallingDrivers,
    
    /// System files copying phase
    CopyingSystemFiles,
    
    /// Bootloader configuration phase
    ConfiguringBootLoader,
    
    /// User account creation phase
    CreatingUsers,
    
    /// Finalization phase
    Finalizing,
    
    /// Completed successfully
    Completed,
    
    /// Failed with error
    Failed {
        error: String,
        can_retry: bool,
    },
    
    /// Cancelled by user
    Cancelled,
    
    /// Rollback in progress
    RollingBack {
        recovery_point: String,
    },
    
    /// Waiting for user input
    WaitingForUser,
    
    /// Rebooting system
    Rebooting,
}

impl fmt::Display for WizardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WizardState::Initializing => write!(f, "Initializing"),
            WizardState::DetectingHardware => write!(f, "Detecting Hardware"),
            WizardState::ConfiguringNetwork => write!(f, "Configuring Network"),
            WizardState::ConfiguringPartitions => write!(f, "Configuring Partitions"),
            WizardState::InstallingDrivers => write!(f, "Installing Drivers"),
            WizardState::CopyingSystemFiles => write!(f, "Copying System Files"),
            WizardState::ConfiguringBootLoader => write!(f, "Configuring Boot Loader"),
            WizardState::CreatingUsers => write!(f, "Creating Users"),
            WizardState::Finalizing => write!(f, "Finalizing Installation"),
            WizardState::Completed => write!(f, "Installation Completed"),
            WizardState::Failed { error, .. } => write!(f, "Installation Failed: {}", error),
            WizardState::Cancelled => write!(f, "Installation Cancelled"),
            WizardState::RollingBack { recovery_point } => write!(f, "Rolling Back: {}", recovery_point),
            WizardState::WaitingForUser => write!(f, "Waiting for User Input"),
            WizardState::Rebooting => write!(f, "Rebooting System"),
        }
    }
}

/// State of the actual installation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationState {
    /// Current installation step
    pub current_step: usize,
    
    /// Total number of steps
    pub total_steps: usize,
    
    /// Current step name
    pub current_step_name: String,
    
    /// Overall progress (0.0 to 1.0)
    pub overall_progress: f32,
    
    /// Current step progress (0.0 to 1.0)
    pub step_progress: f32,
    
    /// Wizard state
    pub wizard_state: WizardState,
    
    /// Start time of installation
    pub start_time: Option<std::time::SystemTime>,
    
    /// Estimated time remaining
    pub estimated_time_remaining: Option<std::time::Duration>,
    
    /// Current operation details
    pub current_operation: Option<String>,
    
    /// Installation log entries
    pub log_entries: Vec<LogEntry>,
    
    /// Installation warnings
    pub warnings: Vec<String>,
    
    /// Installation errors
    pub errors: Vec<String>,
    
    /// Whether installation can be rolled back
    pub can_rollback: bool,
    
    /// Whether recovery is available
    pub recovery_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl Default for InstallationState {
    fn default() -> Self {
        Self {
            current_step: 0,
            total_steps: 0,
            current_step_name: String::new(),
            overall_progress: 0.0,
            step_progress: 0.0,
            wizard_state: WizardState::Initializing,
            start_time: None,
            estimated_time_remaining: None,
            current_operation: None,
            log_entries: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            can_rollback: false,
            recovery_available: false,
        }
    }
}

impl InstallationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate overall progress based on current step and step progress
    pub fn calculate_overall_progress(&mut self) {
        if self.total_steps > 0 {
            let step_progress = (self.current_step as f32) / (self.total_steps as f32);
            let current_step_fraction = self.step_progress / (self.total_steps as f32);
            self.overall_progress = (step_progress + current_step_fraction).min(1.0);
        } else {
            self.overall_progress = 0.0;
        }
    }

    /// Update estimated time remaining
    pub fn update_time_estimate(&mut self) {
        if let Some(start_time) = self.start_time {
            if let Ok(elapsed) = start_time.elapsed() {
                if self.overall_progress > 0.01 {
                    let total_estimated = elapsed.as_secs_f32() / self.overall_progress;
                    let remaining = total_estimated - elapsed.as_secs_f32();
                    self.estimated_time_remaining = Some(std::time::Duration::from_secs_f32(remaining.max(0.0)));
                }
            }
        }
    }

    /// Add a log entry
    pub fn add_log(&mut self, level: LogLevel, message: String, component: String) {
        self.log_entries.push(LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
            component,
        });

        // Keep only the last 1000 log entries to prevent memory issues
        if self.log_entries.len() > 1000 {
            self.log_entries.remove(0);
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        self.add_log(LogLevel::Warning, warning.clone(), "state".to_string());
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error.clone());
        self.add_log(LogLevel::Error, error.clone(), "state".to_string());
    }

    /// Clear errors and warnings
    pub fn clear_warnings_and_errors(&mut self) {
        self.warnings.clear();
        self.errors.clear();
    }

    /// Check if installation is in progress
    pub fn is_in_progress(&self) -> bool {
        match self.wizard_state {
            WizardState::Initializing
            | WizardState::DetectingHardware
            | WizardState::ConfiguringNetwork
            | WizardState::ConfiguringPartitions
            | WizardState::InstallingDrivers
            | WizardState::CopyingSystemFiles
            | WizardState::ConfiguringBootLoader
            | WizardState::CreatingUsers
            | WizardState::Finalizing => true,
            _ => false,
        }
    }

    /// Check if installation is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.wizard_state, WizardState::Completed)
    }

    /// Check if installation failed
    pub fn is_failed(&self) -> bool {
        matches!(self.wizard_state, WizardState::Failed { .. })
    }

    /// Check if installation can be retried
    pub fn can_retry(&self) -> bool {
        match &self.wizard_state {
            WizardState::Failed { can_retry, .. } => *can_retry,
            WizardState::Cancelled => true,
            _ => false,
        }
    }

    /// Get the current status summary
    pub fn get_status_summary(&self) -> String {
        format!(
            "Step {}/{}: {} ({:.1}% complete)",
            self.current_step + 1,
            self.total_steps,
            self.current_step_name,
            self.overall_progress * 100.0
        )
    }

    /// Get recent log entries (last 50)
    pub fn get_recent_logs(&self) -> &[LogEntry] {
        let start = if self.log_entries.len() > 50 {
            self.log_entries.len() - 50
        } else {
            0
        };
        &self.log_entries[start..]
    }

    /// Export state as JSON for external tools
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import state from JSON
    pub fn import_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl WizardState {
    /// Check if this state indicates active work
    pub fn is_active(&self) -> bool {
        match self {
            WizardState::Initializing | WizardState::DetectingHardware | 
            WizardState::ConfiguringNetwork | WizardState::ConfiguringPartitions |
            WizardState::InstallingDrivers | WizardState::CopyingSystemFiles |
            WizardState::ConfiguringBootLoader | WizardState::CreatingUsers |
            WizardState::Finalizing | WizardState::RollingBack { .. } |
            WizardState::Rebooting => true,
            _ => false,
        }
    }

    /// Check if this state allows user interaction
    pub fn allows_user_interaction(&self) -> bool {
        matches!(self, WizardState::Initializing | WizardState::WaitingForUser | WizardState::Completed | WizardState::Failed { .. } | WizardState::Cancelled)
    }

    /// Transition to a new state
    pub fn transition_to(&self, new_state: WizardState) -> Result<(), &'static str> {
        // Validate state transitions
        match (self, &new_state) {
            // Valid transitions
            (WizardState::Initializing, WizardState::DetectingHardware) => Ok(()),
            (WizardState::DetectingHardware, WizardState::ConfiguringNetwork) => Ok(()),
            (WizardState::ConfiguringNetwork, WizardState::ConfiguringPartitions) => Ok(()),
            (WizardState::ConfiguringPartitions, WizardState::InstallingDrivers) => Ok(()),
            (WizardState::InstallingDrivers, WizardState::CopyingSystemFiles) => Ok(()),
            (WizardState::CopyingSystemFiles, WizardState::ConfiguringBootLoader) => Ok(()),
            (WizardState::ConfiguringBootLoader, WizardState::CreatingUsers) => Ok(()),
            (WizardState::CreatingUsers, WizardState::Finalizing) => Ok(()),
            (WizardState::Finalizing, WizardState::Completed) => Ok(()),
            
            // Rollback transitions
            (state, WizardState::RollingBack { .. }) if state.is_active() => Ok(()),
            (WizardState::RollingBack { .. }, WizardState::Initializing) => Ok(()),
            
            // User-initiated transitions
            (_, WizardState::WaitingForUser) => Ok(()),
            (_, WizardState::Cancelled) => Ok(()),
            (WizardState::Failed { .. }, WizardState::Initializing) => Ok(()),
            
            // Invalid transitions
            _ => Err("Invalid state transition"),
        }
    }
}