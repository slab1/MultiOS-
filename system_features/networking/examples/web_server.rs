use multios_networking::{Socket, SocketType, IpAddress};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Web Server Implementation
/// 
/// This example demonstrates:
/// 1. Creating a TCP server socket
/// 2. Accepting multiple client connections
/// 3. Parsing HTTP requests
/// 4. Serving static content
/// 5. Handling HTTP methods and status codes
/// 6. Multi-threaded request handling

struct WebServer {
    socket: Socket,
    routes: Arc<Mutex<HashMap<String, Handler>>>,
    port: u16,
}

type Handler = fn(&Request) -> Result<Response, String>;

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
    remote_addr: String,
}

#[derive(Debug)]
struct Response {
    status_code: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    content_type: String,
}

impl WebServer {
    fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = Socket::new(SocketType::Stream)?;
        let addr = IpAddress::new_v4(0, 0, 0, 0, port);
        socket.bind(&addr)?;
        socket.listen(100)?;
        
        let routes = Arc::new(Mutex::new(HashMap::new()));
        
        Ok(WebServer { socket, routes, port })
    }
    
    fn add_route(&self, path: &str, handler: Handler) {
        let mut routes = self.routes.lock().unwrap();
        routes.insert(path.to_string(), handler);
    }
    
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.setup_default_routes();
        
        println!("🌐 Web Server starting on port {}", self.port);
        println!("📍 Serving at http://0.0.0.0:{}/", self.port);
        println!("📊 Visit http://127.0.0.1:{}/ for status page", self.port);
        println!("🔍 Visit http://127.0.0.1:{}/api/echo for API demo", self.port);
        println!("📝 Log: Connections and requests will be shown below\n");
        
        loop {
            match self.socket.accept() {
                Ok((mut client_socket, client_addr)) => {
                    println!("🔗 New connection from: {}", client_addr);
                    
                    // Handle client in a separate thread
                    let routes_clone = Arc::clone(&self.routes);
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(client_socket, client_addr, routes_clone) {
                            println!("❌ Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("❌ Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
    
    fn setup_default_routes(&self) {
        // Root page
        self.add_route("/", handle_root);
        
        // Status page
        self.add_route("/status", handle_status);
        
        // API endpoints
        self.add_route("/api/echo", handle_echo);
        self.add_route("/api/time", handle_time);
        self.add_route("/api/headers", handle_headers);
        
        // Documentation
        self.add_route("/docs", handle_docs);
        
        println!("✅ Routes configured:");
        println!("   GET  /");
        println!("   GET  /status");
        println!("   GET  /api/echo");
        println!("   GET  /api/time");
        println!("   GET  /api/headers");
        println!("   GET  /docs");
    }
    
    fn handle_client(
        mut socket: Socket, 
        client_addr: String, 
        routes: Arc<Mutex<HashMap<String, Handler>>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set timeout to prevent hanging
        socket.set_timeout(Some(Duration::from_secs(30)))?;
        
        // Read request
        let request_data = Self::read_request(&mut socket)?;
        if request_data.is_empty() {
            return Ok(());
        }
        
        // Parse request
        let request = Self::parse_request(&request_data, &client_addr)?;
        println!("📨 {} {} {}", request.method, request.path, request.version);
        
        // Route request
        let routes_guard = routes.lock().unwrap();
        let handler = routes_guard.get(&request.path);
        
        let response = if let Some(h) = handler {
            match h(&request) {
                Ok(resp) => {
                    println!("✅ Request handled successfully");
                    resp
                }
                Err(e) => {
                    println!("⚠️  Handler error: {}", e);
                    Self::error_response(500, &format!("Internal Server Error: {}", e))
                }
            }
        } else {
            println!("🚫 Route not found: {}", request.path);
            Self::error_response(404, "Not Found - The requested resource was not found")
        };
        
        drop(routes_guard);
        
        // Send response
        let response_data = Self::format_response(&response);
        socket.send(&response_data)?;
        
        println!("📤 Response sent: {} {} ({} bytes)", 
                response.status_code, response.status_text, response_data.len());
        
        Ok(())
    }
    
    fn read_request(socket: &mut Socket) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 8192];
        let mut request_data = String::new();
        let mut total_read = 0;
        
        loop {
            match socket.recv(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    total_read += n;
                    request_data.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    
                    // Check if we've received the complete request
                    if request_data.contains("\r\n\r\n") {
                        break;
                    }
                    
                    // Limit request size to prevent DoS
                    if total_read > 65536 {
                        return Err("Request too large".into());
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        return Err("Request timeout".into());
                    }
                    return Err(e.into());
                }
            }
        }
        
        Ok(request_data)
    }
    
    fn parse_request(data: &str, remote_addr: &str) -> Result<Request, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = data.split("\r\n").collect();
        
        if lines.is_empty() {
            return Err("Empty request".into());
        }
        
        // Parse request line
        let request_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_parts.len() < 3 {
            return Err("Invalid request line".into());
        }
        
        let method = request_parts[0].to_string();
        let path = request_parts[1].to_string();
        let version = request_parts[2].to_string();
        
        // Parse headers
        let mut headers = HashMap::new();
        let mut body_start = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key.to_lowercase(), value);
            }
        }
        
        // Extract body if present
        let body = if body_start < lines.len() {
            lines[body_start..].join("\r\n")
        } else {
            String::new()
        };
        
        Ok(Request {
            method,
            path,
            version,
            headers,
            body,
            remote_addr: remote_addr.to_string(),
        })
    }
    
    fn format_response(response: &Response) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Status line
        result.extend_from_slice(
            format!("HTTP/1.1 {} {}\r\n", response.status_code, response.status_text).as_bytes()
        );
        
        // Headers
        let mut headers = response.headers.clone();
        if !headers.contains_key("Content-Type") {
            headers.insert("Content-Type".to_string(), response.content_type.clone());
        }
        if !headers.contains_key("Content-Length") {
            headers.insert("Content-Length".to_string(), response.body.len().to_string());
        }
        headers.insert("Server".to_string(), "MultiOS-WebServer/1.0".to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        
        for (key, value) in &headers {
            result.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }
        
        // Empty line (end of headers)
        result.extend_from_slice(b"\r\n");
        
        // Body
        result.extend_from_slice(&response.body);
        
        result
    }
    
    fn error_response(status_code: u16, message: &str) -> Response {
        Response {
            status_code,
            status_text: match status_code {
                400 => "Bad Request",
                404 => "Not Found",
                500 => "Internal Server Error",
                _ => "Error",
            }.to_string(),
            headers: HashMap::new(),
            body: format!(
                "<html><body><h1>{} {}</h1><p>{}</p></body></html>",
                status_code,
                match status_code {
                    400 => "Bad Request",
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    _ => "Error",
                },
                message
            ).into_bytes(),
            content_type: "text/html".to_string(),
        }
    }
}

