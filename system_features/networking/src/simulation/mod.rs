//! Network simulation and testing framework
//!
//! This module provides comprehensive network simulation capabilities for educational
//! and testing purposes, including topology simulation, traffic generation,
//! network conditions simulation, and protocol testing.

use crate::{Result, NetworkError};
use crate::core::{IpAddress, NetworkInterface};
use crate::protocols::ip::IpPacket;
use crate::sockets::{TcpSocket, UdpSocket, SocketAddr};
use crate::dns::DnsResolver;
use crate::routing::{RoutingTable, Route};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use rand::Rng;

/// Network topology node
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// Node identifier
    pub id: String,
    /// Node IP address
    pub ip_address: IpAddress,
    /// Node type
    pub node_type: NodeType,
    /// Connected links
    pub connections: Vec<ConnectionId>,
    /// Node configuration
    pub config: NodeConfig,
    /// Statistics
    pub stats: NodeStatistics,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    /// Host computer
    Host,
    /// Router/gateway
    Router,
    /// Switch
    Switch,
    /// Server
    Server,
    /// Firewall
    Firewall,
    /// Load balancer
    LoadBalancer,
    /// Database server
    Database,
    /// Web server
    WebServer,
    /// DNS server
    DnsServer,
    /// File server
    FileServer,
    /// Custom application
    Application(String),
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u32,
    /// Network bandwidth (Mbps)
    pub bandwidth_mbps: u32,
    /// Network latency (ms)
    pub latency_ms: u32,
    /// Packet loss rate (0.0 to 1.0)
    pub packet_loss_rate: f64,
    /// Jitter (ms)
    pub jitter_ms: u32,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Application services
    pub services: Vec<ServiceInfo>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Port number
    pub port: u16,
    /// Protocol
    pub protocol: String,
    /// Service status
    pub enabled: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            cpu_cores: 2,
            memory_mb: 4096,
            bandwidth_mbps: 1000,
            latency_ms: 1,
            packet_loss_rate: 0.0,
            jitter_ms: 0,
            max_connections: 1000,
            services: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeStatistics {
    /// Total packets sent
    pub packets_sent: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Errors encountered
    pub errors: u64,
    /// Connections established
    pub connections_established: u64,
    /// Active connections
    pub active_connections: u32,
    /// Uptime
    pub uptime: Duration,
    /// Last activity
    pub last_activity: Instant,
}

/// Network link/connection between nodes
#[derive(Debug, Clone)]
pub struct NetworkLink {
    /// Connection identifier
    pub id: ConnectionId,
    /// Source node
    pub source_node: String,
    /// Destination node
    pub dest_node: String,
    /// Link configuration
    pub config: LinkConfig,
    /// Link statistics
    pub stats: LinkStatistics,
    /// Current state
    pub state: LinkState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionId {
    pub id: u32,
}

impl ConnectionId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone)]
pub struct LinkConfig {
    /// Link bandwidth (Mbps)
    pub bandwidth_mbps: u32,
    /// Link latency (ms)
    pub latency_ms: u32,
    /// Packet loss rate (0.0 to 1.0)
    pub packet_loss_rate: f64,
    /// Jitter (ms)
    pub jitter_ms: u32,
    /// Queue size (packets)
    pub queue_size: usize,
    /// Link type
    pub link_type: LinkType,
    /// Directional (true = bidirectional, false = unidirectional)
    pub bidirectional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Ethernet
    Ethernet,
    /// Fiber optic
    Fiber,
    /// WiFi
    WiFi,
    /// DSL
   Dsl,
    /// Cable
    Cable,
    /// Satellite
    Satellite,
    /// VPN tunnel
    VpnTunnel,
    /// Serial
    Serial,
    /// Virtual
    Virtual,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            bandwidth_mbps: 1000,
            latency_ms: 1,
            packet_loss_rate: 0.0,
            jitter_ms: 0,
            queue_size: 1000,
            link_type: LinkType::Ethernet,
            bidirectional: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LinkStatistics {
    /// Total packets transmitted
    pub packets_transmitted: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Total bytes transmitted
    pub bytes_transmitted: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Packets dropped
    pub packets_dropped: u64,
    /// Queue size
    pub current_queue_size: usize,
    /// Peak queue size
    pub peak_queue_size: usize,
    /// Link utilization
    pub utilization: f64,
    /// Errors
    pub errors: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// Link is up and operational
    Up,
    /// Link is down
    Down,
    /// Link is congested
    Congested,
    /// Link is in maintenance
    Maintenance,
    /// Link has errors
    Error,
}

/// Network packet in simulation
#[derive(Debug, Clone)]
pub struct SimulatedPacket {
    /// Packet identifier
    pub id: PacketId,
    /// Source node
    pub source: String,
    /// Destination node
    pub dest: String,
    /// Packet data
    pub data: Vec<u8>,
    /// Packet type
    pub packet_type: PacketType,
    /// Creation time
    pub created_at: Instant,
    /// Expected delivery time
    pub expected_delivery: Instant,
    /// Current hop
    pub current_hop: usize,
    /// Route path
    pub route_path: Vec<String>,
    /// Priority
    pub priority: PacketPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketId {
    pub id: u64,
}

impl PacketId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    /// TCP packet
    Tcp,
    /// UDP packet
    Udp,
    /// ICMP packet
    Icmp,
    /// ARP packet
    Arp,
    /// DNS packet
    Dns,
    /// HTTP request
    HttpRequest,
    /// HTTP response
    HttpResponse,
    /// Custom application packet
    Custom(String),
    /// Network control packet
    Control,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

/// Network simulation environment
pub struct NetworkSimulator {
    /// Network topology
    topology: NetworkTopology,
    /// Simulation parameters
    config: SimulationConfig,
    /// Event queue
    event_queue: EventQueue,
    /// Traffic generator
    traffic_generator: TrafficGenerator,
    /// Network conditions
    conditions: NetworkConditions,
    /// Statistics
    stats: SimulationStatistics,
    /// Simulation state
    state: SimulationState,
}

#[derive(Debug, Clone)]
pub struct NetworkTopology {
    /// Network nodes
    pub nodes: HashMap<String, NetworkNode>,
    /// Network links
    pub links: Vec<NetworkLink>,
    /// Network configuration
    pub config: TopologyConfig,
}

#[derive(Debug, Clone)]
pub struct TopologyConfig {
    /// Network name
    pub name: String,
    /// Network type
    pub network_type: NetworkType,
    /// Default configuration
    pub default_config: NodeConfig,
    /// Security settings
    pub security_enabled: bool,
    /// QoS enabled
    pub qos_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    /// Local Area Network
    Lan,
    /// Wide Area Network
    Wan,
    /// Campus Network
    Campus,
    /// Data Center
    DataCenter,
    /// Enterprise Network
    Enterprise,
    /// Internet Simulation
    Internet,
    /// Custom topology
    Custom,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            name: "Default Network".to_string(),
            network_type: NetworkType::Lan,
            default_config: NodeConfig::default(),
            security_enabled: false,
            qos_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Simulation duration
    pub duration: Duration,
    /// Simulation speed multiplier
    pub speed_multiplier: f64,
    /// Enable logging
    pub enable_logging: bool,
    /// Enable statistics collection
    pub collect_stats: bool,
    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
    /// Real-time simulation
    pub real_time: bool,
    /// Packet capture enabled
    pub packet_capture: bool,
    /// Debug mode
    pub debug_mode: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(300),
            speed_multiplier: 1.0,
            enable_logging: true,
            collect_stats: true,
            random_seed: None,
            real_time: false,
            packet_capture: false,
            debug_mode: false,
        }
    }
}

