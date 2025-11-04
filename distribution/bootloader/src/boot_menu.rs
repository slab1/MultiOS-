//! Boot Menu Implementation
//! 
//! Provides interactive boot menu functionality allowing users to select
//! boot options, configure boot parameters, and choose between different boot modes.

use crate::{BootConfig, BootMode, BootError};
use core::fmt;
use spin::Mutex;

/// Boot menu entry
#[derive(Debug, Clone)]
pub struct BootMenuEntry {
    pub id: u8,
    pub label: &'static str,
    pub description: &'static str,
    pub config: BootConfig,
    pub is_default: bool,
    pub is_recovery: bool,
}

/// Boot menu state
#[derive(Debug, Clone)]
pub struct BootMenuState {
    pub entries: Vec<BootMenuEntry>,
    pub selected_entry: usize,
    pub timeout_seconds: u8,
    pub auto_boot: bool,
}

/// Boot menu errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMenuError {
    InvalidEntry,
    TimeoutExpired,
    UserCancelled,
    ConfigurationError,
}

/// Boot menu configuration
#[derive(Debug, Clone)]
pub struct BootMenuConfig {
    pub timeout_seconds: u8,
    pub enable_recovery_mode: bool,
    pub enable_debug_mode: bool,
    pub enable_normal_mode: bool,
    pub default_boot_mode: BootMenuSelection,
}

/// Boot menu selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMenuSelection {
    Normal,
    Recovery,
    Debug,
    Custom(String),
}

/// Global boot menu state
static BOOT_MENU_STATE: Mutex<Option<BootMenuState>> = Mutex::new(None);

impl BootMenuEntry {
    /// Create a new boot menu entry
    pub fn new(
        id: u8,
        label: &'static str,
        description: &'static str,
        config: BootConfig,
        is_default: bool,
        is_recovery: bool,
    ) -> Self {
        Self {
            id,
            label,
            description,
            config,
            is_default,
            is_recovery,
        }
    }

    /// Check if this entry is the default boot option
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Check if this is a recovery mode entry
    pub fn is_recovery_mode(&self) -> bool {
        self.is_recovery
    }

    /// Get the boot configuration for this entry
    pub fn config(&self) -> &BootConfig {
        &self.config
    }
}

impl fmt::Display for BootMenuEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.is_default { "*" } else { " " };
        let recovery_marker = if self.is_recovery { " [RECOVERY]" } else { "" };
        write!(
            f,
            "{}{}. {}{} - {}",
            marker, self.id, self.label, recovery_marker, self.description
        )
    }
}

