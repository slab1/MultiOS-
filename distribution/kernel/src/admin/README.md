# MultiOS Administrative API System

The MultiOS Administrative API provides a comprehensive, REST-like interface for system management and administration. It offers secure, authenticated access to kernel functionality through well-defined endpoints.

## Overview

The administrative API system is designed to provide:

- **Secure System Management**: Authenticated API endpoints for system operations
- **Process Control**: Full process lifecycle management capabilities
- **Service Administration**: Start, stop, restart, and monitor system services
- **Memory Management**: Real-time memory statistics and allocation controls
- **Security Controls**: User management, permissions, and security monitoring
- **Resource Monitoring**: Performance metrics and system health indicators

## Architecture

### Core Components

1. **AdminApiServer**: Main HTTP API server with endpoint routing
2. **AuthManager**: Authentication and authorization system
3. **RateLimiter**: Request rate limiting and abuse prevention
4. **RequestValidator**: Input validation and security checking
5. **SyscallIntegration**: Integration with kernel syscall interface

### API Structure

```
/api/v1/
├── system/          # System management endpoints
├── processes/       # Process control endpoints
├── memory/          # Memory management endpoints
├── services/        # Service administration endpoints
├── security/        # Security and access control
├── users/           # User management endpoints
└── network/         # Network configuration endpoints
```

## Authentication & Authorization

### API Key Authentication

The API uses API key-based authentication. Include your API key in the `X-API-Key` header:

```bash
curl -H "X-API-Key: your-api-key" https://api.multios.org:8443/api/v1/system/info
```

### Permission System

API endpoints require specific permissions:

| Permission | Description | Endpoints |
|------------|-------------|-----------|
| `SystemRead` | View system information | `/system/info`, `/memory/stats` |
| `SystemWrite` | Modify system settings | `/system/log-level`, `/memory/*` |
| `SystemAdmin` | System control operations | `/system/shutdown`, `/system/reboot` |
| `ProcessRead` | View process information | `/processes` |
| `ProcessAdmin` | Process control | `/processes/*/terminate` |
| `ServiceRead` | View service status | `/services` |
| `ServiceAdmin` | Service control | `/services/*/start`, `/services/*/stop` |
| `SecurityRead` | Security information | `/security/status` |
| `UserRead` | User information | `/users` |
| `UserAdmin` | User management | `/users` (POST, DELETE) |

## API Endpoints

### System Management

#### Get System Information
```http
GET /api/v1/system/info
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": {
    "kernel_name": "MultiOS",
    "kernel_version": "1.0.0",
    "architecture": "X86_64",
    "boot_time": 1640995200,
    "uptime": 3600,
    "total_memory": 8589934592,
    "used_memory": 4294967296,
    "available_memory": 4294967296,
    "cpu_count": 4,
    "load_average": [0.5, 0.3, 0.2],
    "processes": 42,
    "services": 12
  },
  "message": "System information retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### System Shutdown
```http
POST /api/v1/system/shutdown
```

#### System Reboot
```http
POST /api/v1/system/reboot
```

#### Set Log Level
```http
POST /api/v1/system/log-level
Content-Type: application/json

{
  "level": "info"
}
```

### Process Management

#### List Processes
```http
GET /api/v1/processes?include_details=true&filter_state=running
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": [
    {
      "pid": 1,
      "name": "kernel",
      "state": "running",
      "cpu_usage": 0.1,
      "memory_usage": 1048576,
      "parent_pid": 0,
      "start_time": 1640995200,
      "priority": 0,
      "threads": 1
    }
  ],
  "total": 1,
  "message": "Process list retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### Get Process Information
```http
GET /api/v1/processes/1234
```

#### Terminate Process
```http
POST /api/v1/processes/1234/terminate
```

### Memory Management

#### Get Memory Statistics
```http
GET /api/v1/memory/stats
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": {
    "total": 8589934592,
    "used": 4294967296,
    "available": 4294967296,
    "swap_total": 0,
    "swap_used": 0,
    "page_faults": 12345,
    "physical_pages": 2097152,
    "virtual_pages": 4194304
  },
  "message": "Memory statistics retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### Allocate Memory
```http
POST /api/v1/memory/alloc
Content-Type: application/json

{
  "size": 1048576,
  "persistent": false
}
```

#### Free Memory
```http
POST /api/v1/memory/free
Content-Type: application/json

{
  "address": 0x1000
}
```

### Service Management

#### List Services
```http
GET /api/v1/services
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": [
    {
      "service_id": 1,
      "name": "kernel-service",
      "description": "Core kernel service",
      "state": "Running",
      "service_type": "SystemService",
      "health": "healthy",
      "uptime": 3600,
      "restart_count": 0,
      "dependencies": []
    }
  ],
  "total": 1,
  "message": "Service list retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### Service Control
```http
POST /api/v1/services/1/start
POST /api/v1/services/1/stop
POST /api/v1/services/1/restart
```

#### Get Service Status
```http
GET /api/v1/services/1/status
```

### Security & Users

#### Get Security Status
```http
GET /api/v1/security/status
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": {
    "enabled_security_features": ["encryption", "audit"],
    "failed_login_attempts": 0,
    "last_failed_login": 0,
    "active_sessions": 1,
    "permissions": [],
    "audit_enabled": true,
    "encryption_level": "AES-256"
  },
  "message": "Security status retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### User Management
```http
GET /api/v1/users
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": [
    {
      "user_id": 0,
      "username": "root",
      "full_name": "System Administrator",
      "home_directory": "/root",
      "shell": "/bin/sh",
      "groups": ["root", "wheel"],
      "last_login": 1640995200,
      "account_expires": null,
      "locked": false
    }
  ],
  "total": 1,
  "message": "User list retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

