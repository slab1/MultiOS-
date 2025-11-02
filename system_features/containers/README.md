# MultiOS Container System

A comprehensive lightweight container support system for MultiOS that enables safe educational experimentation and isolation. This system provides full container functionality including runtime, orchestration, isolation mechanisms, resource management, networking, security, and educational templates.

## Features

- **Container Runtime**: Full-featured container execution engine
- **Orchestration**: Multi-container management and coordination
- **Namespace Isolation**: Process, network, filesystem, and IPC isolation
- **Resource Management**: CPU, memory, and disk I/O limiting and quotas
- **Image Management**: Layered container image format and management
- **Networking**: Virtual networking with bridge management and port forwarding
- **Security**: Capability-based security and sandboxing
- **Educational Templates**: Pre-configured learning environments
- **Lifecycle Management**: Complete container lifecycle operations

## Architecture Overview

The MultiOS Container System is built with a modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Tools & Utilities                  │
├─────────────────────────────────────────────────────────────┤
│  multios-container  │  containerd  │  container-template   │
├─────────────────────────────────────────────────────────────┤
│                     Core Library                          │
├─────────────────────────────────────────────────────────────┤
│  Orchestration  │  Lifecycle  │  Image Manager  │  Runtime │
├─────────────────────────────────────────────────────────────┤
│  Network Manager │  Security  │  Resource Manager │ Templates │
├─────────────────────────────────────────────────────────────┤
│                     Linux Kernel                           │
│  Namespaces (PID, NET, MNT, IPC, UTS, USER) + Cgroups    │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

1. **Container Runtime** (`runtime.rs`): Core execution engine for containers
2. **Orchestration** (`orchestration.rs`): Multi-container management and coordination
3. **Lifecycle Management** (`lifecycle.rs`): Container creation, starting, stopping, and deletion
4. **Image Manager** (`image_manager.rs`): Container image format and layer management
5. **Network Manager** (`network_manager.rs`): Virtual networking and bridge management
6. **Security Manager** (`security.rs`): Capability-based security and sandboxing
7. **Resource Manager** (`resource_manager.rs`): Cgroup-based resource limiting
8. **Namespace Manager** (`namespaces.rs`): Linux namespace isolation
9. **Template Manager** (`templates.rs`): Educational container templates

## Installation

### Prerequisites

- Linux kernel 4.14+ with namespace support
- Rust 1.70+ and Cargo
- Root privileges for container operations
- Required kernel modules: `overlay`, `br_netfilter`, `veth`

### Build from Source

```bash
# Clone the repository
cd /workspace/system_features/containers

# Build the project
cargo build --release

# Install the binaries (requires sudo)
sudo cargo install --path .

# Verify installation
multios-container --version
```

### Quick Start

```bash
# Start the container daemon
sudo multios-containerd --daemon

# List available templates
multios-container template list

# Create and run a Python learning container
multios-container create --template python-learning python-test
multios-container start python-test
```

## Usage

### Container Management

#### Create a Container

```bash
# Create a container from a template
multios-container create --template python-learning my-python

# Create a container with custom resources
multios-container create \
  --name my-app \
  --cpu-limit 2 \
  --memory-limit 1G \
  --disk-limit 10G \
  python-learning

# Create from custom image
multios-container create --image myapp:latest my-app
```

#### Start, Stop, and Manage Containers

```bash
# Start a container
multios-container start my-python

# Stop a container
multios-container stop my-python

# Restart a container
multios-container restart my-python

# Pause a container
multios-container pause my-python

# Resume a container
multios-container resume my-python

# Delete a container
multios-container delete my-python

# Force delete (even if running)
multios-container delete --force my-python
```

#### Container Operations

```bash
# Execute commands in running container
multios-container exec my-python python3 --version
multios-container exec my-python bash

# Attach to container (interactive mode)
multios-container attach my-python

# View container logs
multios-container logs my-python
multios-container logs --follow my-python

# View container status
multios-container ps
multios-container ps --all

# Inspect container details
multios-container inspect my-python
```

### Image Management

```bash
# List available images
multios-container images

# Pull image from registry
multios-container image pull ubuntu:20.04

# Build image from Dockerfile
multios-container image build -t myapp:latest .

# Push image to registry
multios-container image push myapp:latest

# Remove image
multios-container image rm myapp:latest

# Export image
multios-container image export myapp:latest > myapp.tar

# Import image
multios-container image import < myapp.tar
```

### Networking

