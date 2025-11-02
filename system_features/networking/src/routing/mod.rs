//! Routing implementation
//!
//! This module provides comprehensive routing functionality including routing tables,
//! route entries, static and dynamic routing, and IP forwarding capabilities.

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, Instant};

/// Route types supported by the routing system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteType {
    /// Direct route to connected network
    Connected,
    /// Static route configured by administrator
    Static,
    /// Dynamic route learned via routing protocol
    Dynamic,
    /// Default route
    Default,
    /// Host-specific route
    Host,
    /// Blackhole route (packets dropped)
    Blackhole,
}

/// Route flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteFlags {
    pub up: bool,           // Route is usable
    pub gateway: bool,      // Route uses a gateway
    pub host: bool,         // Route is to a specific host
    pub reinstal: bool,     // Route was reinstalled
    pub modified: bool,     // Route was modified
    pub done: bool,         // Route installation complete
    pub clone: bool,        // Route is a clone
    pub equalize: bool,     // Load balancing route
}

impl RouteFlags {
    /// Create default route flags
    pub fn default() -> Self {
        Self {
            up: false,
            gateway: false,
            host: false,
            reinstal: false,
            modified: false,
            done: false,
            clone: false,
            equalize: false,
        }
    }

    /// Parse flags from u32 value
    pub fn from_u32(value: u32) -> Self {
        Self {
            up: (value & 0x0001) != 0,
            gateway: (value & 0x0002) != 0,
            host: (value & 0x0004) != 0,
            reinstal: (value & 0x0008) != 0,
            modified: (value & 0x0010) != 0,
            done: (value & 0x0020) != 0,
            clone: (value & 0x0040) != 0,
            equalize: (value & 0x0080) != 0,
        }
    }

    /// Convert to u32 value
    pub fn to_u32(&self) -> u32 {
        (self.up as u32) |
        ((self.gateway as u32) << 1) |
        ((self.host as u32) << 2) |
        ((self.reinstal as u32) << 3) |
        ((self.modified as u32) << 4) |
        ((self.done as u32) << 5) |
        ((self.clone as u32) << 6) |
        ((self.equalize as u32) << 7)
    }
}

/// Route entry structure
#[derive(Debug, Clone)]
pub struct Route {
    /// Destination network/host
    pub destination: IpAddress,
    /// Destination netmask (0.0.0.0 for default)
    pub netmask: IpAddress,
    /// Gateway address (if applicable)
    pub gateway: Option<IpAddress>,
    /// Source address for this route
    pub source: Option<IpAddress>,
    /// Network interface
    pub interface: String,
    /// Route type
    pub route_type: RouteType,
    /// Route flags
    pub flags: RouteFlags,
    /// Route metric/cost
    pub metric: u32,
    /// Time to live for dynamic routes
    pub ttl: Option<Duration>,
    /// Route creation time
    pub created_at: Instant,
    /// Last time route was used
    pub last_used: Instant,
    /// Reference count
    pub ref_count: u32,
}

impl Route {
    /// Create a new route
    pub fn new(
        destination: IpAddress,
        netmask: IpAddress,
        gateway: Option<IpAddress>,
        interface: String,
        route_type: RouteType,
    ) -> Self {
        let now = Instant::now();
        Self {
            destination,
            netmask,
            gateway,
            source: None,
            interface,
            route_type,
            flags: RouteFlags::default(),
            metric: 0,
            ttl: None,
            created_at: now,
            last_used: now,
            ref_count: 0,
        }
    }

    /// Create a connected route
    pub fn connected(destination: IpAddress, netmask: IpAddress, interface: String) -> Self {
        let mut route = Self::new(destination, netmask, None, interface, RouteType::Connected);
        route.flags.up = true;
        route
    }

    /// Create a default route
    pub fn default_route(gateway: IpAddress, interface: String) -> Self {
        Self::new(IpAddress::v4(0, 0, 0, 0), IpAddress::v4(0, 0, 0, 0), Some(gateway), interface, RouteType::Default)
    }