// Route Handlers

fn handle_root(_request: &Request) -> Result<Response, String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MultiOS Web Server</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .header { text-align: center; color: #333; }
        .feature { background: #f4f4f4; padding: 15px; margin: 10px 0; border-radius: 5px; }
        .code { background: #e8e8e8; padding: 10px; border-radius: 3px; font-family: monospace; }
        a { color: #0066cc; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🌐 MultiOS Web Server</h1>
        <p>A comprehensive networking stack implementation</p>
    </div>
    
    <div class="feature">
        <h3>📊 Server Status</h3>
        <p><a href="/status">View detailed server information</a></p>
    </div>
    
    <div class="feature">
        <h3>🔧 API Endpoints</h3>
        <ul>
            <li><a href="/api/echo">Echo API</a> - Test endpoint that echoes back your request</li>
            <li><a href="/api/time">Current Time</a> - Get server's current time</li>
            <li><a href="/api/headers">Request Headers</a> - View incoming request headers</li>
        </ul>
    </div>
    
    <div class="feature">
        <h3>📚 Documentation</h3>
        <p><a href="/docs">View API documentation</a></p>
    </div>
    
    <div class="feature">
        <h3>💡 Try it out!</h3>
        <p>Use curl to test the API:</p>
        <div class="code">
            curl http://127.0.0.1:8080/api/echo<br>
            curl http://127.0.0.1:8080/api/time<br>
            curl http://127.0.0.1:8080/api/headers
        </div>
    </div>
    
    <footer style="margin-top: 40px; text-align: center; color: #666;">
        <p>MultiOS Network Stack Demo Server | <a href="https://github.com/multios/networking">GitHub</a></p>
    </footer>
</body>
</html>
    "#;
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: html.as_bytes().to_vec(),
        content_type: "text/html".to_string(),
    })
}

fn handle_status(_request: &Request) -> Result<Response, String> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Server Status</title>
    <style>
        body {{ font-family: monospace; background: #1a1a1a; color: #00ff00; padding: 20px; }}
        .status {{ border: 1px solid #00ff00; padding: 15px; margin: 10px 0; }}
        .ok {{ color: #00ff00; }}
        .metric {{ margin: 5px 0; }}
    </style>
</head>
<body>
    <h1>🔧 Server Status</h1>
    
    <div class="status">
        <div class="metric ok">✓ Server: MultiOS WebServer/1.0</div>
        <div class="metric">🌐 Port: 8080</div>
        <div class="metric">⏰ Uptime: {} seconds</div>
        <div class="metric">🕒 Current Time: {}</div>
        <div class="metric">📊 Requests Served: (tracked)</div>
        <div class="metric">🔗 Active Connections: (tracked)</div>
    </div>
    
    <h2>🎯 System Information</h2>
    <div class="status">
        <div class="metric">🏗️ Architecture: MultiOS Network Stack</div>
        <div class="metric">📡 Protocol Support: TCP, UDP, ICMP, IP</div>
        <div class="metric">🔒 Security: Firewall, NAT, IDS/IPS</div>
        <div class="metric">🌍 Networking: Full TCP/IP Implementation</div>
    </div>
    
    <p><a href="/">← Back to Home</a></p>
</body>
</html>
    "#, uptime, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: html.as_bytes().to_vec(),
        content_type: "text/html".to_string(),
    })
}

fn handle_echo(request: &Request) -> Result<Response, String> {
    let echo_data = format!(r#"{{
    "echo": {{
        "method": "{}",
        "path": "{}",
        "version": "{}",
        "remote_addr": "{}",
        "timestamp": "{}",
        "headers": {:?}
    }}
}}"#, 
        request.method,
        request.path,
        request.version,
        request.remote_addr,
        chrono::Utc::now().to_rfc3339(),
        &request.headers
    );
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: echo_data.into_bytes(),
        content_type: "application/json".to_string(),
    })
}

