#!/bin/bash
# Fast optimized version of collect_info.sh

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""
TEMP_DIR=$(mktemp -d)
ENABLE_HASHING=${ENABLE_HASHING:-1}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}

# Cleanup on exit
trap 'rm -rf "$TEMP_DIR"' EXIT

# Fast architecture detection
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

# Fast CRC32 using cksum
fast_crc32() {
    printf '%s' "$1" | cksum | cut -d' ' -f1
}

# Check sudo availability (cached)
check_sudo() {
    if [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]] && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
        echo "1"
    else
        echo "0"
    fi
}

# Execute plugin with error handling
exec_plugin() {
    local plugin="$1"
    local name=$(basename "$plugin" .sh)
    local func="get_${name#??_}"
    local output_file="$TEMP_DIR/${name}.json"
    
    if [[ -f "$plugin" && -x "$plugin" ]]; then
        (
            source "$plugin" 2>/dev/null || exit 1
            if declare -f "$func" >/dev/null 2>&1; then
                local start_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                local data=$($func 2>/dev/null || echo '{}')
                local end_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                
                # Validate basic JSON structure
                if [[ "$data" =~ ^\{.*\}$ ]]; then
                    local plugin_hash="disabled"
                    local data_hash="disabled"
                    
                    if [[ "$ENABLE_HASHING" -eq 1 ]]; then
                        plugin_hash=$(cksum "$plugin" 2>/dev/null | cut -d' ' -f1 || echo "unavailable")
                        data_hash=$(fast_crc32 "$data")
                    fi
                    
                    cat > "$output_file" << END
{
  "data": $data,
  "collection_timestamp": "$start_time",
  "completion_timestamp": "$end_time",
  "plugin_file_hash": "$plugin_hash",
  "function_data_hash": "$data_hash"
}
END
                fi
            fi
        ) &
    fi
}

# Main collection function
collect_info() {
    local arch=$(detect_arch)
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local sudo_available=$(check_sudo)
    
    # Count and execute plugins in parallel
    local plugin_count=0
    for plugin in "$PLUGIN_DIR"/*.sh; do
        if [[ -f "$plugin" && -x "$plugin" ]]; then
            exec_plugin "$plugin"
            ((plugin_count++))
        fi
    done
    
    # Wait for all plugins to complete
    wait
    
    # Generate JSON output efficiently
    cat << EOF
{
  "detected_architecture": "$arch",
  "collection_metadata": {
    "timestamp": "$timestamp",
    "plugin_count": $plugin_count,
    "hashing_enabled": $ENABLE_HASHING,
    "sudo_support_enabled": $ENABLE_SUDO_SUPPORT,
    "sudo_available": $sudo_available
  },
