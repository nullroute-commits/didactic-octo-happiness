#!/bin/bash
# LLDP neighbors, ARP table, and bridge information plugin
# Outputs LLDP/CDP neighbors, ARP table, and bridge info in JSON format

set -e

ARCH="$1"

get_lldp_neighbors() {
    # Function to escape JSON strings
    escape_json() {
        echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\x0//g'
    }

    # Get LLDP neighbors
    local lldp_neighbors="["
    local first_lldp=true

    if command -v lldpctl >/dev/null 2>&1; then
        # Use lldpctl if available
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local interface=$(echo "$line" | awk '{print $1}')
                local chassis_id=$(echo "$line" | awk '{print $2}')
                local port_id=$(echo "$line" | awk '{print $3}')
                local system_name=$(echo "$line" | awk '{$1=$2=$3=""; print $0}' | sed 's/^ *//')

                if [[ -n "$interface" ]] && [[ "$interface" != "Interface" ]]; then
                    if [[ "$first_lldp" == "false" ]]; then
                        lldp_neighbors+=","
                    fi
                    first_lldp=false

                    lldp_neighbors+="{\"local_interface\":\"$(escape_json "$interface")\",\"chassis_id\":\"$(escape_json "$chassis_id")\",\"port_id\":\"$(escape_json "$port_id")\",\"system_name\":\"$(escape_json "$system_name")\",\"protocol\":\"lldp\"}"
                fi
            fi
        done < <(lldpctl 2>/dev/null | grep -A 100 "LLDP neighbors" | grep "^[[:space:]]*[a-zA-Z]" | head -20)
    elif command -v lldptool >/dev/null 2>&1; then
        # Alternative LLDP tool
        while IFS= read -r interface; do
            if [[ -n "$interface" ]]; then
                local lldp_info=$(lldptool -t -i "$interface" 2>/dev/null || echo "")
                if [[ -n "$lldp_info" ]]; then
                    local chassis_id=$(echo "$lldp_info" | grep "Chassis ID" | cut -d: -f2- | tr -d ' ' || echo "unknown")
                    local port_id=$(echo "$lldp_info" | grep "Port ID" | cut -d: -f2- | tr -d ' ' || echo "unknown")
                    local system_name=$(echo "$lldp_info" | grep "System Name" | cut -d: -f2- | sed 's/^ *//' || echo "unknown")

                    if [[ "$first_lldp" == "false" ]]; then
                        lldp_neighbors+=","
                    fi
                    first_lldp=false

                    lldp_neighbors+="{\"local_interface\":\"$(escape_json "$interface")\",\"chassis_id\":\"$(escape_json "$chassis_id")\",\"port_id\":\"$(escape_json "$port_id")\",\"system_name\":\"$(escape_json "$system_name")\",\"protocol\":\"lldp\"}"
                fi
            fi
        done < <(ip link show 2>/dev/null | grep "^[0-9]" | awk -F': ' '{print $2}' | cut -d'@' -f1 | head -10)
    fi

    # Check for CDP (Cisco Discovery Protocol) if available
    if command -v cdpctl >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local interface=$(echo "$line" | awk '{print $1}')
                local device_id=$(echo "$line" | awk '{print $2}')
                local platform=$(echo "$line" | awk '{print $3}')

                if [[ -n "$interface" ]] && [[ "$interface" != "Interface" ]]; then
                    if [[ "$first_lldp" == "false" ]]; then
                        lldp_neighbors+=","
                    fi
                    first_lldp=false

                    lldp_neighbors+="{\"local_interface\":\"$(escape_json "$interface")\",\"chassis_id\":\"$(escape_json "$device_id")\",\"port_id\":\"unknown\",\"system_name\":\"$(escape_json "$platform")\",\"protocol\":\"cdp\"}"
                fi
            fi
        done < <(cdpctl 2>/dev/null | head -20)
    fi

    # If no LLDP/CDP neighbors found, add empty array
    if [[ "$first_lldp" == "true" ]]; then
        lldp_neighbors="[]"
    else
        lldp_neighbors+="]"
    fi

    # Get ARP table
    local arp_table="["
    local first_arp=true

    if command -v ip >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local ip_addr=$(echo "$line" | awk '{print $1}')
                local interface=$(echo "$line" | grep -o "dev [^ ]*" | awk '{print $2}' || echo "unknown")
                local mac_addr=$(echo "$line" | grep -o "[0-9a-f:]\{17\}" || echo "unknown")
                local state=$(echo "$line" | grep -o "REACHABLE\|STALE\|DELAY\|PROBE\|FAILED\|NOARP" || echo "unknown")

                if [[ -n "$ip_addr" ]] && [[ "$ip_addr" != "Address" ]]; then
                    if [[ "$first_arp" == "false" ]]; then
                        arp_table+=","
                    fi
                    first_arp=false

                    arp_table+="{\"ip_address\":\"$(escape_json "$ip_addr")\",\"mac_address\":\"$(escape_json "$mac_addr")\",\"interface\":\"$(escape_json "$interface")\",\"state\":\"$(escape_json "$state")\"}"
                fi
            fi
        done < <(ip neigh show 2>/dev/null | head -50)
    elif command -v arp >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ ! "$line" =~ ^Address ]]; then
                local ip_addr=$(echo "$line" | awk '{print $1}')
                local mac_addr=$(echo "$line" | awk '{print $3}')
                local interface=$(echo "$line" | awk '{print $5}')

                if [[ -n "$ip_addr" ]]; then
                    if [[ "$first_arp" == "false" ]]; then
                        arp_table+=","
                    fi
                    first_arp=false

                    arp_table+="{\"ip_address\":\"$(escape_json "$ip_addr")\",\"mac_address\":\"$(escape_json "$mac_addr")\",\"interface\":\"$(escape_json "$interface")\",\"state\":\"unknown\"}"
                fi
            fi
        done < <(arp -a 2>/dev/null | head -50)
    elif [[ -f /proc/net/arp ]]; then
        while IFS= read -r line; do
            if [[ ! "$line" =~ ^IP ]]; then
                local ip_addr=$(echo "$line" | awk '{print $1}')
                local mac_addr=$(echo "$line" | awk '{print $4}')
                local interface=$(echo "$line" | awk '{print $6}')

                if [[ -n "$ip_addr" ]]; then
                    if [[ "$first_arp" == "false" ]]; then
                        arp_table+=","
                    fi
                    first_arp=false

                    arp_table+="{\"ip_address\":\"$(escape_json "$ip_addr")\",\"mac_address\":\"$(escape_json "$mac_addr")\",\"interface\":\"$(escape_json "$interface")\",\"state\":\"unknown\"}"
                fi
            fi
        done < <(cat /proc/net/arp | head -50)
    fi

    # If no ARP entries found, add empty array
    if [[ "$first_arp" == "true" ]]; then
        arp_table="[]"
    else
        arp_table+="]"
    fi

    # Get bridge information
    local bridge_info="["
    local first_bridge=true

    if command -v brctl >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ ! "$line" =~ ^bridge ]]; then
                local bridge_name=$(echo "$line" | awk '{print $1}')
                local bridge_id=$(echo "$line" | awk '{print $2}')
                local stp=$(echo "$line" | awk '{print $3}')
                local interfaces=$(echo "$line" | awk '{$1=$2=$3=""; print $0}' | sed 's/^ *//')

                if [[ -n "$bridge_name" ]]; then
                    if [[ "$first_bridge" == "false" ]]; then
                        bridge_info+=","
                    fi
                    first_bridge=false

                    bridge_info+="{\"bridge_name\":\"$(escape_json "$bridge_name")\",\"bridge_id\":\"$(escape_json "$bridge_id")\",\"stp_enabled\":\"$(escape_json "$stp")\",\"interfaces\":\"$(escape_json "$interfaces")\"}"
                fi
            fi
        done < <(brctl show 2>/dev/null | head -20)
    elif command -v bridge >/dev/null 2>&1; then
        # Use bridge command from iproute2
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local bridge_name=$(echo "$line" | awk '{print $1}')
                local state=$(echo "$line" | grep -o "state [^ ]*" | awk '{print $2}' || echo "unknown")

                if [[ -n "$bridge_name" ]] && [[ "$bridge_name" != "name" ]]; then
                    # Get bridge details
                    local bridge_id=$(bridge link show master "$bridge_name" 2>/dev/null | head -1 | awk '{print $1}' || echo "unknown")
                    
                    if [[ "$first_bridge" == "false" ]]; then
                        bridge_info+=","
                    fi
                    first_bridge=false

                    bridge_info+="{\"bridge_name\":\"$(escape_json "$bridge_name")\",\"bridge_id\":\"$(escape_json "$bridge_id")\",\"stp_enabled\":\"unknown\",\"interfaces\":\"unknown\"}"
                fi
            fi
        done < <(bridge link show 2>/dev/null | head -20)
    fi

    # Check for Docker bridges
    if command -v docker >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local network_name=$(echo "$line" | awk '{print $1}')
                local driver=$(echo "$line" | awk '{print $2}')
                local scope=$(echo "$line" | awk '{print $3}')

                if [[ "$driver" == "bridge" ]] && [[ -n "$network_name" ]] && [[ "$network_name" != "NETWORK" ]]; then
                    if [[ "$first_bridge" == "false" ]]; then
                        bridge_info+=","
                    fi
                    first_bridge=false

                    bridge_info+="{\"bridge_name\":\"$(escape_json "$network_name")\",\"bridge_id\":\"docker\",\"stp_enabled\":\"unknown\",\"interfaces\":\"docker_managed\"}"
                fi
            fi
        done < <(docker network ls --filter driver=bridge 2>/dev/null | head -10)
    fi

    # If no bridges found, add empty array
    if [[ "$first_bridge" == "true" ]]; then
        bridge_info="[]"
    else
        bridge_info+="]"
    fi

    # Get network namespaces (if available)
    local network_namespaces="["
    local first_netns=true

    if command -v ip >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                if [[ "$first_netns" == "false" ]]; then
                    network_namespaces+=","
                fi
                first_netns=false

                network_namespaces+="\"$(escape_json "$line")\""
            fi
        done < <(ip netns list 2>/dev/null | awk '{print $1}' | head -20)
    fi

    # If no network namespaces found, add empty array
    if [[ "$first_netns" == "true" ]]; then
        network_namespaces="[]"
    else
        network_namespaces+="]"
    fi

    # Architecture-specific adjustments
    case "$ARCH" in
        x86_64|amd64|i386|i686)
            # Standard processing for x86 architectures
            ;;
        arm64|aarch64|aarch32|armv7l|armv8l|arm)
            # ARM-specific network discovery
            # Some ARM devices may have specialized network configurations
            ;;
        ppc64le)
            # PowerPC specific network handling
            ;;
        s390x)
            # IBM Z specific network handling
            ;;
        riscv64|mips64|sparc64|loongarch64)
            # Other architectures - standard handling
            ;;
    esac

    # Output JSON
    cat << EOF
{
  "lldp_neighbors": $lldp_neighbors,
  "arp_table": $arp_table,
  "bridge_info": $bridge_info,
  "network_namespaces": $network_namespaces,
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
get_lldp_neighbors