/// Event queue for simulation events
struct EventQueue {
    events: VecDeque<SimulationEvent>,
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    /// Packet transmission event
    PacketTransmit {
        packet: SimulatedPacket,
        link_id: ConnectionId,
        timestamp: Instant,
    },
    /// Packet delivery event
    PacketDeliver {
        packet_id: PacketId,
        destination: String,
        timestamp: Instant,
    },
    /// Link failure event
    LinkFailure {
        link_id: ConnectionId,
        timestamp: Instant,
    },
    /// Link recovery event
    LinkRecovery {
        link_id: ConnectionId,
        timestamp: Instant,
    },
    /// Network congestion event
    Congestion {
        link_id: ConnectionId,
        timestamp: Instant,
    },
    /// Custom event
    Custom {
        event_type: String,
        data: String,
        timestamp: Instant,
    },
}

impl EventQueue {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    fn add_event(&mut self, event: SimulationEvent) {
        self.events.push_back(event);
    }

    fn get_next_event(&mut self) -> Option<SimulationEvent> {
        self.events.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    fn len(&self) -> usize {
        self.events.len()
    }
}

/// Traffic generator for creating network traffic
pub struct TrafficGenerator {
    /// Traffic patterns
    patterns: Vec<TrafficPattern>,
    /// Active flows
    flows: HashMap<String, TrafficFlow>,
    /// Generator configuration
    config: TrafficGeneratorConfig,
}

#[derive(Debug, Clone)]
pub struct TrafficPattern {
    /// Pattern name
    pub name: String,
    /// Pattern type
    pub pattern_type: TrafficPatternType,
    /// Source node
    pub source_node: String,
    /// Destination node
    pub dest_node: String,
    /// Packet rate (packets per second)
    pub packet_rate: f64,
    /// Packet size range
    pub packet_size_range: (usize, usize),
    /// Duration
    pub duration: Duration,
    /// Start time
    pub start_time: Duration,
    /// Priority
    pub priority: PacketPriority,
}

#[derive(Debug, Clone, Copy)]
pub enum TrafficPatternType {
    /// Constant bit rate
    Constant,
    /// Variable bit rate
    Variable,
    /// Burst traffic
    Burst,
    /// Poisson traffic
    Poisson,
    /// Web traffic pattern
    WebTraffic,
    /// Email traffic pattern
    EmailTraffic,
    /// VoIP traffic pattern
    VoipTraffic,
    /// Video streaming pattern
    VideoStreaming,
    /// File transfer pattern
    FileTransfer,
    /// Random pattern
    Random,
}

#[derive(Debug, Clone)]
pub struct TrafficFlow {
    /// Flow identifier
    pub id: String,
    /// Source node
    pub source: String,
    /// Destination node
    pub destination: String,
    /// Flow type
    pub flow_type: TrafficPatternType,
    /// Packets sent
    pub packets_sent: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Flow start time
    pub start_time: Instant,
    /// Flow status
    pub status: FlowStatus,
    /// Average packet size
    pub avg_packet_size: usize,
    /// Actual packet rate
    pub actual_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TrafficGeneratorConfig {
    /// Maximum concurrent flows
    pub max_concurrent_flows: usize,
    /// Default packet size
    pub default_packet_size: usize,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Flow timeout
    pub flow_timeout: Duration,
}

impl Default for TrafficGeneratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_flows: 1000,
            default_packet_size: 512,
            rate_limiting: true,
            flow_timeout: Duration::from_secs(30),
        }
    }
}

/// Network conditions simulation
pub struct NetworkConditions {
    /// Current conditions
    current_conditions: CurrentConditions,
    /// Condition changes
    condition_changes: Vec<ConditionChange>,
    /// Random event generator
    rng: rand::ThreadRng,
}

#[derive(Debug, Clone)]
pub struct CurrentConditions {
    /// Global latency multiplier
    pub latency_multiplier: f64,
    /// Global packet loss rate
    pub global_loss_rate: f64,
    /// Global bandwidth reduction
    pub bandwidth_reduction: f64,
    /// Active failures
    pub active_failures: Vec<FailureEvent>,
    /// Scheduled maintenance
    pub scheduled_maintenance: Vec<MaintenanceEvent>,
}

