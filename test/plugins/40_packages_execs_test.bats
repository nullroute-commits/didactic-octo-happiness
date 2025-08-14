#!/usr/bin/env bats

# Tests for 40_packages_execs.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/packages_execs_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/40_packages_execs.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/40_packages_execs.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "40_packages_execs.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "40_packages_execs.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Check if output is valid JSON
    echo "$output" | python3 -m json.tool >/dev/null
    [ $? -eq 0 ]
}

@test "40_packages_execs.sh should include required JSON fields" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Check for required top-level fields
    [[ "$output" =~ "installed_packages" ]]
    [[ "$output" =~ "system_executables" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "40_packages_execs.sh should handle x86_64 architecture" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ "x86_64" ]]
}

@test "40_packages_execs.sh should handle arm64 architecture" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ "arm64" ]]
}

@test "40_packages_execs.sh should include package information" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should include package fields in output
    [[ "$output" =~ "package_manager" ]]
    [[ "$output" =~ "version" ]]
    [[ "$output" =~ "config_files" ]]
}

@test "40_packages_execs.sh should include executable information" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should include executable fields
    [[ "$output" =~ "name" ]]
    [[ "$output" =~ "path" ]]
    [[ "$output" =~ "config_files" ]]
}

@test "40_packages_execs.sh should respect MAX_PACKAGES environment variable" {
    cd "$TEST_DIR"
    
    # Test with very low limit
    export MAX_PACKAGES=5
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Count package entries (rough check)
    package_count=$(echo "$output" | grep -o '"name":' | wc -l || echo "0")
    # Should be reasonable number (not too many due to limit)
    [ "$package_count" -lt 50 ]
}

@test "40_packages_execs.sh should respect MAX_EXECUTABLES environment variable" {
    cd "$TEST_DIR"
    
    # Test with very low limit
    export MAX_EXECUTABLES=3
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should complete successfully even with low limits
    [[ "$output" =~ "system_executables" ]]
}

@test "40_packages_execs.sh should detect package manager" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should detect at least one known package manager on Ubuntu systems
    if command -v dpkg >/dev/null 2>&1; then
        [[ "$output" =~ "dpkg" ]]
    elif command -v rpm >/dev/null 2>&1; then
        [[ "$output" =~ "rpm" ]]
    elif command -v brew >/dev/null 2>&1; then
        [[ "$output" =~ "brew" ]]
    else
        [[ "$output" =~ "unknown" ]]
    fi
}

@test "40_packages_execs.sh should handle missing package managers gracefully" {
    cd "$TEST_DIR"
    
    # Save original PATH
    local original_path="$PATH"
    
    # Create a minimal environment without package managers but with essential commands
    mkdir -p /tmp/minimal_path
    export PATH="/tmp/minimal_path:/bin:/usr/bin"
    
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool >/dev/null
    [ $? -eq 0 ]
    
    # Restore original PATH
    export PATH="$original_path"
}

@test "40_packages_execs.sh JSON structure should be consistent" {
    cd "$TEST_DIR"
    run ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
    
    # Parse JSON and verify structure
    echo "$output" | python3 -c "
import json, sys
data = json.load(sys.stdin)
assert 'installed_packages' in data
assert 'system_executables' in data
assert 'architecture' in data
assert isinstance(data['installed_packages'], list)
assert isinstance(data['system_executables'], list)
print('JSON structure valid')
"
}

@test "40_packages_execs.sh should complete within reasonable time" {
    cd "$TEST_DIR"
    
    # Set low limits for faster execution
    export MAX_PACKAGES=10
    export MAX_EXECUTABLES=10
    
    # Should complete within 15 seconds
    run timeout 15 ./40_packages_execs.sh x86_64
    [ "$status" -eq 0 ]
}