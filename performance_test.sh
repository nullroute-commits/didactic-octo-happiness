#!/bin/bash
# Performance test framework

run_performance_test() {
    local test_name="$1"
    local command="$2"
    local iterations="${3:-3}"
    
    echo "Testing $test_name ($iterations runs)..."
    
    local total_time=0
    local min_time=999999
    local max_time=0
    
    for i in $(seq 1 $iterations); do
        local start_time=$(date +%s.%N)
        eval "$command" >/dev/null 2>&1
        local end_time=$(date +%s.%N)
        local run_time=$(echo "$end_time - $start_time" | bc -l)
        
        total_time=$(echo "$total_time + $run_time" | bc -l)
        
        if (( $(echo "$run_time < $min_time" | bc -l) )); then
            min_time=$run_time
        fi
        
        if (( $(echo "$run_time > $max_time" | bc -l) )); then
            max_time=$run_time
        fi
        
        printf "  Run %d: %.3fs\n" "$i" "$run_time"
    done
    
    local avg_time=$(echo "scale=3; $total_time / $iterations" | bc -l)
    
    printf "  Average: %.3fs\n" "$avg_time"
    printf "  Min:     %.3fs\n" "$min_time"
    printf "  Max:     %.3fs\n" "$max_time"
    
    echo "$avg_time"
}

# Test specific components
test_json_generation() {
    echo "Testing JSON generation performance..."
    
    local temp_data='{"test": "data", "numbers": [1, 2, 3], "nested": {"key": "value"}}'
    
    local start_time=$(date +%s.%N)
    for i in {1..1000}; do
        echo "$temp_data" | jq . >/dev/null 2>&1 || echo "$temp_data" >/dev/null
    done
    local end_time=$(date +%s.%N)
    
    local json_time=$(echo "$end_time - $start_time" | bc -l)
    printf "JSON processing (1000 ops): %.3fs\n" "$json_time"
}

test_file_operations() {
    echo "Testing file I/O performance..."
    
    local temp_dir=$(mktemp -d)
    local start_time=$(date +%s.%N)
    
    for i in {1..100}; do
        echo "test data $i" > "$temp_dir/file_$i.txt"
        cat "$temp_dir/file_$i.txt" >/dev/null
    done
    
    local end_time=$(date +%s.%N)
    local file_time=$(echo "$end_time - $start_time" | bc -l)
    
    printf "File I/O (100 ops): %.3fs\n" "$file_time"
    
    rm -rf "$temp_dir"
}

test_command_execution() {
    echo "Testing command execution performance..."
    
    local start_time=$(date +%s.%N)
    for i in {1..50}; do
        uname -m >/dev/null
        date >/dev/null
        whoami >/dev/null
    done
    local end_time=$(date +%s.%N)
    
    local cmd_time=$(echo "$end_time - $start_time" | bc -l)
    printf "Command execution (150 ops): %.3fs\n" "$cmd_time"
}

# Run all tests
echo "🔍 Component Performance Tests"
echo "=============================="
test_json_generation
test_file_operations  
test_command_execution

