# Automation_nation

A comprehensive plugin-based system information collector supporting the top 10 architectures as of Q4 2024.

## Overview

This repository provides a flexible system information collection framework that automatically discovers and executes plugins to gather system data across multiple architectures. The collector outputs comprehensive system information in JSON format.

## Features

- **Plugin-based Architecture**: Extensible design for easy addition of new data collectors
- **Multi-Architecture Support**: Supports 10 major CPU architectures  
- **JSON Output**: Structured, machine-readable output format
- **Automatic Plugin Discovery**: Dynamically finds and executes plugins
- **Comprehensive Testing**: Full test coverage with Bats framework
- **Network Discovery**: Advanced network interface, routing, and neighbor discovery
- **Container Integration**: Docker bridge and network namespace detection
- **Configurable Limits**: Environment variable configuration for performance tuning
- **Graceful Fallbacks**: Continues operation when optional tools are unavailable
- **Cross-Platform**: Works on Linux, macOS, and other Unix-like systems

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
├── collect_info.sh              # Main orchestrator script
├── plugins/                     # Data collection plugins
│   ├── 10_os_info.sh           #   OS and distribution information
│   ├── 20_hardware_info.sh     #   Hardware details (CPU, memory, disk)
│   ├── 30_ip_info.sh           #   Network interface information
│   ├── 31_network_stats.sh     #   Network statistics and routing
│   ├── 32_lldp_neighbors.sh    #   LLDP/ARP/bridge discovery
│   ├── 40_packages_execs.sh    #   Package and executable inventory
│   └── 50_uptime_info.sh       #   System uptime and load
├── test/                        # Test suite (Bats framework)
│   ├── integration/             #   Integration tests
│   └── plugins/                 #   Individual plugin tests
├── README.md                    # This documentation
├── TECHNICAL.md                 # Technical implementation guide
├── CONFIGURATION.md             # Configuration and tuning guide
├── ANALYSIS_SUMMARY.md          # Project analysis documentation
└── proj-SHA512_of_folder_creation_time/  # Placeholder directories (future use)
```

## Quick Start

### Basic Usage

```bash
# Collect system information and output to console
./collect_info.sh

# Save output to a file
./collect_info.sh -o system_info.json

# Display help
./collect_info.sh -h
```

### Example Output

```json
{
  "detected_architecture": "x86_64",
  "os_name": "Ubuntu",
  "os_version": "24.04.2 LTS (Noble Numbat)",
  "distribution": "ubuntu",
  "distribution_version": "24.04",
  "kernel_version": "6.11.0-1018-azure",
  "architecture": "x86_64",
  "cpu_model": "AMD EPYC 7763 64-Core Processor",
  "cpu_cores": "1",
  "cpu_threads": "2",
  "cpu_frequency": "3240.421 MHz",
  "memory_total": "7944 MB",
  "memory_available": "6523 MB",
  "disk_info": [
    {
      "filesystem": "/dev/root",
      "size": "72G",
      "used": "49G",
      "available": "24G",
      "usage": "68%",
      "mountpoint": "/"
    }
  ],
  "network_interfaces": [
    {
      "interface": "eth0",
      "ipv4_addresses": ["10.1.0.215/20"],
      "ipv6_addresses": ["fe80::6245:bdff:fe06:427f/64"],
      "mac_address": "60:45:bd:06:42:7f",
      "mtu": "1500",
      "state": "up"
    }
  ],
  "interface_statistics": [
    {
      "interface": "eth0",
      "rx_bytes": "77457645",
      "rx_packets": "58679",
      "tx_bytes": "4771183",
      "tx_packets": "11917"
    }
  ],
  "ipv4_routes": [
    {
      "destination": "default",
      "gateway": "10.1.0.1",
      "interface": "eth0",
      "metric": "100"
    }
  ],
  "arp_table": [
    {
      "ip_address": "10.1.0.1",
      "mac_address": "12:34:56:78:9a:bc",
      "interface": "eth0",
      "state": "REACHABLE"
    }
  ],
  "uptime_seconds": "3647",
  "uptime_formatted": "1h 0m 47s",
  "boot_time": "1754453845",
  "load_average": "0.15 0.18 0.12"
}
```

## Plugin Architecture

### Plugin Directory Structure

```
plugins/
├── 10_os_info.sh      # OS and distribution information
├── 20_hardware_info.sh # CPU, memory, and disk information
├── 30_ip_info.sh      # Network interface details (IPv4/IPv6)
├── 31_network_stats.sh # Network statistics, routing, multicast
├── 32_lldp_neighbors.sh # LLDP neighbors, ARP table, bridge info
├── 40_packages_execs.sh # Installed packages and executables
└── 50_uptime_info.sh  # System uptime and load information
```

### Plugin Requirements

1. **Executable**: Plugins must be executable shell scripts
2. **Architecture Parameter**: First argument is the detected architecture
3. **JSON Output**: Must output valid JSON to stdout
4. **Error Handling**: Should handle errors gracefully
5. **Naming Convention**: Numeric prefix for execution order

### Plugin Interface

Each plugin receives the detected architecture as the first argument:

```bash
./plugin_name.sh <architecture>
```

Example plugin:

```bash
#!/bin/bash
ARCH="$1"

