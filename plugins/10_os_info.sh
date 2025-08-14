#!/bin/bash
# OS and distribution information plugin
# Outputs OS and distro info in JSON format
# NOTE: Do not use 'set -e' in plugin scripts.
# When plugins are executed via command substitution in the main script,
# 'set -e' can cause unexpected behavior and silent failures.

ARCH="$1"

get_os_info() {
    local os_name=""
    local os_version=""
    local distro=""
    local distro_version=""
    local kernel_version=""
    
    # Get kernel version
    kernel_version=$(uname -r 2>/dev/null || echo "unknown")
    
    # Detect OS and distribution
    if [[ -f /etc/os-release ]]; then
        # Use os-release for modern systems
        . /etc/os-release
        os_name="${NAME:-unknown}"
        os_version="${VERSION:-unknown}"
        distro="${ID:-unknown}"
        distro_version="${VERSION_ID:-unknown}"
    elif [[ -f /etc/redhat-release ]]; then
        # Red Hat family
        distro="rhel"
        distro_version=$(cat /etc/redhat-release | sed 's/.*release \([0-9.]*\).*/\1/' 2>/dev/null || echo "unknown")
        os_name="Red Hat Enterprise Linux"
        os_version="$distro_version"
    elif [[ -f /etc/debian_version ]]; then
        # Debian family
        distro="debian"
        distro_version=$(cat /etc/debian_version 2>/dev/null || echo "unknown")
        os_name="Debian"
        os_version="$distro_version"
    elif [[ -f /etc/SuSE-release ]]; then
        # SUSE family
        distro="suse"
        distro_version=$(grep VERSION /etc/SuSE-release | cut -d= -f2 2>/dev/null || echo "unknown")
        os_name="SUSE"
        os_version="$distro_version"
    elif command -v sw_vers >/dev/null 2>&1; then
        # macOS
        os_name="macOS"
        os_version=$(sw_vers -productVersion 2>/dev/null || echo "unknown")
        distro="macos"
        distro_version="$os_version"
    elif command -v wsl.exe >/dev/null 2>&1; then
        # WSL
        distro="wsl"
        os_name="Windows Subsystem for Linux"
        os_version=$(uname -v 2>/dev/null || echo "unknown")
        distro_version="$os_version"
    else
        # Fallback to uname
        os_name=$(uname -s 2>/dev/null || echo "unknown")
        os_version=$(uname -v 2>/dev/null || echo "unknown")
        distro="unknown"
        distro_version="unknown"
    fi
    
    # Architecture-specific adjustments
    case "$ARCH" in
        x86_64|amd64)
            # Standard x86_64 processing
            ;;
        arm64|aarch64)
            # ARM64 specific detection
            if [[ -f /proc/device-tree/model ]]; then
                local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0' || echo "")
                if [[ "$model" =~ "Raspberry Pi" ]]; then
                    distro="${distro}_rpi"
                fi
            fi
            ;;
        i386|i686)
            # 32-bit x86 specific
            ;;
        ppc64le)
            # PowerPC Little Endian
            if [[ -f /proc/cpuinfo ]] && grep -q "POWER" /proc/cpuinfo; then
                distro="${distro}_power"
            fi
            ;;
        s390x)
            # IBM Z/Architecture
            distro="${distro}_s390x"
            ;;
        riscv64)
            # RISC-V 64-bit
            distro="${distro}_riscv64"
            ;;
        mips64)
            # MIPS 64-bit
            distro="${distro}_mips64"
            ;;
        aarch32|armv7l|armv8l|arm)
            # ARM 32-bit
            if [[ -f /proc/device-tree/model ]]; then
                local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0' || echo "")
                if [[ "$model" =~ "Raspberry Pi" ]]; then
                    distro="${distro}_rpi32"
                fi
            fi
            ;;
        sparc64)
            # SPARC 64-bit
            distro="${distro}_sparc64"
            ;;
        loongarch64)
            # LoongArch 64-bit
            distro="${distro}_loongarch64"
            ;;
    esac
    
    # Output JSON
    cat << EOF
{
  "os_name": "$os_name",
  "os_version": "$os_version",
  "distribution": "$distro",
  "distribution_version": "$distro_version",
  "kernel_version": "$kernel_version",
  "architecture": "$ARCH"
}
EOF
}

# Validate architecture parameter
if [[ -z "$ARCH" ]]; then
    echo "Error: Architecture parameter required" >&2
    exit 1
fi

# Execute main function
get_os_info