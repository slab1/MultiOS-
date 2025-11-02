# Socket Programming Guide

This tutorial provides comprehensive coverage of socket programming using the MultiOS network stack's POSIX-compliant API.

## Learning Objectives

- Master the POSIX socket API
- Implement both TCP and UDP applications
- Handle errors and edge cases properly
- Use advanced socket options and features
- Build scalable network applications

## Socket API Overview

The MultiOS network stack implements a POSIX-compliant socket API with these core functions:

### Creation and Setup

```rust
use multios_networking::{Socket, SocketType, Protocol, IpAddress};

// Create a new socket
let socket = Socket::new(SocketType::Stream)?;        // TCP
let udp_socket = Socket::new(SocketType::Datagram)?;   // UDP
let raw_socket = Socket::new(SocketType::Raw)?;        // Raw socket
```

### Binding and Listening (Server)

```rust
// Bind to a specific address and port
socket.bind(&IpAddress::new_v4(0, 0, 0, 0, 8080))?;

// Start listening for connections
socket.listen(128)?; // Backlog of 128 pending connections

// Accept incoming connections
let (client_socket, client_address) = socket.accept()?;
println!("Connected to: {}", client_address);
```

### Connection (Client)

```rust
// Connect to a server
let server_addr = IpAddress::new_v4(192, 168, 1, 100, 80);
socket.connect(&server_addr)?;
```

## TCP Socket Programming

### Echo Server Implementation

```rust
use multios_networking::{Socket, SocketType, IpAddress};
use std::io::{Read, Write};

pub struct EchoServer {
    socket: Socket,
}

impl EchoServer {
    pub fn new(bind_address: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut socket = Socket::new(SocketType::Stream)?;
        let addr = IpAddress::new_v4(0, 0, 0, 0, port);
        socket.bind(&addr)?;
        socket.listen(100)?;
        
        Ok(EchoServer { socket })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Echo server listening...");
        
        loop {
            // Accept connection
            let (mut client_socket, client_addr) = self.socket.accept()?;
            println!("Client connected: {}", client_addr);
            
            // Handle client in a separate task (pseudo-code)
            spawn(|| self.handle_client(client_socket));
        }
    }
    
    fn handle_client(mut socket: Socket) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 4096];
        
        loop {
            // Receive data
            let bytes_read = socket.recv(&mut buffer)?;
            
            if bytes_read == 0 {
                // Connection closed by client
                println!("Client disconnected");
                break;
            }
            
            // Echo data back
            socket.send(&buffer[..bytes_read])?;
            
            // Log the transaction
            println!("Echoed {} bytes", bytes_read);
        }
        
        Ok(())
    }
}
```

### HTTP Server Implementation

```rust
use multios_networking::{Socket, SocketType, IpAddress};
use std::collections::HashMap;

pub struct HttpServer {
    socket: Socket,
    routes: HashMap<String, String>,
}

impl HttpServer {
    pub fn new(bind_address: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut socket = Socket::new(SocketType::Stream)?;
        let addr = IpAddress::new_v4(0, 0, 0, 0, port);
        socket.bind(&addr)?;
        socket.listen(100)?;
        
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), "Hello, World!".to_string());
        routes.insert("/status".to_string(), "Server is running".to_string());
        
        Ok(HttpServer { socket, routes })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("HTTP server listening on port 80...");
        
        loop {
            let (mut client_socket, client_addr) = self.socket.accept()?;
            println!("HTTP request from: {}", client_addr);
            
            spawn(|| self.handle_http_request(client_socket));
        }
    }
    
    fn handle_http_request(mut socket: Socket) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 8192];
        let bytes_read = socket.recv(&mut buffer)?;
        
        if bytes_read == 0 {
            return Ok(());
        }
        
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let response = self.process_http_request(&request);
        
        socket.send(response.as_bytes())?;
        Ok(())
    }
    
    fn process_http_request(&self, request: &str) -> String {
        // Simple HTTP/1.0 parser
        let lines: Vec<&str> = request.lines().collect();
        
        if lines.is_empty() {
            return self.create_error_response(400, "Bad Request");
        }
        
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        
        if parts.len() < 3 {
            return self.create_error_response(400, "Bad Request");
        }
        
        let method = parts[0];
        let path = parts[1];
        
        if method != "GET" {
            return self.create_error_response(501, "Not Implemented");
        }
        
        // Find matching route
        if let Some(content) = self.routes.get(path) {
            self.create_success_response(content)
        } else {
            self.create_error_response(404, "Not Found")
        }
    }
    
    fn create_success_response(&self, content: &str) -> String {
        format!(
            "HTTP/1.0 200 OK\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            content.len(),
            content
        )
    }
    
    fn create_error_response(&self, status_code: u16, message: &str) -> String {
        format!(
            "HTTP/1.0 {} {}\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            status_code,
            message,
            message.len(),
            message
        )
    }
}
```