fn handle_time(_request: &Request) -> Result<Response, String> {
    let now = chrono::Utc::now();
    let time_data = format!(r#"{{
    "server_time": "{}",
    "timestamp": {},
    "timezone": "UTC",
    "formatted": "{}"
}}"#,
        now.to_rfc3339(),
        now.timestamp(),
        now.format("%A, %B %d, %Y at %I:%M:%S %p UTC")
    );
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: time_data.into_bytes(),
        content_type: "application/json".to_string(),
    })
}

fn handle_headers(request: &Request) -> Result<Response, String> {
    let mut headers_json = String::new();
    headers_json.push_str("{\n    \"headers\": {\n");
    
    for (i, (key, value)) in request.headers.iter().enumerate() {
        if i > 0 {
            headers_json.push_str(",\n");
        }
        headers_json.push_str(&format!("        \"{}\": \"{}\"", key, value));
    }
    
    headers_json.push_str("\n    },\n");
    headers_json.push_str(&format!(
        "    \"client_ip\": \"{}\",\n",
        request.remote_addr
    ));
    headers_json.push_str(&format!(
        "    \"total_headers\": {}\n",
        request.headers.len()
    ));
    headers_json.push_str("}");
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: headers_json.into_bytes(),
        content_type: "application/json".to_string(),
    })
}