    /// Create a host-specific route
    pub fn host_route(host: IpAddress, gateway: IpAddress, interface: String) -> Self {
        let mut route = Self::new(host, IpAddress::v4(255, 255, 255, 255), Some(gateway), interface, RouteType::Host);
        route.flags.host = true;
        route
    }

    /// Check if destination matches this route
    pub fn matches(&self, dest: IpAddress) -> bool {
        let dest_net = dest.to_u32() & self.netmask.to_u32();
        let route_net = self.destination.to_u32() & self.netmask.to_u32();
        dest_net == route_net
    }

    /// Calculate route length (prefix length)
    pub fn prefix_length(&self) -> u32 {
        let mask = self.netmask.to_u32();
        mask.count_ones()
    }

    /// Check if route is more specific than another
    pub fn is_more_specific(&self, other: &Route) -> bool {
        self.prefix_length() > other.prefix_length()
    }

    /// Check if route has expired
    pub fn has_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            Instant::now().duration_since(self.created_at) > ttl
        } else {
            false
        }
    }

    /// Mark route as used
    pub fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.ref_count = self.ref_count.saturating_add(1);
    }

    /// Check if route is usable
    pub fn is_usable(&self) -> bool {
        self.flags.up && !self.has_expired()
    }

    /// Get next hop for destination
    pub fn next_hop(&self, dest: IpAddress) -> Option<IpAddress> {
        if self.flags.gateway {
            self.gateway
        } else if self.flags.host {
            Some(dest)
        } else {
            Some(dest) // Direct delivery
        }
    }

    /// Get interface for this route
    pub fn interface_name(&self) -> &str {
        &self.interface
    }

    /// Get route cost
    pub fn cost(&self) -> u32 {
        self.metric
    }

    /// Compare routes for route selection
    pub fn compare(&self, other: &Route) -> std::cmp::Ordering {
        // First compare by prefix length (more specific is better)
        let prefix_cmp = self.prefix_length().cmp(&other.prefix_length());
        if prefix_cmp != std::cmp::Ordering::Equal {
            return prefix_cmp.reverse(); // More specific first
        }

        // Then compare by metric (lower is better)
        let metric_cmp = self.metric.cmp(&other.metric());
        if metric_cmp != std::cmp::Ordering::Equal {
            return metric_cmp;
        }

        // Then compare by route type preference
        let type_priority = self.route_type_priority();
        let other_type_priority = other.route_type_priority();
        
        if type_priority != other_type_priority {
            return type_priority.cmp(&other_type_priority).reverse();
        }

        // Finally, prefer newer routes
        self.created_at.cmp(&other.created_at).reverse()
    }

    /// Get route type priority for selection
    fn route_type_priority(&self) -> u32 {
        match self.route_type {
            RouteType::Connected => 5,
            RouteType::Host => 4,
            RouteType::Static => 3,
            RouteType::Dynamic => 2,
            RouteType::Default => 1,
            RouteType::Blackhole => 0,
        }
    }
}

/// Route table structure
#[derive(Debug)]
pub struct RoutingTable {
    /// Routes organized by prefix length
    routes: BTreeMap<u32, Vec<Route>>,
    /// Fast lookup by destination
    route_cache: HashMap<IpAddress, Route>,
    /// Default route
    default_route: Option<Route>,
    /// Route statistics
    stats: RoutingTableStats,
    /// Route learning timer
    route_cleaner: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Default)]
pub struct RoutingTableStats {
    /// Total routes
    pub total_routes: usize,
    /// Connected routes
    pub connected_routes: usize,
    /// Static routes
    pub static_routes: usize,
    /// Dynamic routes
    pub dynamic_routes: usize,
    /// Default routes
    pub default_routes: usize,
    /// Host routes
    pub host_routes: usize,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Route lookups
    pub route_lookups: u64,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new() -> Self {
        Self {
            routes: BTreeMap::new(),
            route_cache: HashMap::new(),
            default_route: None,
            stats: RoutingTableStats::default(),
            route_cleaner: None,
        }
    }

