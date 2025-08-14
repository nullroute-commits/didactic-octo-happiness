# Automation Nation

**A comprehensive automation platform for container deployment, system profiling, and infrastructure management**

## Overview

Automation Nation is a sophisticated automation platform that combines system information collection, container orchestration, web-based deployment management, and comprehensive monitoring. Built with a modular architecture using Rust for performance and shell scripts for system integration, it provides enterprise-grade automation capabilities for modern infrastructure.

### Core Capabilities

- **🔍 System Profiling**: Plugin-based system information collection across 10 major CPU architectures
- **🐳 Container Orchestration**: Multi-runtime container management (Docker, Podman, LXC)
- **🌐 Web Interface**: RESTful API and web server for deployment management
- **📊 Monitoring Stack**: Complete observability with ELK, Prometheus, and Grafana
- **🔧 Infrastructure Management**: NetBox integration for network infrastructure tracking
- **🚀 CI/CD Integration**: Comprehensive testing and automated release management

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Automation Nation Platform                   │
├─────────────────────────────────────────────────────────────────┤
│  Web Interface (Rust + Axum)                                   │
│  ├── REST API for container deployment                         │
│  ├── GitHub API integration                                    │
│  ├── System profiling endpoints                                │
│  └── Container runtime management                              │
├─────────────────────────────────────────────────────────────────┤
│  Container Orchestration Layer                                 │
│  ├── Docker Manager                                            │
│  ├── Podman Manager                                            │
│  ├── LXC Manager                                               │
│  └── Runtime Detection & Optimization                          │
├─────────────────────────────────────────────────────────────────┤
│  System Information Collection (collect_info.sh)              │
│  ├── Plugin-based architecture                                 │
│  ├── Multi-architecture support (10 CPU architectures)        │
│  ├── JSON output with integrity verification                   │
│  └── Cross-platform Unix compatibility                         │
├─────────────────────────────────────────────────────────────────┤
│  Monitoring & Observability Stack                              │
│  ├── ELK Stack (Elasticsearch, Logstash, Kibana)             │
│  ├── Prometheus + Grafana                                      │
│  ├── Application metrics and logging                           │
│  └── Container and infrastructure monitoring                   │
├─────────────────────────────────────────────────────────────────┤
│  Infrastructure Management                                      │
│  ├── NetBox (Network infrastructure database)                  │
│  ├── PostgreSQL (Application database)                         │
│  ├── Redis (Caching and session storage)                       │
│  └── Automated backup and maintenance                          │
└─────────────────────────────────────────────────────────────────┘
```

## Features

### System Information Collection
- **Plugin-based Architecture**: Extensible design for easy addition of new data collectors
- **Multi-Architecture Support**: Supports 10 major CPU architectures (x86_64, ARM64, RISC-V, etc.)
- **JSON Output**: Structured, machine-readable output format with integrity verification
- **Cross-Platform**: Works on Linux, macOS, and other Unix-like systems
- **Privilege Management**: Optional sudo support with graceful fallbacks
- **Performance Tuning**: Configurable limits and resource management

### Container Orchestration
- **Multi-Runtime Support**: Docker, Podman, and LXC container management
- **Automated Deployment**: GitHub repository analysis and automatic deployment
- **Resource Management**: Intelligent resource allocation and optimization
- **Security Integration**: Container security scanning and hardening
- **Network Management**: Advanced networking configuration and monitoring

### Web Interface & API
- **RESTful API**: Comprehensive API for automation integration
- **Container Management**: Deploy, manage, and monitor containers via web interface
- **GitHub Integration**: Automatic repository analysis and deployment suggestions
- **System Profiling**: Real-time system information and performance metrics
- **Deployment Profiles**: Pre-configured deployment templates for common applications

### Monitoring & Observability
- **Complete ELK Stack**: Centralized logging with Elasticsearch, Logstash, and Kibana
- **Metrics Collection**: Prometheus-based metrics with Grafana visualization
- **Application Monitoring**: Rust application metrics and performance tracking
- **Infrastructure Monitoring**: Container, network, and system resource monitoring
- **Alerting**: Configurable alerts and notifications

### Infrastructure Management
- **NetBox Integration**: Network infrastructure documentation and management
- **Database Management**: PostgreSQL for application data with automated backups
- **Caching Layer**: Redis for performance optimization and session management
- **Service Discovery**: Automatic service registration and health checking

## Supported Architectures

The system supports the following architectures based on Q4 2024 market data:

1. **x86_64** (AMD64) - Intel/AMD 64-bit
2. **arm64** (aarch64) - ARM 64-bit (Apple Silicon, AWS Graviton, etc.)
3. **i386** (i686) - Intel/AMD 32-bit
4. **ppc64le** - PowerPC 64-bit Little Endian (IBM POWER)
5. **s390x** - IBM Z/Architecture (mainframes)
6. **riscv64** - RISC-V 64-bit (emerging open architecture)
7. **mips64** - MIPS 64-bit (embedded systems, routers)
8. **aarch32** - ARM 32-bit (Raspberry Pi, embedded systems)
9. **sparc64** - SPARC 64-bit (Oracle systems)
10. **loongarch64** - LoongArch 64-bit (Chinese architecture)

## Repository Structure

```
Automation_nation/
├── src/                              # Rust source code
│   ├── bin/                          # Binary applications
│   │   ├── ci_runner.rs              # CI test runner application
│   │   └── web_server.rs             # Web server and API application
│   ├── lib.rs                        # Library root and public API
│   ├── config.rs                     # Configuration management
│   ├── types.rs                      # Common type definitions
│   ├── executor.rs                   # Script execution engine
│   ├── validator.rs                  # Output validation
│   ├── reporter.rs                   # Test result reporting
│   ├── privilege.rs                  # Privilege escalation management
│   ├── os_support.rs                 # Operating system compatibility
│   ├── web_handlers.rs               # Web API route handlers
│   ├── web_types.rs                  # Web application types
│   ├── github_api.rs                 # GitHub API integration
│   ├── system_profiler.rs            # System profiling engine
│   ├── deployment_profiles.rs        # Deployment profile management
│   ├── container_runtime.rs          # Container runtime abstraction
│   ├── docker_manager.rs             # Docker container management
│   ├── podman_manager.rs             # Podman container management
│   └── lxc_manager.rs                # LXC container management
├── collect_info.sh                   # Main system information collector
├── plugins/                          # System information collection plugins
│   ├── 10_os_info.sh                 # OS and distribution information
│   ├── 20_hardware_info.sh           # Hardware details (CPU, memory, PCIe, USB, GPU)
│   ├── 25_virtualization_info.sh     # VM/container platform detection
│   ├── 30_ip_info.sh                 # Network interface information
│   ├── 31_network_stats.sh           # Network statistics and routing
│   ├── 32_lldp_neighbors.sh          # LLDP/ARP/bridge discovery
│   ├── 40_packages_execs.sh          # Package and executable inventory
│   └── 50_uptime_info.sh             # System uptime and load
├── netbox-build/                     # NetBox source build configuration
│   ├── Dockerfile.netbox             # NetBox source build Dockerfile
│   ├── install-plugins.sh            # Plugin installation script
│   ├── docker-entrypoint.sh          # Container entrypoint
│   └── netbox-config/                # NetBox configuration files
├── monitoring/                       # Monitoring stack configuration
│   ├── prometheus/                   # Prometheus configuration
│   ├── grafana/                      # Grafana dashboards and datasources
│   └── logstash/                     # Logstash pipeline configuration
├── test/                             # Test suite
│   ├── integration/                  # Integration tests (BATS)
│   └── plugins/                      # Plugin-specific tests
├── .github/                          # GitHub Actions workflows
│   └── workflows/                    # CI/CD and release automation
├── docker-compose.yml               # Complete stack deployment
├── Dockerfile                       # Main application container
├── Cargo.toml                       # Rust project configuration
├── README.md                        # This documentation
├── TECHNICAL.md                     # Technical implementation details
├── CONFIGURATION.md                 # Configuration and tuning guide
├── IMPLEMENTATION_SUMMARY.md        # Implementation status and features
└── ANALYSIS_SUMMARY.md              # Project analysis and improvements
```

## Quick Start

### Prerequisites

- **Docker and Docker Compose** (recommended for full stack deployment)
- **Rust 1.75+** (for building from source)
- **Git** (for repository management)

### Full Stack Deployment

The quickest way to get started with the complete Automation Nation platform:

```bash
# Clone the repository
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# Copy environment template and customize
cp .env.template .env
# Edit .env with your preferred passwords and configuration

