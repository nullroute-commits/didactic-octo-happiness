# Installation Guide

Complete installation instructions for all deployment methods of the Automation Nation platform.

## 📋 Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores (x86_64 or ARM64)
- **Memory**: 4 GB RAM
- **Storage**: 20 GB available space
- **Network**: Internet access for downloads

#### Recommended Requirements
- **CPU**: 4+ cores
- **Memory**: 8+ GB RAM
- **Storage**: 50+ GB SSD storage
- **Network**: High-speed internet connection

#### Supported Platforms
- **Linux**: Ubuntu 20.04+, CentOS 8+, Debian 11+, Alpine 3.15+
- **Container Platforms**: Docker 20.10+, Podman 3.0+, LXC 4.0+
- **Architectures**: x86_64, ARM64, and 8 additional architectures

### Required Software

#### For Docker Deployment (Recommended)
```bash
# Docker and Docker Compose
sudo apt-get update
sudo apt-get install docker.io docker-compose

# Or use Docker Desktop on macOS/Windows
```

#### For Source Deployment
```bash
# Rust toolchain (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies
sudo apt-get install build-essential pkg-config libssl-dev libsqlite3-dev

# Optional: BATS for testing
sudo apt-get install bats
```

## 🚀 Quick Installation Methods

### Method 1: Docker Compose (Recommended)

The fastest way to get the complete platform running:

```bash
# 1. Clone the repository
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# 2. Copy and customize environment file
cp .env.template .env
editor .env  # Update passwords and settings

# 3. Start the complete stack
docker-compose up -d

# 4. Wait for services to initialize (1-2 minutes)
docker-compose logs -f automation-nation-web

# 5. Access the platform
echo "Web Interface: http://localhost:3000"
echo "NetBox: http://localhost:8080 (admin/admin_password)"
echo "Grafana: http://localhost:3001 (admin/admin_password)"
```

### Method 2: Container-Only Deployment

For minimal deployment without full monitoring stack:

```bash
# 1. Clone and build
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# 2. Build the container
docker build -t automation-nation:latest .

# 3. Run with basic configuration
docker run -d \
  --name automation-nation \
  -p 3000:3000 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -e RUST_LOG=info \
  automation-nation:latest

# 4. Access the web interface
curl http://localhost:3000/health
```

### Method 3: Source Installation

For development or custom deployments:

```bash
# 1. Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Clone and build
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation
cargo build --release

# 3. Run the web server
./target/release/web_server serve --port 3000

# 4. In another terminal, test system profiling
./collect_info.sh | jq .
```

## 🔧 Detailed Installation Steps

### Step 1: Environment Preparation

#### Ubuntu/Debian
```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install base dependencies
sudo apt-get install -y \
  curl \
  git \
  jq \
  bash \
  procps \
  net-tools \
  iproute2

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
newgrp docker
```

#### CentOS/RHEL/Fedora
```bash
# Update system
sudo dnf update -y

# Install base dependencies
sudo dnf install -y \
  curl \
  git \
  jq \
  bash \
  procps-ng \
  net-tools \
  iproute

# Install Docker
sudo dnf install -y docker docker-compose
sudo systemctl enable --now docker
sudo usermod -aG docker $USER
```

#### macOS
```bash
# Install Homebrew if not present
# Download the Homebrew installation script
curl -fsSLo install.sh https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh

# (Optional) Inspect or verify the script before running:
# less install.sh
# You can also check the SHA256 at https://docs.brew.sh/Installation#untar-anywhere

# Run the installation script
/bin/bash install.sh

# For more details, see: https://brew.sh/
# Install dependencies
brew install git jq

# Install Docker Desktop
# Download from https://www.docker.com/products/docker-desktop
```

### Step 2: Repository Setup

```bash
# Clone the repository
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation

# Verify repository integrity
ls -la
chmod +x collect_info.sh plugins/*.sh *.sh

# Test basic functionality
./collect_info.sh | head -20
```

### Step 3: Configuration

#### Environment Variables
```bash
# Copy template and customize
cp .env.template .env

# Essential settings to customize
editor .env
```

**Required Changes in .env:**
```bash
# Database passwords (change these!)
POSTGRES_PASSWORD=your_secure_postgres_password
REDIS_PASSWORD=your_secure_redis_password

# NetBox configuration
NETBOX_SECRET_KEY=your_secret_key_at_least_50_characters_long
NETBOX_ADMIN_PASSWORD=your_netbox_admin_password

# Grafana configuration
GRAFANA_ADMIN_PASSWORD=your_grafana_admin_password

# Optional: GitHub token for enhanced API limits
GITHUB_TOKEN=your_github_token_here
```

#### Advanced Configuration
For customized deployments, see [Configuration Reference](Configuration).

### Step 4: Service Deployment

#### Full Stack Deployment
```bash
# Start all services
docker-compose up -d

# Monitor startup progress
docker-compose logs -f

# Check service health
docker-compose ps
```

#### Individual Service Management
```bash
# Start specific services
docker-compose up -d postgres redis
docker-compose up -d automation-nation-web

# View service logs
docker-compose logs automation-nation-web
docker-compose logs netbox

# Restart a service
docker-compose restart automation-nation-web
```

### Step 5: Verification

