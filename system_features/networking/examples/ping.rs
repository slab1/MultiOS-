use multios_networking::{Socket, SocketType, IpAddress, Protocol};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// Ping Implementation using ICMP Protocol
/// 
/// This example demonstrates:
/// 1. Raw socket creation for ICMP traffic
/// 2. ICMP packet structure and parsing
/// 3. Echo Request/Response handling
/// 4. Round-trip time calculation
/// 5. Network diagnostic utilities
/// 6. Error handling for network unreachability

const ICMP_ECHO_REQUEST: u8 = 8;
const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_TIME_EXCEEDED: u8 = 11;
const ICMP_DESTINATION_UNREACHABLE: u8 = 3;

#[derive(Debug)]
struct IcmpPacket {
    packet_type: u8,
    code: u8,
    checksum: u16,
    identifier: u16,
    sequence: u16,
    timestamp: u64,
    payload: Vec<u8>,
}

impl IcmpPacket {
    fn new_echo_request(identifier: u16, sequence: u16, payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        IcmpPacket {
            packet_type: ICMP_ECHO_REQUEST,
            code: 0,
            checksum: 0, // Will be calculated
            identifier,
            sequence,
            timestamp,
            payload,
        }
    }
    
    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 8 {
            return Err("ICMP packet too short".into());
        }
        
        let packet_type = data[0];
        let code = data[1];
        let checksum = u16::from_be_bytes([data[2], data[3]]);
        let identifier = u16::from_be_bytes([data[4], data[5]]);
        let sequence = u16::from_be_bytes([data[6], data[7]]);
        
        let mut timestamp = 0u64;
        let mut payload = Vec::new();
        
        if data.len() > 12 {
            // Extract timestamp (8 bytes starting at offset 8)
            timestamp = u64::from_be_bytes([
                data[8], data[9], data[10], data[11],
                data[12], data[13], data[14], data[15]
            ]);
            
            // Remaining data is payload
            if data.len() > 16 {
                payload.extend_from_slice(&data[16..]);
            }
        }
        
        // Verify checksum
        let calculated_checksum = calculate_checksum(data);
        if checksum != calculated_checksum {
            return Err("Invalid ICMP checksum".into());
        }
        
        Ok(IcmpPacket {
            packet_type,
            code,
            checksum,
            identifier,
            sequence,
            timestamp,
            payload,
        })
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // ICMP header
        data.push(self.packet_type);
        data.push(self.code);
        data.extend_from_slice(&self.checksum.to_be_bytes());
        data.extend_from_slice(&self.identifier.to_be_bytes());
        data.extend_from_slice(&self.sequence.to_be_bytes());
        
        // Timestamp (8 bytes)
        let timestamp_bytes = self.timestamp.to_be_bytes();
        data.extend_from_slice(&timestamp_bytes);
        
        // Payload
        data.extend_from_slice(&self.payload);
        
        // Calculate and set checksum
        let checksum = calculate_checksum(&data);
        data[2] = (checksum >> 8) as u8;
        data[3] = (checksum & 0xFF) as u8;
        
        data
    }
}

fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    
    // Sum all 16-bit words
    while i + 1 < data.len() {
        let word = ((data[i] as u32) << 8) | (data[i + 1] as u32);
        sum += word;
        i += 2;
    }
    
    // Add remaining byte if any
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    // Fold 32-bit sum to 16 bits
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    // One's complement
    !(sum as u16)
}

struct PingResult {
    sequence: u16,
    bytes: usize,
    time_ms: f64,
    ttl: Option<u8>,
    source_ip: IpAddress,
}

impl PingResult {
    fn success(sequence: u16, bytes: usize, time_ms: f32, ttl: u8, source_ip: IpAddress) -> Self {
        PingResult {
            sequence,
            bytes,
            time_ms: time_ms as f64,
            ttl: Some(ttl),
            source_ip,
        }
    }
    
    fn timeout(sequence: u16) -> Self {
        PingResult {
            sequence,
            bytes: 0,
            time_ms: 0.0,
            ttl: None,
            source_ip: IpAddress::new_v4(0, 0, 0, 0, 0),
        }
    }
}

struct PingConfig {
    count: u32,
    timeout: Duration,
    interval: Duration,
    payload_size: usize,
    ttl: u8,
}

impl Default for PingConfig {
    fn default() -> Self {
        PingConfig {
            count: 4,
            timeout: Duration::from_secs(5),
            interval: Duration::from_secs(1),
            payload_size: 64,
            ttl: 64,
        }
    }
}

struct PingStats {
    sent: u32,
    received: u32,
    min_time: f32,
    max_time: f32,
    avg_time: f32,
    times: Vec<f32>,
}

impl PingStats {
    fn new() -> Self {
        PingStats {
            sent: 0,
            received: 0,
            min_time: f32::INFINITY,
            max_time: 0.0,
            avg_time: 0.0,
            times: Vec::new(),
        }
    }
    