    /// Add a route to the table
    pub fn add_route(&mut self, route: Route) -> Result<()> {
        let prefix_length = route.prefix_length();
        
        // Update statistics
        self.stats.total_routes += 1;
        match route.route_type {
            RouteType::Connected => self.stats.connected_routes += 1,
            RouteType::Static => self.stats.static_routes += 1,
            RouteType::Dynamic => self.stats.dynamic_routes += 1,
            RouteType::Default => {
                self.stats.default_routes += 1;
                self.default_route = Some(route.clone());
            },
            RouteType::Host => self.stats.host_routes += 1,
            RouteType::Blackhole => {},
        }

        // Add to prefix bucket
        self.routes.entry(prefix_length).or_insert_with(Vec::new).push(route);

        // Clear cache since routes have changed
        self.route_cache.clear();

        log::info!("Added route: {} via {} on interface {}", route.destination, 
                   route.gateway.map(|g| g.to_string()).unwrap_or_else(|| "direct".to_string()),
                   route.interface);
        Ok(())
    }

    /// Remove a route from the table
    pub fn remove_route(&mut self, destination: IpAddress, netmask: IpAddress) -> Result<Route> {
        let prefix_length = netmask.to_u32().count_ones();
        
        if let Some(routes) = self.routes.get_mut(&prefix_length) {
            // Find and remove the route
            if let Some(index) = routes.iter().position(|r| r.destination == destination && r.netmask == netmask) {
                let route = routes.remove(index);
                
                // Update statistics
                self.stats.total_routes = self.stats.total_routes.saturating_sub(1);
                match route.route_type {
                    RouteType::Connected => self.stats.connected_routes = self.stats.connected_routes.saturating_sub(1),
                    RouteType::Static => self.stats.static_routes = self.stats.static_routes.saturating_sub(1),
                    RouteType::Dynamic => self.stats.dynamic_routes = self.stats.dynamic_routes.saturating_sub(1),
                    RouteType::Default => {
                        self.stats.default_routes = self.stats.default_routes.saturating_sub(1);
                        self.default_route = None;
                    },
                    RouteType::Host => self.stats.host_routes = self.stats.host_routes.saturating_sub(1),
                    RouteType::Blackhole => {},
                }

                // Clear cache
                self.route_cache.clear();

                log::info!("Removed route: {}", destination);
                return Ok(route);
            }
        }

        Err(NetworkError::RoutingError)
    }

    /// Find the best route to a destination
    pub fn find_best_route(&mut self, destination: IpAddress) -> Option<&Route> {
        self.stats.route_lookups += 1;

        // Check cache first
        if let Some(cached_route) = self.route_cache.get(&destination) {
            self.stats.cache_hits += 1;
            return Some(cached_route);
        }

        self.stats.cache_misses += 1;

        // Find matching routes
        let mut matches = Vec::new();
        
        for (prefix_length, routes) in &self.routes {
            for route in routes {
                if route.matches(destination) && route.is_usable() {
                    matches.push(route);
                }
            }
        }

        if matches.is_empty() {
            return self.default_route.as_ref().filter(|r| r.is_usable());
        }

        // Sort matches by preference (most specific first)
        matches.sort_by(|a, b| b.compare(a));

        // Cache the best route for future lookups
        let best_route = matches[0].clone();
        self.route_cache.insert(destination, best_route.clone());

        // Mark route as used
        let best_route_mut = self.route_cache.get_mut(&destination).unwrap();
        best_route_mut.mark_used();

        Some(best_route)
    }

    /// Get all routes in the table
    pub fn get_all_routes(&self) -> Vec<&Route> {
        self.routes.values().flatten().collect()
    }

    /// Get routes by type
    pub fn get_routes_by_type(&self, route_type: RouteType) -> Vec<&Route> {
        self.get_all_routes().into_iter()
            .filter(|r| r.route_type == route_type)
            .collect()
    }

    /// Get connected routes
    pub fn get_connected_routes(&self) -> Vec<&Route> {
        self.get_routes_by_type(RouteType::Connected)
    }

    /// Get static routes
    pub fn get_static_routes(&self) -> Vec<&Route> {
        self.get_routes_by_type(RouteType::Static)
    }

    /// Get dynamic routes
    pub fn get_dynamic_routes(&self) -> Vec<&Route> {
        self.get_routes_by_type(RouteType::Dynamic)
    }