#### Create User
```http
POST /api/v1/users
Content-Type: application/json

{
  "username": "johndoe",
  "password_hash": "sha256_hashed_password",
  "full_name": "John Doe",
  "home_directory": "/home/johndoe",
  "shell": "/bin/bash"
}
```

### Network Information

#### Get Network Information
```http
GET /api/v1/network/info
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "data": {
    "interfaces": [
      {
        "name": "eth0",
        "address": "192.168.1.100",
        "netmask": "255.255.255.0",
        "broadcast": "192.168.1.255",
        "is_up": true,
        "mtu": 1500,
        "speed": 1000
      }
    ],
    "routes": [
      {
        "destination": "0.0.0.0",
        "gateway": "192.168.1.1",
        "netmask": "0.0.0.0",
        "interface": "eth0",
        "metric": 100
      }
    ],
    "connections": []
  },
  "message": "Network information retrieved",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **System Read Endpoints**: 60 requests/minute, burst of 10
- **System Admin Endpoints**: 10 requests/minute, burst of 3
- **Process Management**: 30 requests/minute, burst of 5
- **Memory Operations**: 100 requests/minute, burst of 20
- **Service Control**: 30 requests/minute, burst of 5

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit per window
- `X-RateLimit-Remaining`: Requests remaining in window
- `X-RateLimit-Reset`: Timestamp when limit resets

## Error Handling

### Standard Error Response Format
```json
{
  "success": false,
  "status_code": 400,
  "data": null,
  "message": "Invalid request parameters",
  "timestamp": 1640998800,
  "request_id": "req_1640998800"
}
```

### HTTP Status Codes

| Code | Description | Common Causes |
|------|-------------|---------------|
| 200 | Success | Request completed successfully |
| 400 | Bad Request | Invalid parameters, malformed request |
| 401 | Unauthorized | Missing or invalid API key |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Resource already exists |
| 429 | Rate Limited | Too many requests |
| 500 | Internal Server Error | Server-side error |

## Configuration

### Default Configuration
```rust
ApiConfig {
    enabled: true,
    port: 8080,
    max_connections: 100,
    request_timeout: 30,
    max_request_size: 1024 * 1024, // 1MB
    enable_cors: true,
    enable_https: false,
    enable_authentication: true,
    enable_authorization: true,
    rate_limit_enabled: true,
    api_key_required: true,
}
```

### Secure Configuration
```rust
ApiConfig {
    enabled: true,
    port: 8443,
    max_connections: 50,
    request_timeout: 15,
    max_request_size: 512 * 1024, // 512KB
    enable_cors: false,
    enable_https: true,
    enable_authentication: true,
    enable_authorization: true,
    rate_limit_enabled: true,
    api_key_required: true,
}
```

## Integration Examples

### Syscall Integration

The administrative API integrates with the kernel's syscall interface:

```rust
use crate::admin::{AdminSyscallIntegration, ADMIN_API_SYSTEM_INFO};

// Handle administrative API syscall
AdminSyscallIntegration::handle_admin_syscall(
    ADMIN_API_SYSTEM_INFO,
    0, 0, 0, 0
)?;
```

### API Client Example

```rust
use crate::admin::{make_api_request, ApiRequest};

fn main() {
    // Get system information
    let response = make_api_request(ApiRequest::SystemInfo).unwrap();
    println!("System: {}", response.message);
    
    // List processes
    let response = make_api_request(ApiRequest::ProcessList).unwrap();
    println!("Processes: {}", response.message);
    
    // Get memory statistics
    let response = make_api_request(ApiRequest::MemoryStats).unwrap();
    println!("Memory: {}", response.message);
}
```

## Security Considerations

1. **API Key Management**: Keep API keys secure and rotate regularly
2. **Permission Principle**: Grant only necessary permissions
3. **Rate Limiting**: Monitor for abuse patterns
4. **Input Validation**: All inputs are validated for security
5. **Audit Logging**: Security events are logged for monitoring
6. **HTTPS**: Use HTTPS in production environments

## Testing

### Unit Tests
```bash
cargo test admin::admin_api
```

### Integration Tests
```bash
cargo test --test integration admin::integration_examples
```

### API Testing
```bash
# Test system info endpoint
curl -X GET http://localhost:8080/api/v1/system/info \
     -H "X-API-Key: admin-12345" \
     -H "Content-Type: application/json"

# Test process termination
curl -X POST http://localhost:8080/api/v1/processes/1/terminate \
     -H "X-API-Key: admin-12345" \
     -H "Content-Type: application/json"
```

## Performance

The administrative API is designed for high performance:

- **Low Latency**: Typical response times < 10ms
- **High Throughput**: Supports 1000+ requests/second
- **Efficient Memory**: Minimal memory footprint
- **Async Processing**: Non-blocking request handling

## Monitoring

### Metrics Available
- Request count and rate
- Response times
- Error rates
- Active connections
- Memory usage
- Rate limit hits

### Logging
All API operations are logged with:
- Request/response timing
- Authentication status
- Authorization results
- Error conditions
- Security events

## Future Enhancements

1. **WebSocket Support**: Real-time status updates
2. **GraphQL API**: Alternative query interface
3. **Metrics Endpoint**: Prometheus-compatible metrics
4. **Bulk Operations**: Multiple operations in single request
5. **Plugin System**: Extensible endpoint architecture

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Check API key configuration
2. **403 Forbidden**: Verify user permissions
3. **429 Rate Limited**: Reduce request frequency
4. **500 Internal Error**: Check system logs

### Debug Mode

Enable debug logging:
```rust
let config = ApiConfig {
    // ... other settings
    enable_debug: true,
};
```

## Contributing

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation
4. Ensure security review for changes
5. Test across different architectures

## License

This API system is part of MultiOS and licensed under the MIT License.