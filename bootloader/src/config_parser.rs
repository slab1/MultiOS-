//! Boot Configuration File Parser
//! 
//! Parses boot configuration files (similar to GRUB.cfg or systemd-boot loader.conf)
//! to support flexible boot configurations, boot parameters, and multi-boot setups.

use crate::{BootConfig, BootMode, BootError};
use core::fmt;

/// Configuration file format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Grub2,
    SystemdBoot,
    Custom,
    Json,
}

/// Boot configuration entry
#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub title: String,
    pub linux: Option<String>,
    pub initrd: Option<String>,
    pub options: Vec<String>,
    pub fallback_options: Vec<String>,
    pub timeout: Option<u32>,
    pub serial_console: bool,
    pub debug_mode: bool,
    pub recovery_mode: bool,
}

/// Parsed boot configuration
#[derive(Debug, Clone)]
pub struct ParsedBootConfig {
    pub default_entry: Option<String>,
    pub timeout: u32,
    pub entries: Vec<ConfigEntry>,
    pub global_options: Vec<String>,
    pub format: ConfigFormat,
}

/// Configuration parser errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigParseError {
    InvalidFormat,
    ParseError,
    EntryNotFound,
    InvalidValue,
    FileNotFound,
}

/// Boot parameter types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootParameter {
    Console(String),
    Root(String),
    Init(String),
    Loglevel(u8),
    Debug,
    Quiet,
    Single,
    Rescue,
    Memtest,
    NoDrivers,
    NoServices,
    Custom(String),
}

/// Global boot configuration state
static BOOT_CONFIG: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl ConfigEntry {
    /// Create a new configuration entry
    pub fn new(title: String) -> Self {
        Self {
            title,
            linux: None,
            initrd: None,
            options: Vec::new(),
            fallback_options: Vec::new(),
            timeout: None,
            serial_console: false,
            debug_mode: false,
            recovery_mode: false,
        }
    }

    /// Set the kernel path
    pub fn linux<T: Into<String>>(mut self, path: T) -> Self {
        self.linux = Some(path.into());
        self
    }

    /// Set the initrd path
    pub fn initrd<T: Into<String>>(mut self, path: T) -> Self {
        self.initrd = Some(path.into());
        self
    }

    /// Add boot options
    pub fn options<T: Into<String>>(mut self, opts: &[T]) -> Self {
        for opt in opts {
            self.options.push(opt.into().into());
        }
        self
    }

    /// Add a single boot option
    pub fn option<T: Into<String>>(mut self, option: T) -> Self {
        self.options.push(option.into());
        self
    }

    /// Add fallback options
    pub fn fallback_options<T: Into<String>>(mut self, opts: &[T]) -> Self {
        for opt in opts {
            self.fallback_options.push(opt.into().into());
        }
        self
    }

    /// Set timeout for this entry
    pub fn timeout(mut self, seconds: u32) -> Self {
        self.timeout = Some(seconds);
        self
    }

    /// Enable serial console
    pub fn serial_console(mut self, enabled: bool) -> Self {
        self.serial_console = enabled;
        self
    }

    /// Enable debug mode
    pub fn debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    /// Mark as recovery mode
    pub fn recovery_mode(mut self, enabled: bool) -> Self {
        self.recovery_mode = enabled;
        self
    }

    /// Convert to BootConfig
    pub fn to_boot_config(&self) -> Result<BootConfig, ConfigParseError> {
        if let Some(ref kernel_path) = self.linux {
            let command_line = if !self.options.is_empty() {
                Some(self.options.join(" "))
            } else {
                None
            };

            Ok(BootConfig {
                mode: BootMode::UEFI, // Default for now
                kernel_path: Box::leak(kernel_path.clone().into_boxed_str()) as &str,
                initrd_path: self.initrd.as_ref().map(|s| Box::leak(s.clone().into_boxed_str()) as &str),
                command_line: command_line.as_ref().map(|s| Box::leak(s.clone().into_boxed_str()) as &str),
                memory_test: self.options.iter().any(|opt| opt.contains("memtest")),
                serial_console: self.serial_console,
            })
        } else {
            Err(ConfigParseError::InvalidValue)
        }
    }
}

impl ParsedBootConfig {
    /// Create a new parsed boot configuration
    pub fn new(format: ConfigFormat) -> Self {
        Self {
            default_entry: None,
            timeout: 10,
            entries: Vec::new(),
            global_options: Vec::new(),
            format,
        }
    }