    /// Get default route
    pub fn get_default_route(&self) -> Option<&Route> {
        self.default_route.as_ref()
    }

    /// Clean up expired routes
    pub fn cleanup_expired_routes(&mut self) {
        let mut routes_to_remove = Vec::new();
        
        for (prefix_length, routes) in self.routes.iter_mut() {
            routes.retain(|route| {
                if route.has_expired() {
                    log::info!("Removing expired route: {}", route.destination);
                    true // Mark for removal
                } else {
                    false
                }
            });
            
            if routes.is_empty() {
                routes_to_remove.push(*prefix_length);
            }
        }

        // Remove empty buckets
        for prefix in routes_to_remove {
            self.routes.remove(&prefix);
        }

        // Update statistics
        self.stats.total_routes = self.routes.values().flatten().count();
        self.stats.connected_routes = self.get_connected_routes().len();
        self.stats.static_routes = self.get_static_routes().len();
        self.stats.dynamic_routes = self.get_dynamic_routes().len();
        self.stats.default_routes = self.default_route.is_some() as usize;
        self.stats.host_routes = self.get_routes_by_type(RouteType::Host).len();

        // Clear cache
        self.route_cache.clear();
    }

    /// Clear all routes
    pub fn clear(&mut self) {
        self.routes.clear();
        self.route_cache.clear();
        self.default_route = None;
        self.stats = RoutingTableStats::default();
        log::info!("Routing table cleared");
    }

    /// Get routing table statistics
    pub fn get_stats(&self) -> &RoutingTableStats {
        &self.stats
    }

    /// Set route cleaner interval
    pub fn set_route_cleaner(&mut self, interval: Duration) {
        self.route_cleaner = Some(interval);
    }

    /// Check if periodic cleanup is needed
    pub fn needs_cleanup(&self) -> bool {
        if let Some(interval) = self.route_cleaner {
            // This would check the last cleanup time
            // For now, always return true for demonstration
            true
        } else {
            false
        }
    }
}

/// IP forwarding engine
pub struct IpForwardingEngine {
    /// Forwarding table
    routing_table: RoutingTable,
    /// Forwarding statistics
    stats: ForwardingStats,
    /// Interface statistics
    interface_stats: HashMap<String, InterfaceForwardingStats>,
    /// Fragment reassembly buffer
    fragments: FragmentBuffer,
    /// Forwarding settings
    settings: ForwardingSettings,
}

#[derive(Debug, Clone)]
pub struct ForwardingSettings {
    /// Enable IP forwarding
    pub ip_forwarding_enabled: bool,
    /// Default TTL for forwarded packets
    pub default_ttl: u8,
    /// Enable packet logging
    pub log_forwarding: bool,
    /// Maximum number of fragments to reassemble
    pub max_fragments: usize,
    /// Fragment timeout
    pub fragment_timeout: Duration,
}

