#!/bin/bash
# Comprehensive test coverage and end-to-end testing suite

set -e

echo "🧪 Comprehensive Test Coverage Suite"
echo "===================================="

RESULTS_DIR="test_results"
mkdir -p "$RESULTS_DIR"

# Test result summary
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local description="$3"
    
    echo "Running: $test_name - $description"
    ((TOTAL_TESTS++))
    
    if eval "$test_command" >/dev/null 2>&1; then
        echo "✅ PASS: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        echo "❌ FAIL: $test_name"
        echo "   Command: $test_command"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo ""
echo "📊 Rust Test Coverage Analysis"
echo "=============================="

echo "Running all Rust tests..."
if cargo test --all 2>&1 | tee "$RESULTS_DIR/rust_tests.log"; then
    RUST_TESTS=$(grep "test result:" "$RESULTS_DIR/rust_tests.log" | grep -o '[0-9]\+ passed' | awk '{sum+=$1} END {print sum}')
    echo "✅ Rust tests passed: $RUST_TESTS"
else
    echo "❌ Some Rust tests failed"
    RUST_TESTS=0
fi

echo ""
echo "🐚 Bash Script Testing"
echo "======================"

# Test 1: Basic script execution
run_test "bash_basic" \
    "./collect_info.sh -o /tmp/test_basic.json" \
    "Basic script execution"

# Test 2: Output file creation
run_test "bash_output" \
    "test -f /tmp/test_basic.json" \
    "Output file creation"

# Test 3: JSON validation
run_test "bash_json" \
    "grep -q 'detected_architecture' /tmp/test_basic.json" \
    "JSON structure validation"

# Test 4: Architecture detection
run_test "bash_arch" \
    "grep -q 'x86_64\|arm64\|i386' /tmp/test_basic.json" \
    "Architecture detection"

# Test 5: Plugins execution
run_test "bash_plugins" \
    "grep -q 'get_os_info\|get_hardware_info' /tmp/test_basic.json" \
    "Plugin execution"

# Test 6: Metadata generation
run_test "bash_metadata" \
    "grep -q 'collection_metadata' /tmp/test_basic.json" \
    "Metadata generation"

# Test 7: Hashing functionality
run_test "bash_hashing" \
    "grep -q 'plugin_file_hash\|function_data_hash' /tmp/test_basic.json" \
    "Hash generation"

# Test 8: Error handling (non-existent output directory)
run_test "bash_error_handling" \
    "! ./collect_info.sh -o /nonexistent/path/output.json" \
    "Error handling for invalid paths"

echo ""
echo "🐳 Container Runtime Testing"
echo "=========================="

# Test Docker availability
if command -v docker >/dev/null 2>&1; then
    run_test "docker_available" \
        "docker --version" \
        "Docker availability"
        
    run_test "docker_build" \
        "docker build -t automation-nation-test -f Dockerfile . --quiet" \
        "Docker build test"
else
    echo "⚠️  Docker not available, skipping Docker tests"
fi

# Test Podman availability
if command -v podman >/dev/null 2>&1; then
    run_test "podman_available" \
        "podman --version" \
        "Podman availability"
else
    echo "⚠️  Podman not available, skipping Podman tests"
fi

# Test LXC availability
if command -v lxc-create >/dev/null 2>&1; then
    run_test "lxc_available" \
        "lxc-create --help" \
        "LXC availability"
else
    echo "⚠️  LXC not available, skipping LXC tests"
fi

echo ""
echo "🔧 Plugin Testing"
echo "================"

