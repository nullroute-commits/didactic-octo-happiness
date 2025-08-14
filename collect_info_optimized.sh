#!/bin/bash
# Simple optimized version of collect_info.sh

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""

# Configuration options
ENABLE_HASHING=${ENABLE_HASHING:-1}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}

# Fast architecture detection using case instead of multiple conditionals
detect_arch() {
  case "$(uname -m)" in
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
    *) uname -m ;;
  esac
}

# Optimized CRC32 calculation - direct cksum without intermediate commands
calculate_crc32() {
    local input="$1"
    if [[ "$ENABLE_HASHING" -eq 1 ]] && command -v cksum >/dev/null 2>&1; then
        printf '%s' "$input" | cksum | awk '{print $1}'
    else
        echo "disabled"
    fi
}

# Hash plugin file content
hash_plugin_content() {
    local plugin_file="$1"
    if [[ "$ENABLE_HASHING" -eq 1 ]] && [[ -f "$plugin_file" ]]; then
        if command -v cksum >/dev/null 2>&1; then
            cksum "$plugin_file" | awk '{print $1}'
        else
            echo "unavailable"
        fi
    else
        echo "disabled"
    fi
}

# Hash function output data
hash_function_data() {
    local data="$1"
    calculate_crc32 "$data"
}

# Check for privilege escalation capabilities (optimized)
check_privilege_support() {
    if [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]] && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
        echo "1"
    else
        echo "0"
    fi
}

usage() {
  echo "Usage: $0 [-o output.json] [-h]"
  echo "Environment variables:"
  echo "  ENABLE_HASHING=0      - Disable CRC32 hashing of datasets (default: 1)"
  echo "  ENABLE_SUDO_SUPPORT=1 - Enable sudo privilege detection (default: 0)"
  exit 1
}

# Basic JSON validation
validate_json() {
    local json_string="$1"
    # Check for basic JSON structure
    [[ "$json_string" =~ ^\{.*\}$ ]]
}

# Main collection function with optimizations
collect_system_info() {
    local detected_arch=$(detect_arch)
    local collection_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local sudo_available=$(check_privilege_support)
    
    # Pre-count plugins for better performance
    local plugin_count=0
    for plugin_file in "$PLUGIN_DIR"/*.sh; do
        [[ -f "$plugin_file" && -x "$plugin_file" ]] && ((plugin_count++))
    done
    
    # Start JSON output efficiently
    cat << EOF
{
  "detected_architecture": "$detected_arch",
  "collection_metadata": {
    "timestamp": "$collection_timestamp",
    "plugin_count": $plugin_count,
    "hashing_enabled": $ENABLE_HASHING,
    "sudo_support_enabled": $ENABLE_SUDO_SUPPORT,
    "sudo_available": $sudo_available
  },
EOF
    
    # Process plugins with improved error handling
    local first_plugin=true
    
    for plugin_file in "$PLUGIN_DIR"/*.sh; do
        if [[ -f "$plugin_file" && -x "$plugin_file" ]]; then
            local plugin_name=$(basename "$plugin_file" .sh)
            local function_name="get_${plugin_name#??_}"
            
            # Add comma separator for JSON
            if [[ "$first_plugin" != true ]]; then
                echo ","
            fi
            
            local collection_timestamp_plugin=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
            
            # Execute plugin with error handling in subshell for isolation
            local plugin_output=""
            if plugin_output=$(source "$plugin_file" 2>/dev/null && declare -f "$function_name" >/dev/null 2>&1 && $function_name 2>/dev/null); then
                local completion_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                
                if validate_json "$plugin_output"; then
                    local plugin_file_hash=$(hash_plugin_content "$plugin_file")
                    local function_data_hash=$(hash_function_data "$plugin_output")
                    
                    # Output plugin result directly without temp files
                    cat << EOF
  "$function_name": {
    "data": $plugin_output,
    "collection_timestamp": "$collection_timestamp_plugin",
    "completion_timestamp": "$completion_timestamp",
    "plugin_file_hash": "$plugin_file_hash",
    "function_data_hash": "$function_data_hash"
  }
EOF
                    first_plugin=false
                else
                    echo "Warning: Plugin $plugin_name returned invalid JSON. Skipping." >&2
                fi
            else
                echo "Warning: Plugin $plugin_name failed to execute. Skipping." >&2
            fi
        fi
    done
    
    echo ""
    echo "}"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            ;;
    esac
done

# Execute main function
if [[ -n "$OUTPUT_FILE" ]]; then
    collect_system_info > "$OUTPUT_FILE"
    echo "System info written to $OUTPUT_FILE"
else
    collect_system_info
fi