# Start the complete stack
docker-compose up -d

# Wait for services to initialize (1-2 minutes)
docker-compose logs -f automation-nation-web

# Access the services:
# - Main Web Interface: http://localhost:3000
# - NetBox: http://localhost:8080 (admin/admin_password)
# - Grafana: http://localhost:3001 (admin/admin_password)
# - Prometheus: http://localhost:9090
# - Kibana: http://localhost:5601
```

### Development Setup

For development and testing:

```bash
# Install Rust dependencies
cargo build --release

# Run the web server locally
cargo run --bin web_server -- serve --port 3000

# Run the CI test suite
cargo run --bin ci_runner -- run --profile dev

# Test system information collection
./collect_info.sh -o system_info.json
```

### Container-Only Deployment

For production deployments with custom configurations:

```bash
# Build custom container
docker build -t automation-nation:custom .

# Run with custom configuration
docker run -d \
  -p 3000:3000 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -e RUST_LOG=info \
  automation-nation:custom
```

## Service Architecture

### Core Services

| Service | Port | Purpose | Dependencies |
|---------|------|---------|--------------|
| **Web Application** | 3000 | Main automation interface, API, container management | PostgreSQL, Redis |
| **NetBox** | 8080 | Network infrastructure database and documentation | PostgreSQL, Redis |
| **PostgreSQL** | 5432 | Primary database for applications | None |
| **Redis** | 6379 | Caching and session storage | None |

### Monitoring Services

| Service | Port | Purpose | Dependencies |
|---------|------|---------|--------------|
| **Prometheus** | 9090 | Metrics collection and storage | None |
| **Grafana** | 3001 | Metrics visualization and dashboards | Prometheus |
| **Elasticsearch** | 9200 | Log storage and search | None |
| **Logstash** | 5044 | Log processing and ingestion | Elasticsearch |
| **Kibana** | 5601 | Log visualization and analysis | Elasticsearch |

## Usage Examples

### System Information Collection

```bash
# Basic system information collection
./collect_info.sh