```bash
# List networks
multios-container network list

# Create custom network
multios-container network create my-network --subnet 172.20.0.0/16

# Connect container to network
multios-container network connect my-network my-python

# Disconnect container from network
multios-container network disconnect my-network my-python

# Inspect network
multios-container network inspect my-network

# Remove network
multios-container network rm my-network

# Port forwarding
multios-container port-forward my-python 8080:80
```

### Resource Management

```bash
# Update container resources
multios-container update my-python --cpu-limit 4 --memory-limit 2G

# View resource usage
multios-container stats my-python

# Set CPU priority
multios-container update my-python --cpu-shares 512

# Set memory limit
multios-container update my-python --memory-limit 512M

# Set disk I/O limit
multios-container update my-python --io-limit 1000
```

### Educational Templates

```bash
# List all available templates
multios-container template list

# Create container from template
multios-container create --template python-learning python-lab

# Template-specific operations
multios-container template inspect python-learning
multios-container template show python-learning

# Create custom template
multios-container template create my-template --base ubuntu:20.04
```

## Educational Templates

### Available Templates

#### Python Learning Environment (`python-learning`)
- **Base**: Ubuntu 20.04 with Python 3.9+
- **Includes**: pip, virtualenv, jupyter notebook, common packages
- **Use Case**: Python programming education, data science, web development
- **Resource**: 1 CPU, 1GB RAM minimum

```bash
multios-container create --template python-learning python-course
multios-container start python-course
multios-container exec python-course python3 -m venv myenv
```

#### Node.js Development (`nodejs-learning`)
- **Base**: Ubuntu 20.04 with Node.js 16+
- **Includes**: npm, yarn, express-generator, common tools
- **Use Case**: JavaScript/Node.js learning, web development
- **Resource**: 1 CPU, 1GB RAM minimum

```bash
multios-container create --template nodejs-learning node-course
multios-container start node-course
multios-container exec node-course npx create-react-app myapp
```

#### Java Development (`java-learning`)
- **Base**: Ubuntu 20.04 with OpenJDK 11
- **Includes**: Maven, Gradle, common libraries
- **Use Case**: Java programming, enterprise development
- **Resource**: 2 CPU, 2GB RAM minimum

```bash
multios-container create --template java-learning java-course
multios-container start java-course
multios-container exec java-course mvn --version
```

#### C/C++ Development (`cpp-learning`)
- **Base**: Ubuntu 20.04 with GCC 9+
- **Includes**: build-essential, gdb, valgrind
- **Use Case**: Systems programming, competitive programming
- **Resource**: 1 CPU, 1GB RAM minimum

```bash
multios-container create --template cpp-learning cpp-course
multios-container start cpp-course
multios-container exec cpp-course gcc --version
```

#### Web Development (`web-learning`)
- **Base**: Ubuntu 20.04 with full web stack
- **Includes**: Apache, PHP, MySQL, Node.js, Python
- **Use Case**: Full-stack web development education
- **Resource**: 2 CPU, 2GB RAM minimum

```bash
multios-container create --template web-learning web-course
multios-container start web-course
multios-container exec web-course apache2ctl start
```

#### Database Learning (`database-learning`)
- **Base**: Ubuntu 20.04 with multiple databases
- **Includes**: MySQL, PostgreSQL, MongoDB, Redis
- **Use Case**: Database administration, SQL learning
- **Resource**: 2 CPU, 2GB RAM minimum

```bash
multios-container create --template database-learning db-course
multios-container start db-course
multios-container exec db-course mysql --version
```

#### Networking Lab (`networking-learning`)
- **Base**: Ubuntu 20.04 with networking tools
- **Includes**: tcpdump, wireshark, netcat, iperf, nmap
- **Use Case**: Network administration, security testing
- **Resource**: 1 CPU, 1GB RAM minimum

```bash
multios-container create --template networking-learning net-course
multios-container start net-course
multios-container exec net-course tcpdump --version
```

## Configuration

### Container Configuration

Containers can be configured through JSON files:

```json
{
  "name": "my-app",
  "image": "python-learning",
  "resources": {
    "cpu_limit": 2.0,
    "memory_limit": "1GB",
    "disk_limit": "10GB",
    "cpu_shares": 512,
    "io_limit": 1000
  },
  "networking": {
    "network": "default",
    "ports": [
      {"container": 8080, "host": 8080},
      {"container": 3000, "host": 3000}
    ]
  },
  "security": {
    "capabilities": ["CHOWN", "SETGID", "SETUID"],
    "seccomp_profile": "default",
    "read_only": false
  },
  "volumes": [
    {"host": "/host/data", "container": "/data", "mode": "rw"},
    {"host": "/host/config", "container": "/config", "mode": "ro"}
  ],
  "environment": {
    "PYTHONPATH": "/app",
    "DEBUG": "true"
  },
  "restart_policy": "unless-stopped"
}
```

