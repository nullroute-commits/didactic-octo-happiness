#!/bin/bash
# Main orchestrator for plugin-based system info collection (JSON output)
# Supports top 10 architectures per 2024 Q4 market reports.

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""
PLUGINS=()

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

usage() {
  echo "Usage: $0 [-o output.json]"
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

# Start with basic structure and collection metadata
JSON="{\"detected_architecture\": \"$ARCH\","
JSON+="\"collection_metadata\": {"
JSON+="\"timestamp\": \"$COLLECTION_START_TIME\","
JSON+="\"plugin_count\": ${#PLUGINS[@]}"
JSON+="},"

FIRST=1
for plugin in "${PLUGINS[@]}"; do
  plugin_basename=$(basename "$plugin")
  function_name=$(extract_function_name "$plugin")
  
  # Capture execution time and output
  start_time=$(get_timestamp)
  OUTPUT="$($plugin "$ARCH")"
  end_time=$(get_timestamp)
  
  if ! validate_json "$OUTPUT" "$plugin_basename"; then
    continue
  fi
  
  # Create the new structure with function name as key
  if [[ $FIRST -eq 1 ]]; then
    FIRST=0
  else
    JSON+=","
  fi
  
  # Strip the outer braces from plugin output to get just the content
  PLUGIN_DATA="${OUTPUT:1:-1}"
  
  # Add plugin data with function name as key and timestamp
  JSON+="\"$function_name\": {"
  JSON+="\"data\": {$PLUGIN_DATA},"
  JSON+="\"collection_timestamp\": \"$start_time\","
  JSON+="\"completion_timestamp\": \"$end_time\""
  JSON+="}"
done

JSON+="}"

if [[ -n "$OUTPUT_FILE" ]]; then
  echo "$JSON" > "$OUTPUT_FILE"
  echo "System info written to $OUTPUT_FILE"
else
  echo "$JSON"
fi