# Save to file with enhanced security
ENABLE_HASHING=1 ./collect_info.sh -o system_info.json

# Collect with sudo privileges (for enhanced hardware info)
ENABLE_SUDO_SUPPORT=1 ./collect_info.sh -o privileged_info.json

# Fast collection without hashing
ENABLE_HASHING=0 ./collect_info.sh
```

### Web API Usage

```bash
# Get system profile
curl http://localhost:3000/api/system/profile

# List available container runtimes
curl http://localhost:3000/api/containers/runtimes

# Deploy a GitHub repository
curl -X POST http://localhost:3000/api/deploy \
  -H "Content-Type: application/json" \
  -d '{"repository": "owner/repo", "runtime": "docker"}'

# Get deployment status
curl http://localhost:3000/api/deployments/status
```

### Container Management

```bash
# Use the web interface for container management
# Navigate to http://localhost:3000

# Or use the CI runner for automated testing
cargo run --bin ci_runner -- run --profile ci

# Test specific container runtime
cargo run --bin ci_runner -- validate --runtime docker
```

## Configuration

### Environment Variables

The platform supports extensive configuration through environment variables:

#### Database Configuration
```bash
POSTGRES_PASSWORD=secure_password        # PostgreSQL password
REDIS_PASSWORD=secure_password           # Redis password
DATABASE_URL=sqlite:///app/data/app.db   # SQLite URL for development
```

#### NetBox Configuration
```bash
NETBOX_SECRET_KEY=your_secret_key                    # NetBox Django secret key
NETBOX_ADMIN_PASSWORD=admin_password                 # NetBox admin password

# Source build options
BUILD_NETBOX_FROM_SOURCE=false                       # Build from source vs. official image
NETBOX_VERSION=v4.1.0                               # NetBox version for source build
NETBOX_ENABLE_PLUGINS=false                         # Enable plugin installation
NETBOX_PLUGIN_LIST=netbox-topology-views,netbox-bgp # Comma-separated plugin list
```

#### Application Configuration
```bash
RUST_LOG=info                           # Logging level (debug, info, warn, error)
ENABLE_HASHING=1                        # Enable CRC32 hashing for data integrity
ENABLE_SUDO_SUPPORT=0                   # Enable sudo privilege escalation
HOST_SYSTEM_ROOT=/host                  # Host system mount point for containers

# Container runtime preferences
PREFERRED_RUNTIMES=podman,docker,lxc    # Comma-separated runtime priority list
```

#### Plugin Limits and Performance Tuning
```bash
# Network interface limits
MAX_INTERFACES=20                       # Maximum network interfaces to process
MAX_ADDRESSES_PER_INTERFACE=10          # Maximum addresses per interface

# Network discovery limits  
MAX_NEIGHBORS=20                        # Maximum LLDP/CDP neighbors
MAX_ARP_ENTRIES=50                      # Maximum ARP table entries
MAX_ROUTES=50                           # Maximum routing table entries

# Package and executable limits
MAX_PACKAGES=30                         # Maximum packages to collect
MAX_EXECUTABLES=50                      # Maximum executables to find
```

### NetBox Plugin Options

When building NetBox from source, you can install additional plugins:

#### Network Management Plugins
- `netbox-topology-views` - Network topology visualization
- `netbox-device-map` - Device location mapping
- `netbox-bgp` - BGP routing information management
- `netbox-dns` - DNS zone management

#### Monitoring Integration Plugins  
- `netbox-prometheus-sd` - Prometheus service discovery
- `netbox-grafana` - Grafana dashboard integration

#### Documentation Plugins
- `netbox-documents` - Document management
- `netbox-qrcode` - QR code generation for devices

#### Cloud Integration Plugins
- `netbox-cloud` - Cloud provider integration
- `netbox-kubernetes` - Kubernetes resource management

See `netbox-build/install-plugins.sh` for the complete list of supported plugins.

## Development

### Building from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation
cargo build --release

# Run tests
cargo test
./comprehensive_test_suite.sh
```