    /// Set the default entry
    pub fn default_entry<T: Into<String>>(mut self, entry: T) -> Self {
        self.default_entry = Some(entry.into());
        self
    }

    /// Set the global timeout
    pub fn timeout(mut self, seconds: u32) -> Self {
        self.timeout = seconds;
        self
    }

    /// Add global options
    pub fn global_options<T: Into<String>>(mut self, opts: &[T]) -> Self {
        for opt in opts {
            self.global_options.push(opt.into().into());
        }
        self
    }

    /// Add a boot entry
    pub fn entry(mut self, entry: ConfigEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Find an entry by title
    pub fn find_entry(&self, title: &str) -> Option<&ConfigEntry> {
        self.entries.iter().find(|e| e.title == title)
    }

    /// Get the default entry
    pub fn get_default_entry(&self) -> Option<&ConfigEntry> {
        if let Some(ref default_title) = self.default_entry {
            self.find_entry(default_title)
        } else {
            self.entries.first()
        }
    }

    /// Convert to boot menu entries
    pub fn to_boot_menu_entries(&self) -> Result<Vec<crate::boot_menu::BootMenuEntry>, ConfigParseError> {
        let mut menu_entries = Vec::new();
        
        for (index, entry) in self.entries.iter().enumerate() {
            let boot_config = entry.to_boot_config()?;
            let is_default = self.get_default_entry().map_or(index == 0, |e| e.title == entry.title);
            
            menu_entries.push(crate::boot_menu::BootMenuEntry::new(
                (index + 1) as u8,
                &entry.title,
                &format!("Boot {}", entry.title),
                boot_config,
                is_default,
                entry.recovery_mode,
            ));
        }
        
        Ok(menu_entries)
    }
}

impl fmt::Display for ConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "title {}", self.title)?;
        
        if let Some(ref linux) = self.linux {
            writeln!(f, "  linux {}", linux)?;
        }
        
        if let Some(ref initrd) = self.initrd {
            writeln!(f, "  initrd {}", initrd)?;
        }
        
        if !self.options.is_empty() {
            writeln!(f, "  options {}", self.options.join(" "))?;
        }
        
        if !self.fallback_options.is_empty() {
            writeln!(f, "  fallback_options {}", self.fallback_options.join(" "))?;
        }
        
        if let Some(timeout) = self.timeout {
            writeln!(f, "  timeout {}", timeout)?;
        }
        
        if self.serial_console {
            writeln!(f, "  serial_console")?;
        }
        
        if self.debug_mode {
            writeln!(f, "  debug_mode")?;
        }
        
        if self.recovery_mode {
            writeln!(f, "  recovery_mode")?;
        }
        
        Ok(())
    }
}

/// Parse boot configuration from string content
pub fn parse_config_content(content: &str, format: ConfigFormat) -> Result<ParsedBootConfig, ConfigParseError> {
    let mut config = ParsedBootConfig::new(format);
    let mut current_entry: Option<ConfigEntry> = None;
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "timeout" => {
                if parts.len() >= 2 {
                    if let Ok(timeout) = parts[1].parse() {
                        config.timeout = timeout;
                    } else {
                        return Err(ConfigParseError::InvalidValue);
                    }
                }
            }
            "default" => {
                if parts.len() >= 2 {
                    config.default_entry = Some(parts[1].to_string());
                }
            }
            "title" => {
                // Save previous entry if exists
                if let Some(entry) = current_entry.take() {
                    config.entries.push(entry);
                }
                
                if parts.len() >= 2 {
                    current_entry = Some(ConfigEntry::new(parts[1].to_string()));
                }
            }
            "linux" => {
                if let Some(ref mut entry) = current_entry {
                    if parts.len() >= 2 {
                        entry.linux = Some(parts[1].to_string());
                    }
                }
            }
            "initrd" => {
                if let Some(ref mut entry) = current_entry {
                    if parts.len() >= 2 {
                        entry.initrd = Some(parts[1].to_string());
                    }
                }
            }
            "options" => {
                if let Some(ref mut entry) = current_entry {
                    if parts.len() >= 2 {
                        entry.options.push(parts[1..].join(" "));
                    }
                }
            }
            "fallback_options" => {
                if let Some(ref mut entry) = current_entry {
                    if parts.len() >= 2 {
                        entry.fallback_options.push(parts[1..].join(" "));
                    }
                }
            }
            "timeout" => {
                if let Some(ref mut entry) = current_entry {
                    if parts.len() >= 2 {
                        if let Ok(timeout) = parts[1].parse() {
                            entry.timeout = Some(timeout);
                        }
                    }
                }
            }
            "serial_console" => {
                if let Some(ref mut entry) = current_entry {
                    entry.serial_console = true;
                }
            }
            "debug_mode" => {
                if let Some(ref mut entry) = current_entry {
                    entry.debug_mode = true;
                }
            }
            "recovery_mode" => {
                if let Some(ref mut entry) = current_entry {
                    entry.recovery_mode = true;
                }
            }
            _ => {
                // Unknown directive
                return Err(ConfigParseError::InvalidFormat);
            }
        }
    }
    
    // Add the last entry
    if let Some(entry) = current_entry {
        config.entries.push(entry);
    }
    
    if config.entries.is_empty() {
        return Err(ConfigParseError::ParseError);
    }
    
    Ok(config)
}

