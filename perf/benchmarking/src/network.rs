//! Network Performance Benchmarks
//! 
//! This module implements network performance benchmarks including:
//! - TCP throughput tests
//! - UDP latency measurements
//! - Socket creation and connection overhead
//! - Network protocol performance

use super::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

#[cfg(feature = "network")]
const TEST_MESSAGE_SIZE: usize = 1024;
const TCP_SERVER_PORT: u16 = 9999;
const UDP_SERVER_PORT: u16 = 9998;

/// TCP connection benchmark
pub struct TcpConnection;

impl TcpConnection {
    pub fn new() -> Self {
        Self
    }
    
    /// Create a simple echo server for testing
    fn start_echo_server(port: u16) -> Result<std::thread::JoinHandle<()>, Box<dyn std::error::Error>> {
        let handle = thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let mut buffer = vec![0u8; 1024];
                        if let Ok(bytes_read) = stream.read(&mut buffer) {
                            let _ = stream.write_all(&buffer[..bytes_read]);
                        }
                    }
                }
            }
        });
        
        // Wait for server to start
        thread::sleep(Duration::from_millis(100));
        
        Ok(handle)
    }
}

impl Benchmark for TcpConnection {
    fn name(&self) -> &str {
        "TCP Connection"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Network
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        #[cfg(feature = "network")]
        {
            // Start echo server
            let server_handle = Self::start_echo_server(TCP_SERVER_PORT)?;
            
            let start = Instant::now();
            let mut successful_connections = 0u64;
            
            for _ in 0..iterations {
                if let Ok(_stream) = TcpStream::connect(format!("127.0.0.1:{}", TCP_SERVER_PORT)) {
                    successful_connections += 1;
                    
                    // Small delay to prevent overwhelming the system
                    thread::sleep(Duration::from_micros(100));
                }
            }
            
            let elapsed = start.elapsed();
            let connections_per_sec = (successful_connections as f64) / elapsed.as_secs_f64();
            
            let mut metadata = HashMap::new();
            metadata.insert("server_port".to_string(), TCP_SERVER_PORT.to_string());
            metadata.insert("successful_connections".to_string(), successful_connections.to_string());
            metadata.insert("connection_rate_limit".to_string(), "100_us".to_string());
            
            // Drop server handle to stop the server
            drop(server_handle);
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: elapsed,
                iterations: successful_connections,
                operations_per_second: connections_per_sec,
                throughput: connections_per_sec,
                unit: "connections/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
        
        #[cfg(not(feature = "network"))]
        {
            // Mock benchmark when network feature is not enabled
            let start = Instant::now();
            thread::sleep(Duration::from_millis(100)); // Simulate work
            let elapsed = start.elapsed();
            
            let connections_per_sec = (iterations as f64) / elapsed.as_secs_f64();
            
            let mut metadata = HashMap::new();
            metadata.insert("note".to_string(), "Network feature disabled - using mock data".to_string());
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: elapsed,
                iterations,
                operations_per_second: connections_per_sec,
                throughput: connections_per_sec,
                unit: "connections/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
    }
}

/// TCP throughput benchmark
pub struct TcpThroughput;

impl TcpThroughput {
    pub fn new() -> Self {
        Self
    }
    
    /// Start TCP server for throughput testing
    fn start_throughput_server(port: u16) -> Result<std::thread::JoinHandle<()>, Box<dyn std::error::Error>> {
        let handle = thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
                    
                    loop {
                        match stream.read(&mut buffer) {
                            Ok(0) | Err(_) => break,
                            Ok(_bytes) => {
                                // Echo back
                                let _ = stream.write_all(&buffer);
                            }
                        }
                    }
                }
            }
        });
        
        thread::sleep(Duration::from_millis(100)); // Wait for server to start
        Ok(handle)
    }
}

impl Benchmark for TcpThroughput {
    fn name(&self) -> &str {
        "TCP Throughput"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Network
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        #[cfg(feature = "network")]
        {
            let server_handle = Self::start_throughput_server(TCP_SERVER_PORT + 1)?;
            
            let test_data = vec![0u8; TEST_MESSAGE_SIZE];
            let start = Instant::now();
            let mut total_bytes = 0u64;
            
            for _ in 0..iterations {
                if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", TCP_SERVER_PORT + 1)) {
                    // Send data
                    stream.write_all(&test_data)?;
                    total_bytes += test_data.len() as u64;
                    
                    // Receive echo
                    let mut buffer = vec![0u8; TEST_MESSAGE_SIZE];
                    if let Ok(_bytes_read) = stream.read(&mut buffer) {
                        total_bytes += _bytes_read as u64;
                    }
                }
            }
            
            let elapsed = start.elapsed();
            let bytes_per_sec = (total_bytes as f64) / elapsed.as_secs_f64();
            
            let mut metadata = HashMap::new();
            metadata.insert("message_size".to_string(), TEST_MESSAGE_SIZE.to_string());
            metadata.insert("total_bytes_transferred".to_string(), total_bytes.to_string());
            
            drop(server_handle);
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: elapsed,
                iterations,
                operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
                throughput: bytes_per_sec,
                unit: "bytes/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
        
        #[cfg(not(feature = "network"))]
        {
            let start = Instant::now();
            thread::sleep(Duration::from_millis(200));
            let elapsed = start.elapsed();
            
            let bytes_per_sec = (iterations as f64 * TEST_MESSAGE_SIZE as f64 * 2.0) / elapsed.as_secs_f64();
            
            let mut metadata = HashMap::new();
            metadata.insert("note".to_string(), "Network feature disabled - using mock data".to_string());
            metadata.insert("message_size".to_string(), TEST_MESSAGE_SIZE.to_string());
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: elapsed,
                iterations,
                operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
                throughput: bytes_per_sec,
                unit: "bytes/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
    }
}

/// UDP latency benchmark
pub struct UdpLatency;

impl UdpLatency {
    pub fn new() -> Self {
        Self
    }
    
    /// Start UDP echo server
    fn start_udp_server(port: u16) -> Result<std::thread::JoinHandle<()>, Box<dyn std::error::Error>> {
        let handle = thread::spawn(move || {
            if let Ok(socket) = UdpSocket::bind(format!("127.0.0.1:{}", port)) {
                let mut buffer = vec![0u8; 1024];
                loop {
                    if let Ok((bytes_read, _)) = socket.recv_from(&mut buffer) {
                        if bytes_read > 0 {
                            let _ = socket.send_to(&buffer[..bytes_read], "127.0.0.1:9999");
                        }
                    }
                }
            }
        });
        
        thread::sleep(Duration::from_millis(100));
        Ok(handle)
    }
}

impl Benchmark for UdpLatency {
    fn name(&self) -> &str {
        "UDP Latency"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Network
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        #[cfg(feature = "network")]
        {
            let server_handle = Self::start_udp_server(UDP_SERVER_PORT)?;
            
            let socket = UdpSocket::bind("127.0.0.1:0")?;
            let test_message = b"UDP latency test message";
            let mut latencies = Vec::new();
            
            for _ in 0..iterations {
                let start = Instant::now();
                
                socket.send_to(test_message, "127.0.0.1:9999")?;
                
                let mut buffer = vec![0u8; 1024];
                if let Ok((_bytes_read, _)) = socket.recv_from(&mut buffer) {
                    let latency = start.elapsed();
                    latencies.push(latency);
                }
            }
            
            let avg_latency = if !latencies.is_empty() {
                latencies.iter().sum::<Duration>() / latencies.len() as u32
            } else {
                Duration::from_millis(1)
            };
            
            let mut metadata = HashMap::new();
            metadata.insert("message_size".to_string(), test_message.len().to_string());
            metadata.insert("avg_latency_us".to_string(), (avg_latency.as_micros()).to_string());
            metadata.insert("latency_samples".to_string(), latencies.len().to_string());
            
            drop(server_handle);
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: avg_latency * iterations,
                iterations,
                operations_per_second: (iterations as f64) / (avg_latency.as_secs_f64()),
                throughput: (iterations as f64) / avg_latency.as_secs_f64(),
                unit: "round_trips/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
        
        #[cfg(not(feature = "network"))]
        {
            let avg_latency = Duration::from_micros(1000); // 1ms mock latency
            
            let mut metadata = HashMap::new();
            metadata.insert("note".to_string(), "Network feature disabled - using mock data".to_string());
            metadata.insert("avg_latency_us".to_string(), (avg_latency.as_micros()).to_string());
            
            Ok(BenchmarkResult {
                name: self.name().to_string(),
                category: self.category(),
                duration: avg_latency * iterations,
                iterations,
                operations_per_second: (iterations as f64) / (avg_latency.as_secs_f64()),
                throughput: (iterations as f64) / avg_latency.as_secs_f64(),
                unit: "round_trips/sec".to_string(),
                metadata,
                timestamp: chrono::Utc::now(),
            })
        }
    }
}

/// Socket creation benchmark
pub struct SocketCreation;

impl SocketCreation {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for SocketCreation {
    fn name(&self) -> &str {
        "Socket Creation"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Network
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut socket_creations = 0u64;
        
        for _ in 0..iterations {
            // Test UDP socket creation (faster and doesn't require server)
            if let Ok(_socket) = UdpSocket::bind("127.0.0.1:0") {
                socket_creations += 1;
            }
        }
        
        let elapsed = start.elapsed();
        let creations_per_sec = (socket_creations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("socket_type".to_string(), "UDP".to_string());
        metadata.insert("successful_creations".to_string(), socket_creations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations: socket_creations,
            operations_per_second: creations_per_sec,
            throughput: creations_per_sec,
            unit: "socket_creations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Network protocol overhead benchmark
pub struct ProtocolOverhead;

impl ProtocolOverhead {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for ProtocolOverhead {
    fn name(&self) -> &str {
        "Protocol Overhead"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Network
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        let test_data = b"Test message for protocol overhead measurement";
        let mut overhead_measurements = Vec::new();
        
        for _ in 0..iterations {
            // Simulate TCP connection lifecycle
            let conn_start = Instant::now();
            
            // Socket creation (mock)
            if let Ok(_socket) = UdpSocket::bind("127.0.0.1:0") {
                // Send data (mock)
                let _ = _socket.send_to(test_data, "127.0.0.1:9999");
                
                let conn_time = conn_start.elapsed();
                overhead_measurements.push(conn_time);
            }
        }
        
        let elapsed = start.elapsed();
        let total_operations = overhead_measurements.len() as f64;
        let ops_per_sec = total_operations / elapsed.as_secs_f64();
        
        let avg_overhead = if !overhead_measurements.is_empty() {
            overhead_measurements.iter().sum::<Duration>() / overhead_measurements.len() as u32
        } else {
            Duration::from_micros(1)
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("message_size".to_string(), test_data.len().to_string());
        metadata.insert("avg_overhead_us".to_string(), (avg_overhead.as_micros()).to_string());
        metadata.insert("operations_measured".to_string(), total_operations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations: total_operations as u64,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec,
            unit: "protocol_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Complete network benchmarking suite
pub struct NetworkBenchmarkSuite {
    pub tcp_connection: TcpConnection,
    pub tcp_throughput: TcpThroughput,
    pub udp_latency: UdpLatency,
    pub socket_creation: SocketCreation,
    pub protocol_overhead: ProtocolOverhead,
}

impl NetworkBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            tcp_connection: TcpConnection::new(),
            tcp_throughput: TcpThroughput::new(),
            udp_latency: UdpLatency::new(),
            socket_creation: SocketCreation::new(),
            protocol_overhead: ProtocolOverhead::new(),
        }
    }
    
    /// Run all network benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use super::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(false);
        let benchmarks: Vec<_> = vec![
            &self.socket_creation,
            &self.protocol_overhead,
            &self.tcp_connection,
            &self.tcp_throughput,
            &self.udp_latency,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_socket_creation() {
        let bench = SocketCreation::new();
        let result = bench.run(100).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
    
    #[test]
    fn test_protocol_overhead() {
        let bench = ProtocolOverhead::new();
        let result = bench.run(100).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
}