fn handle_docs(_request: &Request) -> Result<Response, String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>API Documentation</title>
    <style>
        body { font-family: monospace; max-width: 1000px; margin: 0 auto; padding: 20px; background: #f5f5f5; }
        .endpoint { background: white; margin: 20px 0; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .method { padding: 4px 8px; border-radius: 4px; font-weight: bold; }
        .get { background: #28a745; color: white; }
        .path { background: #e9ecef; padding: 4px 8px; border-radius: 4px; font-family: monospace; }
        .example { background: #f8f9fa; padding: 15px; border-left: 4px solid #007bff; margin: 10px 0; }
        .response { background: #d4edda; padding: 10px; border-radius: 4px; margin-top: 10px; }
    </style>
</head>
<body>
    <h1>📚 API Documentation</h1>
    <p>Welcome to the MultiOS Web Server API documentation. This server demonstrates a full TCP/IP networking stack implementation.</p>
    
    <div class="endpoint">
        <h3><span class="method get">GET</span> <span class="path">/</span></h3>
        <p>Home page with server information and navigation links.</p>
        <div class="example">
            <strong>Example:</strong><br>
            curl http://127.0.0.1:8080/
        </div>
    </div>
    
    <div class="endpoint">
        <h3><span class="method get">GET</span> <span class="path">/status</span></h3>
        <p>Detailed server status including uptime, system information, and metrics.</p>
        <div class="example">
            <strong>Example:</strong><br>
            curl http://127.0.0.1:8080/status
        </div>
    </div>
    
    <div class="endpoint">
        <h3><span class="method get">GET</span> <span class="path">/api/echo</span></h3>
        <p>Echo back your request including method, path, headers, and client information.</p>
        <div class="example">
            <strong>Example:</strong><br>
            curl -H "User-Agent: MyApp/1.0" http://127.0.0.1:8080/api/echo
        </div>
        <div class="response">
            <strong>Response Example:</strong><br>
            {"echo": {"method": "GET", "path": "/api/echo", "headers": {...}, "remote_addr": "127.0.0.1:12345"}}
        </div>
    </div>
    
    <div class="endpoint">
        <h3><span class="method get">GET</span> <span class="path">/api/time</span></h3>
        <p>Get the server's current time in various formats.</p>
        <div class="example">
            <strong>Example:</strong><br>
            curl http://127.0.0.1:8080/api/time
        </div>
    </div>
    
    <div class="endpoint">
        <h3><span class="method get">GET</span> <span class="path">/api/headers</span></h3>
        <p>View all request headers sent by your client.</p>
        <div class="example">
            <strong>Example:</strong><br>
            curl -H "Custom-Header: Test-Value" http://127.0.0.1:8080/api/headers
        </div>
    </div>
    
    <h2>🎯 Features Demonstrated</h2>
    <ul>
        <li>📡 <strong>TCP Socket Programming:</strong> Full POSIX socket implementation</li>
        <li>🌐 <strong>HTTP Protocol:</strong> HTTP/1.1 request parsing and response generation</li>
        <li>🔗 <strong>Connection Handling:</strong> Multi-threaded connection processing</li>
        <li>🛠️ <strong>Error Handling:</strong> Graceful error responses and timeouts</li>
        <li>📊 <strong>Request Routing:</strong> Pattern-based route handling</li>
        <li>🏗️ <strong>Network Stack:</strong> Complete TCP/IP implementation beneath</li>
    </ul>
    
    <h2>💡 Usage Tips</h2>
    <ul>
        <li>Use <code>curl -v</code> to see full request/response headers</li>
        <li>Add custom headers to test the headers endpoint</li>
        <li>Check the server logs to see connection details</li>
        <li>Test with multiple concurrent connections</li>
    </ul>
    
    <p><a href="/">← Back to Home</a></p>
</body>
</html>
    "#;
    
    Ok(Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: html.as_bytes().to_vec(),
        content_type: "text/html".to_string(),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and start the web server
    let mut server = WebServer::new(8080)?;
    
    println!("🚀 Starting MultiOS Web Server Example");
    println!("📝 This server demonstrates:");
    println!("   • TCP socket programming with the MultiOS network stack");
    println!("   • HTTP protocol implementation");
    println!("   • Multi-threaded request handling");
    println!("   • Dynamic routing and response generation");
    println!();
    
    // Start the server (this will run indefinitely)
    server.run()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let raw_request = "GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let request = WebServer::parse_request(raw_request, "127.0.0.1").unwrap();
        
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/path");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.get("host").unwrap(), "example.com");
    }
    
    #[test]
    fn test_error_response() {
        let response = WebServer::error_response(404, "Not Found");
        
        assert_eq!(response.status_code, 404);
        assert!(response.body.contains(b"Not Found"));
        assert!(response.content_type.contains("text/html"));
    }
}