## UDP Socket Programming

### UDP Echo Server

```rust
use multios_networking::{Socket, SocketType, Protocol, IpAddress};

pub struct UdpEchoServer {
    socket: Socket,
}

impl UdpEchoServer {
    pub fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = Socket::new(SocketType::Datagram)?;
        let addr = IpAddress::new_v4(0, 0, 0, 0, port);
        socket.bind(&addr)?;
        
        Ok(UdpEchoServer { socket })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("UDP echo server listening on port 7...");
        
        let mut buffer = [0u8; 4096];
        
        loop {
            // Receive data and sender address
            let (bytes_read, sender_addr) = self.socket.recv_from(&mut buffer)?;
            
            println!("Received {} bytes from: {}", bytes_read, sender_addr);
            
            // Echo data back to sender
            self.socket.send_to(&buffer[..bytes_read], &sender_addr)?;
        }
    }
}
```

### DNS Client Implementation

```rust
use multios_networking::{Socket, SocketType, IpAddress};

pub struct DnsClient {
    socket: Socket,
    dns_servers: Vec<IpAddress>,
}

impl DnsClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let socket = Socket::new(SocketType::Datagram)?;
        
        let dns_servers = vec![
            IpAddress::new_v4(8, 8, 8, 8, 53),      // Google DNS
            IpAddress::new_v4(8, 8, 4, 4, 53),      // Google DNS secondary
        ];
        
        Ok(DnsClient { socket, dns_servers })
    }
    
    pub fn resolve(&mut self, domain: &str) -> Result<Vec<IpAddress>, Box<dyn std::error::Error>> {
        let query = self.create_dns_query(domain);
        
        for dns_server in &self.dns_servers {
            // Send DNS query
            self.socket.send_to(&query, dns_server)?;
            
            // Receive response with timeout
            let mut response = [0u8; 512];
            let (bytes_read, _) = self.socket.recv_from(&mut response)?;
            
            let response_data = &response[..bytes_read];
            
            if let Ok(ip_addresses) = self.parse_dns_response(response_data) {
                if !ip_addresses.is_empty() {
                    return Ok(ip_addresses);
                }
            }
        }
        
        Err("DNS resolution failed".into())
    }
    
    fn create_dns_query(&self, domain: &str) -> Vec<u8> {
        let mut query = Vec::new();
        
        // DNS Header
        let transaction_id = rand::random::<u16>();
        query.extend_from_slice(&transaction_id.to_be_bytes());
        query.push(0x01); // Flags: Standard query
        query.push(0x00);
        query.push(0x00); // Questions: 1
        query.push(0x01);
        query.push(0x00); // Answer RRs: 0
        query.push(0x00);
        query.push(0x00); // Authority RRs: 0
        query.push(0x00);
        query.push(0x00); // Additional RRs: 0
        query.push(0x00);
        
        // Question Section
        for label in domain.split('.') {
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0x00); // End of domain name
        query.push(0x00); // Type: A
        query.push(0x01);
        query.push(0x00); // Class: IN
        query.push(0x01);
        
        query
    }
    
    fn parse_dns_response(&self, response: &[u8]) -> Result<Vec<IpAddress>, Box<dyn std::error::Error>> {
        // Simplified DNS response parser
        if response.len() < 12 {
            return Err("Invalid DNS response".into());
        }
        
        let answer_count = u16::from_be_bytes([response[6], response[7]]);
        let mut ip_addresses = Vec::new();
        
        let mut offset = 12;
        
        // Skip question section
        while offset < response.len() && response[offset] != 0 {
            offset += response[offset] as usize + 1;
        }
        offset += 1; // Skip null byte
        
        // Skip question type and class
        offset += 4;
        
        // Parse answer section
        for _ in 0..answer_count {
            if offset + 10 > response.len() {
                break;
            }
            
            // Skip name pointer
            offset += 2;
            
            let answer_type = u16::from_be_bytes([response[offset], response[offset + 1]]);
            let answer_length = u16::from_be_bytes([response[offset + 8], response[offset + 9]]);
            
            if answer_type == 1 && answer_length == 4 { // Type A record
                offset += 10;
                let ip = IpAddress::new_v4(
                    response[offset],
                    response[offset + 1],
                    response[offset + 2],
                    response[offset + 3],
                    0
                );
                ip_addresses.push(ip);
                offset += 4;
            } else {
                offset += 10 + answer_length as usize;
            }
        }
        
        Ok(ip_addresses)
    }
}
```

