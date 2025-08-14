# Automation_nation

A comprehensive plugin-based system information collector supporting the top 10 architectures as of Q4 2024.

## Overview

This repository provides a flexible system information collection framework that automatically discovers and executes plugins to gather system data across multiple architectures. The collector outputs comprehensive system information in JSON format.

## Features

- **Plugin-based Architecture**: Extensible design for easy addition of new data collectors
- **Multi-Architecture Support**: Supports 10 major CPU architectures  
- **JSON Output**: Structured, machine-readable output format
- **Automatic Plugin Discovery**: Dynamically finds and executes plugins
- **Data Integrity**: Optional CRC32 hashing of plugin content and function outputs
- **Privilege Support**: Optional sudo/privileged user support with graceful fallbacks
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
└── ANALYSIS_SUMMARY.md          # Project analysis documentation
```

## Quick Start

### Basic Usage

```bash
# Collect system information and output to console
./collect_info.sh

# Save output to a file
./collect_info.sh -o system_info.json

# Enable CRC32 hashing for data integrity verification
ENABLE_HASHING=1 ./collect_info.sh -o system_info_with_hashes.json

# Enable sudo support for privileged operations (with fallback)
ENABLE_SUDO_SUPPORT=1 ./collect_info.sh -o system_info_privileged.json

# Enable both hashing and sudo support
ENABLE_HASHING=1 ENABLE_SUDO_SUPPORT=1 ./collect_info.sh -o comprehensive_info.json

# Display help
./collect_info.sh -h
```

### Example Output

The system produces comprehensive JSON output with nested structure including collection metadata and per-plugin timestamps:

```json
{
  "detected_architecture": "x86_64",
  "collection_metadata": {
    "timestamp": "2025-01-15T14:30:45Z",
    "plugin_count": 7,
    "hashing_enabled": 1,
    "sudo_support_enabled": 1,
    "sudo_available": 1
  },
  "get_os_info": {
    "data": {
      "os_name": "Ubuntu",
      "os_version": "24.04.2 LTS (Noble Numbat)",
      "distribution": "ubuntu",
      "distribution_version": "24.04",
      "kernel_version": "6.11.0-1018-azure",
      "architecture": "x86_64"
    },
    "collection_timestamp": "2025-01-15T14:30:45Z",
    "completion_timestamp": "2025-01-15T14:30:45Z",
    "plugin_file_hash": "2054604427",
    "function_data_hash": "2915874064"
  },
  "get_hardware_info": {
    "data": {
      "cpu_model": "AMD EPYC 7763 64-Core Processor",
      "cpu_cores": "2",
      "cpu_threads": "4",
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
      ]
    },
    "collection_timestamp": "2025-01-15T14:30:45Z",
    "completion_timestamp": "2025-01-15T14:30:45Z"
  },
  "get_ip_info": {
    "data": {
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
      "architecture": "x86_64"
    },
    "collection_timestamp": "2025-01-15T14:30:45Z",
    "completion_timestamp": "2025-01-15T14:30:45Z"
  },
  "get_network_stats": {
    "data": {
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
      "listening_ports": [
        {
          "protocol": "tcp",
          "local_address": "0.0.0.0:22",
          "state": "LISTEN"
        }
      ]
    },
    "collection_timestamp": "2025-01-15T14:30:45Z",
    "completion_timestamp": "2025-01-15T14:30:45Z"
  },
  "get_uptime_info": {
    "data": {
      "uptime_seconds": "3647",
      "uptime_formatted": "1h 0m 47s",
      "boot_time": "1754453845",
      "load_average": "0.15 0.18 0.12",
      "architecture": "x86_64"
    },
    "collection_timestamp": "2025-01-15T14:30:45Z",
    "completion_timestamp": "2025-01-15T14:30:45Z"
  }
}
```

## Data Integrity and Security

### CRC32 Hashing

The system supports optional CRC32 hashing for data integrity verification:

- **Plugin File Hashes**: Hash of each plugin script content
- **Function Data Hashes**: Hash of JSON output from each plugin function
- **Backwards Compatible**: Uses standard `cksum` command available on all Unix systems
- **Low Resource**: Minimal CPU and memory overhead
- **Consistent**: Same plugin/data produces identical hash values

```bash
# Enable hashing
ENABLE_HASHING=1 ./collect_info.sh
```

**Use Cases:**
- Verify plugin integrity and detect unauthorized modifications
- Validate data consistency across multiple collection runs
- Security auditing and compliance requirements
- Change detection in system configurations

### Privilege Support

Optional sudo/privileged user support with graceful fallbacks:

- **Backwards Compatible**: Disabled by default, no changes to existing behavior
- **Graceful Fallback**: Attempts privileged execution, falls back to unprivileged
- **No Requirements**: Works without sudo configuration
- **Security Focused**: Only escalates when explicitly enabled

```bash
# Enable sudo support
ENABLE_SUDO_SUPPORT=1 ./collect_info.sh
```

**Privilege Status Indicators:**
- `sudo_support_enabled`: Whether privilege support was requested
- `sudo_available`: Whether sudo is available and configured for current user

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

The system produces structured output with metadata and per-plugin timing information. When all plugins run successfully, the comprehensive JSON includes all system information:

**Note**: The actual output format includes plugin function names as top-level keys (e.g., `get_os_info`, `get_hardware_info`) with nested `data`, `collection_timestamp`, and `completion_timestamp` fields. The example below shows the logical structure with some formatting simplified for readability.

For the actual nested JSON structure with plugin function names and timestamps, run `./collect_info.sh` to see the current format, or refer to the Example Output section above.

```bash
# Generate current format example
./collect_info.sh | python3 -m json.tool
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

