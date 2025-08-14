#!/bin/bash
# Performance analysis tool for collect_info.sh and plugins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAIN_SCRIPT="$SCRIPT_DIR/collect_info.sh"
PLUGIN_DIR="$SCRIPT_DIR/plugins"
RESULTS_DIR="$SCRIPT_DIR/perf_results"
ITERATIONS=5

mkdir -p "$RESULTS_DIR"

echo "🔍 Bash Performance Analysis Tool"
echo "=================================="

# Function to run performance test
run_perf_test() {
    local test_name="$1"
    local command="$2"
    local output_file="$RESULTS_DIR/perf_${test_name}_$(date +%Y%m%d_%H%M%S).log"
    
    echo "📊 Running test: $test_name"
    echo "Command: $command" > "$output_file"
    echo "===================" >> "$output_file"
    
    local total_time=0
    local total_user=0
    local total_sys=0
    
    for i in $(seq 1 $ITERATIONS); do
        echo "  Iteration $i/$ITERATIONS..."
        
        # Run with time and capture output
        local time_output=$(bash -c "time $command" 2>&1 >/dev/null | grep real)
        echo "Iteration $i: $time_output" >> "$output_file"
        
        # Parse time output
        local real_time=$(echo "$time_output" | grep real | awk '{print $2}' | sed 's/m/:/g' | sed 's/s//g')
        local user_time=$(echo "$time_output" | grep user | awk '{print $2}' | sed 's/m/:/g' | sed 's/s//g')
        local sys_time=$(echo "$time_output" | grep sys | awk '{print $2}' | sed 's/m/:/g' | sed 's/s//g')
        
        # Convert to seconds for averaging
        local real_seconds=$(echo "$real_time" | awk -F: '{if(NF==2) print $1*60+$2; else print $1}')
        local user_seconds=$(echo "$user_time" | awk -F: '{if(NF==2) print $1*60+$2; else print $1}')
        local sys_seconds=$(echo "$sys_time" | awk -F: '{if(NF==2) print $1*60+$2; else print $1}')
        
        total_time=$(echo "$total_time + $real_seconds" | bc -l)
        total_user=$(echo "$total_user + $user_seconds" | bc -l)
        total_sys=$(echo "$total_sys + $sys_seconds" | bc -l)
    done
    
    # Calculate averages
    local avg_time=$(echo "scale=3; $total_time / $ITERATIONS" | bc -l)
    local avg_user=$(echo "scale=3; $total_user / $ITERATIONS" | bc -l)
    local avg_sys=$(echo "scale=3; $total_sys / $ITERATIONS" | bc -l)
    
    echo ""
    echo "Average Results for $test_name:" >> "$output_file"
    echo "Real time: ${avg_time}s" >> "$output_file"
    echo "User time: ${avg_user}s" >> "$output_file"
    echo "Sys time:  ${avg_sys}s" >> "$output_file"
    
    echo "  ✅ Average time: ${avg_time}s (real), ${avg_user}s (user), ${avg_sys}s (sys)"
    echo "  📄 Details saved to: $output_file"
    
    # Return average time for comparison
    echo "$avg_time"
}