#[derive(Debug, Clone)]
pub struct FailureEvent {
    /// Failed component
    pub component_id: String,
    /// Failure type
    pub failure_type: FailureType,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Duration,
    /// Recovery probability
    pub recovery_probability: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum FailureType {
    LinkFailure,
    NodeFailure,
    RouterFailure,
    DnsFailure,
    PowerOutage,
    HardwareMalfunction,
    SoftwareCrash,
    DDoSAttack,
    NaturalDisaster,
}

#[derive(Debug, Clone)]
pub struct MaintenanceEvent {
    /// Component under maintenance
    pub component_id: String,
    /// Maintenance type
    pub maintenance_type: MaintenanceType,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Duration,
    /// Impact level
    pub impact_level: ImpactLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum MaintenanceType {
    Planned,
    Emergency,
    Update,
    Security,
    Performance,
    Backup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ConditionChange {
    /// Change type
    pub change_type: ConditionChangeType,
    /// Affected component
    pub component_id: Option<String>,
    /// Change parameters
    pub parameters: HashMap<String, String>,
    /// Scheduled time
    pub scheduled_time: Option<Instant>,
    /// Probability
    pub probability: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionChangeType {
    LatencyIncrease,
    LatencyDecrease,
    PacketLossIncrease,
    PacketLossDecrease,
    BandwidthReduction,
    BandwidthIncrease,
    LinkFailure,
    LinkRecovery,
    NodeFailure,
    NodeRecovery,
    Congestion,
    Decongestion,
}

impl NetworkConditions {
    fn new() -> Self {
        Self {
            current_conditions: CurrentConditions {
                latency_multiplier: 1.0,
                global_loss_rate: 0.0,
                bandwidth_reduction: 1.0,
                active_failures: Vec::new(),
                scheduled_maintenance: Vec::new(),
            },
            condition_changes: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    /// Simulate network conditions
    fn simulate_conditions(&mut self) {
        // Check for random failures
        if self.rng.gen_bool(0.001) { // 0.1% chance per simulation step
            self.trigger_random_failure();
        }

        // Check for scheduled condition changes
        self.process_condition_changes();
    }

    fn trigger_random_failure(&mut self) {
        let failure_types = vec![
            FailureType::LinkFailure,
            FailureType::NodeFailure,
            FailureType::PowerOutage,
            FailureType::DDoSAttack,
        ];

        let failure_type = failure_types[self.rng.gen_range(0..failure_types.len())];
        let duration = Duration::from_secs(self.rng.gen_range(10..300));
        let recovery_probability = self.rng.gen_range(0.5..0.95);

        // Note: In a real implementation, this would affect actual network components
        log::debug!("Simulating {:?} failure for {:?}", failure_type, duration);
    }

    fn process_condition_changes(&mut self) {
        // Process scheduled condition changes
        let now = Instant::now();
        self.condition_changes.retain(|change| {
            if let Some(scheduled_time) = change.scheduled_time {
                if now >= scheduled_time && self.rng.gen_bool(change.probability as f64) {
                    self.apply_condition_change(change);
                    false // Remove after applying
                } else {
                    true // Keep for later
                }
            } else {
                true
            }
        });
    }

    fn apply_condition_change(&self, change: &ConditionChange) {
        // Apply the condition change based on type
        match change.change_type {
            ConditionChangeType::LatencyIncrease => {
                log::info!("Applying latency increase condition");
            }
            ConditionChangeType::PacketLossIncrease => {
                log::info!("Applying packet loss increase condition");
            }
            ConditionChangeType::BandwidthReduction => {
                log::info!("Applying bandwidth reduction condition");
            }
            _ => {
                log::debug!("Applying condition change: {:?}", change.change_type);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimulationStatistics {
    /// Total packets simulated
    pub total_packets: u64,
    /// Packets delivered successfully
    pub packets_delivered: u64,
    /// Packets dropped
    pub packets_dropped: u64,
    /// Average latency
    pub avg_latency: Duration,
    /// Total latency measurement time
    pub total_latency_measurement: Duration,
    /// Latency measurements
    pub latency_measurements: u64,
    /// Network utilization
    pub network_utilization: f64,
    /// Peak network utilization
    pub peak_network_utilization: f64,
    /// Number of flows established
    pub flows_established: u64,
    /// Number of flow failures
    pub flow_failures: u64,
    /// Simulation duration
    pub simulation_duration: Duration,
    /// Events processed
    pub events_processed: u64,
    /// Simulation efficiency
    pub simulation_efficiency: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    /// Simulation is stopped
    Stopped,
    /// Simulation is running
    Running,
    /// Simulation is paused
    Paused,
    /// Simulation is completed
    Completed,
    /// Simulation is in error state
    Error,
}

impl NetworkSimulator {
    /// Create a new network simulator
    pub fn new(config: SimulationConfig) -> Self {
        let topology = NetworkTopology {
            nodes: HashMap::new(),
            links: Vec::new(),
            config: TopologyConfig::default(),
        };

        Self {
            topology,
            config,
            event_queue: EventQueue::new(),
            traffic_generator: TrafficGenerator {
                patterns: Vec::new(),
                flows: HashMap::new(),
                config: TrafficGeneratorConfig::default(),
            },
            conditions: NetworkConditions::new(),
            stats: SimulationStatistics::default(),
            state: SimulationState::Stopped,
        }
    }

    /// Create simulator with default configuration
    pub fn with_default_config() -> Self {
        Self::new(SimulationConfig::default())
    }

    /// Add a network node
    pub fn add_node(&mut self, node: NetworkNode) {
        self.topology.nodes.insert(node.id.clone(), node);
        log::info!("Added network node: {}", node.id);
    }

    /// Add a network link
    pub fn add_link(&mut self, link: NetworkLink) {
        self.topology.links.push(link);
        log::info!("Added network link: {} <-> {}", link.source_node, link.dest_node);
    }

    /// Remove a network node
    pub fn remove_node(&mut self, node_id: &str) -> Result<NetworkNode> {
        if let Some(node) = self.topology.nodes.remove(node_id) {
            // Remove associated links
            self.topology.links.retain(|link| 
                link.source_node != node_id && link.dest_node != node_id
            );
            log::info!("Removed network node: {}", node_id);
            Ok(node)
        } else {
            Err(NetworkError::Other(format!("Node not found: {}", node_id).into()))
        }
    }

    /// Remove a network link
    pub fn remove_link(&mut self, link_id: ConnectionId) -> Result<NetworkLink> {
        if let Some(index) = self.topology.links.iter().position(|link| link.id == link_id) {
            let link = self.topology.links.remove(index);
            log::info!("Removed network link: {} <-> {}", link.source_node, link.dest_node);
            Ok(link)
        } else {
            Err(NetworkError::Other(format!("Link not found: {:?}", link_id).into()))
        }
    }

    /// Start the simulation
    pub fn start(&mut self) -> Result<()> {
        self.state = SimulationState::Running;
        self.stats.simulation_duration = Duration::from_secs(0);
        log::info!("Starting network simulation");
        
        // Initialize traffic generation
        self.initialize_traffic_generation();
        
        Ok(())
    }

    /// Stop the simulation
    pub fn stop(&mut self) {
        self.state = SimulationState::Stopped;
        log::info!("Stopping network simulation");
    }

    /// Pause the simulation
    pub fn pause(&mut self) {
        self.state = SimulationState::Paused;
        log::info!("Pausing network simulation");
    }

    /// Resume the simulation
    pub fn resume(&mut self) {
        self.state = SimulationState::Running;
        log::info!("Resuming network simulation");
    }

    /// Run simulation step
    pub fn step(&mut self, dt: Duration) -> Result<()> {
        if self.state != SimulationState::Running {
            return Ok(());
        }

        let start_time = Instant::now();

        // Update simulation time
        self.stats.simulation_duration += dt;

        // Process events
        self.process_events();

        // Generate traffic
        self.generate_traffic(dt);

        // Simulate network conditions
        self.conditions.simulate_conditions();

        // Process packet delivery
        self.process_packet_delivery();

        // Update statistics
        self.update_statistics();

        // Check for simulation completion
        if self.stats.simulation_duration >= self.config.duration {
            self.state = SimulationState::Completed;
            log::info!("Simulation completed");
        }

        // Calculate simulation efficiency
        let processing_time = start_time.elapsed();
        self.stats.simulation_efficiency = (dt.as_secs_f64() / processing_time.as_secs_f64()) * self.config.speed_multiplier;

        Ok(())
    }

    /// Initialize traffic generation
    fn initialize_traffic_generation(&mut self) {
        // Create default traffic patterns if none exist
        if self.traffic_generator.patterns.is_empty() {
            self.create_default_traffic_patterns();
        }

        log::info!("Initialized traffic generation with {} patterns", self.traffic_generator.patterns.len());
    }

    fn create_default_traffic_patterns(&mut self) {
        let nodes: Vec<String> = self.topology.nodes.keys().cloned().collect();
        
        if nodes.len() >= 2 {
            // Web traffic pattern
            let web_pattern = TrafficPattern {
                name: "Web Traffic".to_string(),
                pattern_type: TrafficPatternType::WebTraffic,
                source_node: nodes[0].clone(),
                dest_node: nodes[1].clone(),
                packet_rate: 10.0,
                packet_size_range: (100, 1500),
                duration: self.config.duration,
                start_time: Duration::from_secs(0),
                priority: PacketPriority::Normal,
            };
            self.traffic_generator.patterns.push(web_pattern);

            // Ping traffic pattern
            let ping_pattern = TrafficPattern {
                name: "Ping Test".to_string(),
                pattern_type: TrafficPatternType::Constant,
                source_node: nodes[0].clone(),
                dest_node: nodes[1].clone(),
                packet_rate: 1.0,
                packet_size_range: (64, 64),
                duration: self.config.duration,
                start_time: Duration::from_secs(0),
                priority: PacketPriority::High,
            };
            self.traffic_generator.patterns.push(ping_pattern);
        }
    }

    /// Process simulation events
    fn process_events(&mut self) {
        let max_events_per_step = 1000; // Prevent event queue explosion
        let mut events_processed = 0;

        while !self.event_queue.is_empty() && events_processed < max_events_per_step {
            if let Some(event) = self.event_queue.get_next_event() {
                self.process_event(&event);
                events_processed += 1;
                self.stats.events_processed += 1;
            }
        }

        if events_processed == max_events_per_step {
            log::warn!("Event queue processing limit reached");
        }
    }

    fn process_event(&mut self, event: &SimulationEvent) {
        match event {
            SimulationEvent::PacketTransmit { packet, link_id, .. } => {
                self.process_packet_transmit(packet, *link_id);
            }
            SimulationEvent::PacketDeliver { packet_id, destination, .. } => {
                self.process_packet_delivery_event(packet_id, destination);
            }
            SimulationEvent::LinkFailure { link_id, .. } => {
                self.process_link_failure(*link_id);
            }
            SimulationEvent::LinkRecovery { link_id, .. } => {
                self.process_link_recovery(*link_id);
            }
            _ => {
                // Handle other event types
            }
        }
    }

    fn process_packet_transmit(&mut self, packet: &SimulatedPacket, link_id: ConnectionId) {
        // Find the link
        if let Some(link) = self.topology.links.iter_mut().find(|l| l.id == link_id) {
            // Update link statistics
            link.stats.packets_transmitted += 1;
            link.stats.bytes_transmitted += packet.data.len() as u64;
            
            // Simulate transmission delay
            let transmission_time = self.calculate_transmission_time(packet, &link.config);
            
            // Schedule delivery event
            let delivery_event = SimulationEvent::PacketDeliver {
                packet_id: packet.id,
                destination: packet.dest.clone(),
                timestamp: Instant::now() + transmission_time,
            };
            self.event_queue.add_event(delivery_event);

            self.stats.total_packets += 1;
        }
    }

    fn process_packet_delivery_event(&mut self, packet_id: &PacketId, destination: &str) {
        // Find destination node
        if let Some(node) = self.topology.nodes.get_mut(destination) {
            node.stats.packets_received += 1;
            node.stats.bytes_received += 1024; // Approximate packet size
            
            // Calculate latency
            let latency = Instant::now().duration_since(Instant::now()); // Placeholder
            self.update_latency_stats(latency);
            
            self.stats.packets_delivered += 1;
            node.stats.last_activity = Instant::now();
        } else {
            self.stats.packets_dropped += 1;
        }
    }

    fn process_link_failure(&mut self, link_id: ConnectionId) {
        if let Some(link) = self.topology.links.iter_mut().find(|l| l.id == link_id) {
            link.state = LinkState::Down;
            log::warn!("Link {} is now down", link_id.id);
        }
    }

    fn process_link_recovery(&mut self, link_id: ConnectionId) {
        if let Some(link) = self.topology.links.iter_mut().find(|l| l.id == link_id) {
            link.state = LinkState::Up;
            log::info!("Link {} has recovered", link_id.id);
        }
    }

    /// Generate traffic
    fn generate_traffic(&mut self, dt: Duration) {
        // Process traffic patterns
        for pattern in &self.traffic_generator.patterns {
            self.process_traffic_pattern(pattern, dt);
        }

        // Clean up completed flows
        self.traffic_generator.flows.retain(|_, flow| {
            flow.status != FlowStatus::Completed
        });
    }

    fn process_traffic_pattern(&mut self, pattern: &TrafficPattern, dt: Duration) {
        // Check if pattern is active
        let elapsed = self.stats.simulation_duration;
        if elapsed < pattern.start_time || elapsed > pattern.start_time + pattern.duration {
            return;
        }

        // Generate packets based on pattern type
        match pattern.pattern_type {
            TrafficPatternType::Constant => {
                self.generate_constant_traffic(pattern, dt);
            }
            TrafficPatternType::Variable => {
                self.generate_variable_traffic(pattern, dt);
            }
            TrafficPatternType::Burst => {
                self.generate_burst_traffic(pattern, dt);
            }
            TrafficPatternType::Poisson => {
                self.generate_poisson_traffic(pattern, dt);
            }
            TrafficPatternType::WebTraffic => {
                self.generate_web_traffic(pattern, dt);
            }
            _ => {
                self.generate_constant_traffic(pattern, dt);
            }
        }
    }

    fn generate_constant_traffic(&mut self, pattern: &TrafficPattern, dt: Duration) {
        let packets_to_generate = (pattern.packet_rate * dt.as_secs_f64()) as u64;
        
        for _ in 0..packets_to_generate {
            self.create_packet(pattern);
        }
    }

    fn generate_variable_traffic(&mut self, pattern: &TrafficPattern, dt: Duration) {
        let mut rng = rand::thread_rng();
        let rate_variation = rng.gen_range(0.5..1.5);
        let adjusted_rate = pattern.packet_rate * rate_variation;
        let packets_to_generate = (adjusted_rate * dt.as_secs_f64()) as u64;
        
        for _ in 0..packets_to_generate {
            self.create_packet(pattern);
        }
    }

    fn generate_burst_traffic(&mut self, pattern: &TrafficPattern, dt: Duration) {
        let mut rng = rand::thread_rng();
        let burst_probability = 0.1; // 10% chance of burst
        
        if rng.gen_bool(burst_probability) {
            let burst_size = rng.gen_range(10..100);
            for _ in 0..burst_size {
                self.create_packet(pattern);
            }
        } else {
            self.generate_constant_traffic(pattern, dt);
        }
    }

    fn generate_poisson_traffic(&mut self, pattern: &TrafficPattern, dt: Duration) {
        let lambda = pattern.packet_rate * dt.as_secs_f64();
        let mut rng = rand::thread_rng();
        
        // Generate number of events using Poisson distribution (simplified)
        let events = rng.gen_range(0..(lambda * 2.0) as u32);
        
        for _ in 0..events {
            self.create_packet(pattern);
        }
    }

    fn generate_web_traffic(&mut self, pattern: &TrafficPattern, dt: Duration) {
        let mut rng = rand::thread_rng();
        
        // Simulate web traffic patterns
        let packet_type = match rng.gen_range(0..10) {
            0..=7 => PacketType::HttpRequest,
            8..=9 => PacketType::HttpResponse,
            _ => PacketType::Tcp,
        };
        
        self.create_packet_with_type(pattern, packet_type);
    }

    fn create_packet(&mut self, pattern: &TrafficPattern) {
        let packet_size = rand::thread_rng().gen_range(pattern.packet_size_range.0..pattern.packet_size_range.1);
        let data = vec![0u8; packet_size];
        
        self.create_packet_with_data(pattern, data);
    }

    fn create_packet_with_type(&mut self, pattern: &TrafficPattern, packet_type: PacketType) {
        let packet_size = rand::thread_rng().gen_range(pattern.packet_size_range.0..pattern.packet_size_range.1);
        let data = vec![0u8; packet_size];
        
        self.create_packet_with_data_and_type(pattern, data, packet_type);
    }

    fn create_packet_with_data(&mut self, pattern: &TrafficPattern, data: Vec<u8>) {
        self.create_packet_with_data_and_type(pattern, data, PacketType::Tcp);
    }

    fn create_packet_with_data_and_type(&mut self, pattern: &TrafficPattern, data: Vec<u8>, packet_type: PacketType) {
        let packet_id = PacketId::new(self.stats.total_packets + 1);
        let route_path = vec![pattern.source_node.clone(), pattern.dest_node.clone()];
        
        let packet = SimulatedPacket {
            id: packet_id,
            source: pattern.source_node.clone(),
            dest: pattern.dest_node.clone(),
            data,
            packet_type,
            created_at: Instant::now(),
            expected_delivery: Instant::now() + Duration::from_millis(100),
            current_hop: 0,
            route_path,
            priority: pattern.priority,
        };

        // Find direct link between nodes
        let link_id = self.find_direct_link(&pattern.source_node, &pattern.dest_node);
        if let Some(link_id) = link_id {
            let transmit_event = SimulationEvent::PacketTransmit {
                packet,
                link_id,
                timestamp: Instant::now(),
            };
            self.event_queue.add_event(transmit_event);
        } else {
            // No direct link, packet is dropped
            self.stats.packets_dropped += 1;
        }
    }

    fn find_direct_link(&self, source: &str, dest: &str) -> Option<ConnectionId> {
        for link in &self.topology.links {
            if (link.source_node == source && link.dest_node == dest) ||
               (link.source_node == dest && link.dest_node == source) {
                return Some(link.id);
            }
        }
        None
    }

    /// Process packet delivery
    fn process_packet_delivery(&mut self) {
        // Process any delivery events in the queue
        // This would be handled in process_events, but we can add additional logic here
    }

    /// Calculate transmission time for a packet
    fn calculate_transmission_time(&self, packet: &SimulatedPacket, link_config: &LinkConfig) -> Duration {
        let data_size_bits = packet.data.len() * 8;
        let bandwidth_bps = link_config.bandwidth_mbps * 1_000_000;
        
        let transmission_time = Duration::from_secs_f64(data_size_bits as f64 / bandwidth_bps as f64);
        let latency = Duration::from_millis(link_config.latency_ms);
        
        transmission_time + latency
    }

    /// Update latency statistics
    fn update_latency_stats(&mut self, latency: Duration) {
        self.stats.total_latency_measurement += latency;
        self.stats.latency_measurements += 1;
        self.stats.avg_latency = Duration::from_nanos(
            self.stats.total_latency_measurement.as_nanos() as u64 / self.stats.latency_measurements.max(1)
        );
    }

    /// Update simulation statistics
    fn update_statistics(&mut self) {
        // Calculate network utilization
        let mut total_utilization = 0.0;
        let mut link_count = 0;
        
        for link in &self.topology.links {
            let utilization = self.calculate_link_utilization(link);
            total_utilization += utilization;
            link_count += 1;
        }
        
        if link_count > 0 {
            self.stats.network_utilization = total_utilization / link_count as f64;
            self.stats.peak_network_utilization = self.stats.peak_network_utilization.max(self.stats.network_utilization);
        }
    }

    fn calculate_link_utilization(&self, link: &NetworkLink) -> f64 {
        // Simplified utilization calculation
        if link.config.bandwidth_mbps > 0 {
            (link.stats.bytes_transmitted + link.stats.bytes_received) as f64 / (link.config.bandwidth_mbps * 125000) as f64
        } else {
            0.0
        }
    }

    /// Add a traffic pattern
    pub fn add_traffic_pattern(&mut self, pattern: TrafficPattern) {
        self.traffic_generator.patterns.push(pattern);
    }

    /// Remove a traffic pattern
    pub fn remove_traffic_pattern(&mut self, pattern_name: &str) -> Result<TrafficPattern> {
        if let Some(index) = self.traffic_generator.patterns.iter().position(|p| p.name == pattern_name) {
            Ok(self.traffic_generator.patterns.remove(index))
        } else {
            Err(NetworkError::Other(format!("Traffic pattern not found: {}", pattern_name).into()))
        }
    }

    /// Get simulation statistics
    pub fn get_statistics(&self) -> &SimulationStatistics {
        &self.stats
    }

    /// Get network topology
    pub fn get_topology(&self) -> &NetworkTopology {
        &self.topology
    }

    /// Get simulation state
    pub fn get_state(&self) -> SimulationState {
        self.state
    }

    /// Get active traffic patterns
    pub fn get_active_patterns(&self) -> Vec<&TrafficPattern> {
        let elapsed = self.stats.simulation_duration;
        self.traffic_generator.patterns.iter()
            .filter(|pattern| {
                elapsed >= pattern.start_time && elapsed <= pattern.start_time + pattern.duration
            })
            .collect()
    }

    /// Create predefined network topologies
    pub fn create_predefined_topology(&mut self, topology_type: NetworkType) -> Result<()> {
        match topology_type {
            NetworkType::Lan => {
                self.create_lan_topology()?;
            }
            NetworkType::Wan => {
                self.create_wan_topology()?;
            }
            NetworkType::DataCenter => {
                self.create_datacenter_topology()?;
            }
            NetworkType::Internet => {
                self.create_internet_topology()?;
            }
            _ => {
                return Err(NetworkError::Other(format!("Topology type {:?} not implemented", topology_type).into()));
            }
        }

        log::info!("Created predefined topology: {:?}", topology_type);
        Ok(())
    }

    fn create_lan_topology(&mut self) -> Result<()> {
        // Clear existing topology
        self.topology.nodes.clear();
        self.topology.links.clear();

        // Create hosts
        for i in 0..10 {
            let host = NetworkNode {
                id: format!("host_{}", i),
                ip_address: IpAddress::v4(192, 168, 1, (i + 10) as u8),
                node_type: NodeType::Host,
                connections: Vec::new(),
                config: NodeConfig {
                    services: vec![ServiceInfo {
                        name: "Web Client".to_string(),
                        port: 80,
                        protocol: "HTTP".to_string(),
                        enabled: true,
                    }],
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(host);
        }

        // Create router
        let router = NetworkNode {
            id: "router".to_string(),
            ip_address: IpAddress::v4(192, 168, 1, 1),
            node_type: NodeType::Router,
            connections: Vec::new(),
            config: NodeConfig {
                cpu_cores: 4,
                memory_mb: 8192,
                bandwidth_mbps: 10000,
                services: vec![
                    ServiceInfo {
                        name: "DHCP Server".to_string(),
                        port: 67,
                        protocol: "UDP".to_string(),
                        enabled: true,
                    },
                    ServiceInfo {
                        name: "DNS Server".to_string(),
                        port: 53,
                        protocol: "UDP/TCP".to_string(),
                        enabled: true,
                    },
                ],
                ..Default::default()
            },
            stats: NodeStatistics::default(),
        };
        self.add_node(router);

        // Create links between hosts and router
        for i in 0..10 {
            let link = NetworkLink {
                id: ConnectionId::new(i),
                source_node: format!("host_{}", i),
                dest_node: "router".to_string(),
                config: LinkConfig {
                    bandwidth_mbps: 1000,
                    latency_ms: 1,
                    ..Default::default()
                },
                stats: LinkStatistics::default(),
                state: LinkState::Up,
            };
            self.add_link(link);
        }

        Ok(())
    }

    fn create_wan_topology(&mut self) -> Result<()> {
        // Create multiple LANs connected by WAN links
        for lan_id in 0..3 {
            let lan_base = 192 + lan_id * 10;
            
            // Create switch for each LAN
            let switch = NetworkNode {
                id: format!("switch_{}", lan_id),
                ip_address: IpAddress::v4(lan_base, 1, 1, 1),
                node_type: NodeType::Switch,
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 2,
                    memory_mb: 2048,
                    bandwidth_mbps: 10000,
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(switch);

            // Connect some hosts to each switch
            for host_id in 0..5 {
                let host_ip = IpAddress::v4(lan_base, 1, 1, (host_id + 10) as u8);
                let host = NetworkNode {
                    id: format!("host_{}_{}", lan_id, host_id),
                    ip_address: host_ip,
                    node_type: NodeType::Host,
                    connections: Vec::new(),
                    config: Default::default(),
                    stats: NodeStatistics::default(),
                };
                self.add_node(host);

                // Connect host to switch
                let link_id = lan_id * 10 + host_id;
                let link = NetworkLink {
                    id: ConnectionId::new(link_id as u32),
                    source_node: format!("host_{}_{}", lan_id, host_id),
                    dest_node: format!("switch_{}", lan_id),
                    config: LinkConfig {
                        bandwidth_mbps: 1000,
                        latency_ms: 1,
                        ..Default::default()
                    },
                    stats: LinkStatistics::default(),
                    state: LinkState::Up,
                };
                self.add_link(link);
            }

            // Create WAN links between switches
            if lan_id > 0 {
                let wan_link = NetworkLink {
                    id: ConnectionId::new(100 + lan_id as u32),
                    source_node: format!("switch_{}", lan_id - 1),
                    dest_node: format!("switch_{}", lan_id),
                    config: LinkConfig {
                        bandwidth_mbps: 100000,
                        latency_ms: 20,
                        ..Default::default()
                    },
                    stats: LinkStatistics::default(),
                    state: LinkState::Up,
                };
                self.add_link(wan_link);
            }
        }

        Ok(())
    }

    fn create_datacenter_topology(&mut self) -> Result<()> {
        // Create a simple 3-tier datacenter architecture
        // Web tier, Application tier, Database tier

        // Load balancer
        let lb = NetworkNode {
            id: "load_balancer".to_string(),
            ip_address: IpAddress::v4(10, 0, 1, 10),
            node_type: NodeType::LoadBalancer,
            connections: Vec::new(),
            config: NodeConfig {
                cpu_cores: 8,
                memory_mb: 16384,
                bandwidth_mbps: 10000,
                services: vec![ServiceInfo {
                    name: "HTTP Load Balancer".to_string(),
                    port: 80,
                    protocol: "TCP".to_string(),
                    enabled: true,
                }],
                ..Default::default()
            },
            stats: NodeStatistics::default(),
        };
        self.add_node(lb);

        // Web servers (3-tier)
        for i in 0..3 {
            let web_server = NetworkNode {
                id: format!("web_server_{}", i),
                ip_address: IpAddress::v4(10, 0, 1, (i + 20) as u8),
                node_type: NodeType::WebServer,
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 4,
                    memory_mb: 8192,
                    bandwidth_mbps: 1000,
                    services: vec![ServiceInfo {
                        name: "Web Server".to_string(),
                        port: 80,
                        protocol: "HTTP".to_string(),
                        enabled: true,
                    }],
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(web_server);

            // Connect web server to load balancer
            let link = NetworkLink {
                id: ConnectionId::new(i),
                source_node: "load_balancer".to_string(),
                dest_node: format!("web_server_{}", i),
                config: LinkConfig {
                    bandwidth_mbps: 1000,
                    latency_ms: 1,
                    ..Default::default()
                },
                stats: LinkStatistics::default(),
                state: LinkState::Up,
            };
            self.add_link(link);
        }

        // Application servers
        for i in 0..2 {
            let app_server = NetworkNode {
                id: format!("app_server_{}", i),
                ip_address: IpAddress::v4(10, 0, 2, (i + 10) as u8),
                node_type: NodeType::Application("Business Logic".to_string()),
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    bandwidth_mbps: 1000,
                    services: vec![ServiceInfo {
                        name: "Application Server".to_string(),
                        port: 8080,
                        protocol: "TCP".to_string(),
                        enabled: true,
                    }],
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(app_server);

            // Connect application servers to web servers
            for web_id in 0..3 {
                let link = NetworkLink {
                    id: ConnectionId::new(10 + i * 3 + web_id),
                    source_node: format!("web_server_{}", web_id),
                    dest_node: format!("app_server_{}", i),
                    config: LinkConfig {
                        bandwidth_mbps: 1000,
                        latency_ms: 2,
                        ..Default::default()
                    },
                    stats: LinkStatistics::default(),
                    state: LinkState::Up,
                };
                self.add_link(link);
            }
        }

        // Database servers
        for i in 0..2 {
            let db_server = NetworkNode {
                id: format!("db_server_{}", i),
                ip_address: IpAddress::v4(10, 0, 3, (i + 10) as u8),
                node_type: NodeType::Database,
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 16,
                    memory_mb: 32768,
                    bandwidth_mbps: 1000,
                    services: vec![ServiceInfo {
                        name: "Database".to_string(),
                        port: 5432,
                        protocol: "TCP".to_string(),
                        enabled: true,
                    }],
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(db_server);

            // Connect database servers to application servers
            for app_id in 0..2 {
                let link = NetworkLink {
                    id: ConnectionId::new(20 + i * 2 + app_id),
                    source_node: format!("app_server_{}", app_id),
                    dest_node: format!("db_server_{}", i),
                    config: LinkConfig {
                        bandwidth_mbps: 1000,
                        latency_ms: 3,
                        ..Default::default()
                    },
                    stats: LinkStatistics::default(),
                    state: LinkState::Up,
                };
                self.add_link(link);
            }
        }

        Ok(())
    }

    fn create_internet_topology(&mut self) -> Result<()> {
        // Create a simplified internet topology with ISPs and regions

        // Tier 1 ISPs
        for i in 0..3 {
            let isp = NetworkNode {
                id: format!("isp_tier1_{}", i),
                ip_address: IpAddress::v4(198, 51, 100, (i + 10) as u8),
                node_type: NodeType::Router,
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 16,
                    memory_mb: 32768,
                    bandwidth_mbps: 100000,
                    services: Vec::new(),
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(isp);
        }

        // Connect Tier 1 ISPs in a mesh
        for i in 0..3 {
            for j in (i + 1)..3 {
                let link = NetworkLink {
                    id: ConnectionId::new(i * 3 + j),
                    source_node: format!("isp_tier1_{}", i),
                    dest_node: format!("isp_tier1_{}", j),
                    config: LinkConfig {
                        bandwidth_mbps: 100000,
                        latency_ms: 5,
                        ..Default::default()
                    },
                    stats: LinkStatistics::default(),
                    state: LinkState::Up,
                };
                self.add_link(link);
            }
        }

        // Regional ISPs
        for region in 0..5 {
            let regional_isp = NetworkNode {
                id: format!("isp_regional_{}", region),
                ip_address: IpAddress::v4(203, 0, 113, (region + 10) as u8),
                node_type: NodeType::Router,
                connections: Vec::new(),
                config: NodeConfig {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    bandwidth_mbps: 10000,
                    ..Default::default()
                },
                stats: NodeStatistics::default(),
            };
            self.add_node(regional_isp);

            // Connect regional ISP to a random Tier 1 ISP
            let tier1_id = rand::thread_rng().gen_range(0..3);
            let link = NetworkLink {
                id: ConnectionId::new(100 + region),
                source_node: format!("isp_regional_{}", region),
                dest_node: format!("isp_tier1_{}", tier1_id),
                config: LinkConfig {
                    bandwidth_mbps: 10000,
                    latency_ms: 15,
                    ..Default::default()
                },
                stats: LinkStatistics::default(),
                state: LinkState::Up,
            };
            self.add_link(link);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_node_creation() {
        let node = NetworkNode {
            id: "test_node".to_string(),
            ip_address: IpAddress::v4(192, 168, 1, 100),
            node_type: NodeType::Host,
            connections: Vec::new(),
            config: NodeConfig::default(),
            stats: NodeStatistics::default(),
        };

        assert_eq!(node.id, "test_node");
        assert_eq!(node.ip_address, IpAddress::v4(192, 168, 1, 100));
        assert_eq!(node.node_type, NodeType::Host);
    }

    #[test]
    fn test_network_link_creation() {
        let link = NetworkLink {
            id: ConnectionId::new(1),
            source_node: "node1".to_string(),
            dest_node: "node2".to_string(),
            config: LinkConfig::default(),
            stats: LinkStatistics::default(),
            state: LinkState::Up,
        };

        assert_eq!(link.id.id, 1);
        assert_eq!(link.source_node, "node1");
        assert_eq!(link.dest_node, "node2");
        assert_eq!(link.state, LinkState::Up);
    }

    #[test]
    fn test_simulated_packet_creation() {
        let packet = SimulatedPacket {
            id: PacketId::new(123),
            source: "source".to_string(),
            dest: "dest".to_string(),
            data: vec![0x01, 0x02, 0x03, 0x04],
            packet_type: PacketType::Tcp,
            created_at: Instant::now(),
            expected_delivery: Instant::now() + Duration::from_secs(1),
            current_hop: 0,
            route_path: vec!["source".to_string(), "dest".to_string()],
            priority: PacketPriority::Normal,
        };

        assert_eq!(packet.id.id, 123);
        assert_eq!(packet.data, vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(packet.packet_type, PacketType::Tcp);
    }

    #[test]
    fn test_network_simulator_creation() {
        let config = SimulationConfig::default();
        let simulator = NetworkSimulator::new(config);

        assert_eq!(simulator.get_state(), SimulationState::Stopped);
        assert_eq!(simulator.topology.nodes.len(), 0);
        assert_eq!(simulator.topology.links.len(), 0);
    }

    #[test]
    fn test_traffic_pattern_creation() {
        let pattern = TrafficPattern {
            name: "Web Traffic".to_string(),
            pattern_type: TrafficPatternType::WebTraffic,
            source_node: "client".to_string(),
            dest_node: "server".to_string(),
            packet_rate: 10.0,
            packet_size_range: (100, 1500),
            duration: Duration::from_secs(300),
            start_time: Duration::from_secs(0),
            priority: PacketPriority::Normal,
        };

        assert_eq!(pattern.name, "Web Traffic");
        assert_eq!(pattern.packet_rate, 10.0);
        assert_eq!(pattern.packet_type, TrafficPatternType::WebTraffic);
    }

    #[test]
    fn test_network_conditions() {
        let mut conditions = NetworkConditions::new();
        
        // Test initial conditions
        assert_eq!(conditions.current_conditions.latency_multiplier, 1.0);
        assert_eq!(conditions.current_conditions.global_loss_rate, 0.0);
        assert_eq!(conditions.current_conditions.bandwidth_reduction, 1.0);
    }

    #[test]
    fn test_simulation_statistics() {
        let mut stats = SimulationStatistics::default();
        
        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.packets_delivered, 0);
        assert_eq!(stats.packets_dropped, 0);
        assert_eq!(stats.avg_latency, Duration::from_secs(0));
    }
}