## Advanced Socket Programming

### Socket Options and Timeouts

```rust
use multios_networking::{Socket, SocketOption};

fn configure_socket(socket: &Socket) -> Result<(), Box<dyn std::error::Error>> {
    // Set socket timeout
    socket.set_timeout(Some(std::time::Duration::from_secs(30)))?;
    
    // Enable address reuse
    socket.set_option(SocketOption::ReuseAddr, true)?;
    
    // Enable TCP no delay (disable Nagle's algorithm)
    socket.set_option(SocketOption::TcpNoDelay, true)?;
    
    // Set receive buffer size
    socket.set_option(SocketOption::ReceiveBufferSize, 65536)?;
    
    // Set send buffer size
    socket.set_option(SocketOption::SendBufferSize, 65536)?;
    
    Ok(())
}
```

### Non-blocking Sockets

```rust
use multios_networking::{Socket, SocketType};

fn non_blocking_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = Socket::new(SocketType::Stream)?;
    
    // Enable non-blocking mode
    socket.set_nonblocking(true)?;
    
    // Attempt to connect (will return immediately)
    let server_addr = IpAddress::new_v4(192, 168, 1, 100, 80);
    match socket.connect(&server_addr) {
        Ok(_) => println!("Connected immediately"),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            println!("Connection in progress...");
            
            // Wait for connection to complete
            wait_for_write_ready(&socket)?;
            
            // Check connection status
            if socket.take_error()?.is_some() {
                return Err("Connection failed".into());
            }
            
            println!("Connection established!");
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

fn wait_for_write_ready(socket: &Socket) -> Result<(), Box<dyn std::error::Error>> {
    // This would use platform-specific mechanisms like epoll, kqueue, etc.
    // For educational purposes, we'll use a simple poll
    loop {
        let mut poll_fds = vec![socket.as_raw_fd()];
        
        match poll(&mut poll_fds, 1000)? {
            0 => continue, // Timeout, try again
            _ => {
                if poll_fds[0].revents & (PollFlags::POLLOUT | PollFlags::POLLERR | PollFlags::POLLHUP) != 0 {
                    break Ok(());
                }
            }
        }
    }
}
```

### Socket Shutdown

```rust
fn shutdown_example(mut socket: Socket) -> Result<(), Box<dyn std::error::Error>> {
    // Send some data
    socket.send(b"Hello, World!")?;
    
    // Shutdown write side (no more sending)
    socket.shutdown(Shutdown::Write)?;
    
    // Read any remaining data
    let mut buffer = [0u8; 1024];
    loop {
        match socket.recv(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => println!("Received {} bytes", n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }
    
    // Shutdown read side (no more receiving)
    socket.shutdown(Shutdown::Read)?;
    
    // Close socket
    drop(socket);
    
    Ok(())
}
```

## Error Handling

### Common Socket Errors

```rust
use multios_networking::{Socket, SocketType, SocketError};

fn robust_socket_usage() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Socket::new(SocketType::Stream)?;
    
    match socket.connect(&IpAddress::new_v4(192, 168, 1, 100, 80)) {
        Ok(_) => {
            // Connection successful
            handle_connection(socket)?;
        }
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::ConnectionRefused => {
                    println!("Connection refused - server not running");
                }
                std::io::ErrorKind::TimedOut => {
                    println!("Connection timed out - network issue");
                }
                std::io::ErrorKind::HostUnreachable => {
                    println!("Host unreachable - routing issue");
                }
                std::io::ErrorKind::PermissionDenied => {
                    println!("Permission denied - insufficient privileges");
                }
                _ => {
                    println!("Connection error: {}", e);
                }
            }
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn handle_connection(mut socket: Socket) -> Result<(), Box<dyn std::error::Error>> {
    // Set reasonable timeouts
    socket.set_timeout(Some(std::time::Duration::from_secs(30)))?;
    
    // Send data with retry logic
    let data = b"Hello, Server!";
    let mut bytes_sent = 0;
    let max_retries = 3;
    
    while bytes_sent < data.len() {
        match socket.send(&data[bytes_sent..]) {
            Ok(n) => {
                bytes_sent += n;
                println!("Sent {} bytes", n);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // Interrupted by signal, retry
                continue;
            }
            Err(e) => {
                if bytes_sent == 0 {
                    return Err(format!("Failed to send data: {}", e).into());
                }
                break; // Partial send, continue with what we have
            }
        }
    }
    
    // Receive response with timeout
    let mut buffer = [0u8; 4096];
    let mut total_received = 0;
    
    loop {
        match socket.recv(&mut buffer) {
            Ok(0) => {
                // Connection closed by peer
                if total_received == 0 {
                    return Err("Connection closed without data".into());
                }
                break; // Normal closure after receiving data
            }
            Ok(n) => {
                total_received += n;
                let response = String::from_utf8_lossy(&buffer[..n]);
                println!("Received: {}", response);
                
                // Check for HTTP response termination
                if response.contains("\r\n\r\n") {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout, handle appropriately
                if total_received == 0 {
                    return Err("Timeout waiting for response".into());
                }
                break; // We have some data, proceed
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                continue; // Interrupted, try again
            }
            Err(e) => {
                return Err(format!("Error receiving data: {}", e).into());
            }
        }
    }
    
    println!("Total bytes received: {}", total_received);
    Ok(())
}
```

