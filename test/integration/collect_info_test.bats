#!/usr/bin/env bats

# Tests for collect_info.sh main script

# Test data setup
setup() {
    # Save original state
    export ORIGINAL_PATH="$PATH"
    export TEST_DIR="/tmp/collect_info_test"
    export TEST_PLUGIN_DIR="$TEST_DIR/plugins"
    
    # Create test environment
    mkdir -p "$TEST_PLUGIN_DIR"
    
    # Copy main script to test location
    cp collect_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/collect_info.sh"
}

teardown() {
    # Clean up test environment
    rm -rf "$TEST_DIR"
    export PATH="$ORIGINAL_PATH"
}

@test "collect_info.sh should detect architecture correctly" {
    # Create a simple test plugin
    cat > "$TEST_PLUGIN_DIR/test_plugin.sh" << 'EOF'
#!/bin/bash
echo '{"test_arch": "'$1'"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/test_plugin.sh"
    
    # Run the script and check architecture detection
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    [[ "$output" =~ "detected_architecture" ]]
}

@test "collect_info.sh should discover plugins in plugins directory" {
    # Create multiple test plugins
    cat > "$TEST_PLUGIN_DIR/01_test.sh" << 'EOF'
#!/bin/bash
echo '{"plugin1": "data1"}'
EOF
    
    cat > "$TEST_PLUGIN_DIR/02_test.sh" << 'EOF'
#!/bin/bash
echo '{"plugin2": "data2"}'
EOF
    
    chmod +x "$TEST_PLUGIN_DIR"/*.sh
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    # With new structure, plugin data is nested inside function names
    [[ "$output" =~ "plugin1" ]]
    [[ "$output" =~ "plugin2" ]]
    # Also check for new structure elements
    [[ "$output" =~ "collection_metadata" ]]
    [[ "$output" =~ "timestamp" ]]
}

@test "collect_info.sh should pass architecture as first argument to plugins" {
    # Create a plugin that echoes the received argument
    cat > "$TEST_PLUGIN_DIR/arch_test.sh" << 'EOF'
#!/bin/bash
echo '{"received_arch": "'$1'"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/arch_test.sh"
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    [[ "$output" =~ "received_arch" ]]
}

@test "collect_info.sh should produce valid JSON output" {
    # Create a test plugin
    cat > "$TEST_PLUGIN_DIR/valid_json.sh" << 'EOF'
#!/bin/bash
echo '{"test": "value"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/valid_json.sh"
    
    # Run the script and validate JSON
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    
    # Check if output is valid JSON by parsing it
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "collect_info.sh should merge multiple plugin outputs" {
    # Create multiple plugins with different JSON content
    cat > "$TEST_PLUGIN_DIR/plugin1.sh" << 'EOF'
#!/bin/bash
echo '{"key1": "value1"}'
EOF
    
    cat > "$TEST_PLUGIN_DIR/plugin2.sh" << 'EOF'
#!/bin/bash
echo '{"key2": "value2"}'
EOF
    
    chmod +x "$TEST_PLUGIN_DIR"/*.sh
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    # With new structure, data is nested under function names
    [[ "$output" =~ "key1" ]]
    [[ "$output" =~ "key2" ]]
    [[ "$output" =~ "detected_architecture" ]]
    # Check for new structure elements
    [[ "$output" =~ "collection_metadata" ]]
    [[ "$output" =~ "timestamp" ]]
    [[ "$output" =~ "data" ]]
    [[ "$output" =~ "collection_timestamp" ]]
    [[ "$output" =~ "completion_timestamp" ]]
}

@test "collect_info.sh should handle missing plugins directory" {
    # Remove plugins directory
    rm -rf "$TEST_PLUGIN_DIR"
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 2 ]
    [[ "$output" =~ "Plugin directory" ]]
}

@test "collect_info.sh should handle no executable plugins" {
    # Create non-executable files in plugins directory
    echo "not executable" > "$TEST_PLUGIN_DIR/not_plugin.txt"
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 3 ]
    [[ "$output" =~ "No plugins found" ]]
}

@test "collect_info.sh should handle invalid JSON from plugins" {
    # Create a plugin that outputs invalid JSON
    cat > "$TEST_PLUGIN_DIR/invalid.sh" << 'EOF'
#!/bin/bash
echo 'not json'
EOF
    chmod +x "$TEST_PLUGIN_DIR/invalid.sh"
    
    # Create a valid plugin too
    cat > "$TEST_PLUGIN_DIR/valid.sh" << 'EOF'
#!/bin/bash
echo '{"valid": "json"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/valid.sh"
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    [[ "$output" =~ "valid" ]]
    [[ "$output" =~ "detected_architecture" ]]
}

@test "collect_info.sh should support output to file with -o option" {
    # Create a test plugin
    cat > "$TEST_PLUGIN_DIR/test.sh" << 'EOF'
#!/bin/bash
echo '{"test": "output"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/test.sh"
    
    # Run the script with output file
    cd "$TEST_DIR"
    run ./collect_info.sh -o output.json
    [ "$status" -eq 0 ]
    [[ "$output" =~ "written to output.json" ]]
    [ -f "output.json" ]
    
    # Verify file content is valid JSON
    cat output.json | python3 -m json.tool > /dev/null
}

@test "collect_info.sh should show usage with -h option" {
    cd "$TEST_DIR"
    run ./collect_info.sh -h
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Usage:" ]]
}

# Test supported architectures
@test "collect_info.sh should support all specified architectures" {
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    # Create a plugin that echoes the architecture
    cat > "$TEST_PLUGIN_DIR/arch_test.sh" << 'EOF'
#!/bin/bash
echo '{"arch": "'$1'"}'
EOF
    chmod +x "$TEST_PLUGIN_DIR/arch_test.sh"
    
    # Test each architecture is recognized in the TOP_ARCHS list
    cd "$TEST_DIR"
    for arch in $archs; do
        # Check if architecture is in the supported list
        grep -q "$arch" collect_info.sh
    done
}

# New test for enhanced JSON structure with function names and timestamps
@test "collect_info.sh should use function names as keys and include timestamps" {
    # Create a plugin with a clear function name
    cat > "$TEST_PLUGIN_DIR/get_test_data.sh" << 'EOF'
#!/bin/bash
get_test_data() {
    echo '{"test_field": "test_value"}'
}
get_test_data
EOF
    chmod +x "$TEST_PLUGIN_DIR/get_test_data.sh"
    
    # Run the script
    cd "$TEST_DIR"
    run ./collect_info.sh
    [ "$status" -eq 0 ]
    
    # Validate new JSON structure
    echo "$output" | python3 -m json.tool > /dev/null  # Valid JSON
    
    # Check for function-name based keys
    [[ "$output" =~ "get_test_data" ]]
    
    # Check for nested data structure  
    [[ "$output" =~ '"data"' ]]
    
    # Check for timestamps
    [[ "$output" =~ '"collection_timestamp"' ]]
    [[ "$output" =~ '"completion_timestamp"' ]]
    [[ "$output" =~ '"collection_metadata"' ]]
    
    # Check timestamp format (ISO 8601) - simplified regex
    [[ "$output" =~ [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z ]]
    
    # Check that the test data is nested under the function name - use simpler checks
    [[ "$output" =~ "get_test_data" ]]
    [[ "$output" =~ "test_field" ]]
    # Ensure the structure is correct by checking the order/pattern
    echo "$output" | grep -q '"get_test_data".*"data".*"test_field"'
}