### Development Workflow

```bash
# Run development server with auto-reload
cargo run --bin web_server -- serve --port 3000 --verbose

# Run CI tests in development mode
cargo run --bin ci_runner -- run --profile dev --verbose

# Test system information collection
./collect_info.sh | jq .

# Run specific test suites
cargo test web_handlers
cargo test container_runtime
```

### Cross-Platform Testing

```bash
# Test on multiple architectures (requires Docker)
cargo run --bin ci_runner -- run \
  --architectures x86_64,arm64 \
  --operating-systems ubuntu,alpine \
  --parallel

# Test container runtime compatibility
cargo run --bin ci_runner -- validate --runtime docker
cargo run --bin ci_runner -- validate --runtime podman
```

## Release Management

The project includes automated release management with datetime-based versioning:

### Automated Releases

- **Schedule**: Daily check for changes at 2 AM UTC
- **Versioning**: DateTime.ISO format (YYYY-MM-DDTHH-MM-SS)
- **Artifacts**: Pre-compiled binaries for multiple architectures
- **Platforms**: x86_64/ARM64 with glibc/musl variants

### Manual Release

```bash
# Trigger release via GitHub Actions
gh workflow run release.yml --field force_release=true
```

### Using Release Binaries

```bash
# Use release-specific container with pre-compiled binaries
docker build -f Dockerfile.release -t automation-nation:2024-03-15T10-30-00 .

# Run with optimized performance
docker run -p 3000:3000 automation-nation:2024-03-15T10-30-00
```

## Monitoring and Observability

### Available Dashboards

- **Grafana** (port 3001): Application and infrastructure metrics
- **Kibana** (port 5601): Centralized log analysis and search
- **Prometheus** (port 9090): Raw metrics and alerting rules

### Key Metrics

- Container deployment success rates
- System profiling performance
- API response times
- Resource utilization
- Network discovery statistics

### Log Aggregation

All services forward logs to the ELK stack:
- Application logs (Rust services)
- Container runtime logs
- NetBox application logs
- System information collection logs

## Integration Examples

### Ansible Integration

```yaml
- name: Deploy with Automation Nation
  uri:
    url: "http://automation-nation:3000/api/deploy"
    method: POST
    body_format: json
    body:
      repository: "myorg/myapp"
      runtime: "docker"
      environment: "production"
```

### GitHub Actions Integration

```yaml
- name: Deploy to Automation Nation
  uses: actions/github-script@v6
  with:
    script: |
      await github.rest.repos.createDispatchEvent({
        owner: 'myorg',
        repo: 'automation-nation',
        event_type: 'deploy',
        client_payload: {
          repository: context.repo.full_name,
          ref: context.ref
        }
      });
```

### Monitoring Integration

```bash
# Export metrics to external Prometheus
curl http://localhost:3000/metrics

## Contributing

Contributions are welcome! Please see the following guidelines:

1. **Fork the repository** and create a feature branch
2. **Add comprehensive tests** for new functionality  
3. **Update documentation** for any API or configuration changes
4. **Ensure all tests pass** including the complete test suite
5. **Follow existing code style** and commenting conventions
6. **Submit a pull request** with detailed description of changes

### Development Setup

```bash
# Clone and set up development environment
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# Install dependencies and build
cargo build --release

# Run the full test suite
cargo test
./comprehensive_test_suite.sh

# Commit your changes
git add .
git commit -m "Brief description of changes"
git push origin feature-branch
```

## Support and Documentation

- **Technical Documentation**: See `TECHNICAL.md` for implementation details
- **Configuration Guide**: See `CONFIGURATION.md` for configuration options
- **Implementation Status**: See `IMPLEMENTATION_SUMMARY.md` for feature status
- **Project Analysis**: See `ANALYSIS_SUMMARY.md` for project improvements

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [Axum](https://github.com/tokio-rs/axum) for the web framework
- Monitoring powered by the [ELK Stack](https://www.elastic.co/what-is/elk-stack) and [Grafana](https://grafana.com/)
- Container orchestration supports [Docker](https://www.docker.com/), [Podman](https://podman.io/), and [LXC](https://linuxcontainers.org/)
- Network infrastructure management via [NetBox](https://netbox.readthedocs.io/)

---

**Automation Nation** - Empowering infrastructure automation with comprehensive system profiling, container orchestration, and enterprise-grade monitoring.
