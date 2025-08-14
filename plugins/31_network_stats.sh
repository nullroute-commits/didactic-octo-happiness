#!/bin/bash
# Network statistics plugin
# Outputs interface statistics, route tables, and multicast group info in JSON format

# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.

ARCH="$1"

# Configuration limits (can be overridden by environment variables)
MAX_INTERFACES=${MAX_INTERFACES:-20}
MAX_ROUTES=${MAX_ROUTES:-50}
MAX_MCAST_GROUPS=${MAX_MCAST_GROUPS:-30}
MAX_LISTENING_PORTS=${MAX_LISTENING_PORTS:-50}

get_network_stats() {
    # Function to escape JSON strings
    escape_json() {
        echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\x0//g'
    }

    # Get interface statistics
    local interface_stats="["
    local first_stat=true

    if [[ -f /proc/net/dev ]]; then
        # Read interface statistics from /proc/net/dev
        while IFS= read -r line; do
            # Skip header lines
            if [[ "$line" =~ ^[[:space:]]*[a-zA-Z0-9_-]+: ]]; then
                local interface=$(echo "$line" | awk -F: '{print $1}' | tr -d ' ')
                local stats=$(echo "$line" | awk -F: '{print $2}')
                local rx_bytes=$(echo "$stats" | awk '{print $1}')
                local rx_packets=$(echo "$stats" | awk '{print $2}')
                local rx_errors=$(echo "$stats" | awk '{print $3}')
                local rx_dropped=$(echo "$stats" | awk '{print $4}')
                local tx_bytes=$(echo "$stats" | awk '{print $9}')
                local tx_packets=$(echo "$stats" | awk '{print $10}')
                local tx_errors=$(echo "$stats" | awk '{print $11}')
                local tx_dropped=$(echo "$stats" | awk '{print $12}')

                if [[ "$first_stat" == "false" ]]; then
                    interface_stats+=","
                fi
                first_stat=false

                interface_stats+="{\"interface\":\"$(escape_json "$interface")\",\"rx_bytes\":\"$rx_bytes\",\"rx_packets\":\"$rx_packets\",\"rx_errors\":\"$rx_errors\",\"rx_dropped\":\"$rx_dropped\",\"tx_bytes\":\"$tx_bytes\",\"tx_packets\":\"$tx_packets\",\"tx_errors\":\"$tx_errors\",\"tx_dropped\":\"$tx_dropped\"}"
            fi
        done < <(tail -n +3 /proc/net/dev | head -${MAX_INTERFACES})
    fi

    # If no interface stats found, add unknown entry
    if [[ "$first_stat" == "true" ]]; then
        interface_stats+="{\"interface\":\"unknown\",\"rx_bytes\":\"0\",\"rx_packets\":\"0\",\"rx_errors\":\"0\",\"rx_dropped\":\"0\",\"tx_bytes\":\"0\",\"tx_packets\":\"0\",\"tx_errors\":\"0\",\"tx_dropped\":\"0\"}"
    fi

    interface_stats+="]"

    # Get IPv4 routing table
    local ipv4_routes="["
    local first_route=true

    if command -v ip >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                # Parse ip route output
                local destination=$(echo "$line" | awk '{print $1}' || echo "unknown")
                local gateway=$(echo "$line" | grep -o "via [^ ]*" | awk '{print $2}' || echo "direct")
                local interface=$(echo "$line" | grep -o "dev [^ ]*" | awk '{print $2}' || echo "unknown")
                local metric=$(echo "$line" | grep -o "metric [0-9]*" | awk '{print $2}' || echo "0")

                if [[ "$first_route" == "false" ]]; then
                    ipv4_routes+=","
                fi
                first_route=false

                ipv4_routes+="{\"destination\":\"$(escape_json "$destination")\",\"gateway\":\"$(escape_json "$gateway")\",\"interface\":\"$(escape_json "$interface")\",\"metric\":\"$(escape_json "$metric")\"}"
            fi
        done < <(ip -4 route show 2>/dev/null | head -${MAX_ROUTES})
    elif command -v route >/dev/null 2>&1; then
        # Fallback to route command
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ ! "$line" =~ ^Kernel ]] && [[ ! "$line" =~ ^Destination ]]; then
                local destination=$(echo "$line" | awk '{print $1}' || echo "unknown")
                local gateway=$(echo "$line" | awk '{print $2}' || echo "direct")
                local interface=$(echo "$line" | awk '{print $8}' || echo "unknown")
                local metric=$(echo "$line" | awk '{print $5}' || echo "0")

                if [[ "$first_route" == "false" ]]; then
                    ipv4_routes+=","
                fi
                first_route=false

                ipv4_routes+="{\"destination\":\"$(escape_json "$destination")\",\"gateway\":\"$(escape_json "$gateway")\",\"interface\":\"$(escape_json "$interface")\",\"metric\":\"$(escape_json "$metric")\"}"
            fi
        done < <(route -n 2>/dev/null | head -${MAX_ROUTES})
    elif [[ -f /proc/net/route ]]; then
        # Fallback to /proc/net/route
        while IFS= read -r line; do
            if [[ ! "$line" =~ ^Iface ]]; then
                local interface=$(echo "$line" | awk '{print $1}')
                local destination=$(echo "$line" | awk '{print $2}')
                local gateway=$(echo "$line" | awk '{print $3}')
                local metric=$(echo "$line" | awk '{print $7}')

                if [[ "$first_route" == "false" ]]; then
                    ipv4_routes+=","
                fi
                first_route=false

                ipv4_routes+="{\"destination\":\"$(escape_json "$destination")\",\"gateway\":\"$(escape_json "$gateway")\",\"interface\":\"$(escape_json "$interface")\",\"metric\":\"$(escape_json "$metric")\"}"
            fi
        done < <(head -${MAX_INTERFACES} /proc/net/route)
    fi

    # If no IPv4 routes found, add unknown entry
    if [[ "$first_route" == "true" ]]; then
        ipv4_routes+="{\"destination\":\"unknown\",\"gateway\":\"unknown\",\"interface\":\"unknown\",\"metric\":\"0\"}"
    fi

    ipv4_routes+="]"

    # Get IPv6 routing table
    local ipv6_routes="["
    local first_ipv6_route=true

    if command -v ip >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local destination=$(echo "$line" | awk '{print $1}' || echo "unknown")
                local gateway=$(echo "$line" | grep -o "via [^ ]*" | awk '{print $2}' || echo "direct")
                local interface=$(echo "$line" | grep -o "dev [^ ]*" | awk '{print $2}' || echo "unknown")
                local metric=$(echo "$line" | grep -o "metric [0-9]*" | awk '{print $2}' || echo "0")

                if [[ "$first_ipv6_route" == "false" ]]; then
                    ipv6_routes+=","
                fi
                first_ipv6_route=false

                ipv6_routes+="{\"destination\":\"$(escape_json "$destination")\",\"gateway\":\"$(escape_json "$gateway")\",\"interface\":\"$(escape_json "$interface")\",\"metric\":\"$(escape_json "$metric")\"}"
            fi
        done < <(ip -6 route show 2>/dev/null | head -${MAX_ROUTES})
    elif [[ -f /proc/net/ipv6_route ]]; then
        # Fallback to /proc/net/ipv6_route
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local destination=$(echo "$line" | awk '{print $1}')
                local gateway=$(echo "$line" | awk '{print $5}')
                local interface_idx=$(echo "$line" | awk '{print $10}')
                local metric=$(echo "$line" | awk '{print $6}')

                if [[ "$first_ipv6_route" == "false" ]]; then
                    ipv6_routes+=","
                fi
                first_ipv6_route=false

                ipv6_routes+="{\"destination\":\"$(escape_json "$destination")\",\"gateway\":\"$(escape_json "$gateway")\",\"interface\":\"$(escape_json "$interface_idx")\",\"metric\":\"$(escape_json "$metric")\"}"
            fi
        done < <(head -${MAX_INTERFACES} /proc/net/ipv6_route)
    fi

    # If no IPv6 routes found, add empty array
    if [[ "$first_ipv6_route" == "true" ]]; then
        ipv6_routes="[]"
    else
        ipv6_routes+="]"
    fi

    # Get multicast groups
    local multicast_groups="["
    local first_mcast=true

    if [[ -f /proc/net/igmp ]]; then
        # Simplified parsing - extract interface and group info more reliably
        local current_interface=""
        while IFS= read -r line; do
            if [[ ! "$line" =~ ^Idx ]] && [[ -n "$line" ]]; then
                # Check if line contains an interface declaration
                if [[ "$line" =~ [[:space:]]+([a-zA-Z0-9_]+)[[:space:]]*: ]]; then
                    current_interface="${BASH_REMATCH[1]}"
                elif [[ -n "$current_interface" ]] && [[ "$line" =~ ^[[:space:]]+[0-9A-F]{8} ]]; then
                    # Extract multicast group address
                    local group=$(echo "$line" | awk '{print $1}' | tr -d ' ')
                    
                    if [[ -n "$group" ]] && [[ "$group" != "Group" ]]; then
                        if [[ "$first_mcast" == "false" ]]; then
                            multicast_groups+=","
                        fi
                        first_mcast=false

                        multicast_groups+="{\"interface\":\"$(escape_json "$current_interface")\",\"group\":\"$(escape_json "$group")\",\"version\":\"ipv4\"}"
                    fi
                fi
            fi
        done < <(cat /proc/net/igmp | head -${MAX_MCAST_GROUPS})
    fi

    if [[ -f /proc/net/igmp6 ]]; then
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                # Parse igmp6 format: idx interface_name multicast_address refcnt flags timer
                local interface_idx=$(echo "$line" | awk '{print $1}')
                local interface_name=$(echo "$line" | awk '{print $2}')
                local group=$(echo "$line" | awk '{print $3}')

                if [[ -n "$interface_name" ]] && [[ -n "$group" ]] && [[ "$interface_name" != "igmp6" ]]; then
                    if [[ "$first_mcast" == "false" ]]; then
                        multicast_groups+=","
                    fi
                    first_mcast=false

                    multicast_groups+="{\"interface\":\"$(escape_json "$interface_name")\",\"group\":\"$(escape_json "$group")\",\"version\":\"ipv6\"}"
                fi
            fi
        done < <(head -${MAX_MCAST_GROUPS} /proc/net/igmp6 2>/dev/null)
    fi

    # If no multicast groups found, add empty array
    if [[ "$first_mcast" == "true" ]]; then
        multicast_groups="[]"
    else
        multicast_groups+="]"
    fi

    # Get network connections (listening ports)
    local listening_ports="["
    local first_port=true

    if command -v ss >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ ! "$line" =~ ^Netid ]]; then
                local protocol=$(echo "$line" | awk '{print $1}' | tr '[:upper:]' '[:lower:]')
                local state=$(echo "$line" | awk '{print $2}')
                local local_addr=$(echo "$line" | awk '{print $5}')

                if [[ "$state" == "LISTEN" ]] || [[ "$protocol" == "udp" ]]; then
                    if [[ "$first_port" == "false" ]]; then
                        listening_ports+=","
                    fi
                    first_port=false

                    listening_ports+="{\"protocol\":\"$(escape_json "$protocol")\",\"local_address\":\"$(escape_json "$local_addr")\",\"state\":\"$(escape_json "$state")\"}"
                fi
            fi
        done < <(ss -tuln 2>/dev/null | head -${MAX_LISTENING_PORTS})
    elif command -v netstat >/dev/null 2>&1; then
        while IFS= read -r line; do
            if [[ -n "$line" ]] && [[ ! "$line" =~ ^Active ]] && [[ ! "$line" =~ ^Proto ]]; then
                local protocol=$(echo "$line" | awk '{print $1}' | tr '[:upper:]' '[:lower:]')
                local local_addr=$(echo "$line" | awk '{print $4}')
                local state=$(echo "$line" | awk '{print $6}')

                # Handle UDP which doesn't have a state column in the same position
                if [[ "$protocol" == "udp" ]]; then
                    state="UNCONN"
                fi

                if [[ "$state" == "LISTEN" ]] || [[ "$protocol" == "udp" ]]; then
                    if [[ "$first_port" == "false" ]]; then
                        listening_ports+=","
                    fi
                    first_port=false

                    listening_ports+="{\"protocol\":\"$(escape_json "$protocol")\",\"local_address\":\"$(escape_json "$local_addr")\",\"state\":\"$(escape_json "$state")\"}"
                fi
            fi
        done < <(netstat -tuln 2>/dev/null | head -${MAX_LISTENING_PORTS})
    fi

    # If no listening ports found, add empty array
    if [[ "$first_port" == "true" ]]; then
        listening_ports="[]"
    else
        listening_ports+="]"
    fi

    # Architecture-specific adjustments
    case "$ARCH" in
        x86_64|amd64|i386|i686)
            # Standard processing for x86 architectures
            ;;
        arm64|aarch64|aarch32|armv7l|armv8l|arm)
            # ARM-specific network handling
            ;;
        ppc64le)
            # PowerPC specific adjustments
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
  "interface_statistics": $interface_stats,
  "ipv4_routes": $ipv4_routes,
  "ipv6_routes": $ipv6_routes,
  "multicast_groups": $multicast_groups,
  "listening_ports": $listening_ports,
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
get_network_stats