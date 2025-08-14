#!/bin/bash
# Package and executable information plugin
# Collects installed packages with versions and config locations,
# and executables with versions and config files

set -e

ARCH="$1"

# Configuration limits
MAX_PACKAGES=${MAX_PACKAGES:-30}
MAX_EXECUTABLES=${MAX_EXECUTABLES:-20}

get_packages_and_executables() {
    local packages_info
    local executables_info
    
    # Detect and collect package information
    packages_info=$(collect_packages)
    
    # Collect executable information
    executables_info=$(collect_executables)
    
    # Output combined JSON
    cat << EOF
{
  "installed_packages": $packages_info,
  "system_executables": $executables_info,
  "architecture": "$ARCH"
}
EOF
}

collect_packages() {
    local packages_json="["
    local first=true
    local count=0
    
    # Debian/Ubuntu (apt/dpkg)
    if command -v dpkg >/dev/null 2>&1; then
        while IFS=$'\t' read -r package version status; do
            # Only include installed packages and respect limit
            if [[ "$status" =~ "install ok installed" ]] && [ $count -lt $MAX_PACKAGES ]; then
                if [ "$first" = false ]; then
                    packages_json+=","
                fi
                first=false
                
                # Escape quotes in version strings
                version_escaped=$(echo "$version" | sed 's/"/\\"/g')
                
                packages_json+='{
      "name": "'$package'",
      "version": "'$version_escaped'",
      "package_manager": "dpkg", 
      "status": "installed",
      "config_files": ["/etc/'$package'.conf", "/etc/'$package'/"]
    }'
                ((count++))
            fi
        done < <(dpkg-query -W -f='${Package}\t${Version}\t${Status}\n' 2>/dev/null | head -$MAX_PACKAGES)
    
    # Red Hat/CentOS/Fedora (rpm)
    elif command -v rpm >/dev/null 2>&1; then
        while read -r line; do
            if [ $count -lt $MAX_PACKAGES ]; then
                local package=$(echo "$line" | awk '{print $1}')
                local version=$(echo "$line" | awk '{print $2}')
                
                if [ "$first" = false ]; then
                    packages_json+=","
                fi
                first=false
                
                # Escape quotes in version strings
                version_escaped=$(echo "$version" | sed 's/"/\\"/g')
                
                packages_json+='{
      "name": "'$package'",
      "version": "'$version_escaped'",
      "package_manager": "rpm",
      "status": "installed",
      "config_files": ["/etc/'$package'.conf", "/etc/'$package'/"]
    }'
                ((count++))
            fi
        done < <(rpm -qa --queryformat '%{NAME} %{VERSION}-%{RELEASE}\n' 2>/dev/null | head -$MAX_PACKAGES)
        
    # macOS (brew)
    elif command -v brew >/dev/null 2>&1; then
        while read -r line; do
            if [ $count -lt $MAX_PACKAGES ]; then
                local package=$(echo "$line" | awk '{print $1}')
                local version=$(echo "$line" | awk '{$1=""; print $0}' | sed 's/^ *//')
                
                if [ "$first" = false ]; then
                    packages_json+=","
                fi
                first=false
                
                packages_json+='{
      "name": "'$package'",
      "version": "'$version'",
      "package_manager": "brew",
      "status": "installed",
      "config_files": ["/usr/local/etc/'$package'", "~/.config/'$package'"]
    }'
                ((count++))
            fi
        done < <(brew list --versions 2>/dev/null | head -$MAX_PACKAGES)
        
    else
        packages_json+='{
      "package_manager": "unknown",
      "message": "No supported package manager found",
      "packages": []
    }'
    fi
    
    packages_json+="]"
    echo "$packages_json"
}

collect_executables() {
    local exec_json="["
    local first=true
    local count=0
    
    # Standard executable paths to search  
    local search_paths="/usr/bin /usr/local/bin /bin"
    
    # Collect executable information efficiently
    for path in $search_paths; do
        if [ -d "$path" ] && [ $count -lt $MAX_EXECUTABLES ]; then
            while read -r executable; do
                if [ $count -ge $MAX_EXECUTABLES ]; then
                    break
                fi
                
                local basename_exec=$(basename "$executable")
                local version="unknown"
                
                # Quick version detection for common tools only
                case "$basename_exec" in
                    bash) version="$(bash --version 2>/dev/null | head -1 | awk '{print $4}' | cut -d'(' -f1 || echo "unknown")" ;;
                    python3) version="$(python3 --version 2>/dev/null | awk '{print $2}' || echo "unknown")" ;;
                    git) version="$(git --version 2>/dev/null | awk '{print $3}' || echo "unknown")" ;;
                    vim) version="$(vim --version 2>/dev/null | head -1 | awk '{print $5}' || echo "unknown")" ;;
                    *) version="unknown" ;;
                esac
                
                if [ "$first" = false ]; then
                    exec_json+=","
                fi
                first=false
                
                exec_json+='{
      "name": "'$basename_exec'",
      "path": "'$executable'",
      "version": "'$version'",
      "config_files": ["/etc/'$basename_exec'.conf", "~/.config/'$basename_exec'", "~/.'$basename_exec'rc"]
    }'
                ((count++))
            done < <(find "$path" -maxdepth 1 -type f -executable 2>/dev/null | head -$((MAX_EXECUTABLES - count)))
        fi
    done
    
    exec_json+="]"
    echo "$exec_json"
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Execute main function
get_packages_and_executables