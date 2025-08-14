#!/bin/bash
# Hardware information plugin
# Outputs CPU, memory, and disk info in JSON format

# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.

ARCH="$1"

get_hardware_info() {
    local cpu_info=""
    local cpu_cores=""
    local cpu_threads=""
    local cpu_freq=""
    local memory_total=""
    local memory_available=""
    local disk_info=""
    
    # Get CPU information
    if [[ -f /proc/cpuinfo ]]; then
        case "$ARCH" in
            x86_64|amd64|i386|i686)
                cpu_info=$(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "unknown")
                cpu_cores=$(grep "cpu cores" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "unknown")
                cpu_threads=$(grep "siblings" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "unknown")
                ;;
            arm64|aarch64|aarch32|armv7l|armv8l|arm)
                cpu_info=$(grep "model name\|Processor\|Hardware" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "ARM Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            ppc64le)
                cpu_info=$(grep "cpu\|model" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "PowerPC Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            s390x)
                cpu_info=$(grep "processor\|vendor_id" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "IBM Z Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            riscv64)
                cpu_info=$(grep "isa\|uarch" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "RISC-V Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            mips64)
                cpu_info=$(grep "cpu model\|system type" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "MIPS Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            sparc64)
                cpu_info=$(grep "cpu\|type" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "SPARC Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            loongarch64)
                cpu_info=$(grep "Model Name\|cpu family" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "LoongArch Processor")
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
            *)
                cpu_info="unknown"
                cpu_cores=$(nproc 2>/dev/null || echo "unknown")
                cpu_threads="$cpu_cores"
                ;;
        esac
        
        # Get CPU frequency (first try scaling, then cpuinfo)
        if [[ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq ]]; then
            local freq_khz=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq 2>/dev/null || echo "0")
            if [[ "$freq_khz" != "0" ]]; then
                if command -v bc >/dev/null 2>&1; then
                    cpu_freq=$(echo "scale=2; $freq_khz / 1000" | bc 2>/dev/null || echo "unknown")
                else
                    # Fallback arithmetic without bc
                    cpu_freq=$((freq_khz / 1000))
                fi
                cpu_freq="${cpu_freq} MHz"
            else
                cpu_freq="unknown"
            fi
        else
            cpu_freq=$(grep "cpu MHz" /proc/cpuinfo | head -1 | cut -d: -f2 | sed 's/^ *//' 2>/dev/null || echo "unknown")
            if [[ "$cpu_freq" != "unknown" ]]; then
                cpu_freq="${cpu_freq} MHz"
            fi
        fi
    elif command -v sysctl >/dev/null 2>&1; then
        # macOS or BSD systems
        cpu_info=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")
        cpu_cores=$(sysctl -n hw.physicalcpu 2>/dev/null || echo "unknown")
        cpu_threads=$(sysctl -n hw.logicalcpu 2>/dev/null || echo "unknown")
        cpu_freq=$(sysctl -n hw.cpufrequency 2>/dev/null | awk '{print $1/1000000 " MHz"}' || echo "unknown")
    else
        cpu_info="unknown"
        cpu_cores="unknown"
        cpu_threads="unknown"
        cpu_freq="unknown"
    fi
    
    # Get memory information
    if [[ -f /proc/meminfo ]]; then
        memory_total=$(grep "MemTotal:" /proc/meminfo | awk '{print $2}' 2>/dev/null || echo "unknown")
        memory_available=$(grep "MemAvailable:" /proc/meminfo | awk '{print $2}' 2>/dev/null || 
                          grep "MemFree:" /proc/meminfo | awk '{print $2}' 2>/dev/null || echo "unknown")
        
        # Convert to MB if we have the values
        if [[ "$memory_total" != "unknown" ]]; then
            if command -v bc >/dev/null 2>&1; then
                memory_total=$(echo "scale=0; $memory_total / 1024" | bc 2>/dev/null || echo "$memory_total")
            else
                memory_total=$((memory_total / 1024))
            fi
            memory_total="${memory_total} MB"
        fi
        if [[ "$memory_available" != "unknown" ]]; then
            if command -v bc >/dev/null 2>&1; then
                memory_available=$(echo "scale=0; $memory_available / 1024" | bc 2>/dev/null || echo "$memory_available")
            else
                memory_available=$((memory_available / 1024))
            fi
            memory_available="${memory_available} MB"
        fi
    elif command -v vm_stat >/dev/null 2>&1; then
        # macOS
        local page_size=$(vm_stat | grep "page size" | awk '{print $8}' 2>/dev/null || echo "4096")
        local total_pages=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//' 2>/dev/null || echo "0")
        if [[ "$total_pages" != "0" ]]; then
            if command -v bc >/dev/null 2>&1; then
                memory_total=$(echo "scale=0; $total_pages * $page_size / 1024 / 1024" | bc 2>/dev/null || echo "unknown")
            else
                memory_total=$((total_pages * page_size / 1024 / 1024))
            fi
            memory_total="${memory_total} MB"
        else
            memory_total="unknown"
        fi
        memory_available="unknown"
    else
        memory_total="unknown"
        memory_available="unknown"
    fi
    
    # Get disk information
    disk_info="["
    local first_disk=true
    
    if command -v df >/dev/null 2>&1; then
        # Use df to get disk usage
        while read -r filesystem size used avail use mountpoint; do
            # Skip header and special filesystems
            if [[ "$filesystem" == "Filesystem" ]] || [[ "$filesystem" =~ ^(tmpfs|udev|devpts|sysfs|proc|cgroup|systemd|none|overlay) ]]; then
                continue
            fi
            
            # Only include real filesystems and mounted on root or standard mount points
            if [[ "$mountpoint" == "/" ]] || [[ "$mountpoint" =~ ^/(home|boot|usr|var|opt|srv|tmp)$ ]] || [[ "$filesystem" =~ ^/dev/ ]]; then
                if [[ "$first_disk" == "false" ]]; then
                    disk_info+=","
                fi
                first_disk=false
                
                disk_info+="{\"filesystem\":\"$filesystem\",\"size\":\"$size\",\"used\":\"$used\",\"available\":\"$avail\",\"usage\":\"$use\",\"mountpoint\":\"$mountpoint\"}"
            fi
        done < <(df -h 2>/dev/null || echo "")
    fi
    
    # If no disks found, add unknown entry
    if [[ "$first_disk" == "true" ]]; then
        disk_info+="{\"filesystem\":\"unknown\",\"size\":\"unknown\",\"used\":\"unknown\",\"available\":\"unknown\",\"usage\":\"unknown\",\"mountpoint\":\"unknown\"}"
    fi
    
    disk_info+="]"
    
    # Architecture-specific hardware adjustments
    case "$ARCH" in
        arm64|aarch64|aarch32|armv7l|armv8l|arm)
            # ARM-specific hardware detection
            if [[ -f /proc/device-tree/model ]]; then
                local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0' || echo "")
                if [[ "$model" =~ "Raspberry Pi" ]]; then
                    cpu_info="$cpu_info (Raspberry Pi)"
                fi
            fi
            ;;
        ppc64le)
            # PowerPC specific
            if [[ -f /proc/ppc64/lparcfg ]]; then
                cpu_info="$cpu_info (LPAR)"
            fi
            ;;
    esac
    
    # Output JSON
    cat << EOF
{
  "cpu_model": "$cpu_info",
  "cpu_cores": "$cpu_cores",
  "cpu_threads": "$cpu_threads",
  "cpu_frequency": "$cpu_freq",
  "memory_total": "$memory_total",
  "memory_available": "$memory_available",
  "disk_info": $disk_info
}
EOF
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Sanitize architecture parameter (allow only alphanumeric and underscore)
if [[ ! "$ARCH" =~ ^[a-zA-Z0-9_]+$ ]]; then
    echo "Error: Architecture parameter contains invalid characters" >&2
    exit 1
fi

# Validate architecture is supported
SUPPORTED_ARCHS="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
if [[ ! " $SUPPORTED_ARCHS " =~ " $ARCH " ]]; then
    echo "Warning: Architecture '$ARCH' not in supported list, proceeding with generic handling" >&2
fi

# Install bc if not available (for calculations)
check_dependencies() {
    local missing_deps=""
    local optional_deps="bc"
    
    # Check for optional dependencies
    for dep in $optional_deps; do
        if ! command -v "$dep" >/dev/null 2>&1; then
            missing_deps="${missing_deps}$dep "
        fi
    done
    
    # Warn about missing dependencies but continue
    if [[ -n "$missing_deps" ]]; then
        echo "Warning: Missing optional dependencies for hardware plugin: $missing_deps" >&2
        echo "Some calculations may fall back to basic arithmetic" >&2
    fi
}

check_dependencies

# Execute main function
get_hardware_info