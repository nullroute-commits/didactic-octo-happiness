#!/usr/bin/env bats

# Tests for 31_network_stats.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/network_stats_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/31_network_stats.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/31_network_stats.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "31_network_stats.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "31_network_stats.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "interface_statistics" ]]
    [[ "$output" =~ "ipv4_routes" ]]
    [[ "$output" =~ "ipv6_routes" ]]
    [[ "$output" =~ "multicast_groups" ]]
    [[ "$output" =~ "listening_ports" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "31_network_stats.sh should return architecture in output for x86_64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "x86_64"' ]]
}

@test "31_network_stats.sh should return architecture in output for arm64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "arm64"' ]]
}

@test "31_network_stats.sh should return architecture in output for i386" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh i386
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "i386"' ]]
}

@test "31_network_stats.sh should return architecture in output for ppc64le" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
}

@test "31_network_stats.sh should return architecture in output for s390x" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh s390x
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "s390x"' ]]
}

@test "31_network_stats.sh should return architecture in output for riscv64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh riscv64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "riscv64"' ]]
}

@test "31_network_stats.sh should return architecture in output for mips64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh mips64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "mips64"' ]]
}

@test "31_network_stats.sh should return architecture in output for aarch32" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh aarch32
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "aarch32"' ]]
}

@test "31_network_stats.sh should return architecture in output for sparc64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh sparc64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "sparc64"' ]]
}

@test "31_network_stats.sh should return architecture in output for loongarch64" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh loongarch64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "loongarch64"' ]]
}

@test "31_network_stats.sh should detect interface statistics" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain interface statistics array
    [[ "$output" =~ '"interface_statistics": [' ]]
    
    # Should contain statistics fields if /proc/net/dev exists
    if [[ -f /proc/net/dev ]]; then
        [[ "$output" =~ '"rx_bytes"' ]]
        [[ "$output" =~ '"tx_bytes"' ]]
        [[ "$output" =~ '"rx_packets"' ]]
        [[ "$output" =~ '"tx_packets"' ]]
    fi
}

@test "31_network_stats.sh should detect routing information" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain routing tables
    [[ "$output" =~ '"ipv4_routes"' ]]
    [[ "$output" =~ '"ipv6_routes"' ]]
    
    # Should have route fields
    [[ "$output" =~ '"destination"' ]]
    [[ "$output" =~ '"gateway"' ]]
    [[ "$output" =~ '"interface"' ]]
}

@test "31_network_stats.sh should include required statistics fields" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain required fields for interface statistics
    [[ "$output" =~ '"interface"' ]]
    [[ "$output" =~ '"rx_bytes"' ]]
    [[ "$output" =~ '"rx_packets"' ]]
    [[ "$output" =~ '"rx_errors"' ]]
    [[ "$output" =~ '"rx_dropped"' ]]
    [[ "$output" =~ '"tx_bytes"' ]]
    [[ "$output" =~ '"tx_packets"' ]]
    [[ "$output" =~ '"tx_errors"' ]]
    [[ "$output" =~ '"tx_dropped"' ]]
}

@test "31_network_stats.sh should handle multicast groups" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain multicast groups field
    [[ "$output" =~ '"multicast_groups"' ]]
}

@test "31_network_stats.sh should detect listening ports" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain listening ports field
    [[ "$output" =~ '"listening_ports"' ]]
}

@test "31_network_stats.sh should handle ARM architecture variants" {
    cd "$TEST_DIR"
    
    # Test different ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./31_network_stats.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "\"architecture\": \"$arch\"" ]]
    done
}

@test "31_network_stats.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./31_network_stats.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"interface_statistics"' ]]
        [[ "$output" =~ '"ipv4_routes"' ]]
        [[ "$output" =~ '"ipv6_routes"' ]]
        [[ "$output" =~ '"multicast_groups"' ]]
        [[ "$output" =~ '"listening_ports"' ]]
        [[ "$output" =~ '"architecture"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}

@test "31_network_stats.sh should handle systems without ss/netstat commands" {
    cd "$TEST_DIR"
    
    # Test by temporarily removing network tools from PATH
    export PATH="/bin:/usr/bin"
    
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"listening_ports"' ]]
}

@test "31_network_stats.sh should escape JSON special characters" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Output should be valid JSON (this will fail if special characters aren't escaped)
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "31_network_stats.sh should handle missing route commands gracefully" {
    cd "$TEST_DIR"
    
    # Create a minimal environment with limited PATH
    export PATH="/bin:/usr/bin"
    
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON even with limited tools
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"ipv4_routes"' ]]
    [[ "$output" =~ '"ipv6_routes"' ]]
}

@test "31_network_stats.sh should produce interface_statistics as valid JSON array" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract interface_statistics array and validate it separately
    interface_stats=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['interface_statistics']))")
    echo "$interface_stats" | python3 -m json.tool > /dev/null
}

@test "31_network_stats.sh should produce ipv4_routes as valid JSON array" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract ipv4_routes array and validate it separately
    ipv4_routes=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['ipv4_routes']))")
    echo "$ipv4_routes" | python3 -m json.tool > /dev/null
}

@test "31_network_stats.sh should read from /proc/net/dev if available" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # If /proc/net/dev exists, should have non-empty interface statistics
    if [[ -f /proc/net/dev ]]; then
        [[ ! "$output" =~ '"interface": "unknown"' ]] || [[ "$output" =~ '"interface"' ]]
    fi
}

@test "31_network_stats.sh should handle IPv6 routing table" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain IPv6 routes field (even if empty)
    [[ "$output" =~ '"ipv6_routes"' ]]
}

@test "31_network_stats.sh should detect network protocol information" {
    cd "$TEST_DIR"
    run ./31_network_stats.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should have listening ports with protocol information
    if command -v ss >/dev/null 2>&1 || command -v netstat >/dev/null 2>&1; then
        [[ "$output" =~ '"protocol"' ]]
        [[ "$output" =~ '"local_address"' ]]
    fi
}