#### Health Checks
```bash
# Web application health
curl http://localhost:3000/health

# API functionality
curl http://localhost:3000/api/system/profile

# NetBox availability
curl http://localhost:8080/api/

# Monitoring stack
curl http://localhost:9090/api/v1/query?query=up
curl http://localhost:3001/api/health
```

#### System Profiling Test
```bash
# Test system information collection
./collect_info.sh | jq .detected_architecture

# Test with file output
./collect_info.sh -o test_output.json
cat test_output.json | jq .collection_metadata
```

#### Container Management Test
```bash
# Test API endpoints
curl -X GET http://localhost:3000/api/containers/runtimes

# Test deployment capability (if GitHub token configured)
curl -X POST http://localhost:3000/api/deploy \
  -H "Content-Type: application/json" \
  -d '{"repository": "hello-world", "runtime": "docker"}'
```

## 🔐 Security Configuration

### Basic Security Setup

#### User Permissions
```bash
# Create dedicated user for the application
sudo useradd -r -s /bin/bash -d /opt/automation_nation automation

# Set proper file permissions
sudo chown -R automation:automation /opt/automation_nation
sudo chmod 755 /opt/automation_nation
sudo chmod 644 /opt/automation_nation/plugins/*
```

#### Network Security
```bash
# Configure firewall (UFW example)
sudo ufw allow 22/tcp       # SSH
sudo ufw allow 3000/tcp     # Web interface
sudo ufw allow 8080/tcp     # NetBox (if external access needed)
sudo ufw --force enable
```

#### TLS/SSL Setup
For production deployments, configure reverse proxy with TLS:

```nginx
# /etc/nginx/sites-available/automation-nation
server {
    listen 443 ssl;
    server_name automation-nation.yourdomain.com;
    
    ssl_certificate /path/to/certificate.crt;
    ssl_certificate_key /path/to/private.key;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 📊 Monitoring Setup

### Accessing Monitoring Dashboards

#### Grafana Setup
1. Access Grafana at `http://localhost:3001`
2. Login with admin/admin_password
3. Import pre-configured dashboards from `monitoring/grafana/dashboards/`
4. Configure data sources pointing to Prometheus

#### Kibana Setup
1. Access Kibana at `http://localhost:5601`
2. Configure index patterns for application logs
3. Import dashboards from `monitoring/kibana/` (if available)

### Custom Monitoring Configuration
See [Monitoring Setup](Monitoring-Setup) for detailed configuration.

## 🐳 Container Runtime Configuration

### Docker Configuration
```bash
# Optimize Docker daemon
sudo mkdir -p /etc/docker
cat <<EOF | sudo tee /etc/docker/daemon.json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2"
}
EOF

sudo systemctl restart docker
```

### Podman Configuration
```bash
# Enable rootless Podman
sudo apt-get install -y podman
podman system migrate

# Test rootless operation
podman run --rm hello-world
```

### LXC Configuration
```bash
# Install and configure LXC
sudo apt-get install -y lxc lxc-templates
sudo systemctl enable --now lxc

# Configure unprivileged containers
echo "$USER veth lxcbr0 10" | sudo tee -a /etc/lxc/lxc-usernet
```

## 🔧 Troubleshooting Installation

### Common Issues

#### Permission Denied Errors
```bash
# Fix Docker socket permissions
sudo usermod -aG docker $USER
newgrp docker

# Fix script permissions
chmod +x collect_info.sh plugins/*.sh
```

#### Port Conflicts
```bash
# Check port usage
sudo netstat -tlnp | grep :3000

# Change ports in docker-compose.yml if needed
```

#### Memory Issues
```bash
# Increase Docker memory limits
# Edit Docker Desktop settings or /etc/docker/daemon.json

# Monitor memory usage
docker stats
```

#### Service Startup Failures
```bash
# Check service logs
docker-compose logs [service_name]

# Restart services
docker-compose restart

# Reset and rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### Log Analysis
```bash
# Application logs
docker-compose logs automation-nation-web

# Database logs
docker-compose logs postgres

# All service logs
docker-compose logs --follow
```

## 📈 Performance Optimization

### Resource Allocation
```yaml
# docker-compose.override.yml
version: '3.8'
services:
  automation-nation-web:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'
```

### Database Optimization
```bash
# PostgreSQL tuning
echo "shared_preload_libraries = 'pg_stat_statements'" >> postgresql.conf
echo "max_connections = 200" >> postgresql.conf
```

## 🚀 Next Steps

After successful installation:

1. **[Quick Start Guide](Quick-Start)** - Learn basic platform usage
2. **[Configuration](Configuration)** - Customize your deployment
3. **[Web Interface Guide](Web-Interface-Guide)** - Explore the web interface
4. **[Container Management](Container-Management)** - Deploy your first container
5. **[Monitoring Setup](Monitoring-Setup)** - Configure comprehensive monitoring

## 📞 Support

If you encounter issues during installation:

1. Check the [Troubleshooting](Troubleshooting) guide
2. Search existing [GitHub Issues](https://github.com/nullroute-commits/Automation_nation/issues)
3. Create a new issue with:
   - Operating system and version
   - Installation method used
   - Complete error messages
   - Configuration file contents (redacted)

---

**Navigation**: [Home](Home) | [Quick Start](Quick-Start) | [Configuration](Configuration)