impl BootMenuState {
    /// Create a new boot menu state
    pub fn new(entries: Vec<BootMenuEntry>, timeout_seconds: u8, auto_boot: bool) -> Self {
        let selected_entry = entries.iter().position(|e| e.is_default).unwrap_or(0);
        
        Self {
            entries,
            selected_entry,
            timeout_seconds,
            auto_boot,
        }
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&BootMenuEntry> {
        self.entries.get(self.selected_entry)
    }

    /// Select the next entry
    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_entry = (self.selected_entry + 1) % self.entries.len();
        }
    }

    /// Select the previous entry
    pub fn select_previous(&mut self) {
        if !self.entries.is_empty() {
            self.selected_entry = if self.selected_entry == 0 {
                self.entries.len() - 1
            } else {
                self.selected_entry - 1
            };
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &[BootMenuEntry] {
        &self.entries
    }

    /// Get timeout configuration
    pub fn timeout_seconds(&self) -> u8 {
        self.timeout_seconds
    }

    /// Check if auto boot is enabled
    pub fn auto_boot(&self) -> bool {
        self.auto_boot
    }

    /// Find entry by ID
    pub fn find_entry_by_id(&self, id: u8) -> Option<&BootMenuEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

impl BootMenuConfig {
    /// Create default boot menu configuration
    pub fn default() -> Self {
        Self {
            timeout_seconds: 10,
            enable_recovery_mode: true,
            enable_debug_mode: true,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Normal,
        }
    }

    /// Create educational lab configuration
    pub fn for_educational_lab() -> Self {
        Self {
            timeout_seconds: 30,
            enable_recovery_mode: true,
            enable_debug_mode: true,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Debug,
        }
    }

    /// Create minimal configuration for embedded systems
    pub fn for_embedded() -> Self {
        Self {
            timeout_seconds: 3,
            enable_recovery_mode: false,
            enable_debug_mode: false,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Normal,
        }
    }
}

/// Initialize the boot menu with default entries
pub fn init_boot_menu(config: BootMenuConfig) -> Result<(), BootMenuError> {
    let mut entries = Vec::new();

    if config.enable_normal_mode {
        entries.push(BootMenuEntry::new(
            1,
            "MultiOS Normal",
            "Standard MultiOS boot with default settings",
            BootConfig {
                mode: BootMode::UEFI,
                kernel_path: "/boot/multios/kernel",
                initrd_path: None,
                command_line: Some("quiet loglevel=3"),
                memory_test: false,
                serial_console: true,
            },
            matches!(config.default_boot_mode, BootMenuSelection::Normal),
            false,
        ));
    }

    if config.enable_debug_mode {
        entries.push(BootMenuEntry::new(
            2,
            "MultiOS Debug",
            "MultiOS boot with debug information enabled",
            BootConfig {
                mode: BootMode::UEFI,
                kernel_path: "/boot/multios/kernel",
                initrd_path: None,
                command_line: Some("debug loglevel=8 console=ttyS0"),
                memory_test: false,
                serial_console: true,
            },
            matches!(config.default_boot_mode, BootMenuSelection::Debug),
            false,
        ));
    }

    if config.enable_recovery_mode {
        entries.push(BootMenuEntry::new(
            3,
            "MultiOS Recovery",
            "Recovery mode for system repair and maintenance",
            BootConfig {
                mode: BootMode::UEFI,
                kernel_path: "/boot/multios/recovery",
                initrd_path: Some("/boot/multios/recovery/initrd"),
                command_line: Some("init=/bin/bash single"),
                memory_test: true,
                serial_console: true,
            },
            matches!(config.default_boot_mode, BootMenuSelection::Recovery),
            true,
        ));
    }

    // Add safe mode option
    entries.push(BootMenuEntry::new(
        4,
        "Safe Mode",
        "Boot with minimal drivers and services",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: None,
            command_line: Some("safe_mode no_drivers no_services"),
            memory_test: true,
            serial_console: true,
        },
        false,
        false,
    ));

    // Add memory test option
    entries.push(BootMenuEntry::new(
        5,
        "Memory Test",
        "Comprehensive memory testing before boot",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: None,
            command_line: Some("memtest only"),
            memory_test: true,
            serial_console: true,
        },
        false,
        false,
    ));

    // Add firmware setup option
    entries.push(BootMenuEntry::new(
        6,
            "Firmware Setup",
            "Enter UEFI/BIOS firmware setup",
            BootConfig {
                mode: BootMode::UEFI,
                kernel_path: "/boot/multios/kernel",
                initrd_path: None,
                command_line: Some("firmware_setup"),
                memory_test: false,
                serial_console: true,
            },
            false,
            false,
    ));

    let menu_state = BootMenuState::new(entries, config.timeout_seconds, true);
    
    let mut menu_state_guard = BOOT_MENU_STATE.lock();
    *menu_state_guard = Some(menu_state);
    
    Ok(())
}

/// Display the boot menu
pub fn display_boot_menu() -> Result<BootMenuEntry, BootMenuError> {
    let menu_state_guard = BOOT_MENU_STATE.lock();
    let menu_state = menu_state_guard.as_ref()
        .ok_or(BootMenuError::ConfigurationError)?;
    
    println!("\n=== MultiOS Boot Menu ===");
    println!("Use arrow keys or number keys to select boot option.");
    println!("Auto-boot in {} seconds... (Press any key to stop timer)\n", 
             menu_state.timeout_seconds);
    
    // Display all entries
    for (index, entry) in menu_state.entries().iter().enumerate() {
        let marker = if index == menu_state.selected_entry { ">" } else { " " };
        println!("{} {}", marker, entry);
    }
    
    println!("\nDefault boot option marked with *");
    
    Ok(menu_state.selected_entry().unwrap().clone())
}

/// Get the default boot entry (without displaying menu)
pub fn get_default_boot_entry() -> Result<BootMenuEntry, BootMenuError> {
    let menu_state_guard = BOOT_MENU_STATE.lock();
    let menu_state = menu_state_guard.as_ref()
        .ok_or(BootMenuError::ConfigurationError)?;
    
    menu_state.selected_entry()
        .cloned()
        .ok_or(BootMenuError::InvalidEntry)
}

