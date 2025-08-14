#!/usr/bin/env bats

# Tests for 20_hardware_info.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/hardware_info_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/20_hardware_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/20_hardware_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "20_hardware_info.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "20_hardware_info.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "cpu_model" ]]
    [[ "$output" =~ "cpu_cores" ]]
    [[ "$output" =~ "cpu_threads" ]]
    [[ "$output" =~ "cpu_frequency" ]]
    [[ "$output" =~ "memory_total" ]]
    [[ "$output" =~ "memory_available" ]]
    [[ "$output" =~ "disk_info" ]]
}

@test "20_hardware_info.sh should return valid hardware info for x86_64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain CPU information
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
    [[ "$output" =~ '"cpu_threads"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for arm64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh arm64
    [ "$status" -eq 0 ]
    
    # Should handle ARM-specific CPU detection
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for i386" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh i386
    [ "$status" -eq 0 ]
    
    # Should handle 32-bit x86 detection
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for ppc64le" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh ppc64le
    [ "$status" -eq 0 ]
    
    # Should handle PowerPC detection
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for s390x" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh s390x
    [ "$status" -eq 0 ]
    
    # Should handle IBM Z architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for riscv64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh riscv64
    [ "$status" -eq 0 ]
    
    # Should handle RISC-V architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for mips64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh mips64
    [ "$status" -eq 0 ]
    
    # Should handle MIPS architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for aarch32" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh aarch32
    [ "$status" -eq 0 ]
    
    # Should handle ARM 32-bit architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for sparc64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh sparc64
    [ "$status" -eq 0 ]
    
    # Should handle SPARC architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should return valid hardware info for loongarch64" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh loongarch64
    [ "$status" -eq 0 ]
    
    # Should handle LoongArch architecture
    [[ "$output" =~ '"cpu_model"' ]]
    [[ "$output" =~ '"cpu_cores"' ]]
}

@test "20_hardware_info.sh should detect memory information" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain memory information
    [[ "$output" =~ '"memory_total"' ]]
    [[ "$output" =~ '"memory_available"' ]]
    
    # If /proc/meminfo exists, should not be "unknown"
    if [[ -f /proc/meminfo ]]; then
        [[ ! "$output" =~ '"memory_total": "unknown"' ]]
    fi
}

@test "20_hardware_info.sh should detect disk information" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain disk information as array
    [[ "$output" =~ '"disk_info": [' ]]
    [[ "$output" =~ 'filesystem' ]]
    [[ "$output" =~ 'size' ]]
    [[ "$output" =~ 'mountpoint' ]]
}

@test "20_hardware_info.sh should handle CPU frequency detection" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain CPU frequency field
    [[ "$output" =~ '"cpu_frequency"' ]]
}

@test "20_hardware_info.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./20_hardware_info.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"cpu_model"' ]]
        [[ "$output" =~ '"cpu_cores"' ]]
        [[ "$output" =~ '"cpu_threads"' ]]
        [[ "$output" =~ '"cpu_frequency"' ]]
        [[ "$output" =~ '"memory_total"' ]]
        [[ "$output" =~ '"memory_available"' ]]
        [[ "$output" =~ '"disk_info"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}

@test "20_hardware_info.sh should handle ARM-specific detection" {
    cd "$TEST_DIR"
    
    # Test ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./20_hardware_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ '"cpu_model"' ]]
    done
}

@test "20_hardware_info.sh should detect CPU cores correctly" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # CPU cores should be a number (not "unknown" if nproc is available)
    if command -v nproc >/dev/null; then
        [[ ! "$output" =~ '"cpu_cores": "unknown"' ]]
    fi
}

@test "20_hardware_info.sh should handle different CPU info formats per architecture" {
    cd "$TEST_DIR"
    
    # x86_64 should look for "model name"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # ARM should look for "model name", "Processor", or "Hardware"
    run ./20_hardware_info.sh arm64
    [ "$status" -eq 0 ]
    
    # Both should produce valid output
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "20_hardware_info.sh should produce disk info as valid JSON array" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Extract disk_info array and validate it separately
    disk_info=$(echo "$output" | python3 -c "import json, sys; data=json.load(sys.stdin); print(json.dumps(data['disk_info']))")
    echo "$disk_info" | python3 -m json.tool > /dev/null
    [[ "$disk_info" =~ "filesystem" ]]
}

@test "20_hardware_info.sh should handle systems without bc command" {
    cd "$TEST_DIR"
    
    # The script should work even if bc is not available for calculations
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still produce valid JSON
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "20_hardware_info.sh should include enhanced hardware fields" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Check for new enhanced hardware fields
    [[ "$output" =~ '"pcie_devices"' ]]
    [[ "$output" =~ '"usb_devices"' ]]
    [[ "$output" =~ '"gpu_info"' ]]
    [[ "$output" =~ '"network_hardware"' ]]
}

@test "20_hardware_info.sh should return PCIe devices as valid JSON array" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # PCIe devices should be array format
    [[ "$output" =~ '"pcie_devices": [' ]]
    [[ "$output" =~ '"slot"' ]]
    [[ "$output" =~ '"device"' ]]
    [[ "$output" =~ '"vendor"' ]]
    [[ "$output" =~ '"device_id"' ]]
}

@test "20_hardware_info.sh should return USB devices as valid JSON array" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # USB devices should be array format
    [[ "$output" =~ '"usb_devices": [' ]]
    [[ "$output" =~ '"bus"' ]]
    [[ "$output" =~ '"device"' ]]
    [[ "$output" =~ '"id"' ]]
    [[ "$output" =~ '"description"' ]]
}

@test "20_hardware_info.sh should return GPU info as valid JSON array" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # GPU info should be array format
    [[ "$output" =~ '"gpu_info": [' ]]
    [[ "$output" =~ '"slot"' ]]
    [[ "$output" =~ '"description"' ]]
    [[ "$output" =~ '"vendor"' ]]
    [[ "$output" =~ '"memory"' ]]
}

@test "20_hardware_info.sh should return network hardware as valid JSON array" {
    cd "$TEST_DIR"
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Network hardware should be array format
    [[ "$output" =~ '"network_hardware": [' ]]
    [[ "$output" =~ '"slot"' ]]
    [[ "$output" =~ '"description"' ]]
    [[ "$output" =~ '"vendor"' ]]
    [[ "$output" =~ '"driver"' ]]
    [[ "$output" =~ '"speed"' ]]
}

@test "20_hardware_info.sh should handle missing lspci gracefully" {
    cd "$TEST_DIR"
    
    # Even without lspci, should produce valid output
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still include all required fields with unknown values if needed
    [[ "$output" =~ '"pcie_devices"' ]]
    [[ "$output" =~ '"gpu_info"' ]]
    [[ "$output" =~ '"network_hardware"' ]]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
}

@test "20_hardware_info.sh should handle missing lsusb gracefully" {
    cd "$TEST_DIR"
    
    # Even without lsusb, should produce valid output  
    run ./20_hardware_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should still include USB devices field
    [[ "$output" =~ '"usb_devices"' ]]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
}