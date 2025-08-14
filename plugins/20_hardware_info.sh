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
    local pcie_devices=""
    local usb_devices=""
    local gpu_info=""
    local network_hardware=""
    
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
    
    # Get PCIe device information
    pcie_devices="["
    local first_pcie=true
    
    if command -v lspci >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local slot=$(echo "$line" | cut -d' ' -f1)
                local device_info=$(echo "$line" | cut -d' ' -f2-)
                local vendor=""
                local device_id=""
                
                # Try to get detailed info
                if lspci -v -s "$slot" >/dev/null 2>&1; then
                    vendor=$(lspci -v -s "$slot" 2>/dev/null | grep "Vendor:" | head -1 | cut -d: -f2- | sed 's/^ *//' || echo "unknown")
                    device_id=$(lspci -n -s "$slot" 2>/dev/null | awk '{print $3}' || echo "unknown")
                fi
                
                if [[ "$first_pcie" == "false" ]]; then
                    pcie_devices+=","
                fi
                first_pcie=false
                
                pcie_devices+="{\"slot\":\"$slot\",\"device\":\"$device_info\",\"vendor\":\"$vendor\",\"device_id\":\"$device_id\"}"
            fi
        done < <(lspci 2>/dev/null | head -20)
    fi
    
    if [[ "$first_pcie" == "true" ]]; then
        pcie_devices+="{\"slot\":\"unknown\",\"device\":\"PCIe information unavailable\",\"vendor\":\"unknown\",\"device_id\":\"unknown\"}"
    fi
    pcie_devices+="]"
    
    # Get USB device information  
    usb_devices="["
    local first_usb=true
    
    if command -v lsusb >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ "$line" =~ Bus.*Device ]]; then
                local bus=$(echo "$line" | grep -o "Bus [0-9]*" | awk '{print $2}')
                local device=$(echo "$line" | grep -o "Device [0-9]*" | awk '{print $2}')
                local id=$(echo "$line" | grep -o "[0-9a-f]\{4\}:[0-9a-f]\{4\}" | head -1)
                local description=$(echo "$line" | sed 's/.*[0-9a-f]\{4\}:[0-9a-f]\{4\} //')
                
                if [[ "$first_usb" == "false" ]]; then
                    usb_devices+=","
                fi
                first_usb=false
                
                usb_devices+="{\"bus\":\"$bus\",\"device\":\"$device\",\"id\":\"$id\",\"description\":\"$description\"}"
            fi
        done < <(lsusb 2>/dev/null | head -15)
    fi
    
    if [[ "$first_usb" == "true" ]]; then
        usb_devices+="{\"bus\":\"unknown\",\"device\":\"unknown\",\"id\":\"unknown\",\"description\":\"USB information unavailable\"}"
    fi
    usb_devices+="]"
    
    # Get GPU/APU information
    gpu_info="["
    local first_gpu=true
    
    # Try multiple methods for GPU detection
    if command -v lspci >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local slot=$(echo "$line" | cut -d' ' -f1)
                local gpu_desc=$(echo "$line" | cut -d' ' -f2-)
                local vendor="unknown"
                local memory="unknown"
                
                # Try to get vendor info
                if echo "$gpu_desc" | grep -qi "nvidia"; then
                    vendor="NVIDIA"
                elif echo "$gpu_desc" | grep -qi "amd\|ati"; then
                    vendor="AMD"
                elif echo "$gpu_desc" | grep -qi "intel"; then
                    vendor="Intel"
                fi
                
                # Try to get memory info if nvidia-smi available
                if [[ "$vendor" == "NVIDIA" ]] && command -v nvidia-smi >/dev/null 2>&1; then
                    memory=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 | sed 's/^ *//' || echo "unknown")
                    if [[ "$memory" != "unknown" ]]; then
                        memory="${memory} MB"
                    fi
                fi
                
                if [[ "$first_gpu" == "false" ]]; then
                    gpu_info+=","
                fi
                first_gpu=false
                
                gpu_info+="{\"slot\":\"$slot\",\"description\":\"$gpu_desc\",\"vendor\":\"$vendor\",\"memory\":\"$memory\"}"
            fi
        done < <(lspci 2>/dev/null | grep -i "vga\|3d\|display" | head -5)
    fi
    
    if [[ "$first_gpu" == "true" ]]; then
        gpu_info+="{\"slot\":\"unknown\",\"description\":\"GPU information unavailable\",\"vendor\":\"unknown\",\"memory\":\"unknown\"}"
    fi
    gpu_info+="]"
    
    # Get detailed network hardware information
    network_hardware="["
    local first_nic=true
    
    if command -v lspci >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local slot=$(echo "$line" | cut -d' ' -f1)
                local nic_desc=$(echo "$line" | cut -d' ' -f2-)
                local vendor="unknown"
                local driver="unknown"
                local speed="unknown"
                
                # Try to get driver info
                if [[ -d "/sys/bus/pci/devices/0000:$slot" ]]; then
                    if [[ -L "/sys/bus/pci/devices/0000:$slot/driver" ]]; then
                        driver=$(readlink "/sys/bus/pci/devices/0000:$slot/driver" 2>/dev/null | sed 's/.*\///' || echo "unknown")
                    fi
                fi
                
                # Extract vendor from description
                if echo "$nic_desc" | grep -qi "intel"; then
                    vendor="Intel"
                elif echo "$nic_desc" | grep -qi "broadcom"; then
                    vendor="Broadcom"
                elif echo "$nic_desc" | grep -qi "realtek"; then
                    vendor="Realtek"
                elif echo "$nic_desc" | grep -qi "qualcomm\|atheros"; then
                    vendor="Qualcomm/Atheros"
                fi
                
                if [[ "$first_nic" == "false" ]]; then
                    network_hardware+=","
                fi
                first_nic=false
                
                network_hardware+="{\"slot\":\"$slot\",\"description\":\"$nic_desc\",\"vendor\":\"$vendor\",\"driver\":\"$driver\",\"speed\":\"$speed\"}"
            fi
        done < <(lspci 2>/dev/null | grep -i "network\|ethernet\|wireless\|wifi" | head -10)
    fi
    
    if [[ "$first_nic" == "true" ]]; then
        network_hardware+="{\"slot\":\"unknown\",\"description\":\"Network hardware information unavailable\",\"vendor\":\"unknown\",\"driver\":\"unknown\",\"speed\":\"unknown\"}"
    fi
    network_hardware+="]"
    
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
  "disk_info": $disk_info,
  "pcie_devices": $pcie_devices,
  "usb_devices": $usb_devices,
  "gpu_info": $gpu_info,
  "network_hardware": $network_hardware
}
EOF
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Install bc if not available (for calculations)
check_dependencies() {
    local missing_deps=""
    
    # Check for bc (used for calculations)
    if ! command -v bc >/dev/null 2>&1; then
        missing_deps="${missing_deps}bc "
    fi
    
    # Warn about missing dependencies but continue
    if [[ -n "$missing_deps" ]]; then
        echo "Warning: Missing optional dependencies: $missing_deps" >&2
        echo "Some calculations may fall back to basic arithmetic" >&2
    fi
}

check_dependencies

# Execute main function
get_hardware_info