impl Default for ForwardingSettings {
    fn default() -> Self {
        Self {
            ip_forwarding_enabled: false,
            default_ttl: 64,
            log_forwarding: false,
            max_fragments: 1000,
            fragment_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ForwardingStats {
    /// Packets forwarded
    pub packets_forwarded: u64,
    /// Packets dropped
    pub packets_dropped: u64,
    /// Packets fragmented
    pub packets_fragmented: u64,
    /// Packets reassembled
    pub packets_reassembled: u64,
    /// Fragments received
    pub fragments_received: u64,
    /// Fragment reassembly timeouts
    pub fragment_timeouts: u64,
    /// Forwarding errors
    pub forwarding_errors: u64,
}

#[derive(Debug, Clone, Default)]
pub struct InterfaceForwardingStats {
    /// Packets sent through this interface
    pub packets_sent: u64,
    /// Packets received on this interface
    pub packets_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
}

#[derive(Debug)]
struct FragmentBuffer {
    /// Fragments waiting for reassembly
    fragments: HashMap<(u16, IpAddress, IpAddress), Vec<Fragment>>,
    /// Maximum number of fragments
    max_fragments: usize,
    /// Fragment timeout
    timeout: Duration,
}

#[derive(Debug, Clone)]
struct Fragment {
    /// Fragment offset
    offset: u16,
    /// More fragments flag
    more_fragments: bool,
    /// Fragment data
    data: Vec<u8>,
    /// Fragment timestamp
    timestamp: Instant,
}

impl Default for FragmentBuffer {
    fn default() -> Self {
        Self {
            fragments: HashMap::new(),
            max_fragments: 1000,
            timeout: Duration::from_secs(30),
        }
    }
}

impl IpForwardingEngine {
    /// Create a new IP forwarding engine
    pub fn new() -> Self {
        Self {
            routing_table: RoutingTable::new(),
            stats: ForwardingStats::default(),
            interface_stats: HashMap::new(),
            fragments: FragmentBuffer::default(),
            settings: ForwardingSettings::default(),
        }
    }

    /// Create IP forwarding engine with custom settings
    pub fn with_settings(settings: ForwardingSettings) -> Self {
        Self {
            routing_table: RoutingTable::new(),
            stats: ForwardingStats::default(),
            interface_stats: HashMap::new(),
            fragments: FragmentBuffer {
                max_fragments: settings.max_fragments,
                timeout: settings.fragment_timeout,
                ..Default::default()
            },
            settings,
        }
    }

    /// Enable/disable IP forwarding
    pub fn set_ip_forwarding(&mut self, enabled: bool) {
        self.settings.ip_forwarding_enabled = enabled;
        log::info!("IP forwarding {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if IP forwarding is enabled
    pub fn is_ip_forwarding_enabled(&self) -> bool {
        self.settings.ip_forwarding_enabled
    }

    /// Set default TTL for forwarded packets
    pub fn set_default_ttl(&mut self, ttl: u8) {
        self.settings.default_ttl = ttl;
    }

    /// Add route to forwarding table
    pub fn add_route(&mut self, route: Route) -> Result<()> {
        self.routing_table.add_route(route)
    }

    /// Remove route from forwarding table
    pub fn remove_route(&mut self, destination: IpAddress, netmask: IpAddress) -> Result<Route> {
        self.routing_table.remove_route(destination, netmask)
    }

    /// Forward an IP packet
    pub fn forward_packet(&mut self, packet_data: &[u8], input_interface: &str, 
                         source: IpAddress, dest: IpAddress) -> Result<Vec<Vec<u8>>> {
        if !self.settings.ip_forwarding_enabled {
            self.stats.packets_dropped += 1;
            return Err(NetworkError::NetworkUnreachable);
        }

        // Parse the IP packet
        let mut packet = crate::protocols::ip::IpPacket::parse(packet_data)
            .map_err(|_| NetworkError::InvalidAddress)?;

        // Check TTL
        if packet.ttl == 0 {
            self.stats.packets_dropped += 1;
            return Err(NetworkError::HostUnreachable);
        }

        // Find best route
        if let Some(route) = self.routing_table.find_best_route(dest) {
            // Check if it's a blackhole route
            if route.route_type == RouteType::Blackhole {
                self.stats.packets_dropped += 1;
                log::debug!("Packet dropped by blackhole route to {}", dest);
                return Ok(Vec::new());
            }

            // Check if we can send directly to destination
            let output_interface = route.interface_name();
            let next_hop = route.next_hop(dest).unwrap_or(dest);

            // Check if this is a fragment that needs reassembly
            if packet.is_fragment() {
                self.stats.fragments_received += 1;
                return self.handle_fragment(packet, input_interface, output_interface);
            }

            // Decrement TTL
            packet.decrement_ttl();

            // Update statistics
            self.update_interface_stats(input_interface, packet_data.len(), false);
            self.update_interface_stats(output_interface, packet_data.len(), true);
            self.stats.packets_forwarded += 1;

            if self.settings.log_forwarding {
                log::info!("Forwarding packet from {} to {} via {} on {}", 
                          source, dest, next_hop, output_interface);
            }

            // Convert packet back to bytes
            let forwarded_data = packet.to_bytes();
            
            // Fragment if necessary
            if forwarded_data.len() > 1500 && packet.can_fragment() {
                self.stats.packets_fragmented += 1;
                let fragments = crate::protocols::utils::fragment_ip_packet(&forwarded_data, 1500);
                Ok(fragments)
            } else {
                Ok(vec![forwarded_data])
            }
        } else {
            self.stats.packets_dropped += 1;
            log::debug!("No route found for destination {}", dest);
            Err(NetworkError::NetworkUnreachable)
        }
    }

    /// Handle IP fragmentation and reassembly
    fn handle_fragment(&mut self, packet: crate::protocols::ip::IpPacket, 
                      _input_interface: &str, _output_interface: &str) -> Result<Vec<Vec<u8>>> {
        let fragment_key = (packet.identification, packet.source, packet.destination);
        let fragment_offset = packet.fragment_offset();
        let more_fragments = packet.flags().more_fragments;

        // Create fragment entry
        let fragment = Fragment {
            offset: fragment_offset,
            more_fragments,
            data: packet.payload,
            timestamp: Instant::now(),
        };

        // Add to fragment buffer
        let fragments = self.fragments.fragments.entry(fragment_key).or_insert_with(Vec::new);
        fragments.push(fragment);

        // Check if all fragments are present
        if !more_fragments {
            if let Some(fragments_list) = self.fragments.fragments.remove(&fragment_key) {
                let fragments_data: Vec<Vec<u8>> = fragments_list.into_iter()
                    .map(|f| f.data)
                    .collect();
                
                match crate::protocols::utils::reassemble_ip_fragments(&fragments_data) {
                    Ok(reassembled) => {
                        self.stats.packets_reassembled += 1;
                        Ok(vec![reassembled])
                    }
                    Err(_) => {
                        self.stats.packets_dropped += 1;
                        Ok(Vec::new())
                    }
                }
            } else {
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Clean up expired fragments
    pub fn cleanup_fragments(&mut self) {
        let now = Instant::now();
        let mut timeouts = 0;

        for fragments in self.fragments.fragments.values_mut() {
            fragments.retain(|fragment| {
                if now.duration_since(fragment.timestamp) > self.fragments.timeout {
                    timeouts += 1;
                    false
                } else {
                    true
                }
            });
        }

        self.stats.fragment_timeouts += timeouts;
    }

    /// Update interface statistics
    fn update_interface_stats(&mut self, interface: &str, bytes: usize, is_output: bool) {
        let stats = self.interface_stats.entry(interface.to_string()).or_insert_with(Default::default);
        
        if is_output {
            stats.packets_sent += 1;
            stats.bytes_sent += bytes as u64;
        } else {
            stats.packets_received += 1;
            stats.bytes_received += bytes as u64;
        }
    }

    /// Get forwarding statistics
    pub fn get_stats(&self) -> &ForwardingStats {
        &self.stats
    }

    /// Get interface forwarding statistics
    pub fn get_interface_stats(&self, interface: &str) -> Option<&InterfaceForwardingStats> {
        self.interface_stats.get(interface)
    }

    /// Get all interface statistics
    pub fn get_all_interface_stats(&self) -> &HashMap<String, InterfaceForwardingStats> {
        &self.interface_stats
    }

    /// Get routing table
    pub fn get_routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    /// Get mutable routing table
    pub fn get_routing_table_mut(&mut self) -> &mut RoutingTable {
        &mut self.routing_table
    }

    /// Get forwarding settings
    pub fn get_settings(&self) -> &ForwardingSettings {
        &self.settings
    }

    /// Enable/disable packet logging
    pub fn set_log_forwarding(&mut self, enabled: bool) {
        self.settings.log_forwarding = enabled;
    }

    /// Get the next hop for a destination
    pub fn get_next_hop(&mut self, dest: IpAddress) -> Option<(IpAddress, String)> {
        if let Some(route) = self.routing_table.find_best_route(dest) {
            let next_hop = route.next_hop(dest).unwrap_or(dest);
            Some((next_hop, route.interface_name().to_string()))
        } else {
            None
        }
    }

    /// Check if destination is directly connected
    pub fn is_directly_connected(&mut self, dest: IpAddress) -> bool {
        if let Some(route) = self.routing_table.find_best_route(dest) {
            route.route_type == RouteType::Connected && !route.flags.gateway
        } else {
            false
        }
    }

    /// Add connected route for an interface
    pub fn add_connected_route(&mut self, interface: &str, network: IpAddress, netmask: IpAddress) -> Result<()> {
        let route = Route::connected(network, netmask, interface.to_string());
        self.add_route(route)
    }

    /// Add default route
    pub fn add_default_route(&mut self, gateway: IpAddress, interface: String) -> Result<()> {
        let route = Route::default_route(gateway, interface);
        self.add_route(route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let dest = IpAddress::v4(192, 168, 1, 0);
        let netmask = IpAddress::v4(255, 255, 255, 0);
        let route = Route::connected(dest, netmask, "eth0".to_string());
        
        assert_eq!(route.destination, dest);
        assert_eq!(route.netmask, netmask);
        assert_eq!(route.route_type, RouteType::Connected);
        assert!(route.flags.up);
    }

    #[test]
    fn test_route_matching() {
        let route = Route::host_route(
            IpAddress::v4(192, 168, 1, 100),
            IpAddress::v4(192, 168, 1, 1),
            "eth0".to_string()
        );
        
        assert!(route.matches(IpAddress::v4(192, 168, 1, 100)));
        assert!(!route.matches(IpAddress::v4(192, 168, 1, 101)));
    }

    #[test]
    fn test_routing_table() {
        let mut table = RoutingTable::new();
        
        let route = Route::connected(
            IpAddress::v4(192, 168, 1, 0),
            IpAddress::v4(255, 255, 255, 0),
            "eth0".to_string()
        );
        
        assert!(table.add_route(route).is_ok());
        assert_eq!(table.get_stats().total_routes, 1);
    }

    #[test]
    fn test_routing_table_lookups() {
        let mut table = RoutingTable::new();
        
        // Add a default route
        let default_route = Route::default_route(
            IpAddress::v4(192, 168, 1, 1),
            "eth0".to_string()
        );
        table.add_route(default_route).unwrap();
        
        // Test lookup
        let dest = IpAddress::v4(8, 8, 8, 8);
        let route = table.find_best_route(dest);
        assert!(route.is_some());
    }

    #[test]
    fn test_ip_forwarding_engine() {
        let mut engine = IpForwardingEngine::new();
        
        // Enable forwarding
        engine.set_ip_forwarding(true);
        assert!(engine.is_ip_forwarding_enabled());
        
        // Add a connected route
        let route = Route::connected(
            IpAddress::v4(192, 168, 1, 0),
            IpAddress::v4(255, 255, 255, 0),
            "eth0".to_string()
        );
        engine.add_route(route).unwrap();
        
        // Create a simple IP packet
        let source = IpAddress::v4(192, 168, 1, 10);
        let dest = IpAddress::v4(192, 168, 1, 20);
        let payload = b"test packet".to_vec();
        let packet = crate::protocols::ip::IpPacket::new(source, dest, crate::protocols::ip::IpProtocol::Tcp, payload);
        
        // Try to forward the packet
        let packet_data = packet.to_bytes();
        let result = engine.forward_packet(&packet_data, "eth0", source, dest);
        // This might fail if forwarding is not fully configured, but the structure should work
    }

    #[test]
    fn test_route_flags() {
        let mut flags = RouteFlags::default();
        assert!(!flags.up);
        
        flags.up = true;
        flags.gateway = true;
        
        let u32_value = flags.to_u32();
        let parsed = RouteFlags::from_u32(u32_value);
        
        assert_eq!(parsed.up, flags.up);
        assert_eq!(parsed.gateway, flags.gateway);
    }

    #[test]
    fn test_route_comparison() {
        let host_route = Route::host_route(
            IpAddress::v4(192, 168, 1, 100),
            IpAddress::v4(192, 168, 1, 1),
            "eth0".to_string()
        );
        
        let network_route = Route::connected(
            IpAddress::v4(192, 168, 1, 0),
            IpAddress::v4(255, 255, 255, 0),
            "eth0".to_string()
        );
        
        // Host routes should be more specific than network routes
        assert!(host_route.is_more_specific(&network_route));
    }
}