# Network Programming Basics

This tutorial introduces fundamental networking concepts and how they're implemented in the MultiOS network stack.

## Learning Objectives

After completing this tutorial, you will understand:
- The OSI and TCP/IP models
- How data flows through the network stack
- Basic networking concepts (IP addresses, ports, protocols)
- Socket programming fundamentals
- Common network programming patterns

## The Network Stack Models

### OSI Model (7 Layers)

```
┌─────────────┬──────────────┬─────────────────┬─────────────────────┐
│ Layer       │ Name         │ Function        │ MultiOS Module      │
├─────────────┼──────────────┼─────────────────┼─────────────────────┤
│ 7           │ Application  │ Network apps    │ Your programs       │
│ 6           │ Presentation │ Data formatting │ DNS/HTTP            │
│ 5           │ Session      │ Connection mgmt │ Socket API          │
│ 4           │ Transport    │ End-to-end      │ TCP/UDP             │
│ 3           │ Network      │ Routing         │ IP                  │
│ 2           │ Data Link    │ Local delivery  │ Ethernet/WiFi       │
│ 1           │ Physical     │ Bits on wire    │ Hardware            │
└─────────────┴──────────────┴─────────────────┴─────────────────────┘
```

### TCP/IP Model (4 Layers)

```
┌─────────────┬──────────────┬─────────────────┬─────────────────────┐
│ Layer       │ Name         │ Function        │ MultiOS Module      │
├─────────────┼──────────────┼─────────────────┼─────────────────────┤
│ 4           │ Application  │ Network apps    │ HTTP/DNS/Sockets    │
│ 3           │ Transport    │ End-to-end      │ TCP/UDP             │
│ 2           │ Internet     │ Routing         │ IP/ICMP             │
│ 1           │ Link         │ Local delivery  │ Ethernet/WiFi       │
└─────────────┴──────────────┴─────────────────┴─────────────────────┘
```

## Understanding IP Addresses

### IPv4 Address Structure

An IPv4 address is a 32-bit number divided into 4 octets:

```
192.168.1.100
│     │     │     │
└─────┴─────┴─────┘  Network Address: 192.168.1.0/24
       │     │           Host Address: 100
       │     └─ Subnet: .1 (router)
       └─ Class B: 192.168.x.x (private network)
```

**Subnet Mask Examples:**
- `/24` (255.255.255.0) - 254 usable hosts
- `/16` (255.255.0.0) - 65,534 usable hosts
- `/30` (255.255.255.252) - 2 usable hosts (point-to-point)

### IPv6 Address Structure

IPv6 addresses are 128-bit addresses:

```
2001:0db8:85a3:0000:0000:8a2e:0370:7334
│    │    │    │    │    │    │    │
└────┴────┴────┴────┴────┴────┴────┘
Global   Subnet  Interface ID
Prefix   ID      (MAC address)
```

**IPv6 Address Types:**
- `::1/128` - Loopback address
- `fe80::/10` - Link-local addresses
- `2000::/3` - Global unicast addresses

## Port Numbers

Ports identify network applications on a host:

```
TCP/UDP Port Ranges:
┌─────────────┬──────────────┬─────────────────┐
│ Range       │ Type         │ Common Uses     │
├─────────────┼──────────────┼─────────────────┤
│ 0-1023      │ Well-known   │ System services │
│             │              │ (HTTP: 80,      │
│             │              │  HTTPS: 443,    │
│             │              │  SSH: 22)       │
├─────────────┼──────────────┼─────────────────┤
│ 1024-49151  │ Registered   │ User apps       │
├─────────────┼──────────────┼─────────────────┤
│ 49152-65535 │ Dynamic      │ Temporary use   │
└─────────────┴──────────────┴─────────────────┘
```

**Socket Addressing:**
```
socket = IP address + port number + protocol
192.168.1.100:8080 (TCP)
```

## Understanding Protocols

### UDP (User Datagram Protocol)

**Characteristics:**
- Connectionless (no setup required)
- Unreliable (no delivery guarantee)
- Fast (minimal overhead)
- Small header (8 bytes)

**Use Cases:**
- DNS queries
- Video streaming
- Real-time gaming
- IoT communications

**UDP Header Structure:**
```
┌─────────────┬─────────────┬─────────────┐
│ Source Port │ Dest Port   │ Length      │
│ (16 bits)   │ (16 bits)   │ (16 bits)   │
├─────────────┼─────────────┼─────────────┤
│ Checksum    │ Data Payload│             │
│ (16 bits)   │ (variable)  │             │
└─────────────┴─────────────┴─────────────┘
```

### TCP (Transmission Control Protocol)

**Characteristics:**
- Connection-oriented (three-way handshake)
- Reliable (guaranteed delivery)
- Ordered (data arrives in sequence)
- Flow control (prevents overwhelming receiver)
- Congestion control (prevents network congestion)

