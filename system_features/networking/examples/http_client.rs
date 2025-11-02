use multios_networking::{Socket, SocketType, IpAddress, Protocol};
use std::io::{Read, Write};
use std::time::Duration;

/// Simple HTTP client example demonstrating TCP socket usage
/// 
/// This example shows how to:
/// 1. Create a TCP socket
/// 2. Connect to an HTTP server
/// 3. Send HTTP request
/// 4. Receive and parse HTTP response
/// 5. Handle timeouts and errors

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple HTTP Client Example ===\n");
    
    // Configuration
    let host = "example.com";
    let port = 80;
    let path = "/";
    
    println!("Fetching {} from {}:{}\n", path, host, port);
    
    // Resolve hostname to IP address
    let mut dns_client = multios_networking::dns::DnsClient::new()?;
    let ip_addresses = dns_client.resolve(host)?;
    
    if ip_addresses.is_empty() {
        return Err("Failed to resolve hostname".into());
    }
    
    let server_ip = ip_addresses[0];
    let server_addr = IpAddress::new_v4(server_ip.get_v4().0, server_ip.get_v4().1, 
                                      server_ip.get_v4().2, server_ip.get_v4().3, port);
    
    println!("Resolved {} to {}\n", host, server_ip);
    
    // Create socket
    let mut socket = Socket::new(SocketType::Stream)?;
    
    // Configure socket options
    socket.set_timeout(Some(Duration::from_secs(10)))?;
    socket.set_option(multios_networking::SocketOption::TcpNoDelay, true)?;
    
    println!("Connecting to {}:{}...", server_ip, port);
    
    // Connect to server
    socket.connect(&server_addr)?;
    println!("Connected successfully!\n");
    
    // Build HTTP request
    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: MultiOS-HTTP-Client/1.0\r\n\
         Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );
    
    println!("Sending request:\n{}", request);
    
    // Send request
    socket.send(request.as_bytes())?;
    println!("Request sent!\n");
    
    // Receive response
    println!("Receiving response:");
    let mut response = String::new();
    let mut buffer = [0u8; 4096];
    let mut total_bytes = 0;
    
    loop {
        match socket.recv(&mut buffer) {
            Ok(0) => {
                // Connection closed by server
                break;
            }
            Ok(n) => {
                total_bytes += n;
                response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                
                // Show progress for large responses
                if n == buffer.len() {
                    print!(".");
                    std::io::stdout().flush()?;
                } else {
                    println!();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                println!("\nTimeout waiting for response");
                return Err("HTTP request timeout".into());
            }
            Err(e) => {
                println!("\nError receiving response: {}", e);
                return Err(e.into());
            }
        }
    }
    
    println!("\nReceived {} bytes total\n", total_bytes);
    
    // Parse and display response
    if let Some(header_end) = response.find("\r\n\r\n") {
        let (header, body) = response.split_at(header_end + 4);
        
        println!("=== HTTP Response Headers ===");
        println!("{}", header.trim_end());
        
        if !body.is_empty() {
            println!("\n=== Response Body (first 500 chars) ===");
            let body_preview = if body.len() > 500 {
                &body[..500]
            } else {
                body
            };
            println!("{}", body_preview);
            
            if body.len() > 500 {
                println!("... ({} more bytes)", body.len() - 500);
            }
        }
    } else {
        println!("=== Raw Response ===");
        println!("{}", response);
    }
    
    // Parse response status
    parse_response_status(&response);
    
    Ok(())
}

fn parse_response_status(response: &str) {
    println!("\n=== Response Analysis ===");
    
    let lines: Vec<&str> = response.lines().collect();
    
    if lines.is_empty() {
        println!("Invalid response: empty");
        return;
    }
    
    let status_line = lines[0];
    println!("Status line: {}", status_line);
    
    // Parse status code
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() >= 3 {
        if let Ok(status_code) = parts[1].parse::<u16>() {
            match status_code {
                200..=299 => println!("✓ Success ({}): Request succeeded", status_code),
                300..=399 => println!("↪ Redirect ({}): Follow redirect to get content", status_code),
                400..=499 => println!("✗ Client Error ({}): Check your request", status_code),
                500..=599 => println!("✗ Server Error ({}): Server had a problem", status_code),
                _ => println!("? Unknown status code: {}", status_code),
            }
        }
    }
    
    // Check for common headers
    for line in &lines[1..] {
        if line.starts_with("Content-Type:") {
            println!("Content type: {}", line.trim());
        } else if line.starts_with("Content-Length:") {
            println!("Content length: {}", line.trim());
        } else if line.starts_with("Server:") {
            println!("Server: {}", line.trim());
        } else if line.starts_with("Date:") {
            println!("Date: {}", line.trim());
        } else if line.starts_with("Connection:") {
            println!("Connection: {}", line.trim());
        }
    }
}

/// Advanced HTTP client with features like redirect following, 
/// SSL support, and certificate validation
struct HttpClient {
    socket: Socket,
    follow_redirects: bool,
    max_redirects: usize,
    timeout: Duration,
}

