#!/bin/bash
# System uptime information plugin
# Outputs uptime info in JSON format

# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.

ARCH="$1"

get_uptime_info() {
    local uptime_seconds=""
    local uptime_formatted=""
    local boot_time=""
    local load_average=""
    
    # Get uptime in seconds
    if [[ -f /proc/uptime ]] && [[ -r /proc/uptime ]]; then
        uptime_seconds=$(awk '{print int($1)}' /proc/uptime 2>/dev/null || echo "unknown")
    elif command -v uptime >/dev/null 2>&1; then
        # Fallback to parsing uptime command output
        local uptime_output=$(uptime 2>/dev/null || echo "")
        if [[ "$uptime_output" =~ up[[:space:]]+([0-9]+)[[:space:]]+days? ]]; then
            local days=${BASH_REMATCH[1]}
            uptime_seconds=$((days * 86400))
        elif [[ "$uptime_output" =~ up[[:space:]]+([0-9]+):([0-9]+) ]]; then
            local hours=${BASH_REMATCH[1]}
            local minutes=${BASH_REMATCH[2]}
            uptime_seconds=$((hours * 3600 + minutes * 60))
        else
            uptime_seconds="unknown"
        fi
    else
        uptime_seconds="unknown"
    fi
    
    # Calculate formatted uptime string
    if [[ "$uptime_seconds" != "unknown" ]] && [[ "$uptime_seconds" =~ ^[0-9]+$ ]]; then
        local days=$((uptime_seconds / 86400))
        local hours=$(((uptime_seconds % 86400) / 3600))
        local minutes=$(((uptime_seconds % 3600) / 60))
        local seconds=$((uptime_seconds % 60))
        
        if [[ $days -gt 0 ]]; then
            uptime_formatted="${days}d ${hours}h ${minutes}m ${seconds}s"
        elif [[ $hours -gt 0 ]]; then
            uptime_formatted="${hours}h ${minutes}m ${seconds}s"
        elif [[ $minutes -gt 0 ]]; then
            uptime_formatted="${minutes}m ${seconds}s"
        else
            uptime_formatted="${seconds}s"
        fi
    else
        uptime_formatted="unknown"
    fi
    
    # Get boot time
    if [[ -f /proc/stat ]] && [[ -r /proc/stat ]]; then
        boot_time=$(grep "^btime " /proc/stat 2>/dev/null | awk '{print $2}' || echo "unknown")
    else
        boot_time="unknown"
    fi
    
    # Get load average
    if [[ -f /proc/loadavg ]] && [[ -r /proc/loadavg ]]; then
        load_average=$(cat /proc/loadavg 2>/dev/null | awk '{print $1, $2, $3}' || echo "unknown")
    elif command -v uptime >/dev/null 2>&1; then
        # Extract load average from uptime command
        local uptime_output=$(uptime 2>/dev/null || echo "")
        if [[ "$uptime_output" =~ load[[:space:]]+averages?:[[:space:]]*([0-9.]+)[[:space:]]*,[[:space:]]*([0-9.]+)[[:space:]]*,[[:space:]]*([0-9.]+) ]]; then
            load_average="${BASH_REMATCH[1]} ${BASH_REMATCH[2]} ${BASH_REMATCH[3]}"
        elif [[ "$uptime_output" =~ load[[:space:]]+average:[[:space:]]*([0-9.]+)[[:space:]]*,[[:space:]]*([0-9.]+)[[:space:]]*,[[:space:]]*([0-9.]+) ]]; then
            load_average="${BASH_REMATCH[1]} ${BASH_REMATCH[2]} ${BASH_REMATCH[3]}"
        else
            load_average="unknown"
        fi
    else
        load_average="unknown"
    fi
    
    # Architecture-specific adjustments
    case "$ARCH" in
        x86_64|amd64|i386|i686)
            # Standard x86 processing
            ;;
        arm64|aarch64|aarch32|armv7l|armv8l|arm)
            # ARM-specific adjustments
            if [[ -f /proc/device-tree/model ]]; then
                local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0' || echo "")
                if [[ "$model" =~ "Raspberry Pi" ]]; then
                    # Raspberry Pi systems may have different uptime behavior
                    :
                fi
            fi
            ;;
        ppc64le|s390x|riscv64|mips64|sparc64|loongarch64)
            # Other architectures - no specific adjustments needed
            ;;
    esac
    
    # Output JSON
    cat << EOF
{
  "uptime_seconds": "$uptime_seconds",
  "uptime_formatted": "$uptime_formatted",
  "boot_time": "$boot_time",
  "load_average": "$load_average",
  "architecture": "$ARCH"
}
EOF
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Execute main function
get_uptime_info