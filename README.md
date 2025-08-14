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
  ]
}
```

## Plugin Architecture

### Plugin Directory Structure

```
plugins/
├── 10_os_info.sh      # OS and distribution information
└── 20_hardware_info.sh # CPU, memory, and disk information
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

## Installation

### Prerequisites

- Bash shell (version 4.0+)
- Standard Unix utilities (`uname`, `grep`, `awk`, etc.)
- Python 3 (for JSON validation in examples)
- Bats testing framework (for running tests)

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

The project includes comprehensive test coverage using the Bats testing framework.

### Test Structure

```
test-collect_info-sh/
└── collect_info_test.bats     # Main script tests

test-10_os_info-sh/
└── os_info_test.bats         # OS plugin tests

test-20_hardware_info-sh/
└── hardware_info_test.bats   # Hardware plugin tests
```

### Running Tests

Install Bats testing framework:
```bash
# Ubuntu/Debian
sudo apt-get install bats

# macOS
brew install bats-core
```

Run all tests:
```bash
# Test main collector script
bats test-collect_info-sh/collect_info_test.bats

# Test OS information plugin
bats test-10_os_info-sh/os_info_test.bats

# Test hardware information plugin
bats test-20_hardware_info-sh/hardware_info_test.bats
```

### Test Coverage

- **Architecture Detection**: Tests for all 10 supported architectures
- **Plugin Discovery**: Automatic plugin detection and execution
- **JSON Validation**: Output format validation
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