## Security Considerations

### Overview

The Automation_nation system information collector is designed with security in mind, but users should be aware of security considerations when deploying and using this tool.

### Privilege Requirements

**No Root Access Required**: All plugins are designed to run as unprivileged users. The system only reads from standard system information sources available to regular users:

- `/proc/` filesystem (read-only)
- `/sys/` filesystem (read-only) 
- `/etc/` configuration files (read-only)
- Standard command-line utilities

**Recommended Practice**: Run the collector as a dedicated non-privileged user account rather than as root or your personal account.

### Plugin Security

**Plugin Execution Safety**:
- ✅ **Fixed**: Removed dangerous `set -e` usage from all plugins that could cause silent failures
- ✅ **Validation**: Enhanced JSON output validation with Python fallback
- ✅ **Input Sanitization**: Architecture parameter validation against known types
- ✅ **Error Isolation**: Plugin failures don't crash the entire collection

**Plugin Development Guidelines**:
```bash
# DO NOT use 'set -e' in plugins executed via command substitution
# ❌ BAD:
#!/bin/bash
set -e

# ✅ GOOD:
#!/bin/bash
# Use explicit error handling instead
```

**Plugin Directory Security**:
- Set directory permissions to `755` with ownership by root or service user
- Set plugin file permissions to `644` to prevent unauthorized modification
- Regularly verify plugin integrity using checksums
- Monitor for unauthorized plugin injection

### Data Privacy and Sensitive Information

**What is NOT collected**:
- Passwords, private keys, or secrets
- User home directory contents
- Application data or databases
- Network traffic or packet contents
- Personal files or documents

**What IS collected**:
- OS version and distribution information
- Hardware specifications (CPU, memory, disk usage)
- Network interface configuration
- Installed packages and system executables
- System uptime and load averages
- Network routing and interface statistics

**Data Handling**:
- All output is in structured JSON format for transparency
- No data is transmitted over the network by the collector itself
- Users control where output is stored (`-o` option)

### Network Security

**Network Information Collection**:
- Interface configurations and IP addresses
- Routing table information
- Listening network services
- ARP table and network neighbors

**Security Notes**:
- Network discovery uses read-only system interfaces
- No active network scanning or probing
- LLDP/CDP neighbor discovery uses passive listening only
- Network namespace enumeration limited to available namespaces

### Deployment Security

**Containerized Environments**:
```dockerfile
# Secure container deployment example
FROM alpine:latest
RUN apk add --no-cache bash python3
RUN adduser -D -s /bin/bash collector
COPY collect_info.sh plugins/ /app/
RUN chown -R root:root /app && chmod 755 /app && chmod 644 /app/plugins/*
USER collector
WORKDIR /app
CMD ["./collect_info.sh"]
```

**System Integration**:
```bash
# Create dedicated service user
sudo useradd -r -s /bin/bash -d /opt/automation_nation automation_collector

# Secure file permissions
sudo chown -R root:automation_collector /opt/automation_nation
sudo chmod 755 /opt/automation_nation
sudo chmod 644 /opt/automation_nation/plugins/*
sudo chmod 755 /opt/automation_nation/collect_info.sh
```

### Output Security

**JSON Output Sanitization**:
- Special characters properly escaped
- Control characters filtered out
- Output format validation prevents injection attacks
- Structured data format prevents command injection in downstream tools

**Integration Security**:
```bash
# Secure output handling
./collect_info.sh | jq -r '.get_os_info.data.os_name' | grep -E '^[a-zA-Z0-9 .-]+$'

# Avoid direct shell evaluation of output
# ❌ BAD: eval "$(./collect_info.sh | jq -r '.some_field')"
# ✅ GOOD: Use structured parsing with validation
```

### Monitoring and Auditing

**Security Monitoring**:
- Monitor plugin execution for unexpected failures
- Log collection timestamps for audit trails
- Validate plugin integrity with checksums
- Monitor for unauthorized modifications to plugin directory

**Audit Recommendations**:
- Regularly review installed plugins
- Monitor system access patterns when collector runs
- Validate output format and content for anomalies
- Track collection frequency and usage patterns

### Threat Model

**Protected Against**:
- Accidental privilege escalation (no sudo required)
- Plugin injection attacks (directory permissions)
- Output injection attacks (JSON escaping)
- Silent failures (explicit error handling)
- Information disclosure beyond system metadata

**Potential Risks**:
- System fingerprinting (by design - this is an information collector)
- Resource exhaustion (configurable limits in place)
- Side-channel information leakage through timing
- Unauthorized access to collected output files

**Mitigation Strategies**:
- Use dedicated service account with minimal privileges
- Secure output file permissions appropriately
- Implement collection frequency limits
- Monitor and audit collector usage

### Compliance Considerations

**Data Classification**: System metadata collection may be subject to organizational data classification policies. Review collected information against your security and privacy requirements.

**Regulatory Compliance**: Consider requirements like GDPR, HIPAA, or SOX when deploying in regulated environments, particularly regarding system inventory and configuration data.

**Change Management**: Implement proper change control for plugin modifications and new plugin deployments.

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
