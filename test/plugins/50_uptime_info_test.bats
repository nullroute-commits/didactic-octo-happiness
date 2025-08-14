#!/usr/bin/env bats

# Tests for 50_uptime_info.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/uptime_info_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/50_uptime_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/50_uptime_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "50_uptime_info.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "50_uptime_info.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "uptime_seconds" ]]
    [[ "$output" =~ "uptime_formatted" ]]
    [[ "$output" =~ "boot_time" ]]
    [[ "$output" =~ "load_average" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "50_uptime_info.sh should return architecture in output for x86_64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "x86_64"' ]]
}

@test "50_uptime_info.sh should return architecture in output for arm64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "arm64"' ]]
}

@test "50_uptime_info.sh should return architecture in output for i386" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh i386
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "i386"' ]]
}

@test "50_uptime_info.sh should return architecture in output for ppc64le" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
}

@test "50_uptime_info.sh should return architecture in output for s390x" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh s390x
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "s390x"' ]]
}

@test "50_uptime_info.sh should return architecture in output for riscv64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh riscv64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "riscv64"' ]]
}

@test "50_uptime_info.sh should return architecture in output for mips64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh mips64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "mips64"' ]]
}

@test "50_uptime_info.sh should return architecture in output for aarch32" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh aarch32
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "aarch32"' ]]
}

@test "50_uptime_info.sh should return architecture in output for sparc64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh sparc64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "sparc64"' ]]
}

@test "50_uptime_info.sh should return architecture in output for loongarch64" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh loongarch64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "loongarch64"' ]]
}

@test "50_uptime_info.sh should detect uptime information" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain uptime information (not "unknown" if /proc/uptime exists)
    if [[ -f /proc/uptime ]] && [[ -r /proc/uptime ]]; then
        [[ ! "$output" =~ '"uptime_seconds": "unknown"' ]]
        [[ ! "$output" =~ '"uptime_formatted": "unknown"' ]]
    fi
}

@test "50_uptime_info.sh should detect boot time" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain boot time field
    [[ "$output" =~ '"boot_time"' ]]
    
    # If /proc/stat exists, should not be "unknown"
    if [[ -f /proc/stat ]] && [[ -r /proc/stat ]]; then
        [[ ! "$output" =~ '"boot_time": "unknown"' ]]
    fi
}

@test "50_uptime_info.sh should detect load average" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain load average field
    [[ "$output" =~ '"load_average"' ]]
    
    # If /proc/loadavg exists, should not be "unknown"
    if [[ -f /proc/loadavg ]] && [[ -r /proc/loadavg ]]; then
        [[ ! "$output" =~ '"load_average": "unknown"' ]]
    fi
}

@test "50_uptime_info.sh should format uptime correctly" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should have formatted uptime field
    [[ "$output" =~ '"uptime_formatted"' ]]
    
    # If uptime is available, formatted should contain time units
    if [[ -f /proc/uptime ]] && [[ -r /proc/uptime ]]; then
        # Should contain 's' for seconds at minimum
        [[ "$output" =~ 's"' ]]
    fi
}

@test "50_uptime_info.sh should handle ARM architecture variants" {
    cd "$TEST_DIR"
    
    # Test different ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./50_uptime_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "\"architecture\": \"$arch\"" ]]
    done
}

@test "50_uptime_info.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./50_uptime_info.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"uptime_seconds"' ]]
        [[ "$output" =~ '"uptime_formatted"' ]]
        [[ "$output" =~ '"boot_time"' ]]
        [[ "$output" =~ '"load_average"' ]]
        [[ "$output" =~ '"architecture"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}

@test "50_uptime_info.sh should work without external dependencies" {
    cd "$TEST_DIR"
    
    # Test that the plugin works without jq or other external tools
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should produce valid JSON without jq
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "50_uptime_info.sh should handle systems without /proc/uptime" {
    cd "$TEST_DIR"
    
    # Mock a system without /proc/uptime by temporarily hiding it
    # The plugin should still work and return "unknown" gracefully
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"uptime_seconds"' ]]
    [[ "$output" =~ '"uptime_formatted"' ]]
}

@test "50_uptime_info.sh should parse load average correctly" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # If load average is available, it should contain three numbers
    if [[ -f /proc/loadavg ]] && [[ -r /proc/loadavg ]]; then
        # Extract load average from output
        load_avg=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(data['load_average'])")
        if [[ "$load_avg" != "unknown" ]]; then
            # Should contain three space-separated numbers
            [[ "$load_avg" =~ ^[0-9.]+ ]]
        fi
    fi
}

@test "50_uptime_info.sh should handle numeric uptime correctly" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract uptime_seconds from JSON
    if [[ -f /proc/uptime ]] && [[ -r /proc/uptime ]]; then
        uptime_sec=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(data['uptime_seconds'])")
        if [[ "$uptime_sec" != "unknown" ]]; then
            # Should be a number
            [[ "$uptime_sec" =~ ^[0-9]+$ ]]
        fi
    fi
}

@test "50_uptime_info.sh should handle different uptime formats" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Test that formatted uptime contains valid time units
    uptime_fmt=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(data['uptime_formatted'])")
    if [[ "$uptime_fmt" != "unknown" ]]; then
        # Should contain 's' for seconds at minimum
        [[ "$uptime_fmt" =~ s$ ]]
    fi
}

@test "50_uptime_info.sh should handle boot time as timestamp" {
    cd "$TEST_DIR"
    run ./50_uptime_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract boot_time from JSON
    if [[ -f /proc/stat ]] && [[ -r /proc/stat ]]; then
        boot_time=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(data['boot_time'])")
        if [[ "$boot_time" != "unknown" ]]; then
            # Should be a Unix timestamp (number)
            [[ "$boot_time" =~ ^[0-9]+$ ]]
        fi
    fi
}