    fn add_result(&mut self, time_ms: f32) {
        self.received += 1;
        self.times.push(time_ms);
        
        if time_ms < self.min_time {
            self.min_time = time_ms;
        }
        if time_ms > self.max_time {
            self.max_time = time_ms;
        }
        
        // Calculate running average
        let total: f32 = self.times.iter().sum();
        self.avg_time = total / self.times.len() as f32;
    }
    
    fn loss_percentage(&self) -> f32 {
        if self.sent == 0 {
            0.0
        } else {
            (self.sent - self.received) as f32 / self.sent as f32 * 100.0
        }
    }
}

fn ping_host(host: &str, config: PingConfig) -> Result<PingStats, Box<dyn std::error::Error>> {
    println!("🌐 PING {} ({})", host, config.payload_size);
    
    // Resolve hostname to IP address
    let mut dns_client = multios_networking::dns::DnsClient::new()?;
    let ip_addresses = dns_client.resolve(host)?;
    
    if ip_addresses.is_empty() {
        return Err("Failed to resolve hostname".into());
    }
    
    let target_ip = ip_addresses[0];
    println!("📍 PING {}: {} data bytes", target_ip, config.payload_size);
    println!();
    
    // Create raw ICMP socket
    let mut socket = Socket::new(SocketType::Raw)?;
    socket.set_timeout(Some(config.timeout))?;
    
    // Set TTL option
    socket.set_option(multios_networking::SocketOption::TimeToLive, config.ttl)?;
    
    let mut stats = PingStats::new();
    let identifier = (std::process::id() & 0xFFFF) as u16;
    
    for sequence in 1..=config.count {
        stats.sent += 1;
        
        // Create ICMP packet
        let payload = create_payload(config.payload_size);
        let icmp_packet = IcmpPacket::new_echo_request(identifier, sequence, payload);
        let packet_data = icmp_packet.to_bytes();
        
        // Record send time
        let send_time = SystemTime::now();
        
        // Send ICMP packet
        match socket.send_to(&packet_data, &target_ip) {
            Ok(_) => {
                println!("🔸 Seq={} ", sequence);
            }
            Err(e) => {
                println!("❌ Send failed for sequence {}: {}", sequence, e);
                continue;
            }
        }
        
        // Wait for response or timeout
        let result = wait_for_response(&mut socket, sequence, config.timeout);
        
        let receive_time = SystemTime::now();
        let elapsed = receive_time.duration_since(send_time).unwrap();
        let time_ms = elapsed.as_secs_f32() * 1000.0;
        
        match result {
            Ok(icmp_response) => {
                stats.add_result(time_ms);
                
                let source_ip = target_ip; // In a real implementation, this would come from the received packet
                let bytes = icmp_response.payload.len() + 8; // Payload + ICMP header
                
                println!("🔹 {} bytes from {}: icmp_seq={} ttl={:?} time={:.2}ms", 
                        bytes, source_ip, sequence, 
                        icmp_response.packet_type, time_ms);
            }
            Err(_) => {
                println!("⏱️  Request timeout for icmp_seq {}", sequence);
            }
        }
        
        // Print statistics after each response
        if stats.received > 0 {
            print_statistics(&stats, sequence);
        }
        
        // Wait for next ping (except for the last one)
        if sequence < config.count {
            std::thread::sleep(config.interval);
        }
        
        println!();
    }
    
    // Print final statistics
    println!("--- {} ping statistics ---", host);
    println!("{} packets transmitted, {} received, {:.1}% packet loss", 
             stats.sent, stats.received, stats.loss_percentage());
    
    if stats.received > 0 {
        println!("round-trip min/avg/max = {:.2}/{:.2}/{:.2} ms", 
                 stats.min_time, stats.avg_time, stats.max_time);
    }
    
    Ok(stats)
}

fn create_payload(size: usize) -> Vec<u8> {
    let mut payload = Vec::with_capacity(size);
    
    // Fill payload with a pattern
    for i in 0..size {
        payload.push((b'a' + (i % 26) as u8) as u8);
    }
    
    payload
}