**TCP Header Structure:**
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│ Source Port │ Dest Port   │ Sequence Number          │
│ (16 bits)   │ (16 bits)   │ (32 bits)               │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ Acknowledgment Number              │ Flags        │
│ (32 bits)                          │ (8 bits)     │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ Window Size │ Checksum    │ Urgent Pointer          │
│ (16 bits)   │ (16 bits)   │ (16 bits)               │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ Options (variable length)                         │
└───────────────────────────────────────────────────┘
```

**TCP Flags:**
- `SYN` - Synchronize sequence numbers
- `ACK` - Acknowledgment
- `FIN` - Finished sending data
- `RST` - Reset connection
- `PSH` - Push data to application
- `URG` - Urgent data

## Socket Programming Fundamentals

### Socket Types

| Domain | Type | Protocol | Use Case |
|--------|------|----------|----------|
| AF_INET | SOCK_STREAM | TCP | Reliable connections |
| AF_INET | SOCK_DGRAM | UDP | Connectionless data |
| AF_INET | SOCK_RAW | any | Raw network access |
| AF_INET6 | SOCK_STREAM | TCP | IPv6 connections |
| AF_INET6 | SOCK_DGRAM | UDP | IPv6 datagrams |

### Basic Socket Operations

**Client Side:**
```
1. socket()    - Create socket
2. connect()   - Connect to server
3. send()/recv() - Exchange data
4. close()     - Close connection
```

**Server Side:**
```
1. socket()    - Create socket
2. bind()      - Bind to port
3. listen()    - Listen for connections
4. accept()    - Accept client connection
5. send()/recv() - Exchange data
6. close()     - Close connection
```

### Working with Addresses

```rust
use multios_networking::{IpAddress, SocketAddr};

// IPv4 address
let ipv4 = IpAddress::new_v4(192, 168, 1, 100, 8080);
let socket_addr = SocketAddr::new(ipv4, Protocol::Tcp);

// IPv6 address
let ipv6 = IpAddress::new_v6(
    0x2001, 0x0db8, 0x85a3, 0x0000,
    0x0000, 0x8a2e, 0x0370, 0x7334,
    8080
);
let socket_addr = SocketAddr::new(ipv6, Protocol::Tcp);

// Address ranges
let any = IpAddress::any_v4(); // 0.0.0.0:0
let localhost = IpAddress::localhost_v4(); // 127.0.0.1
let broadcast = IpAddress::broadcast_v4(); // 255.255.255.255
```

## Network Flow Example

Let's trace how data flows through the network stack:

```
Application: HTTP GET request
    ↓
Socket Layer: send() called
    ↓
TCP Layer: 
  - Create TCP segment
  - Set SYN flag (first packet)
  - Calculate sequence number
    ↓
IP Layer:
  - Create IP packet
  - Determine routing
  - Set destination IP
    ↓
Link Layer:
  - Create Ethernet frame
  - Determine MAC address
  - Send to network interface
```

### Three-Way Handshake

```
Client                          Server
  |                               |
  |---------- SYN seq=100 -------->|
  |                               |
  |<------- SYN-ACK seq=200 ------|
  |        ACK=101                |
  |                               |
  |---------- ACK seq=101 -------->|
  |        ACK=201                |
  |                               |
Connection Established!
```

### Data Transfer

```
Client                          Server
  |                               |
  |--- GET / HTTP/1.1 (seq=101) -->|
  |                               |
  |<-- HTTP/1.1 200 OK (seq=201) -|
  |      (ACK=136)                |
  |                               |
  |--- ACK (seq=136) ------------>|
  |                               |
```

## Common Network Programming Patterns

### Echo Server

```rust
use multios_networking::{Socket, SocketType};

fn echo_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = Socket::new(SocketType::Stream)?;
    listener.bind("0.0.0.0:7")?; // Echo port
    listener.listen(1)?;
    
    let (mut socket, addr) = listener.accept()?;
    println!("Echo client connected: {}", addr);
    
    loop {
        let mut buffer = [0u8; 1024];
        let bytes_read = socket.recv(&mut buffer)?;
        
        if bytes_read == 0 {
            break; // Connection closed
        }
        
        socket.send(&buffer[..bytes_read])?;
        println!("Echoed {} bytes", bytes_read);
    }
    
    Ok(())
}
```

### HTTP Client

```rust
fn http_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = Socket::new(SocketType::Stream)?;
    socket.connect("93.184.216.34:80")?; // example.com
    
    let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    socket.send(request.as_bytes())?;
    
    let mut response = String::new();
    let mut buffer = [0u8; 4096];
    
    loop {
        let bytes_read = socket.recv(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        response.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
        
        // Check if we've received the complete response
        if response.contains("\r\n\r\n") {
            break;
        }
    }
    
    println!("Response: {}", response);
    Ok(())
}
```

## Next Steps

In the next tutorial, we'll cover:
- Advanced socket programming techniques
- Non-blocking and asynchronous I/O
- Socket options and timeouts
- Error handling and debugging

## Exercises

1. **Write a simple echo client** that connects to your echo server
2. **Create a DNS client** that resolves domain names to IP addresses
3. **Build a basic HTTP client** that fetches a webpage
4. **Implement a port scanner** that checks which ports are open

## Key Takeaways

- Networks follow layered models (OSI/TCP-IP)
- IP addresses identify hosts, ports identify applications
- UDP is fast but unreliable, TCP is reliable but slower
- Socket programming follows predictable patterns
- Understanding the network flow helps with debugging

## Additional Resources

- [Socket Programming Tutorial](socket_programming.md)
- [TCP/UDP Protocol Deep Dive](protocols.md)
- RFC 791 (IP), RFC 793 (TCP), RFC 768 (UDP)
- Wireshark for network packet analysis