# Function to profile individual plugins
profile_plugins() {
    echo ""
    echo "🔧 Profiling individual plugins..."
    echo "================================="
    
    local plugin_times=()
    local plugin_names=()
    
    for plugin in "$PLUGIN_DIR"/*.sh; do
        local plugin_name=$(basename "$plugin" .sh)
        echo "Testing plugin: $plugin_name"
        
        # Create a temporary script that sources the plugin and runs the function
        local temp_script=$(mktemp)
        cat > "$temp_script" << EOF
#!/bin/bash
source "$plugin"
get_${plugin_name#??_}
EOF
        chmod +x "$temp_script"
        
        local avg_time=$(run_perf_test "plugin_${plugin_name}" "$temp_script")
        plugin_times+=("$avg_time")
        plugin_names+=("$plugin_name")
        
        rm -f "$temp_script"
        echo ""
    done
    
    # Sort plugins by execution time
    echo "📈 Plugin Performance Summary:"
    echo "============================="
    for i in "${!plugin_names[@]}"; do
        echo "${plugin_times[$i]}s - ${plugin_names[$i]}"
    done | sort -n -r
}

# Function to test parallel execution
test_parallel_execution() {
    echo ""
    echo "⚡ Testing parallel plugin execution..."
    echo "======================================"
    
    # Create a modified version that runs plugins in parallel
    local parallel_script=$(mktemp)
    
    cat > "$parallel_script" << 'EOF'
#!/bin/bash
set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE="/tmp/parallel_test.json"
ENABLE_HASHING=${ENABLE_HASHING:-1}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}

# Architecture detection (same as original)
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

# Hashing functions (same as original)
calculate_crc32() {
    local input="$1"
    if command -v cksum >/dev/null 2>&1; then
        printf '%s' "$input" | cksum | awk '{print $1}'
    else
        echo "unavailable"
    fi
}

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

hash_function_data() {
    local data="$1"
    if [[ "$ENABLE_HASHING" -eq 1 ]]; then
        calculate_crc32 "$data"
    else
        echo "disabled"
    fi
}

check_privilege_support() {
    local has_sudo=0
    if [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]]; then
        if command -v sudo >/dev/null 2>&1; then
            if sudo -n true 2>/dev/null; then
                has_sudo=1
            fi
        fi
    fi
    echo "$has_sudo"
}

# Main execution with parallel plugins
main() {
    local detected_arch=$(detect_arch)
    local collection_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local plugin_count=0
    local sudo_available=$(check_privilege_support)
    
    echo "{"
    echo "  \"detected_architecture\": \"$detected_arch\","
    
    # Collection metadata
    echo "  \"collection_metadata\": {"
    echo "    \"timestamp\": \"$collection_timestamp\","
    echo "    \"plugin_count\": $(find "$PLUGIN_DIR" -name "*.sh" | wc -l),"
    echo "    \"hashing_enabled\": $ENABLE_HASHING,"
    echo "    \"sudo_support_enabled\": $ENABLE_SUDO_SUPPORT,"
    echo "    \"sudo_available\": $sudo_available"
    echo "  },"
    
    # Run plugins in parallel using background processes
    local plugin_results=()
    local plugin_files=()
    local temp_dir=$(mktemp -d)
    
    for plugin_file in "$PLUGIN_DIR"/*.sh; do
        if [[ -f "$plugin_file" && -x "$plugin_file" ]]; then
            local plugin_name=$(basename "$plugin_file" .sh)
            local function_name="get_${plugin_name#??_}"
            local result_file="$temp_dir/${plugin_name}.json"
            
            plugin_files+=("$plugin_file")
            
            # Run plugin in background
            (
                source "$plugin_file"
                local start_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                local data=$($function_name 2>/dev/null || echo '{}')
                local end_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                local plugin_hash=$(hash_plugin_content "$plugin_file")
                local data_hash=$(hash_function_data "$data")
                
                cat > "$result_file" << END
"$function_name": {
  "data": $data,
  "collection_timestamp": "$start_time",
  "completion_timestamp": "$end_time",
  "plugin_file_hash": "$plugin_hash",
  "function_data_hash": "$data_hash"
}
END
            ) &
        fi
    done
    
    # Wait for all background processes to complete
    wait
    
    # Collect results
    local first=true
    for plugin_file in "${plugin_files[@]}"; do
        local plugin_name=$(basename "$plugin_file" .sh)
        local result_file="$temp_dir/${plugin_name}.json"
        
        if [[ -f "$result_file" ]]; then
            if [[ "$first" != true ]]; then
                echo ","
            fi
            cat "$result_file"
            first=false
        fi
    done
    
    echo ""
    echo "}"
    
    # Cleanup
    rm -rf "$temp_dir"
}

main
EOF
    
    chmod +x "$parallel_script"
    
    local parallel_time=$(run_perf_test "parallel_execution" "$parallel_script")
    
    rm -f "$parallel_script"
    
    echo "  📊 Parallel execution time: ${parallel_time}s"
    
    # Compare with original
    echo "  📊 Testing original (sequential) execution..."
    local sequential_time=$(run_perf_test "sequential_execution" "$MAIN_SCRIPT -o /tmp/sequential_test.json")
    
    echo ""
    echo "🏁 Comparison Results:"
    echo "====================="
    echo "Sequential: ${sequential_time}s"
    echo "Parallel:   ${parallel_time}s"
    
    local improvement=$(echo "scale=2; ($sequential_time - $parallel_time) / $sequential_time * 100" | bc -l)
    if (( $(echo "$parallel_time < $sequential_time" | bc -l) )); then
        echo "✅ Improvement: ${improvement}% faster with parallel execution"
    else
        echo "❌ Parallel execution is slower by $(echo "scale=2; ($parallel_time - $sequential_time) / $sequential_time * 100" | bc -l)%"
    fi
}

# Function to check for common bash performance issues
analyze_code_issues() {
    echo ""
    echo "🔍 Analyzing code for performance issues..."
    echo "=========================================="
    
    local issues_found=0
    
    # Check for external command usage in loops
    echo "Checking for external commands in potential loops..."
    for script in "$MAIN_SCRIPT" "$PLUGIN_DIR"/*.sh; do
        local loop_commands=$(grep -n -E "(for|while).*(\`|\$\()" "$script" 2>/dev/null || true)
        if [[ -n "$loop_commands" ]]; then
            echo "⚠️  Potential issue in $(basename "$script"):"
            echo "$loop_commands"
            ((issues_found++))
        fi
    done
    
    # Check for inefficient command usage
    echo ""
    echo "Checking for inefficient command patterns..."
    for script in "$MAIN_SCRIPT" "$PLUGIN_DIR"/*.sh; do
        # Check for unnecessary cat usage
        local cat_pipes=$(grep -n "cat .* |" "$script" 2>/dev/null || true)
        if [[ -n "$cat_pipes" ]]; then
            echo "⚠️  Unnecessary cat usage in $(basename "$script"):"
            echo "$cat_pipes"
            ((issues_found++))
        fi
        
        # Check for subprocess spawning in command substitution
        local heavy_commands=$(grep -n -E "\$\(.*(grep|awk|sed|cut).*\|.*(grep|awk|sed|cut)" "$script" 2>/dev/null || true)
        if [[ -n "$heavy_commands" ]]; then
            echo "⚠️  Heavy command chaining in $(basename "$script"):"
            echo "$heavy_commands"
            ((issues_found++))
        fi
    done
    
    if [[ $issues_found -eq 0 ]]; then
        echo "✅ No obvious performance issues found"
    else
        echo "⚠️  Found $issues_found potential performance issues"
    fi
}

# Main execution
main() {
    echo "Starting performance analysis with $ITERATIONS iterations per test..."
    echo ""
    
    # Check if bc is available for calculations
    if ! command -v bc >/dev/null 2>&1; then
        echo "❌ 'bc' calculator is required for this script"
        exit 1
    fi
    
    # Baseline test
    echo "📊 Baseline performance test..."
    run_perf_test "baseline" "$MAIN_SCRIPT -o /tmp/baseline_test.json"
    
    # Profile individual plugins
    profile_plugins
    
    # Test parallel execution
    test_parallel_execution
    
    # Analyze code for issues
    analyze_code_issues
    
    echo ""
    echo "🎉 Performance analysis complete!"
    echo "📁 Results saved in: $RESULTS_DIR"
    echo ""
    echo "💡 Recommendations:"
    echo "  1. Consider parallel plugin execution for multi-core systems"
    echo "  2. Cache results of expensive operations"
    echo "  3. Optimize plugins with highest execution times"
    echo "  4. Use bash built-ins instead of external commands where possible"
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi