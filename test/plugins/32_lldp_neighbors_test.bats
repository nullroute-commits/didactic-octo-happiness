#!/usr/bin/env bats

# Tests for 32_lldp_neighbors.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/lldp_neighbors_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/32_lldp_neighbors.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/32_lldp_neighbors.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "32_lldp_neighbors.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "32_lldp_neighbors.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "lldp_neighbors" ]]
    [[ "$output" =~ "arp_table" ]]
    [[ "$output" =~ "bridge_info" ]]
    [[ "$output" =~ "network_namespaces" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for x86_64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "x86_64"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for arm64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "arm64"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for i386" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh i386
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "i386"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for ppc64le" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for s390x" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh s390x
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "s390x"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for riscv64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh riscv64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "riscv64"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for mips64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh mips64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "mips64"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for aarch32" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh aarch32
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "aarch32"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for sparc64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh sparc64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "sparc64"' ]]
}

@test "32_lldp_neighbors.sh should return architecture in output for loongarch64" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh loongarch64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "loongarch64"' ]]
}

@test "32_lldp_neighbors.sh should detect LLDP neighbors information" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain LLDP neighbors array
    [[ "$output" =~ '"lldp_neighbors": [' ]]
}

@test "32_lldp_neighbors.sh should detect ARP table" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain ARP table array
    [[ "$output" =~ '"arp_table": [' ]]
    
    # Should have ARP fields if available
    if command -v ip >/dev/null 2>&1 || command -v arp >/dev/null 2>&1 || [[ -f /proc/net/arp ]]; then
        [[ "$output" =~ '"ip_address"' ]]
        [[ "$output" =~ '"mac_address"' ]]
    fi
}

@test "32_lldp_neighbors.sh should include required ARP fields" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain required fields for ARP entries
    [[ "$output" =~ '"ip_address"' ]]
    [[ "$output" =~ '"mac_address"' ]]
    [[ "$output" =~ '"interface"' ]]
    [[ "$output" =~ '"state"' ]]
}

@test "32_lldp_neighbors.sh should detect bridge information" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain bridge info array
    [[ "$output" =~ '"bridge_info": [' ]]
}

@test "32_lldp_neighbors.sh should include network namespaces" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain network namespaces field
    [[ "$output" =~ '"network_namespaces"' ]]
}

@test "32_lldp_neighbors.sh should handle missing LLDP tools gracefully" {
    cd "$TEST_DIR"
    
    # Test with limited PATH (no lldpctl)
    export PATH="/bin:/usr/bin"
    
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"lldp_neighbors"' ]]
}

@test "32_lldp_neighbors.sh should handle ARM architecture variants" {
    cd "$TEST_DIR"
    
    # Test different ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./32_lldp_neighbors.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "\"architecture\": \"$arch\"" ]]
    done
}

@test "32_lldp_neighbors.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./32_lldp_neighbors.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"lldp_neighbors"' ]]
        [[ "$output" =~ '"arp_table"' ]]
        [[ "$output" =~ '"bridge_info"' ]]
        [[ "$output" =~ '"network_namespaces"' ]]
        [[ "$output" =~ '"architecture"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}

@test "32_lldp_neighbors.sh should escape JSON special characters" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Output should be valid JSON (this will fail if special characters aren't escaped)
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "32_lldp_neighbors.sh should handle missing bridge tools gracefully" {
    cd "$TEST_DIR"
    
    # Create a minimal environment with limited PATH
    export PATH="/bin:/usr/bin"
    
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON even with limited tools
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"bridge_info"' ]]
}

@test "32_lldp_neighbors.sh should produce lldp_neighbors as valid JSON array" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract lldp_neighbors array and validate it separately
    lldp_neighbors=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['lldp_neighbors']))")
    echo "$lldp_neighbors" | python3 -m json.tool > /dev/null
}

@test "32_lldp_neighbors.sh should produce arp_table as valid JSON array" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract arp_table array and validate it separately
    arp_table=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['arp_table']))")
    echo "$arp_table" | python3 -m json.tool > /dev/null
}

@test "32_lldp_neighbors.sh should read ARP from /proc/net/arp if available" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # If /proc/net/arp exists, should have ARP information
    if [[ -f /proc/net/arp ]]; then
        [[ "$output" =~ '"arp_table"' ]]
    fi
}

@test "32_lldp_neighbors.sh should handle Docker bridges" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should detect Docker bridges if Docker is available
    if command -v docker >/dev/null 2>&1; then
        [[ "$output" =~ '"bridge_info"' ]]
    fi
}

@test "32_lldp_neighbors.sh should detect network namespaces" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain network namespaces array
    [[ "$output" =~ '"network_namespaces": [' ]]
}

@test "32_lldp_neighbors.sh should handle different LLDP protocols" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should have protocol fields for LLDP entries (if any exist)
    [[ "$output" =~ '"protocol"' ]] || [[ "$output" =~ '"lldp_neighbors": []' ]]
}

@test "32_lldp_neighbors.sh should include required LLDP fields" {
    cd "$TEST_DIR"
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Structure should be ready for LLDP fields
    [[ "$output" =~ '"local_interface"' ]] || [[ "$output" =~ '"lldp_neighbors": []' ]]
    [[ "$output" =~ '"chassis_id"' ]] || [[ "$output" =~ '"lldp_neighbors": []' ]]
    [[ "$output" =~ '"port_id"' ]] || [[ "$output" =~ '"lldp_neighbors": []' ]]
    [[ "$output" =~ '"system_name"' ]] || [[ "$output" =~ '"lldp_neighbors": []' ]]
}

@test "32_lldp_neighbors.sh should handle systems without network discovery tools" {
    cd "$TEST_DIR"
    
    # Test by removing most network tools from PATH
    export PATH="/bin:/usr/bin"
    
    run ./32_lldp_neighbors.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ '"lldp_neighbors"' ]]
    [[ "$output" =~ '"arp_table"' ]]
    [[ "$output" =~ '"bridge_info"' ]]
}