### Resource Limits

- **CPU**: Limit CPU usage (cores or percentage)
- **Memory**: Limit RAM usage (bytes with K, M, G suffixes)
- **Disk**: Limit disk space (bytes with K, M, G suffixes)
- **I/O**: Limit disk I/O operations per second

### Security Configuration

- **Capabilities**: Linux capabilities to grant/deny
- **Seccomp**: System call filtering
- **AppArmor**: Mandatory access control profiles
- **Read-only**: Mount root filesystem as read-only
- **User namespace**: Map container user to host user

## Container Daemon

The `multios-containerd` daemon provides background container management:

```bash
# Start daemon
sudo multios-containerd --daemon

# Start with custom configuration
sudo multios-containerd --daemon --config /etc/multios/containers.conf

# Run as foreground process (for debugging)
multios-containerd --foreground --log-level debug

# Stop daemon
multios-containerd stop

# Reload configuration
multios-containerd reload
```

### Daemon Configuration

```toml
[daemon]
data_root = "/var/lib/multios/containers"
exec_root = "/var/run/multios/containers"
pid_file = "/var/run/multios/containers.pid"

[logging]
level = "info"
file = "/var/log/multios/containers.log"

[security]
seccomp_profile = "/etc/multios/seccomp.json"
apparmor_profile = "multios-default"

[default_resources]
cpu_limit = 2.0
memory_limit = "1GB"
disk_limit = "10GB"

[networking]
bridge = "multios0"
subnet = "172.17.0.0/16"
```

## Monitoring and Debugging

### Container Statistics

```bash
# Real-time container stats
multios-container stats

# Stats for specific container
multios-container stats my-python

# Detailed container information
multios-container inspect my-python
```

### Logging

```bash
# View container logs
multios-container logs my-python

# Follow logs in real-time
multios-container logs --follow my-python

# View daemon logs
journalctl -u multios-containerd
```

### Debugging Tools

```bash
# Access container filesystem
multios-container shell my-python

# Check container processes
multios-container top my-python

# Validate container configuration
multios-container validate my-python-config.json

# Export container filesystem
multios-container export my-python > container-fs.tar
```

## Troubleshooting

### Common Issues

#### Container fails to start
```bash
# Check container logs
multios-container logs container-name

# Validate configuration
multios-container inspect container-name

# Check system resources
multios-container stats
```

#### Network connectivity issues
```bash
# Check network configuration
multios-container network list

# Inspect container network
multios-container inspect container-name | grep Network

# Test connectivity
multios-container exec container-name ping 8.8.8.8
```

#### Resource limit exceeded
```bash
# Check resource usage
multios-container stats container-name

# Update resource limits
multios-container update container-name --memory-limit 2G
```

### Kernel Requirements

Ensure your system has the required kernel features:

```bash
# Check namespace support
cat /proc/self/status | grep CapEff

# Check cgroup support
mount | grep cgroup

# Verify overlay filesystem
lsmod | grep overlay

# Check network capabilities
lsmod | grep br_netfilter
```

## Examples

### Complete Python Learning Workflow

See [Python Learning Example](examples/python-learning.md) for a complete step-by-step guide.

### Multi-Container Application

```bash
# Create network for application
multios-container network create app-network

# Create web server container
multios-container create --template web-learning \
  --network app-network \
  --name web-server \
  --port 8080:80 \
  web-learning

# Create database container
multios-container create --template database-learning \
  --network app-network \
  --name database \
  database-learning

# Start both containers
multios-container start web-server
multios-container start database

# Connect web server to database
multios-container exec web-server mysql -h database -u root -p
```

### Development Environment

```bash
# Create development environment
multios-container create \
  --name dev-environment \
  --cpu-limit 4 \
  --memory-limit 4G \
  --disk-limit 50G \
  --volume /host/workspace:/workspace \
  --volume /host/.ssh:/root/.ssh:ro \
  python-learning

# Start development environment
multios-container start dev-environment

# Attach and develop
multios-container exec dev-environment bash
cd /workspace
git clone https://github.com/example/project.git
```

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the MultiOS project.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:
- GitHub Issues: [MultiOS Issues](https://github.com/multios/issues)
- Documentation: [MultiOS Docs](https://docs.multios.org)
- Community: [MultiOS Community](https://community.multios.org)

---

**MultiOS Container System** - Enabling safe educational experimentation through container technology.