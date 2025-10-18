#!/bin/bash
# Documentation Validation Script
# This script verifies that the documented commands and procedures in README.md
# and Installation Guide actually work as described.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to print test result
print_result() {
    local test_name="$1"
    local result="$2"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗${NC} $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

echo "======================================"
echo "  Documentation Validation Suite"
echo "======================================"
echo ""

# Test 1: Check if required files exist
echo -e "${BLUE}Checking Required Files...${NC}"
if [ -f "README.md" ]; then
    print_result "README.md exists" "PASS"
else
    print_result "README.md exists" "FAIL"
fi

if [ -f "wiki/Installation-Guide.md" ]; then
    print_result "Installation-Guide.md exists" "PASS"
else
    print_result "Installation-Guide.md exists" "FAIL"
fi

if [ -f "collect_info.sh" ]; then
    print_result "collect_info.sh exists" "PASS"
else
    print_result "collect_info.sh exists" "FAIL"
fi

if [ -f ".env.template" ]; then
    print_result ".env.template exists" "PASS"
else
    print_result ".env.template exists" "FAIL"
fi

if [ -f "docker-compose.yml" ]; then
    print_result "docker-compose.yml exists" "PASS"
else
    print_result "docker-compose.yml exists" "FAIL"
fi

if [ -f "Cargo.toml" ]; then
    print_result "Cargo.toml exists" "PASS"
else
    print_result "Cargo.toml exists" "FAIL"
fi

echo ""

# Test 2: Check if collect_info.sh works
echo -e "${BLUE}Testing System Information Collection...${NC}"

if timeout 60 ./collect_info.sh > /tmp/collect_test.json 2>&1; then
    print_result "collect_info.sh executes successfully" "PASS"
    
    # Validate JSON output
    if command -v jq >/dev/null 2>&1; then
        if jq empty /tmp/collect_test.json 2>/dev/null; then
            print_result "collect_info.sh produces valid JSON" "PASS"
        else
            print_result "collect_info.sh produces valid JSON" "FAIL"
        fi
        
        # Check for expected fields
        if jq -e '.detected_architecture' /tmp/collect_test.json >/dev/null 2>&1; then
            print_result "JSON contains detected_architecture field" "PASS"
        else
            print_result "JSON contains detected_architecture field" "FAIL"
        fi
        
        if jq -e '.collection_metadata' /tmp/collect_test.json >/dev/null 2>&1; then
            print_result "JSON contains collection_metadata field" "PASS"
        else
            print_result "JSON contains collection_metadata field" "FAIL"
        fi
    fi
else
    print_result "collect_info.sh executes successfully" "FAIL"
fi

# Test with output file
if timeout 60 ./collect_info.sh -o /tmp/collect_output_test.json >/dev/null 2>&1; then
    print_result "collect_info.sh -o flag works" "PASS"
    rm -f /tmp/collect_output_test.json
else
    print_result "collect_info.sh -o flag works" "FAIL"
fi

# Test with hashing enabled
if timeout 90 bash -c 'ENABLE_HASHING=1 ./collect_info.sh -o /tmp/collect_hash_test.json >/dev/null 2>&1'; then
    print_result "collect_info.sh with ENABLE_HASHING=1 works" "PASS"
    rm -f /tmp/collect_hash_test.json
else
    print_result "collect_info.sh with ENABLE_HASHING=1 works" "FAIL"
fi

# Test with hashing disabled
if timeout 90 bash -c 'ENABLE_HASHING=0 ./collect_info.sh > /tmp/collect_nohash_test.json 2>&1'; then
    print_result "collect_info.sh with ENABLE_HASHING=0 works" "PASS"
    rm -f /tmp/collect_nohash_test.json
else
    print_result "collect_info.sh with ENABLE_HASHING=0 works" "FAIL"
fi

echo ""

# Test 3: Check if Rust builds
echo -e "${BLUE}Testing Rust Build Process...${NC}"

if command -v cargo >/dev/null 2>&1; then
    print_result "Cargo is installed" "PASS"
    
    # Check if already built
    if [ -f "target/debug/web_server" ] && [ -f "target/debug/ci_runner" ]; then
        print_result "Rust binaries already built" "PASS"
    else
        echo "Building Rust project (this may take a few minutes)..."
        if timeout 600 cargo build >/dev/null 2>&1; then
            print_result "cargo build succeeds" "PASS"
        else
            print_result "cargo build succeeds" "FAIL"
        fi
    fi
    
    # Check if binaries exist
    if [ -f "target/debug/web_server" ]; then
        print_result "web_server binary exists" "PASS"
    else
        print_result "web_server binary exists" "FAIL"
    fi
    
    if [ -f "target/debug/ci_runner" ]; then
        print_result "ci_runner binary exists" "PASS"
    else
        print_result "ci_runner binary exists" "FAIL"
    fi
else
    print_result "Cargo is installed" "FAIL"
fi

echo ""

# Test 4: Check if binary commands match documentation
echo -e "${BLUE}Testing Binary Commands...${NC}"

if [ -f "target/debug/web_server" ]; then
    # Test web_server --help
    if timeout 5 ./target/debug/web_server --help >/dev/null 2>&1; then
        print_result "web_server --help works" "PASS"
    else
        print_result "web_server --help works" "FAIL"
    fi
    
    # Check if serve subcommand exists
    if timeout 5 ./target/debug/web_server --help 2>&1 | grep -q "serve"; then
        print_result "web_server has serve subcommand" "PASS"
    else
        print_result "web_server has serve subcommand" "FAIL"
    fi
fi

if [ -f "target/debug/ci_runner" ]; then
    # Test ci_runner --help
    if timeout 5 ./target/debug/ci_runner --help >/dev/null 2>&1; then
        print_result "ci_runner --help works" "PASS"
    else
        print_result "ci_runner --help works" "FAIL"
    fi
    
    # Check if run subcommand exists
    if timeout 5 ./target/debug/ci_runner --help 2>&1 | grep -q "run"; then
        print_result "ci_runner has run subcommand" "PASS"
    else
        print_result "ci_runner has run subcommand" "FAIL"
    fi
    
    # Check if validate subcommand exists
    if timeout 5 ./target/debug/ci_runner --help 2>&1 | grep -q "validate"; then
        print_result "ci_runner has validate subcommand" "PASS"
    else
        print_result "ci_runner has validate subcommand" "FAIL"
    fi
    
    # Check if run subcommand has --profile option
    if timeout 5 ./target/debug/ci_runner run --help 2>&1 | grep -q "profile"; then
        print_result "ci_runner run has --profile option" "PASS"
    else
        print_result "ci_runner run has --profile option" "FAIL"
    fi
fi

echo ""

# Test 5: Check plugins
echo -e "${BLUE}Testing Plugins...${NC}"

if [ -d "plugins" ]; then
    print_result "plugins directory exists" "PASS"
    
    plugin_count=$(ls -1 plugins/*.sh 2>/dev/null | wc -l)
    if [ "$plugin_count" -gt 0 ]; then
        print_result "Plugin scripts exist ($plugin_count found)" "PASS"
    else
        print_result "Plugin scripts exist" "FAIL"
    fi
else
    print_result "plugins directory exists" "FAIL"
fi

echo ""

# Test 6: Check other scripts
echo -e "${BLUE}Testing Other Scripts...${NC}"

if [ -f "comprehensive_test_suite.sh" ]; then
    print_result "comprehensive_test_suite.sh exists" "PASS"
else
    print_result "comprehensive_test_suite.sh exists" "FAIL"
fi

if [ -f "quick_start.sh" ]; then
    print_result "quick_start.sh exists" "PASS"
else
    print_result "quick_start.sh exists" "FAIL"
fi

echo ""

# Test 7: Check Docker/Podman availability (informational)
echo -e "${BLUE}Checking Container Runtimes (Informational)...${NC}"

if command -v docker >/dev/null 2>&1; then
    docker_version=$(docker --version 2>/dev/null || echo "unknown")
    print_result "Docker is available: $docker_version" "PASS"
else
    echo -e "${YELLOW}ℹ${NC} Docker is not available (optional)"
fi

if command -v podman >/dev/null 2>&1; then
    podman_version=$(podman --version 2>/dev/null || echo "unknown")
    print_result "Podman is available: $podman_version" "PASS"
else
    echo -e "${YELLOW}ℹ${NC} Podman is not available (optional)"
fi

if command -v docker-compose >/dev/null 2>&1; then
    compose_version=$(docker-compose --version 2>/dev/null || echo "unknown")
    print_result "docker-compose is available: $compose_version" "PASS"
elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    compose_version=$(docker compose version 2>/dev/null || echo "unknown")
    print_result "docker compose plugin is available: $compose_version" "PASS"
else
    echo -e "${YELLOW}ℹ${NC} Docker Compose is not available (optional)"
fi

echo ""

# Summary
echo "======================================"
echo "  Validation Summary"
echo "======================================"
echo -e "Total Tests:  ${TESTS_TOTAL}"
echo -e "${GREEN}Passed:       ${TESTS_PASSED}${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed:       ${TESTS_FAILED}${NC}"
else
    echo -e "Failed:       ${TESTS_FAILED}"
fi

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ All documentation validation tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Some documentation validation tests failed.${NC}"
    exit 1
fi