/// Select a boot entry by ID
pub fn select_boot_entry(entry_id: u8) -> Result<BootMenuEntry, BootMenuError> {
    let mut menu_state_guard = BOOT_MENU_STATE.lock();
    let menu_state = menu_state_guard.as_mut()
        .ok_or(BootMenuError::ConfigurationError)?;
    
    if let Some(entry) = menu_state.find_entry_by_id(entry_id) {
        Ok(entry.clone())
    } else {
        Err(BootMenuError::InvalidEntry)
    }
}

/// Handle keyboard input for boot menu navigation
pub fn handle_menu_input(input: char) -> Result<Option<BootMenuEntry>, BootMenuError> {
    let mut menu_state_guard = BOOT_MENU_STATE.lock();
    let menu_state = menu_state_guard.as_mut()
        .ok_or(BootMenuError::ConfigurationError)?;
    
    match input {
        '1'..='9' => {
            let id = input.to_digit(10).unwrap() as u8;
            if let Some(entry) = menu_state.find_entry_by_id(id) {
                return Ok(Some(entry.clone()));
            }
        }
        'q' | 'Q' => return Err(BootMenuError::UserCancelled),
        '\x1B' => { // Escape sequence (arrow keys)
            // In a real implementation, we'd read more bytes for arrow keys
            // For now, treat as navigation
        }
        _ => {
            // Handle other keys as needed
        }
    }
    
    Ok(None)
}

/// Get boot menu configuration
pub fn get_boot_menu_config() -> Result<BootMenuConfig, BootMenuError> {
    let menu_state_guard = BOOT_MENU_STATE.lock();
    let menu_state = menu_state_guard.as_ref()
        .ok_or(BootMenuError::ConfigurationError)?;
    
    Ok(BootMenuConfig {
        timeout_seconds: menu_state.timeout_seconds(),
        enable_recovery_mode: menu_state.entries().iter().any(|e| e.is_recovery_mode()),
        enable_debug_mode: true, // Could be determined from entries
        enable_normal_mode: true, // Could be determined from entries
        default_boot_mode: BootMenuSelection::Normal, // Could be determined from entries
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_menu_entry_creation() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
        };

        let entry = BootMenuEntry::new(1, "Test Entry", "Test Description", config, true, false);
        
        assert_eq!(entry.id, 1);
        assert_eq!(entry.label, "Test Entry");
        assert_eq!(entry.description, "Test Description");
        assert!(entry.is_default());
        assert!(!entry.is_recovery_mode());
    }

    #[test]
    fn test_boot_menu_state_creation() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
        };

        let entries = vec![
            BootMenuEntry::new(1, "Entry 1", "Description 1", config.clone(), true, false),
            BootMenuEntry::new(2, "Entry 2", "Description 2", config.clone(), false, true),
        ];

        let state = BootMenuState::new(entries, 10, true);
        
        assert_eq!(state.entries().len(), 2);
        assert_eq!(state.selected_entry(), 0); // Should select the default entry
        assert_eq!(state.timeout_seconds(), 10);
        assert!(state.auto_boot());
    }

    #[test]
    fn test_boot_menu_config_defaults() {
        let config = BootMenuConfig::default();
        
        assert_eq!(config.timeout_seconds, 10);
        assert!(config.enable_recovery_mode);
        assert!(config.enable_debug_mode);
        assert!(config.enable_normal_mode);
        assert_eq!(config.default_boot_mode, BootMenuSelection::Normal);
    }

    #[test]
    fn test_find_entry_by_id() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
        };

        let entries = vec![
            BootMenuEntry::new(1, "Entry 1", "Description 1", config.clone(), true, false),
            BootMenuEntry::new(2, "Entry 2", "Description 2", config.clone(), false, true),
        ];

        let state = BootMenuState::new(entries, 10, true);
        
        let entry1 = state.find_entry_by_id(1).unwrap();
        assert_eq!(entry1.label, "Entry 1");
        
        let entry2 = state.find_entry_by_id(2).unwrap();
        assert_eq!(entry2.label, "Entry 2");
        
        assert!(state.find_entry_by_id(99).is_none());
    }
}