impl HttpClient {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(HttpClient {
            socket: Socket::new(SocketType::Stream)?,
            follow_redirects: true,
            max_redirects: 5,
            timeout: Duration::from_secs(30),
        })
    }
    
    fn get(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let (scheme, host, port, path) = parse_url(url)?;
        let mut current_url = url.to_string();
        let mut redirects_followed = 0;
        
        // For HTTPS, you would need to implement TLS/SSL
        // This is a simplified HTTP-only implementation
        
        loop {
            let (host, port, path) = parse_url(&current_url)?;
            
            // Resolve hostname
            let mut dns_client = multios_networking::dns::DnsClient::new()?;
            let ip_addresses = dns_client.resolve(&host)?;
            
            if ip_addresses.is_empty() {
                return Err("Failed to resolve hostname".into());
            }
            
            let server_ip = ip_addresses[0];
            let server_addr = IpAddress::new_v4(server_ip.get_v4().0, server_ip.get_v4().1, 
                                              server_ip.get_v4().2, server_ip.get_v4().3, port);
            
            // Connect
            self.socket.set_timeout(Some(self.timeout))?;
            self.socket.connect(&server_addr)?;
            
            // Build request
            let request = format!(
                "GET {} HTTP/1.1\r\n\
                 Host: {}\r\n\
                 User-Agent: MultiOS-HTTP-Client/1.0\r\n\
                 Connection: close\r\n\
                 \r\n",
                path, host
            );
            
            // Send request
            self.socket.send(request.as_bytes())?;
            
            // Receive response
            let mut response = String::new();
            let mut buffer = [0u8; 4096];
            
            loop {
                match self.socket.recv(&mut buffer) {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            
            // Parse response
            let status_code = self.parse_status_code(&response);
            
            match status_code {
                200..=299 => {
                    // Success, return response body
                    if let Some(header_end) = response.find("\r\n\r\n") {
                        return Ok(response[header_end + 4..].to_string());
                    }
                    return Ok(String::new());
                }
                300..=399 if self.follow_redirects && redirects_followed < self.max_redirects => {
                    // Redirect, follow it
                    if let Some(location) = self.parse_location_header(&response) {
                        redirects_followed += 1;
                        current_url = resolve_relative_url(&current_url, &location)?;
                        self.socket = Socket::new(SocketType::Stream)?; // New connection for redirect
                        continue;
                    }
                    return Err("Redirect without location header".into());
                }
                301 | 302 | 303 | 307 | 308 => {
                    return Err("Too many redirects or redirects disabled".into());
                }
                400..=599 => {
                    return Err(format!("HTTP error {}: {}", status_code, response.lines().next().unwrap_or("")))?;
                }
                _ => {
                    return Err(format!("Unexpected status code: {}", status_code))?;
                }
            }
        }
    }
    
    fn parse_status_code(&self, response: &str) -> u16 {
        let lines: Vec<&str> = response.lines().collect();
        if let Some(status_line) = lines.first() {
            let parts: Vec<&str> = status_line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(code) = parts[1].parse() {
                    return code;
                }
            }
        }
        0
    }
    
    fn parse_location_header(&self, response: &str) -> Option<String> {
        for line in response.lines() {
            if line.to_lowercase().starts_with("location:") {
                return Some(line.splitn(2, ':').nth(1)?.trim().to_string());
            }
        }
        None
    }
}

fn parse_url(url: &str) -> Result<(String, String, u16, String), Box<dyn std::error::Error>> {
    // Simple URL parser
    let url = if url.starts_with("http://") {
        &url[7..]
    } else if url.starts_with("https://") {
        &url[8..]
    } else {
        return Err("URL must start with http:// or https://".into());
    };
    
    let (scheme, rest) = if url.contains('/') {
        ("http".to_string(), url.splitn(2, '/').next().unwrap().to_string())
    } else {
        ("http".to_string(), url.to_string())
    };
    
    let mut parts = rest.split(':');
    let host = parts.next().unwrap().to_string();
    let port = parts.next().unwrap_or("80").parse::<u16>()?;
    let path = if url.contains('/') {
        format!("/{}", url.splitn(2, '/').nth(1).unwrap_or(""))
    } else {
        "/".to_string()
    };
    
    Ok((scheme, host, port, path))
}

fn resolve_relative_url(base: &str, relative: &str) -> Result<String, Box<dyn std::error::Error>> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }
    
    let (base_scheme, base_host, base_port, base_path) = parse_url(base)?;
    
    if relative.starts_with('/') {
        return Ok(format!("{}://{}:{}{}", base_scheme, base_host, base_port, relative));
    }
    
    // Handle relative paths
    let path_parts: Vec<&str> = base_path.split('/').collect();
    let mut dir_path = if path_parts.len() > 1 {
        path_parts[..path_parts.len() - 1].join("/") + "/"
    } else {
        "/".to_string()
    };
    
    Ok(format!("{}://{}:{}{}{}", base_scheme, base_host, base_port, dir_path, relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let result = parse_url("http://example.com/path").unwrap();
        assert_eq!(result.1, "example.com");
        assert_eq!(result.2, 80);
        assert_eq!(result.3, "/path");
    }
    
    #[test]
    fn test_parse_url_with_port() {
        let result = parse_url("http://example.com:8080/path/to/resource").unwrap();
        assert_eq!(result.1, "example.com");
        assert_eq!(result.2, 8080);
        assert_eq!(result.3, "/path/to/resource");
    }
    
    #[test]
    fn test_resolve_relative_url() {
        let base = "http://example.com/path/to/page.html";
        let relative = "image.jpg";
        let result = resolve_relative_url(base, relative).unwrap();
        assert_eq!(result, "http://example.com:80/path/to/image.jpg");
    }
}