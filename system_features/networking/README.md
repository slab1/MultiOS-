# MultiOS Network Stack

A comprehensive, POSIX-compliant TCP/IP network stack implementation for the MultiOS operating system, written in Rust.

## Overview

This networking stack provides a complete implementation of the TCP/IP protocol suite with full socket programming capabilities. It's designed for educational purposes and production use in embedded/systems programming scenarios.

## Features

### Core Networking Stack
- **IP Protocol**: IPv4 and IPv6 support with fragmentation and reassembly
- **TCP Protocol**: Full TCP implementation with state machine, flow control, and congestion control
- **UDP Protocol**: Connectionless datagram transport
- **ICMP Protocol**: Network diagnostic and error reporting (ping, traceroute)

### POSIX-Compliant Socket API
- Berkeley sockets interface (`socket`, `bind`, `listen`, `accept`, `connect`, `send`, `recv`)
- Support for TCP, UDP, and raw sockets
- Non-blocking and asynchronous socket operations
- Socket options and timeouts

### Network Interface Drivers
- Hardware abstraction layer for network devices
- Support for Ethernet, WiFi, and virtual interfaces
- Packet buffer management and DMA support
- Interrupt handling and polling modes

### Routing and Forwarding
- Dynamic routing table management
- Multiple routing protocols support
- IP forwarding between interfaces
- Policy-based routing

### DNS Resolution
- DNS client implementation
- Recursive and iterative resolution
- DNS caching and TTL management
- Support for A, AAAA, CNAME, MX, TXT records

### Network Security
- Firewall with stateful packet inspection
- Network Address Translation (NAT)
- Intrusion Detection/Prevention System (IDS/IPS)
- Port scanning detection and prevention

### Network Simulation and Testing
- Virtual network topologies
- Network behavior simulation
- Packet loss, latency, and bandwidth modeling
- Educational testing scenarios

## Quick Start

### Basic Socket Usage

```rust
use multios_networking::{Socket, SocketType, IpAddress};

// Create a TCP socket
let socket = Socket::new(SocketType::Stream).unwrap();

// Connect to a server
let server_addr = IpAddress::from_str("192.168.1.100:80").unwrap();
socket.connect(&server_addr).unwrap();

// Send HTTP request
let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
socket.send(request.as_bytes()).unwrap();

// Receive response
let mut buffer = [0u8; 4096];
let bytes_received = socket.recv(&mut buffer).unwrap();
let response = String::from_utf8_lossy(&buffer[..bytes_received]);
```

### Creating a Server

```rust
use multios_networking::{Socket, SocketType, IpAddress};

// Create a listening socket
let listener = Socket::new(SocketType::Stream).unwrap();
let bind_addr = IpAddress::from_str("0.0.0.0:8080").unwrap();
listener.bind(&bind_addr).unwrap();
listener.listen(128).unwrap();

// Accept connections
loop {
    let (client_socket, client_addr) = listener.accept().unwrap();
    println!("Connection from: {}", client_addr);
    
    // Handle client in a separate task/coroutine
    spawn(async move {
        handle_client(client_socket).await;
    });
}
```

### Using the Network Stack

```rust
use multios_networking::{
    NetworkStack, NetworkInterface, IpAddress, 
    Protocol, RoutingTable
};

// Initialize the network stack
let mut stack = NetworkStack::new();

// Add network interfaces
let eth0 = NetworkInterface::new("eth0").unwrap();
stack.add_interface(eth0);

// Configure IP addresses
let interface = stack.get_interface("eth0").unwrap();
interface.set_ip_address("192.168.1.10/24").unwrap();

// Start the network stack
stack.start().unwrap();
```

## Architecture

### Layer Structure

```
┌─────────────────────────────────────┐
│         Application Layer           │  ← Your programs
├─────────────────────────────────────┤
│        Socket API Layer             │  ← POSIX sockets
├─────────────────────────────────────┤
│       Protocol Processing           │  ← TCP/UDP/ICMP
├─────────────────────────────────────┤
│        IP Processing                │  ← Routing/Forwarding
├─────────────────────────────────────┤
│       Network Interface Layer       │  ← Drivers/Hardware
└─────────────────────────────────────┘
```

### Module Organization

- `core.rs`: Core data structures and network stack initialization
- `sockets.rs`: POSIX-compliant socket API implementation
- `protocols/`: Protocol implementations (TCP, UDP, ICMP, IP)
- `routing/`: Routing table and forwarding engine
- `dns/`: DNS resolution system
- `security/`: Firewall, NAT, and security features
- `drivers/`: Network interface driver framework
- `simulation/`: Network simulation and testing tools

## Educational Content

### Tutorials
- [Network Programming Basics](tutorials/basics.md)
- [Socket Programming Guide](tutorials/socket_programming.md)
- [TCP/UDP Protocol Deep Dive](tutorials/protocols.md)
- [Building Network Applications](tutorials/applications.md)
- [Network Security Essentials](tutorials/security.md)

### Examples
- [Simple HTTP Client](examples/http_client.rs)
- [Web Server Implementation](examples/web_server.rs)
- [Network Scanner](examples/scanner.rs)
- [Ping Implementation](examples/ping.rs)
- [DNS Client](examples/dns_client.rs)

## Testing

Run the test suite:
```bash
cargo test
```

Network simulation tests:
```bash
cargo test simulation
```

Protocol compliance tests:
```bash
cargo test protocols
```

## Performance

The network stack is optimized for:
- Low memory footprint (suitable for embedded systems)
- High throughput (supporting gigabit networks)
- Low latency (real-time applications)
- Minimal CPU overhead

### Benchmarks

Run performance benchmarks:
```bash
cargo bench
```

## Security Considerations

- All network traffic is processed in kernel space
- Memory-safe implementation prevents buffer overflows
- Built-in firewall and intrusion detection
- Secure protocol implementations
- Protection against common network attacks

## Compatibility

- POSIX-compliant socket API
- RFC-compliant protocol implementations
- Compatible with existing network tools
- Cross-platform support (Linux, Windows, macOS, embedded)

## Contributing

This is an educational project. Contributions are welcome for:
- Bug fixes and improvements
- New protocol implementations
- Performance optimizations
- Documentation and tutorials
- Test cases and examples

## License

MIT License - see LICENSE file for details

## Educational Resources

### Recommended Reading
- "TCP/IP Illustrated" by W. Richard Stevens
- "UNIX Network Programming" by W. Richard Stevens
- "Computer Networks" by Andrew Tanenbaum
- RFC Documents (RFC 791, RFC 793, RFC 768, RFC 792)

### Online Resources
- [OSI Model Explained](https://example.com/osi-model)
- [Socket Programming Tutorial](https://example.com/socket-tutorial)
- [Network Protocol Deep Dive](https://example.com/protocols)

## Support

For questions, issues, or contributions:
- Create an issue on GitHub
- Check the documentation in the `docs/` directory
- Review the examples in the `examples/` directory
- Follow the tutorials in the `tutorials/` directory

---

**Note**: This is an educational implementation designed to teach networking concepts. For production use, consider using mature networking stacks like Linux's TCP/IP implementation.