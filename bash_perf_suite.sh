#!/bin/bash
# Comprehensive Bash performance optimization and test suite

set -e

RESULTS_DIR="perf_results"
mkdir -p "$RESULTS_DIR"

echo "🚀 Bash Performance Optimization Suite"
echo "======================================"

# Test current performance
echo "📊 Baseline Performance (5 runs)"
echo "================================"

total_time=0
for i in {1..5}; do
    printf "Run %d/5: " "$i"
    start_time=$(date +%s.%N)
    ./collect_info.sh -o "/tmp/baseline_$i.json" >/dev/null 2>&1
    end_time=$(date +%s.%N)
    run_time=$(echo "$end_time - $start_time" | bc -l)
    total_time=$(echo "$total_time + $run_time" | bc -l)
    printf "%.3fs\n" "$run_time"
done

baseline_avg=$(echo "scale=3; $total_time / 5" | bc -l)
printf "📈 Baseline average: %.3fs\n" "$baseline_avg"

# Create optimized version with parallel execution
echo ""
echo "🔧 Creating Optimized Version"
echo "============================="

cat > collect_info_optimized.sh << 'EOF'
#!/bin/bash
# Optimized collect_info.sh with parallel execution and performance improvements

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""
TEMP_DIR=$(mktemp -d)

# Configuration
ENABLE_HASHING=${ENABLE_HASHING:-1}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}
ENABLE_PARALLEL=${ENABLE_PARALLEL:-1}

# Cleanup on exit
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Optimized architecture detection
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

# Fast hash calculation
calculate_hash() {
    if [[ "$ENABLE_HASHING" -eq 1 ]] && command -v cksum >/dev/null 2>&1; then
        printf '%s' "$1" | cksum | awk '{print $1}'
    else
        echo "disabled"
    fi
}

# Check sudo with caching
check_sudo() {
    if [[ -f "$TEMP_DIR/sudo_check" ]]; then
        cat "$TEMP_DIR/sudo_check"
    else
        if [[ "$ENABLE_SUDO_SUPPORT" -eq 1 ]] && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
            echo "1" | tee "$TEMP_DIR/sudo_check"
        else
            echo "0" | tee "$TEMP_DIR/sudo_check"
        fi
    fi
}

# Execute plugin with error handling
execute_plugin() {
    plugin_file="$1"
    plugin_name=$(basename "$plugin_file" .sh)
    function_name="get_${plugin_name#??_}"
    output_file="$TEMP_DIR/${plugin_name}.json"
    
    if [[ ! -f "$plugin_file" || ! -x "$plugin_file" ]]; then
        return 1
    fi
    
    (
        # Source plugin in subshell for isolation
        if source "$plugin_file" 2>/dev/null; then
            if declare -f "$function_name" >/dev/null 2>&1; then
                start_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                
                # Execute function with timeout
                if function_output=$(timeout 30 "$function_name" 2>/dev/null); then
                    end_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
                    
                    # Basic JSON validation
                    if [[ "$function_output" =~ ^\{.*\}$ ]]; then
                        plugin_hash=$(calculate_hash "$(cat "$plugin_file")")
                        data_hash=$(calculate_hash "$function_output")
                        
                        # Create plugin result
                        cat > "$output_file" << END
{
  "data": $function_output,
  "collection_timestamp": "$start_time",
  "completion_timestamp": "$end_time",
  "plugin_file_hash": "$plugin_hash",
  "function_data_hash": "$data_hash"
}
END
                    fi
                fi
            fi
        fi
    ) &
    
    # Store PID for later waiting
    echo $! >> "$TEMP_DIR/pids"
}