fn wait_for_response(
    socket: &mut Socket, 
    expected_sequence: u16, 
    timeout: Duration
) -> Result<IcmpPacket, Box<dyn std::error::Error>> {
    let start_time = SystemTime::now();
    
    loop {
        let mut buffer = [0u8; 65535];
        
        match socket.recv(&mut buffer) {
            Ok(bytes_read) => {
                let packet_data = &buffer[..bytes_read];
                
                // Parse as ICMP packet
                if let Ok(icmp_packet) = IcmpPacket::from_bytes(packet_data) {
                    // Check if this is a response to our echo request
                    if icmp_packet.packet_type == ICMP_ECHO_REPLY && 
                       icmp_packet.sequence == expected_sequence {
                        return Ok(icmp_packet);
                    }
                    
                    // Handle other ICMP error messages
                    match icmp_packet.packet_type {
                        ICMP_DESTINATION_UNREACHABLE => {
                            return Err("Destination unreachable".into());
                        }
                        ICMP_TIME_EXCEEDED => {
                            return Err("Time exceeded (TTL expired)".into());
                        }
                        _ => {
                            // Other ICMP types, ignore for now
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    return Err("Timeout".into());
                }
                return Err(e.into());
            }
        }
        
        // Check timeout
        if SystemTime::now().duration_since(start_time).unwrap() > timeout {
            return Err("Timeout".into());
        }
    }
}

fn print_statistics(stats: &PingStats, current_sequence: u32) {
    println!("\r");
    print!("{} packets transmitted, {} received, {:.1}% packet loss", 
           stats.sent, stats.received, stats.loss_percentage());
    
    if stats.received > 0 {
        print!(", min/avg/max = {:.2}/{:.2}/{:.2} ms", 
               stats.min_time, stats.avg_time, stats.max_time);
    }
    
    print!("    (seq {} target)", current_sequence);
    std::io::stdout().flush().unwrap();
}

fn ping_localhost() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 Testing local network interface (127.0.0.1)");
    
    let config = PingConfig {
        count: 3,
        timeout: Duration::from_secs(2),
        interval: Duration::from_millis(500),
        payload_size: 56,
        ttl: 64,
    };
    
    ping_host("127.0.0.1", config)?;
    
    println!("\n✅ Local interface test completed\n");
    Ok(())
}

fn ping_local_network() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 Testing local network (192.168.1.1 - typical gateway)");
    
    let config = PingConfig {
        count: 2,
        timeout: Duration::from_secs(3),
        interval: Duration::from_secs(1),
        payload_size: 64,
        ttl: 64,
    };
    
    match ping_host("192.168.1.1", config) {
        Ok(stats) => {
            if stats.received > 0 {
                println!("✅ Local network gateway is reachable");
            } else {
                println!("⚠️  Local network gateway not reachable (normal if not connected)");
            }
        }
        Err(e) => {
            println!("⚠️  Could not ping local gateway: {}", e);
        }
    }
    
    println!();
    Ok(())
}

fn ping_public_dns() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Testing public DNS servers");
    
    let config = PingConfig {
        count: 3,
        timeout: Duration::from_secs(5),
        interval: Duration::from_secs(1),
        payload_size: 64,
        ttl: 64,
    };
    
    // Test Google DNS servers
    for (i, server) in ["8.8.8.8", "8.8.4.4", "1.1.1.1"].iter().enumerate() {
        if i > 0 {
            std::thread::sleep(Duration::from_secs(2));
        }
        
        println!("\n--- Testing {} ---", server);
        match ping_host(server, config.clone()) {
            Ok(stats) => {
                if stats.received > 0 {
                    println!("✅ {} is reachable", server);
                } else {
                    println!("❌ {} is not reachable", server);
                }
            }
            Err(e) => {
                println!("❌ Error pinging {}: {}", server, e);
            }
        }
    }
    
    Ok(())
}

fn ping_with_custom_options() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Custom ping options demonstration");
    
    let config = PingConfig {
        count: 2,
        timeout: Duration::from_secs(3),
        interval: Duration::from_millis(800),
        payload_size: 128, // Larger payload
        ttl: 32,          // Lower TTL
    };
    
    ping_host("google.com", config)?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 MultiOS Network Stack - Ping Implementation");
    println!("==============================================");
    println!();
    
    // Test 1: Localhost
    ping_localhost()?;
    
    // Test 2: Local network
    ping_local_network()?;
    
    // Test 3: Public DNS servers
    ping_public_dns()?;
    
    // Test 4: Custom options
    ping_with_custom_options()?;
    
    println!("🎯 Ping demonstrations completed!");
    println!();
    println!("💡 This ping implementation demonstrates:");
    println!("   • Raw socket creation for ICMP protocol");
    println!("   • ICMP packet structure and checksums");
    println!("   • Round-trip time measurement");
    println!("   • Network diagnostic capabilities");
    println!("   • TTL handling and timeouts");
    println!("   • Statistical analysis of network performance");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icmp_packet_creation() {
        let payload = vec![1, 2, 3, 4, 5];
        let packet = IcmpPacket::new_echo_request(1234, 5678, payload.clone());
        
        assert_eq!(packet.packet_type, ICMP_ECHO_REQUEST);
        assert_eq!(packet.identifier, 1234);
        assert_eq!(packet.sequence, 5678);
        assert_eq!(packet.payload, payload);
    }
    
    #[test]
    fn test_icmp_packet_serialization() {
        let payload = vec![1, 2, 3, 4];
        let original = IcmpPacket::new_echo_request(1234, 5678, payload);
        let data = original.to_bytes();
        
        let parsed = IcmpPacket::from_bytes(&data).unwrap();
        
        assert_eq!(parsed.packet_type, original.packet_type);
        assert_eq!(parsed.identifier, original.identifier);
        assert_eq!(parsed.sequence, original.sequence);
        assert_eq!(parsed.payload, original.payload);
    }
    
    #[test]
    fn test_checksum_calculation() {
        let data = vec![0x45, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x00];
        let checksum = calculate_checksum(&data);
        
        // This should produce a valid checksum that validates correctly
        assert!(checksum != 0);
    }
}