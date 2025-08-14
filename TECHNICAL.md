# Technical Implementation Guide

This document provides in-depth technical details about the Automation_nation system architecture, implementation patterns, and internal workings.

## System Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    collect_info.sh                          │
│                   (Main Orchestrator)                       │
├─────────────────────────────────────────────────────────────┤
│ 1. Architecture Detection (detect_arch())                  │
│ 2. Plugin Discovery (scan plugins/ directory)              │
│ 3. Plugin Execution (sequential, ordered by filename)      │
│ 4. JSON Aggregation (merge plugin outputs)                 │
│ 5. Output Management (stdout or file)                      │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Ecosystem                         │
├─────────────────────────────────────────────────────────────┤
│ plugins/10_os_info.sh     - OS/Distribution Detection      │
│ plugins/20_hardware_info.sh - Hardware Information         │
│ plugins/30_ip_info.sh     - Network Interface Details      │
│ plugins/31_network_stats.sh - Network Statistics/Routing   │
│ plugins/32_lldp_neighbors.sh - LLDP/ARP/Bridge Information │
│ plugins/50_uptime_info.sh - System Uptime Information      │
│ plugins/[NN]_*.sh         - Future Extensions              │
└─────────────────────────────────────────────────────────────┘
```

## Architecture Detection Engine

### Detection Algorithm

```bash
detect_arch() {
  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;; 
    aarch64|arm64) echo "arm64" ;; 
    i386|i686) echo "i386" ;; 
    ppc64le) echo "ppc64le" ;; 
    s390x) echo "s390x" ;; 
    riscv64) echo "riscv64" ;; 
    mips64) echo "mips64" ;; 
    armv7l|armv8l|arm) echo "aarch32" ;; 
    sparc64) echo "sparc64" ;; 
    loongarch64) echo "loongarch64" ;; 
    *) echo "$arch" ;;
  esac
}
```

### Architecture Mapping Strategy

The system normalizes various `uname -m` outputs to standardized architecture identifiers:

| Raw `uname -m` | Normalized | Market Context |
|----------------|------------|----------------|
| `x86_64`, `amd64` | `x86_64` | Intel/AMD 64-bit (dominant server/desktop) |
| `aarch64`, `arm64` | `arm64` | Apple Silicon, AWS Graviton, server ARM |
| `i386`, `i686` | `i386` | Legacy 32-bit x86 |
| `ppc64le` | `ppc64le` | IBM POWER (enterprise) |
| `s390x` | `s390x` | IBM Z mainframes |
| `riscv64` | `riscv64` | RISC-V 64-bit (emerging open ISA) |
| `mips64` | `mips64` | MIPS 64-bit (embedded/networking) |
| `armv7l`, `armv8l`, `arm` | `aarch32` | ARM 32-bit (IoT/embedded) |
| `sparc64` | `sparc64` | Oracle SPARC systems |
| `loongarch64` | `loongarch64` | Chinese LoongArch architecture |

## Plugin System Implementation

### Plugin Discovery Mechanism

```bash
PLUGINS=()
for file in "$PLUGIN_DIR"/*; do
  [ -x "$file" ] && PLUGINS+=("$file")
done
```

**Key Implementation Details:**
- **Executable Check**: Only files with execute permissions are considered plugins
- **Alphabetical Ordering**: Natural shell globbing provides execution order
- **Numeric Prefixes**: Convention `NN_name.sh` ensures predictable sequencing
- **Dynamic Discovery**: No hardcoded plugin list - fully extensible

### Plugin Execution Protocol

#### Input Contract
- **Argument 1**: Detected architecture string (required)
- **Environment**: Clean environment with standard PATH
- **Working Directory**: Plugin's directory context

#### Output Contract
- **Format**: Valid JSON object `{"key": "value", ...}`
- **Structure**: Self-contained object (no arrays at root level)
- **Encoding**: UTF-8 text output to stdout
- **Error Handling**: stderr for warnings, non-zero exit for failures

#### Execution Flow
```bash
for plugin in "${PLUGINS[@]}"; do
  # Capture stdout and stderr separately
  OUTPUT="$("$plugin" "$ARCH" 2> >(cat >&3))" 3>plugin_stderr.log
  PLUGIN_EXIT_CODE=$?
  PLUGIN_STDERR=$(cat plugin_stderr.log)
  rm -f plugin_stderr.log

  if [[ $PLUGIN_EXIT_CODE -ne 0 ]]; then
    # Sanitize error output before logging
    echo "Error: Plugin $plugin failed to execute. See logs for details." >&2
    # Optionally, log sanitized stderr to a secure location
    # echo "$PLUGIN_STDERR" | sed 's/[[:cntrl:]]//g' >> /var/log/automation_nation/plugin_errors.log
    continue
  fi
  if [[ ! "$OUTPUT" =~ ^\{.*\}$ ]]; then
    echo "Warning: Plugin $plugin did not return valid JSON. Skipping." >&2
    continue
  fi
  # JSON merging logic...
done
```

### JSON Aggregation Algorithm

The system merges plugin outputs using a sophisticated JSON concatenation approach:

```bash
JSON="{\"detected_architecture\": \"$ARCH\","

FIRST=1
for plugin in "${PLUGINS[@]}"; do
  OUTPUT="$($plugin "$ARCH")"
  FRAGMENT="$(echo "$OUTPUT" | jq -c 'to_entries | map("\(.key):\(.value|tojson)") | join(",")')"
  if [[ $FIRST -eq 1 ]]; then
    JSON+="$FRAGMENT"
    FIRST=0
  else
    JSON+=", $FRAGMENT"
  fi
done

JSON+="}"
```

**Technical Notes:**
- **Brace Stripping**: `${OUTPUT:1:-1}` removes first and last characters
- **Comma Management**: First plugin gets no leading comma, subsequent ones do
- **Architecture Injection**: System-level `detected_architecture` field always included
- **Order Preservation**: Plugin execution order maintained in final JSON

## Plugin Implementation Patterns

### OS Information Plugin (10_os_info.sh)

#### Detection Strategy Hierarchy
1. **Modern Systems**: `/etc/os-release` (systemd standard)
2. **Red Hat Family**: `/etc/redhat-release`
3. **Debian Family**: `/etc/debian_version`
4. **SUSE Family**: `/etc/SuSE-release`
5. **macOS**: `sw_vers` command
6. **WSL Detection**: `wsl.exe` availability
7. **Fallback**: `uname` system calls

#### Architecture-Specific Enhancements
```bash
case "$ARCH" in
    arm64|aarch64)
        if [[ -f /proc/device-tree/model ]]; then
            local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0')
            if [[ "$model" =~ "Raspberry Pi" ]]; then
                distro="${distro}_rpi"
            fi
        fi
        ;;
    ppc64le)
        if [[ -f /proc/cpuinfo ]] && grep -q "POWER" /proc/cpuinfo; then
            distro="${distro}_power"
        fi
        ;;
esac
```

### Network Interface Plugin (30_ip_info.sh)

#### Interface Discovery Strategy
1. **Primary**: `ip link show` for modern Linux systems
2. **Fallback**: `/proc/net/dev` for systems without iproute2
3. **Alternative**: `ifconfig -a` for legacy systems

#### Address Collection Algorithm
```bash
get_interface_info() {
    local interface="$1"
    
    # IPv4 address collection
    ip -4 addr show "$interface" | grep "inet " | awk '{print $2}'
    
    # IPv6 address collection  
    ip -6 addr show "$interface" | grep "inet6 " | awk '{print $2}'
    
    # MAC, MTU, and state from link information
    ip link show "$interface"
}
```

#### Architecture-Specific Enhancements
- **ARM Systems**: Enhanced Raspberry Pi detection via device tree
- **Embedded Platforms**: Special handling for non-standard interface naming
- **Container Environments**: Docker and LXC interface detection

### Network Statistics Plugin (31_network_stats.sh)

#### Multi-Source Statistics Collection

| Data Source | Primary Tool | Fallback | Coverage |
|-------------|--------------|----------|----------|
| Interface Stats | `/proc/net/dev` | None | Universal Linux |
| IPv4 Routes | `ip route` | `route -n`, `/proc/net/route` | Cross-platform |
| IPv6 Routes | `ip -6 route` | `/proc/net/ipv6_route` | Modern systems |
| Listening Ports | `ss -tuln` | `netstat -tuln` | Service discovery |
| Multicast Groups | `/proc/net/igmp`, `/proc/net/igmp6` | None | Group membership |

#### Route Parsing Implementation
```bash
# IPv4 route parsing with multiple fallbacks
parse_ipv4_routes() {
    if command -v ip >/dev/null 2>&1; then
        ip -4 route show | while read -r line; do
            destination=$(echo "$line" | awk '{print $1}')
            gateway=$(echo "$line" | grep -o "via [^ ]*" | awk '{print $2}' || echo "direct")
            interface=$(echo "$line" | grep -o "dev [^ ]*" | awk '{print $2}')
            metric=$(echo "$line" | grep -o "metric [0-9]*" | awk '{print $2}' || echo "0")
        done
    elif command -v route >/dev/null 2>&1; then
        # Fallback to route command
    fi
}
```

### LLDP/ARP Discovery Plugin (32_lldp_neighbors.sh)

#### Network Discovery Hierarchy
1. **LLDP Discovery**: `lldpctl` → `lldptool` → per-interface queries
2. **ARP Table**: `ip neigh` → `arp -a` → `/proc/net/arp`
3. **Bridge Detection**: `brctl show` → `bridge link` → Docker bridge API
4. **Network Namespaces**: `ip netns list`

#### LLDP Protocol Support
```bash
# Multi-protocol neighbor discovery
discover_neighbors() {
    # LLDP (Link Layer Discovery Protocol)
    if command -v lldpctl >/dev/null 2>&1; then
        lldpctl | parse_lldp_output
    fi
    
    # CDP (Cisco Discovery Protocol) 
    if command -v cdpctl >/dev/null 2>&1; then
        cdpctl | parse_cdp_output  
    fi
}
```

#### Bridge Information Collection
- **Linux Bridges**: Native kernel bridge detection
- **Docker Bridges**: Container network bridge enumeration
- **STP Status**: Spanning Tree Protocol state detection
- **Port Membership**: Bridge port and interface relationships

### Hardware Information Plugin (20_hardware_info.sh)

#### CPU Detection Matrix

| Architecture | Primary Source | Fallback | Special Handling |
|--------------|----------------|----------|------------------|
| x86_64/i386 | `/proc/cpuinfo` "model name" | `nproc` | Cores vs threads detection |
| ARM variants | `/proc/cpuinfo` "Processor" | Device tree model | Raspberry Pi identification |
| PowerPC | `/proc/cpuinfo` "cpu" | LPAR detection | `/proc/ppc64/lparcfg` |
| RISC-V | `/proc/cpuinfo` "isa" | ISA string parsing | Emerging standard handling |
| MIPS | `/proc/cpuinfo` "cpu model" | System type detection | Endianness considerations |
| IBM Z | `/proc/cpuinfo` "processor" | Z-specific parsing | Mainframe context |

#### Memory Detection Strategy
```bash
if [[ -f /proc/meminfo ]]; then
    memory_total=$(grep "MemTotal:" /proc/meminfo | awk '{print $2}')
    memory_available=$(grep "MemAvailable:" /proc/meminfo | awk '{print $2}' || 
                      grep "MemFree:" /proc/meminfo | awk '{print $2}')
elif command -v vm_stat >/dev/null 2>&1; then
    # macOS handling with page size calculations
fi
```

#### Disk Information Collection
- **Primary**: `df -h` for filesystem usage
- **Filtering**: Excludes virtual filesystems (tmpfs, devpts, etc.)
- **Scope**: Root filesystem and standard mount points only
- **Format**: Array of filesystem objects with usage metrics

## Testing Framework Architecture

### Bats Testing Structure

```
test/
├── integration/
│   └── collect_info_test.bats          # Main orchestrator tests
└── plugins/
    ├── 10_os_info_test.bats           # OS plugin tests
    ├── 20_hardware_info_test.bats     # Hardware plugin tests  
    ├── 30_ip_info_test.bats           # Network interface tests
    ├── 31_network_stats_test.bats     # Network statistics tests
    ├── 32_lldp_neighbors_test.bats    # LLDP/ARP plugin tests
    └── 50_uptime_info_test.bats       # Uptime plugin tests
```

### Test Environment Isolation

```bash
setup() {
    export ORIGINAL_PATH="$PATH"
    export TEST_DIR="/tmp/collect_info_test"
    export TEST_PLUGIN_DIR="$TEST_DIR/plugins"
    mkdir -p "$TEST_PLUGIN_DIR"
    cp collect_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/collect_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
    export PATH="$ORIGINAL_PATH"
}
```

### Test Categories

1. **Architecture Detection Tests**
   - Validates all 10 supported architectures
   - Tests `detect_arch()` function mapping
   - Verifies architecture parameter passing

2. **Plugin Discovery Tests** 
   - Executable file detection
   - Ordering verification
   - Missing directory handling

3. **JSON Validation Tests**
   - Output format compliance
   - Merge algorithm correctness
   - Invalid JSON handling

4. **Error Condition Tests**
   - Missing plugins directory (exit code 2)
   - No executable plugins (exit code 3)
   - Malformed plugin output (graceful degradation)

5. **Integration Tests**
   - End-to-end workflow validation
   - Output file generation (-o option)
   - Command-line argument processing

## Performance Characteristics

### Execution Profile
- **Plugin Discovery**: O(n) directory scan
- **Architecture Detection**: O(1) case matching
- **Plugin Execution**: O(n) sequential, where n = plugin count
- **JSON Merging**: O(m) string operations, where m = total output size

### Resource Usage
- **Memory**: Minimal - bash arrays and string variables only
- **Disk I/O**: Read-only access to system information files
- **Network**: None (purely local system inspection)
- **CPU**: Lightweight text processing operations

### Scalability Considerations
- **Plugin Count**: Linear scaling, no practical limits
- **Output Size**: Limited by system memory for JSON aggregation
- **Execution Time**: Dominated by slowest plugin (typically hardware detection)

## Error Handling Strategy

### Exit Codes
- `0`: Success
- `1`: Usage/help display
- `2`: Plugin directory not found
- `3`: No executable plugins found

### Error Recovery
- **Invalid JSON**: Plugin skipped with warning, execution continues
- **Plugin Failure**: Non-zero exit from plugin logs warning, continues
- **System Errors**: Graceful fallbacks to "unknown" values

### Logging Strategy
- **stdout**: JSON output only (when successful)
- **stderr**: Warnings, errors, and diagnostic messages
- **Verbosity**: Minimal by design - warnings for operational issues only

## Extension Points

### Custom Plugin Development

#### Minimum Viable Plugin
```bash
#!/bin/bash
set -e
ARCH="$1"
[[ -z "$ARCH" ]] && { echo "Error: Architecture required" >&2; exit 1; }

# Collection logic here
data="collected_value"

cat << EOF
{
  "plugin_identifier": "$data",
  "architecture": "$ARCH"
}
EOF
```

#### Advanced Plugin Template

> **Note:**  
> Do **not** use `set -e` in plugin scripts. When plugins are executed via command substitution in the main orchestrator script, `set -e` can cause unexpected and hard-to-diagnose failures.  
> Instead, use explicit error handling (e.g., check exit codes and handle errors directly) within your plugin scripts.

```bash
#!/bin/bash
# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.
# Instead, use explicit error handling as shown below.

ARCH="$1"

validate_input() {
    [[ -z "$ARCH" ]] && {
        echo "Error: Architecture parameter required" >&2
        exit 1
    }
}

collect_arch_specific_data() {
    case "$ARCH" in
        x86_64|amd64)
            # x86-specific collection
            ;;
        arm64|aarch64)
            # ARM-specific collection
            ;;
        *)
            # Generic fallback
            ;;
    esac
}

output_json() {
    local data="$1"
    cat << EOF
{
  "custom_metric": "$data",
  "collection_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source_architecture": "$ARCH"
}
EOF
}

# Main execution
validate_input
data=$(collect_arch_specific_data)
output_json "$data"
```

### System Integration Patterns

#### Configuration Management Integration
```bash
# Ansible facts integration
./collect_info.sh -o /etc/ansible/facts.d/system_info.fact

# Puppet external facts
./collect_info.sh -o /opt/puppetlabs/facter/facts.d/system_info.json
```

#### Monitoring System Integration
```bash
# Prometheus node_exporter textfile collector
./collect_info.sh | jq -r 'to_entries[] | select(.key|test("^[a-zA-Z_:][a-zA-Z0-9_:]*$")) | select(.value|type=="number") | "\(.key) \(.value)"' > system_info.prom

# Telegraf exec input plugin
./collect_info.sh | telegraf --config telegraf.conf
```

#### Container Integration
```dockerfile
# Dockerfile example for containerized collection
FROM alpine:latest
RUN apk add --no-cache bash jq
COPY collect_info.sh plugins/ ./
CMD ["./collect_info.sh"]
```

## Security Considerations

### Input Validation
- **Architecture Parameter**: Validated against known architecture list
- **Plugin Execution**: Only executable files from designated directory
- **JSON Output**: Regex validation before aggregation

### Privilege Requirements
- **Standard User**: All plugins designed for unprivileged execution
- **File Access**: Read-only access to `/proc`, `/sys`, `/etc` system directories
- **No Elevation**: No `sudo` or privilege escalation required

### Attack Surface
- **Plugin Directory**: Set directory permissions to `755` and plugin file permissions to `644`, with ownership by `root` (or the designated service user). Regularly verify plugin integrity using checksums or digital signatures to prevent unauthorized plugin injection.
- **System Information**: No sensitive data collection (passwords, keys, etc.)
- **Output Sanitization**: JSON escaping prevents injection in downstream systems

## Maintenance and Debugging

### Debug Mode Enhancement
Add to plugins for troubleshooting:
```bash
DEBUG=${DEBUG:-0}
debug_log() {
    [[ "$DEBUG" -eq 1 ]] && echo "DEBUG: $*" >&2
}

debug_log "Architecture detected: $ARCH"
debug_log "Data source: /proc/cpuinfo"
```

### Validation Tools
```bash
# Validate JSON output
./collect_info.sh | python3 -m json.tool

# Check plugin executable status
find plugins/ -type f ! -executable

# Test individual plugin
./plugins/10_os_info.sh x86_64 | jq .
```

### Common Issues and Solutions

| Issue | Symptom | Solution |
|-------|---------|----------|
| Plugin not found | "No plugins found" | Check execute permissions |
| Invalid JSON | Warning message | Validate plugin JSON output |
| Architecture unknown | Falls back to `uname -m` | Add mapping to `detect_arch()` |
| Missing dependencies | Plugin errors | Install required tools (bc, jq, etc.) |

This technical guide provides the implementation details needed for system administrators, developers, and contributors to understand, modify, and extend the Automation_nation framework.