/// Parse boot configuration from file path
pub fn parse_config_file(path: &str, format: ConfigFormat) -> Result<ParsedBootConfig, ConfigParseError> {
    // In a real implementation, this would read from the file system
    // For now, return a sample configuration
    let sample_config = match format {
        ConfigFormat::Grub2 => get_sample_grub2_config(),
        ConfigFormat::SystemdBoot => get_sample_systemd_boot_config(),
        ConfigFormat::Custom => get_sample_custom_config(),
        ConfigFormat::Json => get_sample_json_config(),
    };
    
    parse_config_content(&sample_config, format)
}

/// Parse command line parameters
pub fn parse_boot_parameters(params: &str) -> Vec<BootParameter> {
    let mut parameters = Vec::new();
    
    for param in params.split_whitespace() {
        let param = param.trim();
        
        if param.is_empty() {
            continue;
        }
        
        if param == "debug" {
            parameters.push(BootParameter::Debug);
        } else if param == "quiet" {
            parameters.push(BootParameter::Quiet);
        } else if param == "single" {
            parameters.push(BootParameter::Single);
        } else if param == "rescue" {
            parameters.push(BootParameter::Rescue);
        } else if param == "memtest" {
            parameters.push(BootParameter::Memtest);
        } else if param.starts_with("console=") {
            parameters.push(BootParameter::Console(param.to_string()));
        } else if param.starts_with("root=") {
            parameters.push(BootParameter::Root(param.to_string()));
        } else if param.starts_with("init=") {
            parameters.push(BootParameter::Init(param.to_string()));
        } else if param.starts_with("loglevel=") {
            if let Ok(level) = param[9..].parse() {
                parameters.push(BootParameter::Loglevel(level));
            }
        } else if param == "no_drivers" {
            parameters.push(BootParameter::NoDrivers);
        } else if param == "no_services" {
            parameters.push(BootParameter::NoServices);
        } else {
            parameters.push(BootParameter::Custom(param.to_string()));
        }
    }
    
    parameters
}

/// Convert boot parameters back to string
pub fn parameters_to_string(params: &[BootParameter]) -> String {
    params.iter()
        .map(|p| match p {
            BootParameter::Console(console) => console.clone(),
            BootParameter::Root(root) => root.clone(),
            BootParameter::Init(init) => init.clone(),
            BootParameter::Loglevel(level) => format!("loglevel={}", level),
            BootParameter::Debug => "debug".to_string(),
            BootParameter::Quiet => "quiet".to_string(),
            BootParameter::Single => "single".to_string(),
            BootParameter::Rescue => "rescue".to_string(),
            BootParameter::Memtest => "memtest".to_string(),
            BootParameter::NoDrivers => "no_drivers".to_string(),
            BootParameter::NoServices => "no_services".to_string(),
            BootParameter::Custom(custom) => custom.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sample GRUB2 configuration
fn get_sample_grub2_config() -> String {
    r#"timeout=10
default=MultiOS

title MultiOS Normal
  linux /boot/multios/kernel
  options quiet loglevel=3 console=ttyS0

title MultiOS Debug  
  linux /boot/multios/kernel
  options debug loglevel=8 console=ttyS0

title MultiOS Recovery
  linux /boot/multios/recovery
  initrd /boot/multios/recovery/initrd
  options init=/bin/bash single
  recovery_mode

title Memory Test
  linux /boot/multios/kernel
  options memtest
"#.to_string()
}

/// Sample systemd-boot configuration
fn get_sample_systemd_boot_config() -> String {
    r#"timeout 10
default multios-normal

title MultiOS Normal
  linux /boot/multios/kernel
  options quiet loglevel=3

title MultiOS Debug
  linux /boot/multios/kernel
  options debug loglevel=8

title MultiOS Recovery
  linux /boot/multios/recovery
  initrd /boot/multios/recovery/initrd
  options init=/bin/bash single
  recovery_mode
"#.to_string()
}

/// Sample custom configuration
fn get_sample_custom_config() -> String {
    r#"timeout=30
default=normal

title MultiOS Educational Lab
  linux /boot/multios/kernel
  options debug loglevel=8 console=ttyAMA0
  serial_console
  debug_mode

title MultiOS Production
  linux /boot/multios/kernel
  options quiet loglevel=3
  timeout 5

title MultiOS Safe Mode
  linux /boot/multios/kernel
  options safe_mode no_drivers no_services
"#.to_string()
}

/// Sample JSON configuration
fn get_sample_json_config() -> String {
    r#"{
  "timeout": 10,
  "default_entry": "multios-normal",
  "entries": [
    {
      "title": "MultiOS Normal",
      "linux": "/boot/multios/kernel",
      "options": ["quiet", "loglevel=3"],
      "serial_console": true
    },
    {
      "title": "MultiOS Debug",
      "linux": "/boot/multios/kernel", 
      "options": ["debug", "loglevel=8"],
      "debug_mode": true
    },
    {
      "title": "MultiOS Recovery",
      "linux": "/boot/multios/recovery",
      "initrd": "/boot/multios/recovery/initrd",
      "options": ["init=/bin/bash", "single"],
      "recovery_mode": true
    }
  ]
}"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_entry_creation() {
        let entry = ConfigEntry::new("Test Entry".to_string())
            .linux("/boot/kernel")
            .initrd("/boot/initrd")
            .options(&["quiet", "loglevel=3"])
            .serial_console(true)
            .debug_mode(true);
            
        assert_eq!(entry.title, "Test Entry");
        assert_eq!(entry.linux, Some("/boot/kernel".to_string()));
        assert_eq!(entry.initrd, Some("/boot/initrd".to_string()));
        assert_eq!(entry.options, vec!["quiet", "loglevel=3"]);
        assert!(entry.serial_console);
        assert!(entry.debug_mode);
    }

    #[test]
    fn test_parse_grub2_config() {
        let config_content = r#"timeout=10
default=normal

title MultiOS Normal
  linux /boot/kernel
  options quiet loglevel=3

title MultiOS Debug
  linux /boot/kernel
  options debug loglevel=8
  debug_mode
"#;
        
        let config = parse_config_content(config_content, ConfigFormat::Grub2).unwrap();
        
        assert_eq!(config.timeout, 10);
        assert_eq!(config.default_entry, Some("normal".to_string()));
        assert_eq!(config.entries.len(), 2);
        
        let normal_entry = config.find_entry("MultiOS Normal").unwrap();
        assert_eq!(normal_entry.linux, Some("/boot/kernel".to_string()));
        assert_eq!(normal_entry.options, vec!["quiet loglevel=3"]);
        assert!(!normal_entry.debug_mode);
        
        let debug_entry = config.find_entry("MultiOS Debug").unwrap();
        assert!(debug_entry.debug_mode);
    }

    #[test]
    fn test_parse_boot_parameters() {
        let params = "debug console=ttyS0 loglevel=8 quiet single";
        let parsed = parse_boot_parameters(params);
        
        assert!(parsed.contains(&BootParameter::Debug));
        assert!(parsed.contains(&BootParameter::Console("console=ttyS0".to_string())));
        assert!(parsed.contains(&BootParameter::Loglevel(8)));
        assert!(parsed.contains(&BootParameter::Quiet));
        assert!(parsed.contains(&BootParameter::Single));
    }

    #[test]
    fn test_parameters_to_string() {
        let params = vec![
            BootParameter::Debug,
            BootParameter::Console("console=ttyS0".to_string()),
            BootParameter::Loglevel(8),
            BootParameter::Quiet,
        ];
        
        let result = parameters_to_string(&params);
        assert!(result.contains("debug"));
        assert!(result.contains("console=ttyS0"));
        assert!(result.contains("loglevel=8"));
        assert!(result.contains("quiet"));
    }

    #[test]
    fn test_config_entry_to_boot_config() {
        let entry = ConfigEntry::new("Test".to_string())
            .linux("/boot/kernel")
            .options(&["quiet", "loglevel=3"])
            .serial_console(true);
            
        let boot_config = entry.to_boot_config().unwrap();
        assert_eq!(boot_config.kernel_path, "/boot/kernel");
        assert!(boot_config.serial_console);
        assert!(!boot_config.memory_test);
    }
}