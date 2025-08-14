#!/bin/bash
# Virtualization and container platform detection plugin
# Outputs VM platform, container runtime, and deployment info in JSON format

# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.

ARCH="$1"

get_virtualization_info() {
    local vm_platform="none"
    local hypervisor="none"
    local container_runtime=""
    local container_platform=""
    local deployment_info=""
    local virtualization_type="bare_metal"
    
    # Function to escape JSON strings
    escape_json() {
        echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\x0//g'
    }
    
    # Detect virtualization platform
    detect_vm_platform() {
        # Check systemd-detect-virt first (most reliable)
        if command -v systemd-detect-virt >/dev/null 2>&1; then
            local virt_result=$(systemd-detect-virt 2>/dev/null || echo "none")
            case "$virt_result" in
                "vmware")
                    vm_platform="VMware"
                    hypervisor="VMware ESXi/Workstation"
                    virtualization_type="full_virtualization"
                    ;;
                "kvm"|"qemu")
                    vm_platform="KVM/QEMU"
                    hypervisor="KVM"
                    virtualization_type="full_virtualization"
                    ;;
                "xen")
                    vm_platform="Xen"
                    hypervisor="Xen Hypervisor"
                    virtualization_type="paravirtualization"
                    ;;
                "microsoft")
                    vm_platform="Hyper-V"
                    hypervisor="Microsoft Hyper-V"
                    virtualization_type="full_virtualization"
                    ;;
                "oracle")
                    vm_platform="VirtualBox"
                    hypervisor="Oracle VirtualBox"
                    virtualization_type="full_virtualization"
                    ;;
                "lxc"|"lxc-libvirt")
                    vm_platform="LXC"
                    hypervisor="Linux Containers"
                    virtualization_type="containerization"
                    ;;
                "docker")
                    vm_platform="Docker"
                    hypervisor="Docker Engine"
                    virtualization_type="containerization"
                    ;;
                "container-other")
                    vm_platform="Container"
                    hypervisor="Unknown Container Runtime"
                    virtualization_type="containerization"
                    ;;
                "none")
                    vm_platform="none"
                    hypervisor="none"
                    virtualization_type="bare_metal"
                    ;;
                *)
                    vm_platform="$virt_result"
                    hypervisor="Unknown"
                    virtualization_type="unknown"
                    ;;
            esac
        fi
        
        # Fallback detection methods
        if [[ "$vm_platform" == "none" ]]; then
            # Check DMI information
            if [[ -f /sys/class/dmi/id/sys_vendor ]]; then
                local sys_vendor=$(cat /sys/class/dmi/id/sys_vendor 2>/dev/null | tr '[:upper:]' '[:lower:]')
                local product_name=$(cat /sys/class/dmi/id/product_name 2>/dev/null | tr '[:upper:]' '[:lower:]')
                
                if [[ "$sys_vendor" == *"vmware"* ]] || [[ "$product_name" == *"vmware"* ]]; then
                    vm_platform="VMware"
                    hypervisor="VMware ESXi/Workstation"
                    virtualization_type="full_virtualization"
                elif [[ "$sys_vendor" == *"qemu"* ]] || [[ "$product_name" == *"qemu"* ]]; then
                    vm_platform="QEMU/KVM"
                    hypervisor="QEMU/KVM"
                    virtualization_type="full_virtualization"
                elif [[ "$sys_vendor" == *"microsoft"* ]] || [[ "$product_name" == *"virtual machine"* ]]; then
                    vm_platform="Hyper-V"
                    hypervisor="Microsoft Hyper-V"
                    virtualization_type="full_virtualization"
                elif [[ "$sys_vendor" == *"innotek"* ]] || [[ "$product_name" == *"virtualbox"* ]]; then
                    vm_platform="VirtualBox"
                    hypervisor="Oracle VirtualBox"
                    virtualization_type="full_virtualization"
                fi
            fi
            
            # Check for AWS/cloud metadata
            if [[ -f /sys/hypervisor/uuid ]] && [[ $(cat /sys/hypervisor/uuid 2>/dev/null | cut -c1-3) == "ec2" ]]; then
                vm_platform="AWS EC2"
                hypervisor="Xen/Nitro"
                virtualization_type="cloud_virtualization"
            fi
            
            # Check for container environments
            if [[ -f /.dockerenv ]]; then
                vm_platform="Docker"
                hypervisor="Docker Engine"
                virtualization_type="containerization"
            elif [[ "$container" == "lxc" ]] || [[ -f /proc/1/environ ]] && grep -q "container=lxc" /proc/1/environ 2>/dev/null; then
                vm_platform="LXC"
                hypervisor="Linux Containers"
                virtualization_type="containerization"
            fi
        fi
    }
    
    # Detect container runtime
    detect_container_runtime() {
        container_runtime="["
        local first_runtime=true
        
        # Check for Docker
        if command -v docker >/dev/null 2>&1; then
            local docker_version=$(docker --version 2>/dev/null | awk '{print $3}' | sed 's/,$//' || echo "unknown")
            
            # Detect Docker Compose availability and type
            local compose_info=""
            if command -v docker-compose >/dev/null 2>&1; then
                local compose_version=$(docker-compose --version 2>/dev/null | awk '{print $3}' | sed 's/,$//' || echo "unknown")
                compose_info="\"compose_type\":\"standalone\",\"compose_version\":\"$compose_version\""
            elif docker compose version >/dev/null 2>&1; then
                local compose_version=$(docker compose version 2>/dev/null | grep -o 'v[0-9][0-9.]*' | head -1 || echo "unknown")
                compose_info="\"compose_type\":\"plugin\",\"compose_version\":\"$compose_version\""
            else
                compose_info="\"compose_type\":\"not_available\",\"compose_version\":\"none\""
            fi
            
            if [[ "$first_runtime" == "false" ]]; then
                container_runtime+=","
            fi
            first_runtime=false
            container_runtime+="{\"name\":\"Docker\",\"version\":\"$docker_version\",\"status\":\"installed\",$compose_info}"
        fi
        
        # Check for Podman
        if command -v podman >/dev/null 2>&1; then
            local podman_version=$(podman --version 2>/dev/null | awk '{print $3}' || echo "unknown")
            if [[ "$first_runtime" == "false" ]]; then
                container_runtime+=","
            fi
            first_runtime=false
            container_runtime+="{\"name\":\"Podman\",\"version\":\"$podman_version\",\"status\":\"installed\"}"
        fi
        
        # Check for containerd
        if command -v containerd >/dev/null 2>&1; then
            local containerd_version=$(containerd --version 2>/dev/null | awk '{print $3}' || echo "unknown")
            if [[ "$first_runtime" == "false" ]]; then
                container_runtime+=","
            fi
            first_runtime=false
            container_runtime+="{\"name\":\"containerd\",\"version\":\"$containerd_version\",\"status\":\"installed\"}"
        fi
        
        # Check for CRI-O
        if command -v crio >/dev/null 2>&1; then
            local crio_version=$(crio --version 2>/dev/null | head -1 | awk '{print $3}' || echo "unknown")
            if [[ "$first_runtime" == "false" ]]; then
                container_runtime+=","
            fi
            first_runtime=false
            container_runtime+="{\"name\":\"CRI-O\",\"version\":\"$crio_version\",\"status\":\"installed\"}"
        fi
        
        if [[ "$first_runtime" == "true" ]]; then
            container_runtime+="{\"name\":\"none\",\"version\":\"unknown\",\"status\":\"not_detected\"}"
        fi
        
        container_runtime+="]"
    }
    
    # Detect container platform/orchestration
    detect_container_platform() {
        container_platform="["
        local first_platform=true
        
        # Check for Kubernetes
        if command -v kubectl >/dev/null 2>&1; then
            local kubectl_version=$(kubectl version --client --short 2>/dev/null | awk '{print $3}' || echo "unknown")
            if [[ "$first_platform" == "false" ]]; then
                container_platform+=","
            fi
            first_platform=false
            container_platform+="{\"name\":\"Kubernetes\",\"version\":\"$kubectl_version\",\"status\":\"client_installed\"}"
        fi
        
        # Check if running in Kubernetes
        if [[ -n "$KUBERNETES_SERVICE_HOST" ]] || [[ -f /var/run/secrets/kubernetes.io/serviceaccount/token ]]; then
            if [[ "$first_platform" == "false" ]]; then
                container_platform+=","
            fi
            first_platform=false
            local namespace=$(cat /var/run/secrets/kubernetes.io/serviceaccount/namespace 2>/dev/null || echo "unknown")
            container_platform+="{\"name\":\"Kubernetes\",\"version\":\"runtime\",\"status\":\"running_in_cluster\",\"namespace\":\"$namespace\"}"
        fi
        
        # Check for Docker Swarm
        if command -v docker >/dev/null 2>&1; then
            local swarm_status=$(docker info --format '{{.Swarm.LocalNodeState}}' 2>/dev/null || echo "inactive")
            if [[ "$swarm_status" != "inactive" ]]; then
                if [[ "$first_platform" == "false" ]]; then
                    container_platform+=","
                fi
                first_platform=false
                container_platform+="{\"name\":\"Docker Swarm\",\"version\":\"unknown\",\"status\":\"$swarm_status\"}"
            fi
        fi
        
        # Check for OpenShift
        if command -v oc >/dev/null 2>&1; then
            local oc_version=$(oc version --client 2>/dev/null | grep "Client Version" | awk '{print $3}' || echo "unknown")
            if [[ "$first_platform" == "false" ]]; then
                container_platform+=","
            fi
            first_platform=false
            container_platform+="{\"name\":\"OpenShift\",\"version\":\"$oc_version\",\"status\":\"client_installed\"}"
        fi
        
        if [[ "$first_platform" == "true" ]]; then
            container_platform+="{\"name\":\"none\",\"version\":\"unknown\",\"status\":\"not_detected\"}"
        fi
        
        container_platform+="]"
    }
    
    # Enhanced environment perspective detection for containerized deployments
    detect_host_environment() {
        local host_info="{"
        local has_host_access="false"
        local host_os_release=""
        local host_kernel=""
        local host_hostname=""
        local host_architecture=""
        local container_network_info=""
        
        # Check if we have access to host system information via mounted volumes
        if [[ -n "${HOST_SYSTEM_ROOT}" ]] && [[ -d "${HOST_SYSTEM_ROOT}" ]]; then
            has_host_access="true"
            
            # Read host OS information if available
            if [[ -f "${HOST_SYSTEM_ROOT}/etc/os-release" ]]; then
                host_os_release=$(grep '^PRETTY_NAME=' "${HOST_SYSTEM_ROOT}/etc/os-release" 2>/dev/null | cut -d'"' -f2 || echo "unknown")
            fi
            
            # Read host kernel version if available
            if [[ -f "${HOST_SYSTEM_ROOT}/proc/version" ]]; then
                host_kernel=$(head -1 "${HOST_SYSTEM_ROOT}/proc/version" 2>/dev/null | awk '{print $3}' || echo "unknown")
            fi
            
            # Read host hostname if available
            if [[ -f "${HOST_SYSTEM_ROOT}/etc/hostname" ]]; then
                host_hostname=$(cat "${HOST_SYSTEM_ROOT}/etc/hostname" 2>/dev/null || echo "unknown")
            fi
            
            # Detect host architecture from uname if accessible
            if [[ -f "${HOST_SYSTEM_ROOT}/proc/cpuinfo" ]]; then
                host_architecture=$(grep '^processor' "${HOST_SYSTEM_ROOT}/proc/cpuinfo" | wc -l 2>/dev/null || echo "unknown")
                if [[ "$host_architecture" != "unknown" ]]; then
                    host_architecture="${host_architecture} cores"
                fi
            fi
        fi
        
        # Detect container network configuration and external connectivity
        container_network_info="["
        local first_net=true
        
        # Get default gateway for external IP detection
        local default_gateway=$(ip route show default 2>/dev/null | awk '/default/ {print $3}' | head -1 || echo "unknown")
        local external_ip=""
        
        # Attempt to discover external IP through multiple methods
        if [[ "$default_gateway" != "unknown" ]]; then
            # Try to get external IP via HTTP requests with short timeouts
            external_ip=$(curl -m 3 -s http://checkip.amazonaws.com 2>/dev/null || \
                         curl -m 3 -s http://ipecho.net/plain 2>/dev/null || \
                         curl -m 3 -s http://icanhazip.com 2>/dev/null || \
                         echo "unknown")
        fi
        
        # Container network interface analysis
        for iface in $(ip link show 2>/dev/null | grep '^[0-9]' | cut -d':' -f2 | tr -d ' '); do
            if [[ "$iface" != "lo" ]]; then
                local iface_ip=$(ip addr show "$iface" 2>/dev/null | grep 'inet ' | awk '{print $2}' | head -1 || echo "unknown")
                local iface_mtu=$(ip link show "$iface" 2>/dev/null | grep 'mtu' | awk '{print $5}' || echo "unknown")
                
                if [[ "$first_net" == "false" ]]; then
                    container_network_info+=","
                fi
                first_net=false
                container_network_info+="{\"interface\":\"$iface\",\"ip\":\"$iface_ip\",\"mtu\":\"$iface_mtu\"}"
            fi
        done
        
        if [[ "$first_net" == "true" ]]; then
            container_network_info+="{\"interface\":\"none\",\"ip\":\"unknown\",\"mtu\":\"unknown\"}"
        fi
        container_network_info+="]"
        
        host_info+="\"has_host_access\":$has_host_access,"
        host_info+="\"host_os_release\":\"$(escape_json "$host_os_release")\","
        host_info+="\"host_kernel\":\"$(escape_json "$host_kernel")\","
        host_info+="\"host_hostname\":\"$(escape_json "$host_hostname")\","
        host_info+="\"host_architecture\":\"$(escape_json "$host_architecture")\","
        host_info+="\"default_gateway\":\"$(escape_json "$default_gateway")\","
        host_info+="\"external_ip\":\"$(escape_json "$external_ip")\","
        host_info+="\"container_network\":$container_network_info"
        host_info+="}"
        
        echo "$host_info"
    }

    # Collect deployment information
    collect_deployment_info() {
        deployment_info="{"
        
        # Check for cloud provider metadata
        local cloud_provider="unknown"
        local instance_type="unknown"
        local region="unknown"
        
        # AWS metadata
        if curl -m 2 -s http://169.254.169.254/latest/meta-data/ >/dev/null 2>&1; then
            cloud_provider="AWS"
            instance_type=$(curl -m 2 -s http://169.254.169.254/latest/meta-data/instance-type 2>/dev/null || echo "unknown")
            region=$(curl -m 2 -s http://169.254.169.254/latest/meta-data/placement/region 2>/dev/null || echo "unknown")
        # Google Cloud metadata
        elif curl -m 2 -s -H "Metadata-Flavor: Google" http://metadata.google.internal/computeMetadata/v1/ >/dev/null 2>&1; then
            cloud_provider="GCP"
            instance_type=$(curl -m 2 -s -H "Metadata-Flavor: Google" http://metadata.google.internal/computeMetadata/v1/instance/machine-type 2>/dev/null | cut -d'/' -f4 || echo "unknown")
            region=$(curl -m 2 -s -H "Metadata-Flavor: Google" http://metadata.google.internal/computeMetadata/v1/instance/zone 2>/dev/null | cut -d'/' -f4 | sed 's/-[^-]*$//' || echo "unknown")
        # Azure metadata
        elif curl -m 2 -s -H "Metadata: true" http://169.254.169.254/metadata/instance/compute/vmSize 2>/dev/null | grep -q "."; then
            cloud_provider="Azure"
            instance_type=$(curl -m 2 -s -H "Metadata: true" http://169.254.169.254/metadata/instance/compute/vmSize 2>/dev/null || echo "unknown")
            region=$(curl -m 2 -s -H "Metadata: true" http://169.254.169.254/metadata/instance/compute/location 2>/dev/null || echo "unknown")
        fi
        
        deployment_info+="\"cloud_provider\":\"$cloud_provider\","
        deployment_info+="\"instance_type\":\"$instance_type\","
        deployment_info+="\"region\":\"$region\""
        deployment_info+="}"
    }
    
    # Execute detection functions
    detect_vm_platform
    detect_container_runtime
    detect_container_platform
    collect_deployment_info
    local host_environment=$(detect_host_environment)
    
    # Output JSON
    cat << EOF
{
  "virtualization_type": "$(escape_json "$virtualization_type")",
  "vm_platform": "$(escape_json "$vm_platform")",
  "hypervisor": "$(escape_json "$hypervisor")",
  "container_runtime": $container_runtime,
  "container_platform": $container_platform,
  "deployment_info": $deployment_info,
  "host_environment": $host_environment,
  "architecture": "$(escape_json "$ARCH")"
}
EOF
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Execute main function
get_virtualization_info