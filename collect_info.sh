#!/bin/bash
# Main orchestrator for plugin-based system info collection (JSON output)
# Supports top 10 architectures per 2024 Q4 market reports.

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""
PLUGINS=()

# Configuration options
ENABLE_HASHING=${ENABLE_HASHING:-0}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}

TOP_ARCHS="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"

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

# CRC32 hash calculation using cksum (most backwards compatible)
calculate_crc32() {
    local input="$1"
    if command -v cksum >/dev/null 2>&1; then
        printf '%s' "$input" | cksum | awk '{print $1}'
    else
        echo "unavailable"
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
    if [[ "$ENABLE_HASHING" -eq 1 ]]; then
        calculate_crc32 "$data"
    else
        echo "disabled"
    fi
}

# Check for privilege escalation capabilities
check_privilege_support() {
    local has_sudo=0
    if [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]]; then
        if command -v sudo >/dev/null 2>&1; then
            # Test if sudo is available and configured
            if sudo -n true 2>/dev/null; then
                has_sudo=1
            fi
        fi
    fi
    echo "$has_sudo"
}

usage() {
  echo "Usage: $0 [-o output.json] [-h]"
  echo "Environment variables:"
  echo "  ENABLE_HASHING=1      - Enable CRC32 hashing of datasets (default: 0)"
  echo "  ENABLE_SUDO_SUPPORT=1 - Enable sudo privilege detection (default: 0)"
  exit 1
}

# Enhanced JSON validation function
validate_json() {
    local json_string="$1"
    local plugin_name="$2"
    
    # Basic structure check
    if [[ ! "$json_string" =~ ^\{.*\}$ ]]; then
        echo "Warning: Plugin $plugin_name did not return valid JSON structure. Skipping." >&2
        return 1
    fi
    
    # Try to validate with python if available
    if command -v python3 >/dev/null 2>&1; then
        if ! echo "$json_string" | python3 -m json.tool >/dev/null 2>&1; then
            echo "Warning: Plugin $plugin_name returned malformed JSON. Skipping." >&2
            return 1
        fi
    fi
    
    return 0
}

# Extract main function name from plugin
extract_function_name() {
    local plugin_file="$1"
    local function_name=""
    
    # Look for the main function call at the end of the file
    # This searches for lines that are just function names (after validation comments)
    function_name=$(tail -5 "$plugin_file" | grep -E "^[a-zA-Z_][a-zA-Z0-9_]*$" | tail -1)
    
    # If that doesn't work, extract from function definition and look for the main one
    if [[ -z "$function_name" ]]; then
        # Get all function definitions and find the one that matches the plugin purpose
        local basename_plugin=$(basename "$plugin_file" .sh)
        local plugin_type="${basename_plugin#*_}"  # Remove numeric prefix
        
        # Try to find a function that matches the plugin type
        function_name=$(grep -E "^[a-zA-Z_][a-zA-Z0-9_]*\(\)" "$plugin_file" | grep -i "$plugin_type" | head -1 | cut -d'(' -f1)
        
        # If still not found, get the first function definition that starts with 'get_'
        if [[ -z "$function_name" ]]; then
            function_name=$(grep -E "^get_[a-zA-Z_][a-zA-Z0-9_]*\(\)" "$plugin_file" | head -1 | cut -d'(' -f1)
        fi
    fi
    
    # Final fallback: use filename-based naming
    if [[ -z "$function_name" ]]; then
        local basename_plugin=$(basename "$plugin_file" .sh)
        local plugin_suffix="${basename_plugin#*_}"  # Remove numeric prefix
        function_name="get_${plugin_suffix}"
    fi
    
    echo "$function_name"
}

# Generate ISO 8601 timestamp
get_timestamp() {
    date -u +"%Y-%m-%dT%H:%M:%SZ"
}

while getopts "o:h" opt; do
  case "$opt" in
    o) OUTPUT_FILE="$OPTARG" ;; 
    h) usage ;; 
    *) usage ;; 
  esac

done

if [[ ! -d "$PLUGIN_DIR" ]]; then
  echo "Plugin directory $PLUGIN_DIR not found." >&2
  exit 2
fi

PLUGINS=()
for file in "$PLUGIN_DIR"/*; do
  [ -x "$file" ] && PLUGINS+=("$file")
done

if [[ ${#PLUGINS[@]} -eq 0 ]]; then
  echo "No plugins found in $PLUGIN_DIR." >&2
  exit 3
fi

ARCH=$(detect_arch)
COLLECTION_START_TIME=$(get_timestamp)
HAS_SUDO=$(check_privilege_support)

# Start with basic structure and collection metadata
JSON="{\"detected_architecture\": \"$ARCH\","
JSON+="\"collection_metadata\": {"
JSON+="\"timestamp\": \"$COLLECTION_START_TIME\","
JSON+="\"plugin_count\": ${#PLUGINS[@]},"
JSON+="\"hashing_enabled\": $ENABLE_HASHING,"
JSON+="\"sudo_support_enabled\": $ENABLE_SUDO_SUPPORT,"
JSON+="\"sudo_available\": $HAS_SUDO"
JSON+="},"

FIRST=1
for plugin in "${PLUGINS[@]}"; do
  plugin_basename=$(basename "$plugin")
  function_name=$(extract_function_name "$plugin")
  
  # Calculate plugin file hash
  plugin_hash=$(hash_plugin_content "$plugin")
  
  # Capture execution time and output
  start_time=$(get_timestamp)
  if [[ "$HAS_SUDO" -eq 1 ]] && [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]]; then
    # Try with sudo first, fallback to regular execution, but capture and log sudo errors
    SUDO_OUTPUT=""
    SUDO_ERROR=""
    SUDO_OUTPUT="$(sudo "$plugin" "$ARCH" 2> >(SUDO_ERROR=$(cat); typeset -p SUDO_ERROR >&2))" || {
      if [[ -n "$SUDO_ERROR" ]]; then
        echo "Warning: sudo execution of $plugin failed with error:" >&2
        echo "$SUDO_ERROR" >&2
      fi
      SUDO_OUTPUT="$("$plugin" "$ARCH")"
    }
    OUTPUT="$SUDO_OUTPUT"
  else
    OUTPUT="$($plugin "$ARCH")"
  fi
  end_time=$(get_timestamp)
  
  if ! validate_json "$OUTPUT" "$plugin_basename"; then
    continue
  fi
  
  # Calculate function data hash
  function_hash=$(hash_function_data "$OUTPUT")
  
  # Create the new structure with function name as key
  if [[ $FIRST -eq 1 ]]; then
    FIRST=0
  else
    JSON+=","
  fi
  
  # Strip the outer braces from plugin output to get just the content
  PLUGIN_DATA="${OUTPUT:1:-1}"
  
  # Add plugin data with function name as key, timestamps, and hashes
  JSON+="\"$function_name\": {"
  JSON+="\"data\": {$PLUGIN_DATA},"
  JSON+="\"collection_timestamp\": \"$start_time\","
  JSON+="\"completion_timestamp\": \"$end_time\","
  JSON+="\"plugin_file_hash\": \"$plugin_hash\","
  JSON+="\"function_data_hash\": \"$function_hash\""
  JSON+="}"
done

JSON+="}"

if [[ -n "$OUTPUT_FILE" ]]; then
  echo "$JSON" > "$OUTPUT_FILE"
  echo "System info written to $OUTPUT_FILE"
else
  echo "$JSON"
fi