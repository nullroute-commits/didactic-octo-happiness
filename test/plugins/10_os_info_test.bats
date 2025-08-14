#!/usr/bin/env bats

# Tests for 10_os_info.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/os_info_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/10_os_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/10_os_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "10_os_info.sh should require architecture parameter" {
    cd "$TEST_DIR"
    run ./10_os_info.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Architecture parameter required" ]]
}

@test "10_os_info.sh should produce valid JSON output" {
    cd "$TEST_DIR"
    run ./10_os_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Validate JSON structure
    echo "$output" | python3 -m json.tool > /dev/null
    [[ "$output" =~ "os_name" ]]
    [[ "$output" =~ "os_version" ]]
    [[ "$output" =~ "distribution" ]]
    [[ "$output" =~ "distribution_version" ]]
    [[ "$output" =~ "kernel_version" ]]
    [[ "$output" =~ "architecture" ]]
}

@test "10_os_info.sh should return architecture in output for x86_64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh x86_64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "x86_64"' ]]
}

@test "10_os_info.sh should return architecture in output for arm64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh arm64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "arm64"' ]]
}

@test "10_os_info.sh should return architecture in output for i386" {
    cd "$TEST_DIR"
    run ./10_os_info.sh i386
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "i386"' ]]
}

@test "10_os_info.sh should return architecture in output for ppc64le" {
    cd "$TEST_DIR"
    run ./10_os_info.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
}

@test "10_os_info.sh should return architecture in output for s390x" {
    cd "$TEST_DIR"
    run ./10_os_info.sh s390x
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "s390x"' ]]
}

@test "10_os_info.sh should return architecture in output for riscv64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh riscv64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "riscv64"' ]]
}

@test "10_os_info.sh should return architecture in output for mips64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh mips64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "mips64"' ]]
}

@test "10_os_info.sh should return architecture in output for aarch32" {
    cd "$TEST_DIR"
    run ./10_os_info.sh aarch32
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "aarch32"' ]]
}

@test "10_os_info.sh should return architecture in output for sparc64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh sparc64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "sparc64"' ]]
}

@test "10_os_info.sh should return architecture in output for loongarch64" {
    cd "$TEST_DIR"
    run ./10_os_info.sh loongarch64
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "loongarch64"' ]]
}

@test "10_os_info.sh should detect OS information" {
    cd "$TEST_DIR"
    run ./10_os_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain reasonable OS information (not all "unknown")
    if [[ -f /etc/os-release ]]; then
        [[ ! "$output" =~ '"os_name": "unknown"' ]]
        [[ ! "$output" =~ '"distribution": "unknown"' ]]
    fi
}

@test "10_os_info.sh should detect kernel version" {
    cd "$TEST_DIR"
    run ./10_os_info.sh x86_64
    [ "$status" -eq 0 ]
    
    # Should contain kernel version that's not "unknown"
    [[ ! "$output" =~ '"kernel_version": "unknown"' ]]
}

@test "10_os_info.sh should handle ARM architecture variants" {
    cd "$TEST_DIR"
    
    # Test different ARM variants
    for arch in arm64 aarch32 armv7l armv8l arm; do
        run ./10_os_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "\"architecture\": \"$arch\"" ]]
    done
}

@test "10_os_info.sh should add architecture-specific distribution suffixes" {
    cd "$TEST_DIR"
    
    # Test that architecture-specific suffixes are added for specialized architectures
    for arch in s390x riscv64 mips64 sparc64 loongarch64; do
        run ./10_os_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" =~ "_${arch}" ]]
    done
}

@test "10_os_info.sh should handle PowerPC architecture" {
    cd "$TEST_DIR"
    run ./10_os_info.sh ppc64le
    [ "$status" -eq 0 ]
    [[ "$output" =~ '"architecture": "ppc64le"' ]]
    # May add "_power" suffix to distribution if POWER is detected
}

@test "10_os_info.sh should produce consistent JSON structure across architectures" {
    cd "$TEST_DIR"
    local archs="x86_64 arm64 i386 ppc64le s390x riscv64 mips64 aarch32 sparc64 loongarch64"
    
    for arch in $archs; do
        run ./10_os_info.sh "$arch"
        [ "$status" -eq 0 ]
        
        # Verify all required fields are present
        [[ "$output" =~ '"os_name"' ]]
        [[ "$output" =~ '"os_version"' ]]
        [[ "$output" =~ '"distribution"' ]]
        [[ "$output" =~ '"distribution_version"' ]]
        [[ "$output" =~ '"kernel_version"' ]]
        [[ "$output" =~ '"architecture"' ]]
        
        # Validate JSON
        echo "$output" | python3 -m json.tool > /dev/null
    done
}