//! Container Network Management
//! 
//! This module provides comprehensive container networking including bridge creation,
//! network isolation, DNS configuration, and network policy enforcement.

use super::*;
use std::net::{IpAddr, Ipv4Addr};
use std::collections::HashMap;

/// Container Network Manager - Handles all networking operations for containers
pub struct NetworkManager {
    bridge_name: String,
    network_range: Ipv4Addr,
    dns_servers: Vec<Ipv4Addr>,
    active_networks: Arc<Mutex<HashMap<String, ContainerNetwork>>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new() -> ContainerResult<Self> {
        let bridge_name = "multios-br0".to_string();
        let network_range = Ipv4Addr::new(172, 17, 0, 0);
        let dns_servers = vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)];

        let manager = Self {
            bridge_name,
            network_range,
            dns_servers,
            active_networks: Arc::new(Mutex::new(HashMap::new())),
        };

        // Initialize networking infrastructure
        manager.initialize_networking().await?;

        Ok(manager)
    }

    /// Initialize networking infrastructure
    async fn initialize_networking(&self) -> ContainerResult<()> {
        // Create bridge if it doesn't exist
        if !self.bridge_exists(&self.bridge_name).await? {
            self.create_bridge(&self.bridge_name).await?;
        }

        // Configure bridge
        self.configure_bridge(&self.bridge_name).await?;

        // Enable IP forwarding
        self.enable_ip_forwarding().await?;

        // Setup NAT rules
        self.setup_nat_rules().await?;

        Ok(())
    }

    /// Create network interface for a container
    pub async fn create_network_interface(&self, container_id: &str, config: &NetworkConfig) -> ContainerResult<()> {
        match config.network_mode {
            NetworkMode::Bridge => self.create_bridge_network_interface(container_id, config).await,
            NetworkMode::Host => self.create_host_network_interface(container_id).await,
            NetworkMode::None => self.create_none_network_interface(container_id).await,
            NetworkMode::Custom(ref network_id) => {
                self.create_custom_network_interface(container_id, config, network_id).await
            }
        }
    }

    /// Prepare network environment for container
    pub async fn prepare_network_environment(&self, container_id: &str) -> ContainerResult<()> {
        // This method is called when a container is about to start
        // It ensures the network interface is properly configured
        let active_networks = self.active_networks.lock().unwrap();
        if let Some(network) = active_networks.get(container_id) {
            self.configure_container_network(container_id, network).await?;
        }
        Ok(())
    }

    /// Configure networking for container based on mode
    async fn create_bridge_network_interface(&self, container_id: &str, config: &NetworkConfig) -> ContainerResult<()> {
        // Create virtual ethernet pair
        let veth_name = format!("veth_{}", container_id);
        self.create_veth_pair(&veth_name).await?;

        // Assign IP address
        let ip_address = self.allocate_ip_address(container_id).await?;
        
        // Configure container interface
        self.configure_container_interface(&veth_name, &ip_address).await?;

        // Add to bridge
        self.add_to_bridge(&self.bridge_name, &veth_name).await?;

        // Configure port mappings if specified
        for port_mapping in &config.port_mappings {
            self.setup_port_forwarding(container_id, port_mapping).await?;
        }

        // Store network configuration
        let network = ContainerNetwork {
            id: container_id.to_string(),
            mode: NetworkMode::Bridge,
            interface_name: veth_name,
            ip_address,
            mac_address: self.generate_mac_address(container_id),
            dns_servers: config.dns_servers.clone(),
            port_mappings: config.port_mappings.clone(),
            created_at: SystemTime::now(),
        };

        {
            let mut active_networks = self.active_networks.lock().unwrap();
            active_networks.insert(container_id.to_string(), network);
        }

        Ok(())
    }

    /// Create host network interface (shares host networking)
    async fn create_host_network_interface(&self, container_id: &str) -> ContainerResult<()> {
        let network = ContainerNetwork {
            id: container_id.to_string(),
            mode: NetworkMode::Host,
            interface_name: String::new(), // Use host interface
            ip_address: Ipv4Addr::new(127, 0, 0, 1),
            mac_address: String::new(), // Use host MAC
            dns_servers: vec![],
            port_mappings: vec![],
            created_at: SystemTime::now(),
        };

        {
            let mut active_networks = self.active_networks.lock().unwrap();
            active_networks.insert(container_id.to_string(), network);
        }

        Ok(())
    }

    /// Create isolated network interface (no external networking)
    async fn create_none_network_interface(&self, container_id: &str) -> ContainerResult<()> {
        let network = ContainerNetwork {
            id: container_id.to_string(),
            mode: NetworkMode::None,
            interface_name: String::new(),
            ip_address: Ipv4Addr::new(127, 0, 0, 1),
            mac_address: String::new(),
            dns_servers: vec![],
            port_mappings: vec![],
            created_at: SystemTime::now(),
        };

        {
            let mut active_networks = self.active_networks.lock().unwrap();
            active_networks.insert(container_id.to_string(), network);
        }

        Ok(())
    }

    /// Create custom network interface
    async fn create_custom_network_interface(&self, container_id: &str, config: &NetworkConfig, network_id: &str) -> ContainerResult<()> {
        // For custom networks, we'd load the network definition and configure accordingly
        // This is a simplified implementation
        
        let ip_address = if let Some(ref ip) = config.ip_address {
            ip.parse().map_err(|_| ContainerError::NetworkError("Invalid IP address".to_string()))?
        } else {
            self.allocate_ip_address(container_id).await?
        };

        let network = ContainerNetwork {
            id: container_id.to_string(),
            mode: NetworkMode::Custom(network_id.to_string()),
            interface_name: format!("veth_{}", container_id),
            ip_address,
            mac_address: self.generate_mac_address(container_id),
            dns_servers: config.dns_servers.clone(),
            port_mappings: config.port_mappings.clone(),
            created_at: SystemTime::now(),
        };

        {
            let mut active_networks = self.active_networks.lock().unwrap();
            active_networks.insert(container_id.to_string(), network);
        }

        Ok(())
    }

    /// Remove network interface for a container
    pub async fn cleanup_network_interface(&self, container_id: &str) -> ContainerResult<()> {
        let mut active_networks = self.active_networks.lock().unwrap();
        
        if let Some(network) = active_networks.remove(container_id) {
            // Remove interface
            if !network.interface_name.is_empty() {
                self.remove_interface(&network.interface_name).await?;
            }

            // Release IP address
            self.release_ip_address(container_id).await?;

            // Remove port mappings
            for port_mapping in &network.port_mappings {
                self.remove_port_forwarding(port_mapping).await?;
            }
        }

        Ok(())
    }

    /// Get network information for a container
    pub async fn get_network_info(&self, container_id: &str) -> ContainerResult<NetworkInfo> {
        let active_networks = self.active_networks.lock().unwrap();
        let network = active_networks.get(container_id)
            .ok_or(ContainerError::NotFound(format!("Container {} network not found", container_id)))?;

        let stats = self.get_network_stats(&network.interface_name).await?;

        Ok(NetworkInfo {
            container_id: network.id.clone(),
            network_mode: network.mode.clone(),
            interface_name: network.interface_name.clone(),
            ip_address: network.ip_address.to_string(),
            mac_address: network.mac_address.clone(),
            dns_servers: network.dns_servers.clone(),
            port_mappings: network.port_mappings.clone(),
            stats,
            created_at: network.created_at,
        })
    }

    /// List all container networks
    pub async fn list_networks(&self) -> Vec<NetworkInfo> {
        let active_networks = self.active_networks.lock().unwrap();
        active_networks.values().map(|network| {
            NetworkInfo {
                container_id: network.id.clone(),
                network_mode: network.mode.clone(),
                interface_name: network.interface_name.clone(),
                ip_address: network.ip_address.to_string(),
                mac_address: network.mac_address.clone(),
                dns_servers: network.dns_servers.clone(),
                port_mappings: network.port_mappings.clone(),
                stats: NetworkStats::default(),
                created_at: network.created_at,
            }
        }).collect()
    }

    // Private helper methods

    async fn bridge_exists(&self, bridge_name: &str) -> ContainerResult<bool> {
        unsafe {
            let cmd = format!("ip link show {} 2>/dev/null | grep -q {}\0", bridge_name, bridge_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            Ok(result == 0)
        }
    }

    async fn create_bridge(&self, bridge_name: &str) -> ContainerResult<()> {
        unsafe {
            let cmd = format!("ip link add {} type bridge\n", bridge_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                return Err(ContainerError::NetworkError(
                    format!("Failed to create bridge {}", bridge_name)
                ));
            }
        }
        Ok(())
    }

    async fn configure_bridge(&self, bridge_name: &str) -> ContainerResult<()> {
        unsafe {
            // Bring bridge up
            let cmd1 = format!("ip link set {} up\n", bridge_name);
            libc::system(cmd1.as_ptr() as *const libc::c_char);

            // Assign IP address to bridge
            let bridge_ip = Ipv4Addr::new(172, 17, 0, 1);
            let cmd2 = format!("ip addr add {}/16 dev {}\n", bridge_ip, bridge_name);
            let result = libc::system(cmd2.as_ptr() as *const libc::c_char);

            if result != 0 {
                return Err(ContainerError::NetworkError(
                    format!("Failed to configure bridge {}", bridge_name)
                ));
            }
        }
        Ok(())
    }

    async fn enable_ip_forwarding(&self) -> ContainerResult<()> {
        unsafe {
            let cmd = "echo 1 > /proc/sys/net/ipv4/ip_forward\n";
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                return Err(ContainerError::NetworkError(
                    "Failed to enable IP forwarding".to_string()
                ));
            }
        }
        Ok(())
    }

    async fn setup_nat_rules(&self) -> ContainerResult<()> {
        // Setup NAT rules for outbound traffic
        unsafe {
            let cmd = "iptables -t nat -A POSTROUTING -s 172.17.0.0/16 ! -o multios-br0 -j MASQUERADE\n";
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                log::warn!("Failed to setup NAT rules (this may be normal in some environments)");
            }
        }
        Ok(())
    }

    async fn create_veth_pair(&self, veth_name: &str) -> ContainerResult<()> {
        unsafe {
            let cmd = format!("ip link add {} type veth peer name {}\n", veth_name, veth_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                return Err(ContainerError::NetworkError(
                    format!("Failed to create veth pair {}", veth_name)
                ));
            }

            // Bring up the interface
            let up_cmd = format!("ip link set {} up\n", veth_name);
            libc::system(up_cmd.as_ptr() as *const libc::c_char);
        }
        Ok(())
    }

    async fn allocate_ip_address(&self, container_id: &str) -> ContainerResult<Ipv4Addr> {
        // Simple IP allocation - in reality, we'd maintain a proper allocation table
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        container_id.hash(&mut hasher);
        let hash = hasher.finish();

        // Generate IP in the 172.17.0.x range
        let offset = (hash % 65000) as u32 + 2; // Start from .2 to avoid bridge IP
        let ip = Ipv4Addr::new(172, 17, (offset >> 8) as u8, (offset & 0xFF) as u8);

        Ok(ip)
    }

    async fn configure_container_interface(&self, interface_name: &str, ip_address: &Ipv4Addr) -> ContainerResult<()> {
        unsafe {
            let cmd = format!("ip addr add {}/24 dev {}\n", ip_address, interface_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);

            if result != 0 {
                return Err(ContainerError::NetworkError(
                    format!("Failed to configure interface {}", interface_name)
                ));
            }
        }
        Ok(())
    }

    async fn add_to_bridge(&self, bridge_name: &str, interface_name: &str) -> ContainerResult<()> {
        unsafe {
            let cmd = format!("ip link set {} master {}\n", interface_name, bridge_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);

            if result != 0 {
                return Err(ContainerError::NetworkError(
                    format!("Failed to add interface to bridge: {} -> {}", interface_name, bridge_name)
                ));
            }
        }
        Ok(())
    }

    async fn setup_port_forwarding(&self, container_id: &str, port_mapping: &PortMapping) -> ContainerResult<()> {
        // Get container IP
        let active_networks = self.active_networks.lock().unwrap();
        let network = active_networks.get(container_id)
            .ok_or(ContainerError::NotFound(format!("Container {} network not found", container_id)))?;

        let protocol = if port_mapping.protocol.to_lowercase() == "udp" {
            "-u"
        } else {
            "-t" // Default to TCP
        };

        unsafe {
            // Setup iptables rules for port forwarding
            let cmd = format!(
                "iptables -t nat -A PREROUTING {} -p {} --dport {} -j DNAT --to-dest {}:{}\n",
                protocol, port_mapping.protocol, port_mapping.host_port, network.ip_address, port_mapping.container_port
            );
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);

            if result != 0 {
                log::warn!("Failed to setup port forwarding for container {}:{}", container_id, port_mapping.host_port);
            }
        }

        Ok(())
    }

    async fn remove_port_forwarding(&self, port_mapping: &PortMapping) -> ContainerResult<()> {
        unsafe {
            let protocol = if port_mapping.protocol.to_lowercase() == "udp" {
                "-u"
            } else {
                "-t"
            };

            let cmd = format!(
                "iptables -t nat -D PREROUTING {} -p {} --dport {} -j DNAT --to-dest {}\n",
                protocol, port_mapping.protocol, port_mapping.host_port, port_mapping.container_port
            );
            libc::system(cmd.as_ptr() as *const libc::c_char);
        }
        Ok(())
    }

    async fn remove_interface(&self, interface_name: &str) -> ContainerResult<()> {
        unsafe {
            let cmd = format!("ip link delete {}\n", interface_name);
            let result = libc::system(cmd.as_ptr() as *const libc::c_char);
            
            if result != 0 {
                log::warn!("Failed to remove interface {}", interface_name);
            }
        }
        Ok(())
    }

    async fn release_ip_address(&self, container_id: &str) -> ContainerResult<()> {
        // In a real implementation, we'd maintain an IP allocation table
        // and remove the IP from it here
        Ok(())
    }

    async fn configure_container_network(&self, container_id: &str, network: &ContainerNetwork) -> ContainerResult<()> {
        // This method would configure the container's network namespace
        // with the appropriate networking configuration
        
        if network.mode == NetworkMode::None {
            // Configure loopback only
            unsafe {
                libc::system("ip link set lo up\0".as_ptr() as *const libc::c_char);
            }
        } else if !network.interface_name.is_empty() {
            // Configure the main interface
            unsafe {
                let cmd = format!("ip link set {} up\n", network.interface_name);
                libc::system(cmd.as_ptr() as *const libc::c_char);

                // Add default route if not host mode
                if !matches!(network.mode, NetworkMode::Host) {
                    let route_cmd = format!("ip route add default via 172.17.0.1 dev {}\n", network.interface_name);
                    libc::system(route_cmd.as_ptr() as *const libc::c_char);
                }
            }
        }

        Ok(())
    }

    fn generate_mac_address(&self, container_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        container_id.hash(&mut hasher);
        let hash = hasher.finish();

        // Generate a MAC address with the multicast bit set
        let mac_bytes = [
            0x02, // Locally administered
            (hash >> 40) as u8,
            (hash >> 32) as u8,
            (hash >> 24) as u8,
            (hash >> 16) as u8,
            (hash >> 8) as u8,
        ];

        format!("{}", mac_bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(":"))
    }

    async fn get_network_stats(&self, interface_name: &str) -> ContainerResult<NetworkStats> {
        // Read network statistics from /proc/net/dev
        let stats_file = "/proc/net/dev";
        let content = std::fs::read_to_string(stats_file)
            .map_err(|e| ContainerError::System(format!("Failed to read network stats: {}", e)))?;

        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        let mut rx_packets = 0;
        let mut tx_packets = 0;

        for line in content.lines() {
            if line.starts_with(interface_name) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 16 {
                    rx_bytes = parts[1].parse().unwrap_or(0);
                    rx_packets = parts[2].parse().unwrap_or(0);
                    tx_bytes = parts[9].parse().unwrap_or(0);
                    tx_packets = parts[10].parse().unwrap_or(0);
                }
                break;
            }
        }

        Ok(NetworkStats {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors: 0,
            tx_errors: 0,
        })
    }
}

/// Container network information
#[derive(Debug, Clone)]
pub struct ContainerNetwork {
    pub id: String,
    pub mode: NetworkMode,
    pub interface_name: String,
    pub ip_address: Ipv4Addr,
    pub mac_address: String,
    pub dns_servers: Vec<String>,
    pub port_mappings: Vec<PortMapping>,
    pub created_at: SystemTime,
}

/// Network information for a container
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub container_id: String,
    pub network_mode: NetworkMode,
    pub interface_name: String,
    pub ip_address: String,
    pub mac_address: String,
    pub dns_servers: Vec<String>,
    pub port_mappings: Vec<PortMapping>,
    pub stats: NetworkStats,
    pub created_at: SystemTime,
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Network policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub name: String,
    pub ingress: Vec<IngressRule>,
    pub egress: Vec<EgressRule>,
}

/// Ingress rule for network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub from: Vec<String>,
    pub ports: Vec<u16>,
    pub protocols: Vec<String>,
}

/// Egress rule for network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressRule {
    pub to: Vec<String>,
    pub ports: Vec<u16>,
    pub protocols: Vec<String>,
}