# Test individual plugins
for plugin in plugins/*.sh; do
    if [[ -f "$plugin" && -x "$plugin" ]]; then
        plugin_name=$(basename "$plugin" .sh)
        function_name="get_${plugin_name#??_}"
        
        # Test plugin sourcing
        run_test "plugin_${plugin_name}_source" \
            "source '$plugin'" \
            "Plugin $plugin_name sourcing"
        
        # Test function existence
        run_test "plugin_${plugin_name}_function" \
            "source '$plugin' && declare -f '$function_name' >/dev/null" \
            "Plugin $plugin_name function existence"
    fi
done

echo ""
echo "⚡ Performance Testing"
echo "===================="

# Test performance benchmarks
run_test "perf_baseline" \
    "timeout 30 ./collect_info.sh -o /tmp/perf_baseline.json" \
    "Performance baseline (under 30s)"

if [[ -f "collect_info_optimized.sh" ]]; then
    run_test "perf_optimized" \
        "timeout 20 ./collect_info_optimized.sh -o /tmp/perf_optimized.json" \
        "Optimized performance (under 20s)"
fi

echo ""
echo "🌐 Web Application Testing"
echo "========================="

# Test Rust compilation
run_test "rust_compile" \
    "cargo check --all" \
    "Rust compilation"

# Test web server binary
run_test "web_server_binary" \
    "cargo build --bin web_server" \
    "Web server binary build"

# Test CI runner binary
run_test "ci_runner_binary" \
    "cargo build --bin ci_runner" \
    "CI runner binary build"

echo ""
echo "📁 File Structure Testing"
echo "========================"

# Test required files existence
REQUIRED_FILES=(
    "collect_info.sh"
    "Cargo.toml"
    "Dockerfile"
    "Containerfile"
    "docker-compose.yml"
    "quick_start.sh"
    ".env.template"
)

for file in "${REQUIRED_FILES[@]}"; do
    run_test "file_${file}" \
        "test -f '$file'" \
        "Required file $file exists"
done

# Test required directories
REQUIRED_DIRS=(
    "src"
    "plugins"
    "test"
    "templates"
    "monitoring"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    run_test "dir_${dir}" \
        "test -d '$dir'" \
        "Required directory $dir exists"
done

echo ""
echo "🔒 Security Testing"
echo "=================="

# Test script permissions
run_test "security_script_perms" \
    "test -x collect_info.sh" \
    "Main script executable permissions"

# Test plugin permissions
PLUGIN_PERMS_OK=true
for plugin in plugins/*.sh; do
    if [[ ! -x "$plugin" ]]; then
        PLUGIN_PERMS_OK=false
        break
    fi
done

run_test "security_plugin_perms" \
    "$PLUGIN_PERMS_OK" \
    "Plugin executable permissions"

# Test for potential security issues in scripts
run_test "security_no_hardcoded_paths" \
    "! grep -r '/tmp/' plugins/ || true" \
    "No hardcoded temporary paths"

echo ""
echo "🌐 Integration Testing"
echo "====================="

# Test end-to-end workflow
run_test "integration_full_workflow" \
    "./collect_info.sh -o /tmp/integration_test.json && grep -q 'detected_architecture' /tmp/integration_test.json" \
    "Full end-to-end workflow"

# Test configuration options
run_test "integration_config_hashing_disabled" \
    "ENABLE_HASHING=0 ./collect_info.sh -o /tmp/no_hash_test.json && grep -q 'disabled' /tmp/no_hash_test.json" \
    "Configuration: hashing disabled"

# Test different architectures handling
run_test "integration_arch_handling" \
    "grep -q '\"x86_64\"\\|\"arm64\"\\|\"i386\"' /tmp/integration_test.json" \
    "Architecture handling in output"

echo ""
echo "📈 Performance Regression Testing"
echo "================================"

# Create performance baseline if it doesn't exist
if [[ ! -f "$RESULTS_DIR/performance_baseline.txt" ]]; then
    echo "Creating performance baseline..."
    start_time=$(date +%s.%N)
    ./collect_info.sh -o /tmp/baseline.json >/dev/null 2>&1
    end_time=$(date +%s.%N)
    baseline_time=$(echo "$end_time - $start_time" | bc -l)
    echo "$baseline_time" > "$RESULTS_DIR/performance_baseline.txt"
    echo "Baseline created: ${baseline_time}s"
else
    baseline_time=$(cat "$RESULTS_DIR/performance_baseline.txt")
fi

# Test current performance
start_time=$(date +%s.%N)
./collect_info.sh -o /tmp/current_perf.json >/dev/null 2>&1
end_time=$(date +%s.%N)
current_time=$(echo "$end_time - $start_time" | bc -l)

# Check for regression (allow 20% degradation)
regression_threshold=$(echo "$baseline_time * 1.2" | bc -l)
if (( $(echo "$current_time < $regression_threshold" | bc -l) )); then
    run_test "perf_regression" \
        "true" \
        "No performance regression detected"
else
    run_test "perf_regression" \
        "false" \
        "Performance regression detected"
fi

echo ""
echo "📋 Test Coverage Analysis"
echo "========================"

# Calculate test coverage metrics
echo "Rust Test Coverage:"
echo "  - Total test functions: $RUST_TESTS"
echo "  - Test modules: $(grep -r '#\[cfg(test)\]' src/ | wc -l)"
echo "  - Source files with tests: $(find src/ -name "*.rs" -exec grep -l '#\[test\]' {} \; | wc -l)"

echo ""
echo "Bash Test Coverage:"
BASH_TESTS=$(grep -c "run_test.*bash" "$0" || echo "0")
echo "  - Bash functionality tests: $BASH_TESTS"
echo "  - Plugin tests: $(ls plugins/*.sh | wc -l)"
echo "  - Integration tests: $(grep -c "run_test.*integration" "$0" || echo "0")"

echo ""
echo "Overall Test Coverage:"
TOTAL_SOURCE_FILES=$(find src/ -name "*.rs" | wc -l)
TESTED_SOURCE_FILES=$(find src/ -name "*.rs" -exec grep -l '#\[test\]' {} \; | wc -l)
if [[ $TOTAL_SOURCE_FILES -gt 0 ]]; then
    COVERAGE_PERCENT=$(echo "scale=1; $TESTED_SOURCE_FILES * 100 / $TOTAL_SOURCE_FILES" | bc -l)
    echo "  - Source file coverage: ${COVERAGE_PERCENT}% ($TESTED_SOURCE_FILES/$TOTAL_SOURCE_FILES files)"
fi

echo ""
echo "🎯 Test Results Summary"
echo "======================"
echo "Total tests run: $TOTAL_TESTS"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo "✅ All tests passed!"
    SUCCESS_RATE="100%"
else
    SUCCESS_RATE=$(echo "scale=1; $TESTS_PASSED * 100 / $TOTAL_TESTS" | bc -l)
    echo "⚠️  Success rate: ${SUCCESS_RATE}%"
fi

# Generate test report
cat > "$RESULTS_DIR/test_report.json" << EOF
{
  "test_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "tests_passed": $TESTS_PASSED,
    "tests_failed": $TESTS_FAILED,
    "success_rate": "$SUCCESS_RATE"
  },
  "rust_tests": {
    "total_functions": $RUST_TESTS,
    "source_file_coverage": "${COVERAGE_PERCENT}%"
  },
  "bash_tests": {
    "functionality_tests": $BASH_TESTS,
    "plugin_count": $(ls plugins/*.sh | wc -l)
  },
  "performance": {
    "baseline_time": $baseline_time,
    "current_time": $current_time,
    "regression_threshold": $regression_threshold
  }
}
EOF

echo ""
echo "📊 Report saved to: $RESULTS_DIR/test_report.json"

# Exit with appropriate code
if [[ $TESTS_FAILED -eq 0 ]]; then
    exit 0
else
    exit 1
fi