## Socket Security

### Certificate Validation (TLS)

```rust
use multios_networking::{Socket, SecurityConfig, Certificate};

fn secure_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Create secure socket
    let mut socket = Socket::new(SocketType::Stream)?;
    
    // Configure security settings
    let mut config = SecurityConfig::new();
    config.set_tls_version("1.2");
    config.set_verify_hostname(true);
    config.set_certificate_validation(true);
    
    // Add trusted certificates
    let ca_cert = Certificate::from_pem_file("ca.pem")?;
    config.add_trusted_certificate(&ca_cert);
    
    // Connect with security
    let server_addr = IpAddress::new_v4(192, 168, 1, 100, 443);
    socket.connect_secure(&server_addr, &config)?;
    
    // Send secure request
    socket.send(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")?;
    
    // Receive secure response
    let mut buffer = [0u8; 8192];
    let bytes_read = socket.recv(&mut buffer)?;
    
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Secure response: {}", response);
    
    Ok(())
}
```

## Performance Considerations

### Connection Pooling

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ConnectionPool {
    connections: Arc<Mutex<HashMap<String, Socket>>>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        }
    }
    
    pub fn get_connection(&mut self, host: &str, port: u16) 
        -> Result<Socket, Box<dyn std::error::Error>> {
        let key = format!("{}:{}", host, port);
        
        let mut pool = self.connections.lock().unwrap();
        
        // Check if we have a cached connection
        if let Some(mut socket) = pool.remove(&key) {
            // Test if connection is still valid
            if socket.ping()? {
                return Ok(socket);
            }
            // Connection is dead, drop it
        }
        
        // Create new connection
        let addr = format!("{}:{}", host, port);
        let mut socket = Socket::new(SocketType::Stream)?;
        socket.connect(&IpAddress::from_str(&addr)?)?;
        
        Ok(socket)
    }
    
    pub fn return_connection(&mut self, host: &str, port: u16, socket: Socket) {
        let key = format!("{}:{}", host, port);
        let mut pool = self.connections.lock().unwrap();
        
        // Only cache if we're under the limit
        if pool.len() < self.max_connections {
            pool.insert(key, socket);
        }
    }
}
```

## Testing Socket Applications

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_echo_server() -> Result<(), Box<dyn std::error::Error>> {
        let server_thread = thread::spawn(|| {
            let mut server = EchoServer::new("127.0.0.1", 0).unwrap(); // Port 0 = random port
            server.handle_client_echo_test().unwrap();
        });
        
        thread::sleep(Duration::from_millis(100));
        
        // Test client connection
        let mut client = TcpStream::connect("127.0.0.1:0")?;
        client.write_all(b"Hello, World!")?;
        
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer)?;
        
        assert_eq!(String::from_utf8_lossy(&buffer[..bytes_read]), "Hello, World!");
        
        Ok(())
    }
}
```

## Exercises

1. **Chat Server**: Build a multi-client chat server using TCP
2. **File Transfer**: Implement a file transfer protocol using UDP
3. **Web Server**: Create a threaded HTTP server
4. **Network Monitor**: Build a tool that monitors network connections
5. **Load Balancer**: Implement a simple TCP load balancer

## Best Practices

1. **Always handle errors gracefully**
2. **Use timeouts to prevent hanging**
3. **Close connections properly**
4. **Implement proper logging**
5. **Test with realistic network conditions**
6. **Use connection pooling for performance**
7. **Validate input and sanitize output**
8. **Consider security implications**

## Next Steps

- [TCP/UDP Protocol Deep Dive](protocols.md)
- [Network Security Tutorial](security.md)
- [Building Network Applications](applications.md)

## Additional Resources

- POSIX.1-2017 socket specification
- Stevens' "UNIX Network Programming"
- Beej's Guide to Network Programming