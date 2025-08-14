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
│ plugins/40_packages_execs.sh - Package and Executable Info │
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

The system merges plugin outputs using a structured JSON approach with metadata and timestamps:

```bash
# Start with basic structure and collection metadata
JSON="{\"detected_architecture\": \"$ARCH\","
JSON+="\"collection_metadata\": {"
JSON+="\"timestamp\": \"$COLLECTION_START_TIME\","
JSON+="\"plugin_count\": ${#PLUGINS[@]}"
JSON+="},"

FIRST=1
for plugin in "${PLUGINS[@]}"; do
  plugin_basename=$(basename "$plugin")
  function_name=$(extract_function_name "$plugin")
  
  # Capture execution time and output
  start_time=$(get_timestamp)
  OUTPUT="$($plugin "$ARCH")"
  end_time=$(get_timestamp)
  
  if ! validate_json "$OUTPUT" "$plugin_basename"; then
    continue
  fi
  
  # Create the new structure with function name as key
  if [[ $FIRST -eq 1 ]]; then
    FIRST=0
  else
    JSON+=","
  fi
  
  # Strip the outer braces from plugin output to get just the content
  PLUGIN_DATA="${OUTPUT:1:-1}"
  
  # Add plugin data with function name as key and timestamp
  JSON+="\"$function_name\": {"
  JSON+="\"data\": {$PLUGIN_DATA},"
  JSON+="\"collection_timestamp\": \"$start_time\","
  JSON+="\"completion_timestamp\": \"$end_time\""
  JSON+="}"
done

JSON+="}"
```

**Technical Notes:**
- **Function Name Extraction**: Advanced algorithm extracts meaningful function names from plugins
- **Nested Structure**: Plugin data wrapped in function-named objects with metadata
- **Timing Information**: Per-plugin collection and completion timestamps
- **Collection Metadata**: Top-level metadata including plugin count and collection start time
- **Enhanced Validation**: Python-powered JSON validation with fallbacks
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

### Package and Executable Plugin (40_packages_execs.sh)

#### Package Detection Strategy
1. **Package Manager Detection**: Automatic detection of available package managers
2. **Cross-Platform Support**: Handles multiple package managers per system
3. **Version Parsing**: Extracts version information in standardized format
4. **Configuration Discovery**: Maps package-specific configuration locations

#### Supported Package Managers

| Package Manager | Systems | Query Command | Version Format |
|-----------------|---------|---------------|----------------|
| dpkg | Debian/Ubuntu | `dpkg-query -W` | Package version with distribution |
| rpm | Red Hat/CentOS/Fedora | `rpm -qa` | Name-Version-Release |
| brew | macOS | `brew list --versions` | Package version |
| pacman | Arch Linux | `pacman -Q` | Package version |
| apk | Alpine Linux | `apk list -I` | Package-version |
| pkg | FreeBSD | `pkg info` | Package version |

#### Executable Discovery Algorithm
```bash
# Search standard executable paths
search_paths="/usr/bin /usr/local/bin /bin"

# Per-path discovery
for path in $search_paths; do
    find "$path" -maxdepth 1 -type f -executable
done
```

#### Version Detection Strategy
- **Common Tools**: Specific version detection for bash, python3, git, vim
- **Standard Flags**: Attempts `--version`, `-V`, `-v` flags
- **Fallback**: Reports "unknown" when version unavailable
- **Performance**: Quick detection to avoid timeouts

#### Configuration File Location Mapping
```bash
# Package-based configuration locations
"/etc/$package.conf"
"/etc/$package/"

# Executable-based configuration locations  
"/etc/$executable.conf"
"~/.config/$executable"
"~/.$executable rc"
```

#### Resource Management
- **Configurable Limits**: `MAX_PACKAGES` and `MAX_EXECUTABLES` environment variables
- **Efficient Processing**: Stream-based processing to handle large package lists
- **Memory Conservation**: Limited output buffering
- **Timeout Prevention**: Restricted version detection attempts

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
    ├── 40_packages_execs_test.bats    # Package/executable tests
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

**Architecture Parameter Validation**:
```bash
# validate against known architectures
TOP_ARCHS="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
for arch in $TOP_ARCHS; do
    if [[ "$DETECTED_ARCH" == "$arch" ]]; then
        VALID_ARCH=1
        break
    fi
done
```

**Plugin Discovery Security**:
- Only executable files from designated directory are considered plugins
- No dynamic plugin loading from external sources
- Plugin ordering based on filename prevents injection attacks

**JSON Output Validation**:
- Regex validation: `^\{.*\}$` for basic structure
- Python JSON parsing validation when available
- Malformed output gracefully handled and logged

### Plugin Security Model

**Safe Execution Environment**:
```bash
# Enhanced error handling - no 'set -e' in plugins
validate_input() {
    [[ -z "$ARCH" ]] && {
        echo "Error: Architecture parameter required" >&2
        exit 1
    }
}

# Explicit error handling instead of 'set -e'
if ! command -v required_tool >/dev/null 2>&1; then
    echo "Warning: required_tool not found, using fallback" >&2
    use_fallback_method
fi
```

**Output Sanitization**:
```bash
# JSON string escaping function used in all plugins
escape_json() {
    echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\x0//g'
}

# Usage in plugin output
cat << EOF
{
  "field": "$(escape_json "$user_data")",
  "safe_field": "$validated_data"
}
EOF
```

### System Access Patterns

**File System Access**:
- **Read-only**: `/proc/`, `/sys/`, `/etc/` system directories
- **No write access**: Temporary files only in `/tmp` if needed
- **No traversal**: Fixed paths, no user-controlled directory traversal

**Command Execution**:
- **Whitelisted commands**: Only standard system utilities
- **No shell injection**: All user data properly escaped
- **Fallback handling**: Graceful degradation when tools unavailable

**Network Access**:
- **Local only**: No remote network connections initiated
- **Passive discovery**: Read-only access to network configuration
- **No active scanning**: Uses existing system network state

### Attack Surface Analysis

**Plugin Directory**:
```bash
# Recommended secure setup
chown root:automation_collector /opt/automation_nation/plugins
chmod 755 /opt/automation_nation/plugins
chmod 644 /opt/automation_nation/plugins/*.sh
chmod 755 /opt/automation_nation/collect_info.sh

# Integrity verification
find plugins/ -type f -exec sha256sum {} \; > plugins.checksums
# Verify before each run
sha256sum -c plugins.checksums
```

**Input Attack Vectors**:
- **Architecture parameter**: Validated against known list
- **Environment variables**: Plugin-specific limits with defaults
- **File system state**: Read-only access minimizes impact

**Output Attack Vectors**:
- **JSON injection**: Prevented by proper escaping
- **Command injection**: Structured output prevents shell evaluation
- **Information disclosure**: Limited to intended system metadata

### Secure Deployment Patterns

**Service User Configuration**:
```bash
# Create dedicated service user
useradd -r -s /bin/bash -d /opt/automation_nation \
        -c "Automation Nation Collector" automation_collector

# Minimal file permissions
install -o root -g automation_collector -m 755 collect_info.sh /opt/automation_nation/
install -o root -g automation_collector -m 644 plugins/*.sh /opt/automation_nation/plugins/
```

**Container Security**:
```dockerfile
FROM alpine:latest
RUN adduser -D -s /bin/bash collector
# Use specific tool versions for reproducibility
RUN apk add --no-cache bash=5.2.15-r5 python3=3.11.8-r0
USER collector
# No privileged operations in container
```

**Monitoring Integration**:
```bash
# Secure output handling in monitoring systems
./collect_info.sh | \
  jq -c '.' | \
  logger -t automation_nation -p local0.info

# Rate limiting
if [[ -f /tmp/.automation_nation_lock ]]; then
    echo "Collection already in progress" >&2
    exit 1
fi
touch /tmp/.automation_nation_lock
trap 'rm -f /tmp/.automation_nation_lock' EXIT
```

### Privilege Requirements

**No Elevation Required**:
- All plugins designed for unprivileged execution
- Standard user permissions sufficient for all data sources
- No `sudo` or privilege escalation needed anywhere in codebase

**Minimal Access Principle**:
- Read-only access to system information files
- No access to user data or application files
- No network privileges or special capabilities required

**Service Account Recommendations**:
- Dedicated non-login service account
- Minimal group memberships
- No home directory or shell access
- Restricted file system access through chroot if desired

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
