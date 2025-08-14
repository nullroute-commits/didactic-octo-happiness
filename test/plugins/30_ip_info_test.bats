#!/usr/bin/env bats

# Tests for 30_ip_info.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/ip_info_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/30_ip_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/30_ip_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "30_ip_info.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "30_ip_info.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "network_interfaces" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "30_ip_info.sh should return architecture in output for x86_64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "x86_64"' ]]
}

@test "30_ip_info.sh should return architecture in output for arm64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "arm64"' ]]
}

@test "30_ip_info.sh should return architecture in output for i386" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh i386
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "i386"' ]]
}

@test "30_ip_info.sh should return architecture in output for ppc64le" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
}

@test "30_ip_info.sh should return architecture in output for s390x" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh s390x
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "s390x"' ]]
}

@test "30_ip_info.sh should return architecture in output for riscv64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh riscv64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "riscv64"' ]]
}

@test "30_ip_info.sh should return architecture in output for mips64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh mips64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "mips64"' ]]
}

@test "30_ip_info.sh should return architecture in output for aarch32" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh aarch32
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "aarch32"' ]]
}

@test "30_ip_info.sh should return architecture in output for sparc64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh sparc64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "sparc64"' ]]
}

@test "30_ip_info.sh should return architecture in output for loongarch64" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh loongarch64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "loongarch64"' ]]
}

@test "30_ip_info.sh should detect network interfaces" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain network interfaces array
    [[ "$output" =~ '"network_interfaces": [' ]]
    
    # Should contain interface information (not all "unknown")
    if command -v ip >/dev/null 2>&1 || [[ -f /proc/net/dev ]]; then
        [[ ! "$output" =~ '"interface": "unknown"' ]] || [[ "$output" =~ '"interface": "lo"' ]]
    fi
}

@test "30_ip_info.sh should include required interface fields" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain required fields for interfaces
    [[ "$output" =~ '"interface"' ]]
    [[ "$output" =~ '"ipv4_addresses"' ]]
    [[ "$output" =~ '"ipv6_addresses"' ]]
    [[ "$output" =~ '"mac_address"' ]]
    [[ "$output" =~ '"mtu"' ]]
    [[ "$output" =~ '"state"' ]]
}

@test "30_ip_info.sh should detect loopback interface" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should detect loopback interface
    [[ "$output" =~ '"interface": "lo"' ]]
}

@test "30_ip_info.sh should handle ARM architecture variants" {
    cd "$TEST_DIR"
    
    # Test different ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./30_ip_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "\"architecture\": \"$arch\"" ]]
    done
}

@test "30_ip_info.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./30_ip_info.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"network_interfaces"' ]]
        [[ "$output" =~ '"architecture"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}

@test "30_ip_info.sh should handle IPv4 and IPv6 addresses" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should have fields for both IPv4 and IPv6
    [[ "$output" =~ '"ipv4_addresses"' ]]
    [[ "$output" =~ '"ipv6_addresses"' ]]
}

@test "30_ip_info.sh should escape JSON special characters" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Output should be valid JSON (this will fail if special characters aren't escaped)
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "30_ip_info.sh should handle systems without ip command" {
    cd "$TEST_DIR"
    
    # Test by temporarily moving ip command out of PATH (if it exists)
    local ip_path=""
    if command -v ip >/dev/null 2>&1; then
        ip_path=$(which ip)
        export PATH="${PATH//${ip_path%/*}:/}"
    fi
    
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool > /dev/null
    
    # Restore PATH
    if [[ -n "$ip_path" ]]; then
        export PATH="${PATH}:${ip_path%/*}"
    fi
}

@test "30_ip_info.sh should handle missing network tools gracefully" {
    cd "$TEST_DIR"
    
    # Create a minimal environment with limited PATH
    export PATH="/bin:/usr/bin"
    
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON even with limited tools
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"network_interfaces"' ]]
}

@test "30_ip_info.sh should produce network_interfaces as valid JSON array" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract network_interfaces array and validate it separately
    network_interfaces=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['network_interfaces']))")
    echo "$network_interfaces" | python3 -m json.tool > /dev/null
    [[ "$network_interfaces" =~ "interface" ]]
}

@test "30_ip_info.sh should include external IPv4 detection" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should include external_ipv4 field
    [[ "$output" =~ '"external_ipv4"' ]]
    [[ "$output" =~ '"ip"' ]]
    [[ "$output" =~ '"detection_method"' ]]
}

@test "30_ip_info.sh should return valid external IP structure" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # External IP should be object format with ip and detection_method
    [[ "$output" =~ '"external_ipv4": {' ]]
    [[ "$output" =~ '"ip":' ]]
    [[ "$output" =~ '"detection_method":' ]]
    
    # Extract external_ipv4 object and validate it separately
    external_ipv4=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['external_ipv4']))")
    echo "$external_ipv4" | python3 -m json.tool > /dev/null
}

@test "30_ip_info.sh should handle external IP detection gracefully" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should always include external IP info even if detection fails
    [[ "$output" =~ '"external_ipv4"' ]]
    
    # IP field should exist (can be "unknown", "behind-nat", or actual IP)
    [[ "$output" =~ '"ip":' ]]
    
    # Detection method should indicate what was attempted
    [[ "$output" =~ '"detection_method":' ]]
}

@test "30_ip_info.sh should include all required interface fields" {
    cd "$TEST_DIR"
    run ./30_ip_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should include both network interfaces and external IP
    [[ "$output" =~ '"network_interfaces"' ]]
    [[ "$output" =~ '"external_ipv4"' ]]
    [[ "$output" =~ '"architecture"' ]]
    
    # Network interfaces should include required interface fields
    [[ "$output" =~ '"interface"' ]]
    [[ "$output" =~ '"ipv4_addresses"' ]]
    [[ "$output" =~ '"ipv6_addresses"' ]]
    [[ "$output" =~ '"mac_address"' ]]
    [[ "$output" =~ '"mtu"' ]]
    [[ "$output" =~ '"state"' ]]
}
}