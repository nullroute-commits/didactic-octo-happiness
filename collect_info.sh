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
JSON="{\"detected_architecture\": \"$ARCH\","

FIRST=1
for plugin in "${PLUGINS[@]}"; do
  OUTPUT="$($plugin "$ARCH")"
  plugin_basename=$(basename "$plugin")
  
  if ! validate_json "$OUTPUT" "$plugin_basename"; then
    continue
  fi
  
  FRAGMENT="${OUTPUT:1:-1}"
  if [[ $FIRST -eq 1 ]]; then
    JSON+="$FRAGMENT"
    FIRST=0
  else
    JSON+=", $FRAGMENT"
  fi
done

JSON+="}"

if [[ -n "$OUTPUT_FILE" ]]; then
  echo "$JSON" > "$OUTPUT_FILE"
  echo "System info written to $OUTPUT_FILE"
else
  echo "$JSON"
fi