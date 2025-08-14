#!/usr/bin/env bats

# Tests for 25_virtualization_info.sh plugin

# Test data setup
setup() {
    export TEST_DIR="/tmp/virtualization_info_test"
    mkdir -p "$TEST_DIR"
    
    # Copy plugin to test location
    cp plugins/25_virtualization_info.sh "$TEST_DIR/"
    chmod +x "$TEST_DIR/25_virtualization_info.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "virtualization plugin exists and is executable" {
    [ -x "plugins/25_virtualization_info.sh" ]
}

@test "virtualization plugin requires architecture parameter" {
    run plugins/25_virtualization_info.sh
    [ "$status" -eq 1 ]
    [[ "$output" == *"Architecture parameter required"* ]]
}

@test "virtualization plugin outputs valid JSON" {
    run plugins/25_virtualization_info.sh "x86_64"
    [ "$status" -eq 0 ]
    
    # Test JSON validity
    echo "$output" | python3 -m json.tool >/dev/null
}

@test "virtualization plugin includes required fields" {
    run plugins/25_virtualization_info.sh "x86_64"
    [ "$status" -eq 0 ]
    
    # Check for required top-level fields
    [[ "$output" == *"virtualization_type"* ]]
    [[ "$output" == *"vm_platform"* ]]
    [[ "$output" == *"hypervisor"* ]]
    [[ "$output" == *"container_runtime"* ]]
    [[ "$output" == *"container_platform"* ]]
    [[ "$output" == *"deployment_info"* ]]
    [[ "$output" == *"architecture"* ]]
}

@test "virtualization plugin detects container runtime array format" {
    run plugins/25_virtualization_info.sh "x86_64"
    [ "$status" -eq 0 ]
    
    # Container runtime should be array format with at least one entry
    [[ "$output" == *'"container_runtime": ['* ]]
    [[ "$output" == *'"name":'* ]]
    [[ "$output" == *'"version":'* ]]
    [[ "$output" == *'"status":'* ]]
}

@test "virtualization plugin detects platform array format" {
    run plugins/25_virtualization_info.sh "x86_64"
    [ "$status" -eq 0 ]
    
    # Container platform should be array format
    [[ "$output" == *'"container_platform": ['* ]]
}

@test "virtualization plugin works with different architectures" {
    local architectures=("x86_64" "arm64" "i386" "ppc64le")
    
    for arch in "${architectures[@]}"; do
        run plugins/25_virtualization_info.sh "$arch"
        [ "$status" -eq 0 ]
        [[ "$output" == *"\"architecture\": \"$arch\""* ]]
        
        # Validate JSON for each architecture
        echo "$output" | python3 -m json.tool >/dev/null
    done
}

@test "virtualization plugin includes deployment info structure" {
    run plugins/25_virtualization_info.sh "x86_64"
    [ "$status" -eq 0 ]
    
    # Deployment info should include cloud provider fields
    [[ "$output" == *'"deployment_info":'* ]]
    [[ "$output" == *'"cloud_provider":'* ]]
    [[ "$output" == *'"instance_type":'* ]]
    [[ "$output" == *'"region":'* ]]
}