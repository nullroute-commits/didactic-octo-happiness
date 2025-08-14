#!/bin/bash
# LXC container setup script for Automation Nation

set -e

CONTAINER_NAME="automation-nation"
ROOTFS_PATH="/var/lib/lxc/${CONTAINER_NAME}/rootfs"

echo "Setting up LXC container: ${CONTAINER_NAME}"

# Create container with Ubuntu base
lxc-create -n "${CONTAINER_NAME}" -t ubuntu -- --release focal

# Copy LXC configuration
cp lxc.conf "/var/lib/lxc/${CONTAINER_NAME}/config"

# Install dependencies in container
lxc-attach -n "${CONTAINER_NAME}" -- bash << 'EOF'
export DEBIAN_FRONTEND=noninteractive

# Update system
apt-get update && apt-get upgrade -y

# Install runtime dependencies
apt-get install -y \
    ca-certificates \
    curl \
    sqlite3 \
    bash \
    procps \
    net-tools \
    iproute2 \
    lshw \
    pciutils \
    usbutils \
    systemd \
    systemctl

# Create app user
useradd -m -u 1000 appuser

# Create directories
mkdir -p /app/plugins /app/templates /app/data /host/proc /host/sys
chown -R appuser:appuser /app
EOF

# Copy application files to container
echo "Copying application files..."

# Build the application first
echo "Building Rust application..."
cargo build --release --bin web_server --bin ci_runner

# Copy binaries
mkdir -p "${ROOTFS_PATH}/app/bin"
cp target/release/web_server target/release/ci_runner "${ROOTFS_PATH}/app/bin/"

# Copy application files
cp collect_info.sh "${ROOTFS_PATH}/app/"
cp -r plugins/ "${ROOTFS_PATH}/app/"
cp -r templates/ "${ROOTFS_PATH}/app/"

# Make scripts executable
chmod +x "${ROOTFS_PATH}/app/collect_info.sh"
chmod +x "${ROOTFS_PATH}/app/plugins/"*.sh
chmod +x "${ROOTFS_PATH}/app/bin/"*

# Set ownership
chown -R 1000:1000 "${ROOTFS_PATH}/app"

# Create systemd service file
cat > "${ROOTFS_PATH}/etc/systemd/system/automation-nation.service" << 'EOF'
[Unit]
Description=Automation Nation Web Application
After=network.target
Wants=network.target

[Service]
Type=simple
User=appuser
Group=appuser
WorkingDirectory=/app
ExecStart=/app/bin/web_server serve --host 0.0.0.0 --port 3000
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=ENABLE_HASHING=1
Environment=ENABLE_SUDO_SUPPORT=0
Environment=HOST_SYSTEM_ROOT=/host

[Install]
WantedBy=multi-user.target
EOF

# Enable the service
lxc-attach -n "${CONTAINER_NAME}" -- systemctl enable automation-nation.service

# Create host information collection script that works with mounted host system
cat > "${ROOTFS_PATH}/app/collect_host_info.sh" << 'EOF'
#!/bin/bash
# Modified collect_info.sh for LXC container to read from host system

set -e

PLUGIN_DIR="./plugins"
OUTPUT_FILE=""
PLUGINS=()

# Configuration options
ENABLE_HASHING=${ENABLE_HASHING:-1}
ENABLE_SUDO_SUPPORT=${ENABLE_SUDO_SUPPORT:-0}

# Override system paths to read from host mounts
export HOST_PROC_PATH="/host/proc"
export HOST_SYS_PATH="/host/sys"

# Source the original script but with modified paths
source /app/collect_info.sh "$@"
EOF

chmod +x "${ROOTFS_PATH}/app/collect_host_info.sh"
chown 1000:1000 "${ROOTFS_PATH}/app/collect_host_info.sh"

# Create network configuration for port forwarding
echo "Setting up network configuration..."

# Create iptables rules for port forwarding (host system)
cat > setup_port_forward.sh << 'EOF'
#!/bin/bash
# Setup port forwarding for LXC container

CONTAINER_IP="10.0.3.100"
HOST_PORT="3000"
CONTAINER_PORT="3000"

# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# Add iptables rules for port forwarding
iptables -t nat -A PREROUTING -p tcp --dport ${HOST_PORT} -j DNAT --to-destination ${CONTAINER_IP}:${CONTAINER_PORT}
iptables -t nat -A POSTROUTING -s ${CONTAINER_IP} -j MASQUERADE
iptables -A FORWARD -p tcp -d ${CONTAINER_IP} --dport ${CONTAINER_PORT} -j ACCEPT

echo "Port forwarding setup complete: localhost:${HOST_PORT} -> ${CONTAINER_IP}:${CONTAINER_PORT}"
EOF

chmod +x setup_port_forward.sh

echo "LXC container setup complete!"
echo ""
echo "To start the container:"
echo "  sudo lxc-start -n ${CONTAINER_NAME} -d"
echo ""
echo "To setup port forwarding (run as root):"
echo "  sudo ./setup_port_forward.sh"
echo ""
echo "To access the application:"
echo "  http://localhost:3000"
echo ""
echo "To enter the container:"
echo "  sudo lxc-attach -n ${CONTAINER_NAME}"
echo ""
echo "To stop the container:"
echo "  sudo lxc-stop -n ${CONTAINER_NAME}"