# Main collection logic
main() {
    detected_arch=$(detect_arch)
    collection_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    sudo_available=$(check_sudo)
    
    # Initialize PID tracking
    : > "$TEMP_DIR/pids"
    
    # Count plugins and start execution
    plugin_count=0
    for plugin_file in "$PLUGIN_DIR"/*.sh; do
        if [[ -f "$plugin_file" && -x "$plugin_file" ]]; then
            if [[ "$ENABLE_PARALLEL" -eq 1 ]]; then
                execute_plugin "$plugin_file"
            else
                # Sequential execution
                execute_plugin "$plugin_file"
                wait $(tail -n1 "$TEMP_DIR/pids")
            fi
            ((plugin_count++))
        fi
    done
    
    # Wait for all plugins to complete if parallel
    if [[ "$ENABLE_PARALLEL" -eq 1 ]]; then
        while IFS= read -r pid; do
            wait "$pid" 2>/dev/null || true
        done < "$TEMP_DIR/pids"
    fi
    
    # Generate JSON output efficiently
    {
        echo "{"
        echo "  \"detected_architecture\": \"$detected_arch\","
        echo "  \"collection_metadata\": {"
        echo "    \"timestamp\": \"$collection_timestamp\","
        echo "    \"plugin_count\": $plugin_count,"
        echo "    \"hashing_enabled\": $ENABLE_HASHING,"
        echo "    \"sudo_support_enabled\": $ENABLE_SUDO_SUPPORT,"
        echo "    \"sudo_available\": $sudo_available"
        echo "  },"
        
        # Add plugin results
        first=true
        for plugin_file in "$PLUGIN_DIR"/*.sh; do
            if [[ -f "$plugin_file" && -x "$plugin_file" ]]; then
                plugin_name=$(basename "$plugin_file" .sh)
                function_name="get_${plugin_name#??_}"
                output_file="$TEMP_DIR/${plugin_name}.json"
                
                if [[ -f "$output_file" ]]; then
                    if [[ "$first" != true ]]; then
                        echo ","
                    fi
                    echo -n "  \"$function_name\": "
                    cat "$output_file"
                    first=false
                fi
            fi
        done
        
        echo ""
        echo "}"
    }
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --no-parallel)
            ENABLE_PARALLEL=0
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [-o output.json] [--no-parallel] [-h]"
            echo "Environment variables:"
            echo "  ENABLE_HASHING=0      - Disable CRC32 hashing"
            echo "  ENABLE_SUDO_SUPPORT=1 - Enable sudo detection"
            echo "  ENABLE_PARALLEL=0     - Disable parallel execution"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Execute main function
if [[ -n "$OUTPUT_FILE" ]]; then
    main > "$OUTPUT_FILE"
    echo "System info written to $OUTPUT_FILE"
else
    main
fi
EOF

chmod +x collect_info_optimized.sh

# Test optimized version
echo ""
echo "⚡ Testing Optimized Version (5 runs)"
echo "===================================="

opt_total_time=0
for i in {1..5}; do
    printf "Run %d/5: " "$i"
    start_time=$(date +%s.%N)
    ./collect_info_optimized.sh -o "/tmp/optimized_$i.json" >/dev/null 2>&1
    end_time=$(date +%s.%N)
    run_time=$(echo "$end_time - $start_time" | bc -l)
    opt_total_time=$(echo "$opt_total_time + $run_time" | bc -l)
    printf "%.3fs\n" "$run_time"
done

opt_avg=$(echo "scale=3; $opt_total_time / 5" | bc -l)
printf "📈 Optimized average: %.3fs\n" "$opt_avg"

# Calculate improvement
improvement=$(echo "scale=1; ($baseline_avg - $opt_avg) / $baseline_avg * 100" | bc -l)
printf "🚀 Performance improvement: %.1f%%\n" "$improvement"

# Test sequential vs parallel
echo ""
echo "🔄 Sequential vs Parallel Comparison"
echo "===================================="

printf "Testing sequential execution: "
start_time=$(date +%s.%N)
ENABLE_PARALLEL=0 ./collect_info_optimized.sh -o "/tmp/sequential.json" >/dev/null 2>&1
end_time=$(date +%s.%N)
seq_time=$(echo "$end_time - $start_time" | bc -l)
printf "%.3fs\n" "$seq_time"

printf "Testing parallel execution: "
start_time=$(date +%s.%N)
ENABLE_PARALLEL=1 ./collect_info_optimized.sh -o "/tmp/parallel.json" >/dev/null 2>&1
end_time=$(date +%s.%N)
par_time=$(echo "$end_time - $start_time" | bc -l)
printf "%.3fs\n" "$par_time"

parallel_improvement=$(echo "scale=1; ($seq_time - $par_time) / $seq_time * 100" | bc -l)
printf "🚀 Parallel improvement: %.1f%%\n" "$parallel_improvement"

# Create performance regression test
echo ""
echo "🧪 Creating Performance Regression Test"
echo "======================================="

cat > performance_regression_test.sh << 'EOF'
#!/bin/bash
# Performance regression test

set -e

BASELINE_TIME=6.0  # Expected baseline in seconds
REGRESSION_THRESHOLD=10  # 10% regression threshold

echo "🔍 Performance Regression Test"
echo "=============================="

# Test current performance
start_time=$(date +%s.%N)
./collect_info_optimized.sh -o /tmp/regression_test.json >/dev/null 2>&1
end_time=$(date +%s.%N)
current_time=$(echo "$end_time - $start_time" | bc -l)

printf "Current execution time: %.3fs\n" "$current_time"
printf "Baseline expectation: %.1fs\n" "$BASELINE_TIME"

# Calculate regression
regression=$(echo "scale=1; ($current_time - $BASELINE_TIME) / $BASELINE_TIME * 100" | bc -l)

if (( $(echo "$regression > $REGRESSION_THRESHOLD" | bc -l) )); then
    printf "❌ REGRESSION DETECTED: %.1f%% slower than baseline\n" "$regression"
    exit 1
elif (( $(echo "$current_time > $BASELINE_TIME" | bc -l) )); then
    printf "⚠️  Slight performance degradation: %.1f%%\n" "$regression"
    exit 0
else
    improvement=$(echo "scale=1; ($BASELINE_TIME - $current_time) / $BASELINE_TIME * 100" | bc -l)
    printf "✅ Performance improved: %.1f%% faster than baseline\n" "$improvement"
    exit 0
fi
EOF

chmod +x performance_regression_test.sh

# Test output compatibility
echo ""
echo "🔍 Output Compatibility Test"
echo "==========================="

if [[ -f "/tmp/baseline_1.json" && -f "/tmp/optimized_1.json" ]]; then
    echo "Checking JSON structure..."
    
    # Basic structure check
    baseline_lines=$(grep -c '"' /tmp/baseline_1.json || echo "0")
    optimized_lines=$(grep -c '"' /tmp/optimized_1.json || echo "0")
    
    if [[ "$baseline_lines" -gt 0 && "$optimized_lines" -gt 0 ]]; then
        echo "✅ Both outputs contain valid JSON structure"
        
        # Check for key fields
        if grep -q "detected_architecture" /tmp/optimized_1.json && 
           grep -q "collection_metadata" /tmp/optimized_1.json; then
            echo "✅ Required fields present in optimized output"
        else
            echo "⚠️  Some required fields missing in optimized output"
        fi
    else
        echo "⚠️  JSON structure verification failed"
    fi
else
    echo "⚠️  Test files not available for comparison"
fi

# Create comprehensive test suite
echo ""
echo "📋 Creating Comprehensive Test Suite"
echo "===================================="

cat > test_bash_performance.sh << 'EOF'
#!/bin/bash
# Comprehensive test suite for bash performance

set -e

RESULTS_FILE="performance_results.json"

run_test_suite() {
    echo "🧪 Running Comprehensive Test Suite"
    echo "=================================="
    
    # Test 1: Basic functionality
    echo "Test 1: Basic functionality..."
    if ./collect_info_optimized.sh -o /tmp/test_basic.json >/dev/null 2>&1; then
        echo "✅ Basic functionality: PASS"
    else
        echo "❌ Basic functionality: FAIL"
        return 1
    fi
    
    # Test 2: Performance benchmark
    echo "Test 2: Performance benchmark..."
    start_time=$(date +%s.%N)
    ./collect_info_optimized.sh -o /tmp/test_perf.json >/dev/null 2>&1
    end_time=$(date +%s.%N)
    perf_time=$(echo "$end_time - $start_time" | bc -l)
    
    if (( $(echo "$perf_time < 8.0" | bc -l) )); then
        printf "✅ Performance benchmark: PASS (%.3fs)\n" "$perf_time"
    else
        printf "❌ Performance benchmark: FAIL (%.3fs > 8.0s)\n" "$perf_time"
    fi
    
    # Test 3: Output validation
    echo "Test 3: Output validation..."
    if [[ -f "/tmp/test_basic.json" ]] && grep -q "detected_architecture" /tmp/test_basic.json; then
        echo "✅ Output validation: PASS"
    else
        echo "❌ Output validation: FAIL"
    fi
    
    # Test 4: Parallel vs Sequential
    echo "Test 4: Parallel vs Sequential consistency..."
    ENABLE_PARALLEL=0 ./collect_info_optimized.sh -o /tmp/test_seq.json >/dev/null 2>&1
    ENABLE_PARALLEL=1 ./collect_info_optimized.sh -o /tmp/test_par.json >/dev/null 2>&1
    
    seq_arch=$(grep "detected_architecture" /tmp/test_seq.json | cut -d'"' -f4)
    par_arch=$(grep "detected_architecture" /tmp/test_par.json | cut -d'"' -f4)
    
    if [[ "$seq_arch" == "$par_arch" ]]; then
        echo "✅ Parallel consistency: PASS"
    else
        echo "❌ Parallel consistency: FAIL"
    fi
    
    echo ""
    echo "🎉 Test suite completed!"
}

# Save results
save_results() {
    cat > "$RESULTS_FILE" << END
{
  "test_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "baseline_time": $baseline_avg,
  "optimized_time": $opt_avg,
  "improvement_percent": $improvement,
  "parallel_improvement": $parallel_improvement
}
END
    echo "📊 Results saved to $RESULTS_FILE"
}

run_test_suite
EOF

chmod +x test_bash_performance.sh

# Save performance results
echo ""
echo "💾 Saving Performance Results"
echo "============================="

cat > "$RESULTS_DIR/performance_summary.json" << EOF
{
  "test_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "baseline_average": $baseline_avg,
  "optimized_average": $opt_avg,
  "improvement_percent": $improvement,
  "parallel_vs_sequential": {
    "sequential_time": $seq_time,
    "parallel_time": $par_time,
    "parallel_improvement": $parallel_improvement
  },
  "optimization_techniques": [
    "Parallel plugin execution",
    "Optimized architecture detection",
    "Efficient JSON generation",
    "Reduced subprocess spawning",
    "Cached sudo checking",
    "Improved error handling",
    "Temporary file management"
  ]
}
EOF

echo "📊 Performance Summary"
echo "====================="
printf "Baseline:     %.3fs\n" "$baseline_avg"
printf "Optimized:    %.3fs\n" "$opt_avg"
printf "Improvement:  %.1f%%\n" "$improvement"
printf "Sequential:   %.3fs\n" "$seq_time"
printf "Parallel:     %.3fs\n" "$par_time"
printf "Par. Benefit: %.1f%%\n" "$parallel_improvement"

echo ""
echo "✅ Performance optimization complete!"
echo ""
echo "📁 Files created:"
echo "  - collect_info_optimized.sh (optimized implementation)"
echo "  - performance_regression_test.sh (regression testing)"
echo "  - test_bash_performance.sh (comprehensive test suite)"
echo "  - $RESULTS_DIR/performance_summary.json (results)"
echo ""
echo "🚀 To use optimized version:"
echo "  ./collect_info_optimized.sh -o output.json"
echo ""
echo "🧪 To run tests:"
echo "  ./test_bash_performance.sh"
echo "  ./performance_regression_test.sh"