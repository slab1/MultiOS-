pub mod error;

use crate::core::config::NetworkConfig;
use crate::hardware::NetworkInterface;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::fs;

/// Network manager for handling network configuration during installation
pub struct NetworkManager;

impl NetworkManager {
    pub fn new() -> Self {
        Self
    }

    /// Configure network based on provided configuration
    pub async fn configure(&self, config: NetworkConfig) -> Result<()> {
        log::info!("Configuring network");
        
        // Configure hostname
        self.configure_hostname(&config.hostname).await?;
        
        // Configure network interfaces
        if config.dhcp {
            self.configure_dhcp().await?;
        } else {
            self.configure_static_ip(&config).await?;
        }
        
        // Configure DNS servers
        self.configure_dns(&config.dns_servers).await?;
        
        // Configure network services
        self.configure_network_services().await?;
        
        log::info!("Network configuration completed");
        Ok(())
    }

    /// Get available network interfaces
    pub async fn get_available_interfaces(&self) -> Result<Vec<NetworkInterfaceInfo>> {
        log::info!("Detecting available network interfaces");
        
        let mut interfaces = Vec::new();
        
        // Detect interfaces using system commands
        let output = Command::new("ip")
            .args(&["link", "show"])
            .output()?;
            
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            interfaces = self.parse_ip_link_output(&output_str)?;
        }
        
        log::info!("Found {} network interfaces", interfaces.len());
        Ok(interfaces)
    }

    /// Test network connectivity
    pub async fn test_connectivity(&self) -> Result<NetworkTestResult> {
        log::info!("Testing network connectivity");
        
        let mut result = NetworkTestResult {
            localhost: false,
            gateway: false,
            dns: false,
            internet: false,
            details: String::new(),
        };
        
        // Test localhost
        result.localhost = self.test_localhost().await;
        
        // Test gateway
        result.gateway = self.test_gateway().await;
        
        // Test DNS
        result.dns = self.test_dns().await;
        
        // Test internet connectivity
        result.internet = self.test_internet().await;
        
        // Generate detailed report
        result.details = self.generate_connectivity_report(&result);
        
        log::info!("Network connectivity test completed");
        Ok(result)
    }

    /// Configure hostname
    async fn configure_hostname(&self, hostname: &str) -> Result<()> {
        log::info!("Configuring hostname: {}", hostname);
        
        // Set hostname in /etc/hostname
        tokio::fs::write("/etc/hostname", hostname).await?;
        
        // Set hostname in /etc/hosts
        let hosts_content = format!(
            "127.0.0.1 localhost\n\
             127.0.1.1 {}\n\
             ::1 localhost ip6-localhost",
            hostname
        );
        tokio::fs::write("/etc/hosts", hosts_content).await?;
        
        // Set hostname using hostnamectl
        let output = Command::new("hostnamectl")
            .args(&["set-hostname", hostname])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to set hostname via hostnamectl: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Configure DHCP for all interfaces
    async fn configure_dhcp(&self) -> Result<()> {
        log::info!("Configuring DHCP");
        
        // Configure all available interfaces for DHCP
        let interfaces = self.get_available_interfaces().await?;
        
        for interface in interfaces {
            if !interface.loopback && interface.up {
                self.configure_dhcp_interface(&interface.name).await?;
            }
        }
        
        Ok(())
    }

    /// Configure DHCP for specific interface
    async fn configure_dhcp_interface(&self, interface_name: &str) -> Result<()> {
        // Configure interface for DHCP
        let netplan_config = format!(
            "network:\n\
             version: 2\n\
             ethernets:\n\
             {}:\n\
             dhcp4: true\n\
             dhcp6: true",
            interface_name
        );
        
        let config_path = format!("/etc/netplan/50-multios-installer.yaml");
        tokio::fs::write(&config_path, netplan_config).await?;
        
        // Apply configuration
        let output = Command::new("netplan")
            .args(&["apply"])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to apply DHCP configuration: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Configure static IP configuration
    async fn configure_static_ip(&self, config: &NetworkConfig) -> Result<()> {
        log::info!("Configuring static IP configuration");
        
        if let Some(static_ip) = &config.static_ip {
            let netplan_config = format!(
                "network:\n\
                 version: 2\n\
                 ethernets:\n\
                 eth0:\n\
                 dhcp4: false\n\
                 addresses:\n\
                 - {}/{}",
                static_ip,
                config.netmask.as_deref().unwrap_or("24")
            );
            
            if let Some(gateway) = &config.gateway {
                netplan_config.push_str(&format!("\n\ngateway4: {}", gateway));
            }
            
            let config_path = "/etc/netplan/50-multios-installer.yaml";
            tokio::fs::write(config_path, netplan_config).await?;
            
            // Apply configuration
            let output = Command::new("netplan")
                .args(&["apply"])
                .output()?;
                
            if !output.status.success() {
                return Err(anyhow!("Failed to apply static IP configuration: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        Ok(())
    }

    /// Configure DNS servers
    async fn configure_dns(&self, dns_servers: &[String]) -> Result<()> {
        log::info!("Configuring DNS servers: {:?}", dns_servers);
        
        if dns_servers.is_empty() {
            return Ok(());
        }
        
        // Configure DNS in resolv.conf
        let resolv_content = dns_servers.iter()
            .map(|dns| format!("nameserver {}", dns))
            .collect::<Vec<_>>()
            .join("\n");
            
        tokio::fs::write("/etc/resolv.conf", resolv_content).await?;
        
        // Also configure in netplan if using systemd-networkd
        if !dns_servers.is_empty() {
            let nameservers = dns_servers.join(", ");
            log::info!("DNS servers configured: {}", nameservers);
        }
        
        Ok(())
    }

    /// Configure network services
    async fn configure_network_services(&self) -> Result<()> {
        log::info!("Configuring network services");
        
        // Start and enable network services
        let services = ["systemd-networkd", "systemd-resolved"];
        
        for service in &services {
            // Enable service
            let output = Command::new("systemctl")
                .args(&["enable", service])
                .output()?;
                
            if !output.status.success() {
                log::warn!("Failed to enable service {}: {}", service, 
                    String::from_utf8_lossy(&output.stderr));
            }
            
            // Start service
            let output = Command::new("systemctl")
                .args(&["start", service])
                .output()?;
                
            if !output.status.success() {
                log::warn!("Failed to start service {}: {}", service, 
                    String::from_utf8_lossy(&output.stderr));
            }
        }
        
        Ok(())
    }

    /// Test localhost connectivity
    async fn test_localhost(&self) -> bool {
        let output = Command::new("ping")
            .args(&["-c", "1", "127.0.0.1"])
            .output()
            .ok();
            
        match output {
            Some(output) => output.status.success(),
            None => false,
        }
    }

    /// Test gateway connectivity
    async fn test_gateway(&self) -> bool {
        // Get default gateway
        let output = Command::new("ip")
            .args(&["route", "show", "default"])
            .output()
            .ok();
            
        match output {
            Some(output) => {
                if output.status.success() {
                    let route_output = String::from_utf8_lossy(&output.stdout);
                    if let Some(gateway) = route_output.lines()
                        .find(|line| line.contains("default"))
                        .and_then(|line| line.split_whitespace().nth(2)) {
                        
                        let ping_output = Command::new("ping")
                            .args(&["-c", "1", gateway])
                            .output()
                            .ok();
                            
                        match ping_output {
                            Some(ping_output) => ping_output.status.success(),
                            None => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Test DNS resolution
    async fn test_dns(&self) -> bool {
        let output = Command::new("nslookup")
            .args(&["google.com"])
            .output()
            .ok();
            
        match output {
            Some(output) => output.status.success(),
            None => false,
        }
    }

    /// Test internet connectivity
    async fn test_internet(&self) -> bool {
        let output = Command::new("ping")
            .args(&["-c", "1", "8.8.8.8"])
            .output()
            .ok();
            
        match output {
            Some(output) => output.status.success(),
            None => false,
        }
    }

    /// Generate connectivity report
    fn generate_connectivity_report(&self, result: &NetworkTestResult) -> String {
        format!(
            "Network Connectivity Report:\n\
             - Localhost: {}\n\
             - Gateway: {}\n\
             - DNS: {}\n\
             - Internet: {}",
            if result.localhost { "✓" } else { "✗" },
            if result.gateway { "✓" } else { "✗" },
            if result.dns { "✓" } else { "✗" },
            if result.internet { "✓" } else { "✗" }
        )
    }

    /// Parse ip link show output
    fn parse_ip_link_output(&self, output: &str) -> Result<Vec<NetworkInterfaceInfo>> {
        let mut interfaces = Vec::new();
        
        for line in output.lines() {
            if line.starts_with(&format!("{}:", std::process::id())) {
                continue; // Skip the header
            }
            
            if line.starts_with(" ") {
                continue; // Skip continuation lines
            }
            
            if let Some((name, details)) = self.parse_interface_line(line) {
                interfaces.push(NetworkInterfaceInfo {
                    name,
                    description: details.get("description").cloned().unwrap_or_default(),
                    mac_address: details.get("link/ether").cloned().unwrap_or_default(),
                    mtu: details.get("mtu").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1500),
                    state: details.get("state").cloned().unwrap_or("unknown".to_string()),
                    loopback: details.get("LOOPBACK").is_some(),
                    up: details.get("state") == Some(&"UP".to_string()),
                });
            }
        }
        
        Ok(interfaces)
    }

    /// Parse individual interface line
    fn parse_interface_line(&self, line: &str) -> Option<(String, std::collections::HashMap<String, String>)> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 3 {
            return None;
        }
        
        let name_parts: Vec<&str> = parts[1].trim().split('@').collect();
        let name = name_parts[0].to_string();
        
        let mut details = std::collections::HashMap::new();
        
        // Extract interface details
        let interface_data = parts[2].trim();
        
        // Parse flags and other details
        if let Some(flags_start) = interface_data.find('<') {
            if let Some(flags_end) = interface_data.find('>') {
                let flags = &interface_data[flags_start + 1..flags_end];
                details.insert("flags".to_string(), flags.to_string());
                
                if flags.contains("LOOPBACK") {
                    details.insert("LOOPBACK".to_string(), "true".to_string());
                }
            }
        }
        
        // Look for MTU
        for part in interface_data.split_whitespace() {
            if part.starts_with("mtu") {
                if let Some(mtu_value) = part.split('=').nth(1) {
                    details.insert("mtu".to_string(), mtu_value.to_string());
                }
            }
        }
        
        Some((name, details))
    }

    /// Get recommended network configuration
    pub fn get_recommended_config() -> NetworkConfig {
        NetworkConfig {
            dhcp: true,
            static_ip: None,
            netmask: None,
            gateway: None,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            hostname: "multios".to_string(),
        }
    }
}

// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub mtu: u32,
    pub state: String,
    pub loopback: bool,
    pub up: bool,
}

// Network test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTestResult {
    pub localhost: bool,
    pub gateway: bool,
    pub dns: bool,
    pub internet: bool,
    pub details: String,
}