# Validate input
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Collect data based on architecture
case "$ARCH" in
    x86_64|arm64|i386)
        # Architecture-specific logic
        ;;
    *)
        # Default handling
        ;;
esac

# Output JSON
cat << EOF
{
  "plugin_name": "example",
  "architecture": "$ARCH",
  "data": "value"
}
EOF
```

## Built-in Plugins

### 10_os_info.sh

Collects operating system and distribution information:

- **os_name**: Operating system name
- **os_version**: OS version string
- **distribution**: Distribution identifier
- **distribution_version**: Distribution version
- **kernel_version**: Kernel version
- **architecture**: Target architecture

**Architecture-specific features:**
- ARM variants: Detects Raspberry Pi models
- PowerPC: Identifies POWER systems and LPAR configurations
- Specialized architectures: Adds architecture suffixes to distribution names

### 20_hardware_info.sh

Collects hardware information:

- **cpu_model**: CPU model name/identifier
- **cpu_cores**: Number of physical CPU cores
- **cpu_threads**: Number of logical CPU threads
- **cpu_frequency**: CPU frequency in MHz
- **memory_total**: Total system memory in MB
- **memory_available**: Available memory in MB
- **disk_info**: Array of disk/filesystem information

**Architecture-specific features:**
- x86/x64: Uses `/proc/cpuinfo` model name and core detection
- ARM: Detects ARM-specific processor information and Raspberry Pi models
- PowerPC: Handles POWER-specific CPU detection
- RISC-V/MIPS/SPARC: Architecture-specific CPU model parsing
- Cross-platform memory and disk detection

### 30_ip_info.sh

Collects detailed network interface information:

- **network_interfaces**: Array of network interface details including:
  - **interface**: Interface name (e.g., "eth0", "wlan0")
  - **ipv4_addresses**: Array of IPv4 addresses with CIDR notation
  - **ipv6_addresses**: Array of IPv6 addresses with prefix length
  - **mac_address**: Hardware MAC address
  - **mtu**: Maximum Transmission Unit size
  - **state**: Interface state (up/down/unknown)
- **architecture**: Target architecture

**Architecture-specific features:**
- Cross-platform interface detection using `ip`, `ifconfig`, or `/proc/net/dev`
- ARM-specific handling for embedded systems and Raspberry Pi
- PowerPC and IBM Z network interface specifics
- Graceful fallback for systems without modern network tools

### 31_network_stats.sh

Collects comprehensive network statistics and routing information:

- **interface_statistics**: Per-interface traffic statistics including:
  - **rx_bytes/rx_packets/rx_errors/rx_dropped**: Receive statistics
  - **tx_bytes/tx_packets/tx_errors/tx_dropped**: Transmit statistics
- **ipv4_routes**: IPv4 routing table entries with destination, gateway, interface, metric
- **ipv6_routes**: IPv6 routing table entries
- **multicast_groups**: Active multicast group memberships
- **listening_ports**: Network services and listening ports
- **architecture**: Target architecture

**Architecture-specific features:**
- Multi-source routing information (`ip route`, `route`, `/proc/net/route`)
- Cross-platform network statistics from `/proc/net/dev`
- Multicast group detection for IPv4 and IPv6
- Network service discovery using `ss` or `netstat`

### 32_lldp_neighbors.sh

Collects network discovery and bridge information:

- **lldp_neighbors**: LLDP (Link Layer Discovery Protocol) neighbor devices
- **arp_table**: ARP table entries with IP-to-MAC mappings
- **bridge_info**: Network bridge configurations including:
  - Bridge names and IDs
  - STP (Spanning Tree Protocol) status
  - Connected interfaces
  - Docker bridge detection
- **network_namespaces**: Available network namespaces
- **architecture**: Target architecture

**Architecture-specific features:**
- LLDP/CDP neighbor discovery using `lldpctl` or `lldptool`
- ARP table parsing from `ip neigh`, `arp`, or `/proc/net/arp`
- Bridge detection using `brctl`, `bridge`, or Docker network inspection
- Network namespace enumeration for containerized environments

### 40_packages_execs.sh

Collects installed packages and system executables information:

- **installed_packages**: Array of installed software packages including:
  - **name**: Package name
  - **version**: Package version string
  - **package_manager**: Package manager used (dpkg, rpm, brew, etc.)
  - **status**: Installation status
  - **config_files**: Array of potential configuration file locations
- **system_executables**: Array of system executables including:
  - **name**: Executable name
  - **path**: Full path to executable
  - **version**: Version information (when available)
  - **config_files**: Array of potential configuration file locations
- **architecture**: Target architecture

**Package manager support:**
- **Linux**: dpkg (Debian/Ubuntu), rpm (Red Hat/CentOS/Fedora), pacman (Arch), apk (Alpine)
- **macOS**: Homebrew (brew)
- **FreeBSD**: pkg

**Architecture-specific features:**
- Cross-platform package manager detection
- Version extraction for common executables (bash, python3, git, vim)
- Configuration file location mapping based on package conventions
- Configurable limits via environment variables

### 50_uptime_info.sh

Collects system uptime and load information:

- **uptime_seconds**: System uptime in seconds since boot
- **uptime_formatted**: Human-readable uptime format (e.g., "1h 30m 45s")
- **boot_time**: System boot time as Unix timestamp
- **load_average**: Current system load averages (1, 5, 15 minutes)
- **architecture**: Target architecture

**Architecture-specific features:**
- Cross-platform uptime detection using /proc/uptime or uptime command
- Boot time calculation from /proc/stat or system utilities
- Load average parsing from /proc/loadavg
- Graceful fallback for systems without /proc filesystem
- Consistent JSON structure across all supported architectures

## Installation

### Prerequisites

- Bash shell (version 4.0+)
- Standard Unix utilities (`uname`, `grep`, `awk`, etc.)
- Python 3 (for JSON validation in examples and enhanced validation)
- Bats testing framework (for running tests)

#### Optional Dependencies

These tools enhance functionality but have fallbacks if unavailable:

- `bc` - For precise CPU frequency and memory calculations (falls back to basic arithmetic)
- `ip` (iproute2) - For modern network interface discovery (falls back to `ifconfig` or `/proc`)
- `ss` - For network port discovery (falls back to `netstat`)
- `lldpctl` - For LLDP neighbor discovery (network discovery still works without it)
- `docker` - For Docker bridge detection (other bridge detection methods used)

The system will warn about missing optional dependencies but continue to function with fallback methods.

### Setup

1. Clone the repository:
```bash
git clone https://github.com/nullroute-commits/Automation_nation.git
cd Automation_nation
```

2. Make scripts executable:
```bash
chmod +x collect_info.sh plugins/*.sh
```

3. Run the collector:
```bash
./collect_info.sh
```

## Testing

The project includes comprehensive test coverage using the Bats testing framework for all plugins and the main collection script.

### Test Structure

The testing framework covers all major components with dedicated test suites:

```
test/
├── integration/
│   └── collect_info_test.bats     # Main script integration tests
└── plugins/
    ├── 10_os_info_test.bats       # OS plugin tests
    ├── 20_hardware_info_test.bats # Hardware plugin tests
    ├── 30_ip_info_test.bats       # Network interface plugin tests
    ├── 31_network_stats_test.bats # Network statistics plugin tests
    ├── 32_lldp_neighbors_test.bats # LLDP/ARP plugin tests
    └── 50_uptime_info_test.bats   # Uptime plugin tests
```

### Installing and Running the Bats Testing Framework

#### Installation Options

**Ubuntu/Debian:**
```bash
# Option 1: Package manager (recommended)
sudo apt-get update && sudo apt-get install bats

# Option 2: Install from source
git clone https://github.com/bats-core/bats-core.git
cd bats-core
sudo ./install.sh /usr/local
```

**macOS:**
```bash
# Option 1: Homebrew (recommended)
brew install bats-core

# Option 2: MacPorts
sudo port install bats-core
```

**CentOS/RHEL/Fedora:**
```bash
# Fedora
sudo dnf install bats

# CentOS/RHEL (requires EPEL)
sudo yum install epel-release
sudo yum install bats
```

**Manual Installation:**
```bash
# Clone and install manually
git clone https://github.com/bats-core/bats-core.git
cd bats-core
sudo ./install.sh /usr/local
```

### Running Tests

Execute individual test suites:
```bash
# Test main collector script
bats test/integration/collect_info_test.bats

# Test individual plugins
bats test/plugins/10_os_info_test.bats
bats test/plugins/20_hardware_info_test.bats
bats test/plugins/30_ip_info_test.bats
bats test/plugins/31_network_stats_test.bats
bats test/plugins/32_lldp_neighbors_test.bats
bats test/plugins/50_uptime_info_test.bats
```

Run all tests at once:
```bash
# Run all tests in sequence
bats test/integration/collect_info_test.bats \
     test/plugins/*.bats

# Or use find to run all .bats files
find test/ -name "*.bats" -exec bats {} \;
```

### Sample Complete JSON Output

When all plugins run successfully, the system produces comprehensive JSON output including OS information, hardware details, network configuration, and uptime statistics:

```json
{
  "detected_architecture": "x86_64",
  "os_name": "Ubuntu",
  "os_version": "24.04.2 LTS (Noble Numbat)",
  "distribution": "ubuntu",
  "distribution_version": "24.04",
  "kernel_version": "6.11.0-1018-azure",
  "architecture": "x86_64",
  "cpu_model": "AMD EPYC 7763 64-Core Processor",
  "cpu_cores": "1",
  "cpu_threads": "2",
  "cpu_frequency": "3244.330 MHz",
  "memory_total": "7944 MB",
  "memory_available": "6591 MB",
  "disk_info": [
    {
      "filesystem": "/dev/root",
      "size": "72G",
      "used": "48G",
      "available": "24G",
      "usage": "67%",
      "mountpoint": "/"
    },
    {
      "filesystem": "/dev/sda16",
      "size": "881M",
      "used": "60M",
      "available": "760M",
      "usage": "8%",
      "mountpoint": "/boot"
    },
    {
      "filesystem": "/dev/sda15",
      "size": "105M",
      "used": "6.2M",
      "available": "99M",
      "usage": "6%",
      "mountpoint": "/boot/efi"
    }
  ],
  "network_interfaces": [
    {
      "interface": "eth0",
      "ipv4_addresses": ["10.1.0.215/20"],
      "ipv6_addresses": ["fe80::6245:bdff:fe06:427f/64"],
      "mac_address": "60:45:bd:06:42:7f",
      "mtu": "1500",
      "state": "up"
    },
    {
      "interface": "docker0",
      "ipv4_addresses": ["172.17.0.1/16"],
      "ipv6_addresses": [],
      "mac_address": "02:42:a1:b2:c3:d4",
      "mtu": "1500",
      "state": "down"
    }
  ],
  "interface_statistics": [
    {
      "interface": "eth0",
      "rx_bytes": "77457645",
      "rx_packets": "58679",
      "rx_errors": "0",
      "rx_dropped": "0",
      "tx_bytes": "4771183",
      "tx_packets": "11917",
      "tx_errors": "0",
      "tx_dropped": "0"
    }
  ],
  "ipv4_routes": [
    {
      "destination": "default",
      "gateway": "10.1.0.1",
      "interface": "eth0",
      "metric": "100"
    },
    {
      "destination": "10.1.0.0/20",
      "gateway": "direct",
      "interface": "eth0",
      "metric": "100"
    }
  ],
  "ipv6_routes": [
    {
      "destination": "fe80::/64",
      "gateway": "direct",
      "interface": "eth0",
      "metric": "256"
    }
  ],
  "multicast_groups": [
    {
      "interface": "eth0",
      "group": "224.0.0.1",
      "version": "ipv4"
    }
  ],
  "listening_ports": [
    {
      "protocol": "tcp",
      "local_address": "0.0.0.0:22",
      "state": "LISTEN"
    }
  ],
  "lldp_neighbors": [],
  "arp_table": [
    {
      "ip_address": "10.1.0.1",
      "mac_address": "12:34:56:78:9a:bc",
      "interface": "eth0",
      "state": "REACHABLE"
    }
  ],
  "bridge_info": [
    {
      "bridge_name": "docker0",
      "bridge_id": "8000.0242a1b2c3d4",
      "stp_enabled": "no",
      "interfaces": ""
    }
  ],
  "network_namespaces": [],
  "uptime_seconds": "112",
  "uptime_formatted": "1m 52s",
  "boot_time": "1754453845",
  "load_average": "0.36 0.20 0.08"
}
```

### Test Coverage

The comprehensive test suite validates:

- **Architecture Detection**: Tests for all 10 supported architectures (x86_64, arm64, i386, ppc64le, s390x, riscv64, mips64, aarch32, sparc64, loongarch64)
- **Plugin Discovery**: Automatic plugin detection and execution
- **JSON Validation**: Output format validation and structure consistency
- **Main Script Functionality**: 
  - Architecture detection and parameter passing
  - Plugin merging and output generation
  - Command-line options (-o for output file, -h for help)
- **OS Information Plugin**: 
  - Operating system and distribution detection
  - Kernel version extraction
  - Architecture-specific handling
- **Hardware Information Plugin**: 
  - CPU model, cores, and frequency detection
  - Memory and disk information gathering
  - Cross-architecture hardware detection
- **Network Interface Plugin**: 
  - Network interface discovery and IPv4/IPv6 address detection
  - MAC address and MTU information
  - Cross-platform interface detection
- **Network Statistics Plugin**: 
  - Interface traffic statistics and routing table parsing
  - Multicast group detection and listening port discovery
  - IPv4/IPv6 route management
- **LLDP/ARP Plugin**: 
  - Network neighbor discovery and ARP table parsing
  - Bridge information and network namespace detection
  - Docker network integration
- **Uptime Information Plugin**: 
  - System uptime calculation and formatting
  - Boot time detection
  - Load average monitoring
- **Error Handling**: Missing directories, invalid plugins, malformed JSON
- **Cross-Architecture**: Consistent behavior across different architectures
- **Edge Cases**: Systems without specific tools or information sources

## Creating Custom Plugins

### Plugin Template

```bash
#!/bin/bash
# Custom plugin template
# Description: Your plugin description

set -e

ARCH="$1"

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Your data collection logic here
collect_data() {
    local data=""
    
    # Architecture-specific collection
    case "$ARCH" in
        x86_64|amd64)
            # x86_64 specific logic
            data="x86_64_data"
            ;;
        arm64|aarch64)
            # ARM64 specific logic
            data="arm64_data"
            ;;
        *)
            # Default handling
            data="generic_data"
            ;;
    esac
    
    echo "$data"
}

# Main execution
data=$(collect_data)

# Output JSON
cat << EOF
{
  "plugin_name": "custom_plugin",
  "architecture": "$ARCH",
  "custom_data": "$data"
}
EOF
```

### Plugin Best Practices

1. **Error Handling**: Always validate input parameters
2. **Architecture Awareness**: Handle architecture-specific data sources
3. **Fallback Values**: Provide sensible defaults when data is unavailable
4. **Performance**: Minimize resource usage and execution time
5. **Documentation**: Comment complex logic and architecture-specific code
6. **Testing**: Create corresponding test files in the appropriate test directory

## Configuration

### Environment Variables

The plugins support configuration through environment variables to control resource limits and behavior:

#### Network Interface Plugin (30_ip_info.sh)
- `MAX_INTERFACES=20` - Maximum number of network interfaces to process
- `MAX_ADDRESSES_PER_INTERFACE=10` - Maximum IPv4/IPv6 addresses per interface

#### Network Statistics Plugin (31_network_stats.sh)  
- `MAX_INTERFACES=20` - Maximum number of interfaces for statistics
- `MAX_ROUTES=50` - Maximum number of routing table entries (IPv4/IPv6)
- `MAX_MCAST_GROUPS=30` - Maximum multicast group entries
- `MAX_LISTENING_PORTS=50` - Maximum listening ports to report

#### LLDP/ARP Plugin (32_lldp_neighbors.sh)
- `MAX_NEIGHBORS=20` - Maximum LLDP/CDP neighbors to discover  
- `MAX_ARP_ENTRIES=50` - Maximum ARP table entries
- `MAX_BRIDGES=20` - Maximum bridge configurations
- `MAX_NETNS=20` - Maximum network namespaces
- `MAX_DOCKER_NETWORKS=10` - Maximum Docker bridge networks

#### Package and Executable Plugin (40_packages_execs.sh)
- `MAX_PACKAGES=30` - Maximum number of packages to collect
- `MAX_EXECUTABLES=20` - Maximum number of executables to collect

### Usage Examples

```bash
# Limit network discovery for performance
MAX_INTERFACES=5 MAX_ROUTES=20 ./collect_info.sh

# Comprehensive discovery for detailed analysis
MAX_INTERFACES=50 MAX_ROUTES=100 MAX_ARP_ENTRIES=200 ./collect_info.sh

# Container-focused configuration
MAX_DOCKER_NETWORKS=20 MAX_NETNS=50 ./collect_info.sh -o container-info.json

# Software inventory focus
MAX_PACKAGES=100 MAX_EXECUTABLES=50 ./collect_info.sh -o software-inventory.json
```

## Advanced Usage

### Custom Plugin Directory

Modify the `PLUGIN_DIR` variable in `collect_info.sh` to use a custom plugin directory:

```bash
# Edit collect_info.sh
PLUGIN_DIR="/path/to/custom/plugins"
```

### Architecture Override

For testing purposes, you can modify the `detect_arch()` function to return a specific architecture:

```bash
detect_arch() {
    echo "arm64"  # Force specific architecture
}
```

### Integration with Monitoring Systems

The JSON output can be easily integrated with monitoring and configuration management systems:

```bash
# Send to monitoring system
./collect_info.sh | curl -X POST -H "Content-Type: application/json" \
    -d @- https://monitoring.example.com/api/systems

# Use with configuration management
./collect_info.sh -o /etc/system-facts.json
```

## Troubleshooting

### Common Issues

1. **No plugins found**: Ensure plugins directory exists and contains executable files
2. **Invalid JSON**: Check plugin output format and error messages
3. **Architecture not detected**: Verify `uname -m` output and architecture mapping
4. **Permission denied**: Ensure scripts have execute permissions

### Debug Mode

Add debug output to plugins for troubleshooting:

```bash
# Add to plugin for debugging
echo "Debug: Architecture=$ARCH" >&2
echo "Debug: Data source=/proc/cpuinfo" >&2
```

### Validation

Validate JSON output:
```bash
./collect_info.sh | python3 -m json.tool
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is available under the terms specified in the LICENSE file.

## Architecture Support Roadmap

- **Current**: Top 10 architectures as of Q4 2024
- **Planned**: Additional emerging architectures as they gain market adoption
- **Considerations**: WebAssembly, custom silicon, and edge computing architectures
