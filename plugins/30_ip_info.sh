#!/bin/bash
# IP information plugin
# Outputs detailed IP interface info (IPv4/IPv6) for all network interfaces in JSON format

set -e

ARCH="$1"

# Configuration limits (can be overridden by environment variables)
MAX_INTERFACES=${MAX_INTERFACES:-20}
MAX_ADDRESSES_PER_INTERFACE=${MAX_ADDRESSES_PER_INTERFACE:-10}

get_ip_info() {
    local interfaces_data="["
    local first_interface=true

    # Function to escape JSON strings
    escape_json() {
        echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\x0//g'
    }

    # Function to get interface information using ip command
    get_interface_info() {
        local interface="$1"
        local ipv4_addresses="[]"
        local ipv6_addresses="[]"
        local mac_address="unknown"
        local mtu="unknown"
        local state="unknown"
        
        # Get IPv4 addresses
        if command -v ip >/dev/null 2>&1; then
            local ipv4_list=""
            local first_ipv4=true
            while IFS= read -r line; do
                if [[ -n "$line" ]]; then
                    if [[ "$first_ipv4" == "false" ]]; then
                        ipv4_list+=","
                    fi
                    first_ipv4=false
                    ipv4_list+="\"$(escape_json "$line")\""
                fi
            done < <(ip -4 addr show "$interface" 2>/dev/null | grep "inet " | awk '{print $2}' | head -${MAX_ADDRESSES_PER_INTERFACE})
            
            if [[ "$first_ipv4" == "false" ]]; then
                ipv4_addresses="[$ipv4_list]"
            fi

            # Get IPv6 addresses
            local ipv6_list=""
            local first_ipv6=true
            while IFS= read -r line; do
                if [[ -n "$line" ]]; then
                    if [[ "$first_ipv6" == "false" ]]; then
                        ipv6_list+=","
                    fi
                    first_ipv6=false
                    ipv6_list+="\"$(escape_json "$line")\""
                fi
            done < <(ip -6 addr show "$interface" 2>/dev/null | grep "inet6 " | awk '{print $2}' | head -${MAX_ADDRESSES_PER_INTERFACE})
            
            if [[ "$first_ipv6" == "false" ]]; then
                ipv6_addresses="[$ipv6_list]"
            fi

            # Get MAC address, MTU, and state
            local link_info=$(ip link show "$interface" 2>/dev/null | head -1)
            if [[ -n "$link_info" ]]; then
                mac_address=$(echo "$link_info" | grep -o "link/ether [^ ]*" | awk '{print $2}' || echo "unknown")
                mtu=$(echo "$link_info" | grep -o "mtu [0-9]*" | awk '{print $2}' || echo "unknown")
                if [[ "$link_info" =~ "state UP" ]]; then
                    state="up"
                elif [[ "$link_info" =~ "state DOWN" ]]; then
                    state="down"
                else
                    state="unknown"
                fi
            fi
        fi

        # Fallback to /proc/net if ip command failed
        if [[ "$ipv4_addresses" == "[]" ]] && [[ -f /proc/net/dev ]]; then
            # Try to get basic info from /proc/net/dev
            if grep -q "^ *${interface}:" /proc/net/dev 2>/dev/null; then
                # Interface exists, try ifconfig as fallback
                if command -v ifconfig >/dev/null 2>&1; then
                    local ifconfig_output=$(ifconfig "$interface" 2>/dev/null || echo "")
                    if [[ -n "$ifconfig_output" ]]; then
                        local ipv4_addr=$(echo "$ifconfig_output" | grep "inet " | awk '{print $2}' | head -1)
                        if [[ -n "$ipv4_addr" ]]; then
                            ipv4_addresses="[\"$(escape_json "$ipv4_addr")\"]"
                        fi
                        
                        local ipv6_addr=$(echo "$ifconfig_output" | grep "inet6 " | awk '{print $2}' | head -1)
                        if [[ -n "$ipv6_addr" ]]; then
                            ipv6_addresses="[\"$(escape_json "$ipv6_addr")\"]"
                        fi
                        
                        local mac_addr=$(echo "$ifconfig_output" | grep "ether " | awk '{print $2}' | head -1)
                        if [[ -n "$mac_addr" ]]; then
                            mac_address="$mac_addr"
                        fi
                        
                        local mtu_val=$(echo "$ifconfig_output" | grep "MTU:" | sed 's/.*MTU:\([0-9]*\).*/\1/' | head -1)
                        if [[ -n "$mtu_val" ]]; then
                            mtu="$mtu_val"
                        fi
                        
                        if echo "$ifconfig_output" | grep -q "UP"; then
                            state="up"
                        else
                            state="down"
                        fi
                    fi
                fi
            fi
        fi

        # Architecture-specific adjustments
        case "$ARCH" in
            x86_64|amd64|i386|i686)
                # Standard processing for x86 architectures
                ;;
            arm64|aarch64|aarch32|armv7l|armv8l|arm)
                # ARM-specific network interface handling
                # Some ARM systems may have different interface naming
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

        # Output interface JSON
        cat << EOF
{
  "interface": "$(escape_json "$interface")",
  "ipv4_addresses": $ipv4_addresses,
  "ipv6_addresses": $ipv6_addresses,
  "mac_address": "$(escape_json "$mac_address")",
  "mtu": "$(escape_json "$mtu")",
  "state": "$(escape_json "$state")"
}
EOF
    }

    # Get list of network interfaces
    if command -v ip >/dev/null 2>&1; then
        # Use ip command to list interfaces
        while IFS= read -r interface; do
            if [[ -n "$interface" ]] && [[ "$interface" != "lo" ]]; then
                if [[ "$first_interface" == "false" ]]; then
                    interfaces_data+=","
                fi
                first_interface=false
                interfaces_data+=$(get_interface_info "$interface")
            fi
        done < <(ip link show 2>/dev/null | grep "^[0-9]" | awk -F': ' '{print $2}' | cut -d'@' -f1 | head -${MAX_INTERFACES})
        
        # Also include loopback
        if [[ "$first_interface" == "false" ]]; then
            interfaces_data+=","
        fi
        first_interface=false
        interfaces_data+=$(get_interface_info "lo")
    elif [[ -f /proc/net/dev ]]; then
        # Fallback to /proc/net/dev
        while IFS= read -r line; do
            local interface=$(echo "$line" | awk -F: '{print $1}' | tr -d ' ')
            if [[ -n "$interface" ]] && [[ "$interface" != "Inter-|face" ]] && [[ "$interface" != "Inter-" ]]; then
                if [[ "$first_interface" == "false" ]]; then
                    interfaces_data+=","
                fi
                first_interface=false
                interfaces_data+=$(get_interface_info "$interface")
            fi
        done < <(tail -n +3 /proc/net/dev | head -${MAX_INTERFACES})
    elif command -v ifconfig >/dev/null 2>&1; then
        # Fallback to ifconfig -a
        while IFS= read -r interface; do
            if [[ -n "$interface" ]]; then
                if [[ "$first_interface" == "false" ]]; then
                    interfaces_data+=","
                fi
                first_interface=false
                interfaces_data+=$(get_interface_info "$interface")
            fi
        done < <(ifconfig -a 2>/dev/null | grep "^[a-zA-Z]" | awk '{print $1}' | sed 's/:$//' | head -${MAX_INTERFACES})
    fi

    # If no interfaces found, add unknown entry
    if [[ "$first_interface" == "true" ]]; then
        interfaces_data+="{\"interface\":\"unknown\",\"ipv4_addresses\":[],\"ipv6_addresses\":[],\"mac_address\":\"unknown\",\"mtu\":\"unknown\",\"state\":\"unknown\"}"
    fi

    interfaces_data+="]"

    # Output final JSON
    cat << EOF
{
  "network_interfaces": $interfaces_data,
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
get_ip_info