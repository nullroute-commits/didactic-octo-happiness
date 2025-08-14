#!/bin/bash
# Plugin installation script for NetBox

set -e

PLUGIN_LIST="$1"

if [ -z "$PLUGIN_LIST" ]; then
    echo "No plugins specified for installation"
    exit 0
fi

echo "Installing NetBox plugins: $PLUGIN_LIST"

# Convert comma-separated list to array
IFS=',' read -ra PLUGINS <<< "$PLUGIN_LIST"

# Common NetBox plugins and their installation methods
declare -A PLUGIN_REPOS=(
    # Device and Infrastructure Management
    ["netbox-topology-views"]="netbox-community/netbox-topology-views"
    ["netbox-device-map"]="netbox-community/netbox-device-map"
    ["netbox-device-lifecycle"]="netbox-community/netbox-device-lifecycle"
    
    # Network Automation
    ["netbox-napalm-plugin"]="netbox-community/netbox-napalm-plugin"
    ["netbox-bgp"]="netbox-community/netbox-bgp"
    ["netbox-dns"]="netbox-community/netbox-dns"
    
    # Documentation and Diagrams
    ["netbox-documents"]="netbox-community/netbox-documents"
    ["netbox-qrcode"]="netbox-community/netbox-qr-code"
    
    # Inventory and Asset Management
    ["netbox-inventory"]="netbox-community/netbox-inventory"
    ["netbox-contract"]="netbox-community/netbox-contract"
    
    # Monitoring Integration
    ["netbox-prometheus-sd"]="netbox-community/netbox-prometheus-sd"
    ["netbox-grafana"]="netbox-community/netbox-grafana"
    
    # Cloud Integration
    ["netbox-cloud"]="netbox-community/netbox-cloud"
    ["netbox-kubernetes"]="netbox-community/netbox-kubernetes"
    
    # Security and Compliance
    ["netbox-secrets"]="netbox-community/netbox-secrets"
    ["netbox-scanner"]="netbox-community/netbox-scanner"
)

# Function to install plugin from GitHub
install_plugin_from_github() {
    local plugin_name="$1"
    local repo_path="$2"
    
    echo "Installing $plugin_name from GitHub repository: $repo_path"
    
    # Clone the plugin repository
    cd /tmp
    git clone "https://github.com/$repo_path.git" "$plugin_name"
    cd "$plugin_name"
    
    # Install the plugin
    pip install .
    
    # Clean up
    cd /tmp
    rm -rf "$plugin_name"
}

# Function to install plugin from PyPI
install_plugin_from_pypi() {
    local plugin_name="$1"
    
    echo "Installing $plugin_name from PyPI"
    pip install "$plugin_name"
}

# Install each plugin
for plugin in "${PLUGINS[@]}"; do
    plugin=$(echo "$plugin" | xargs)  # Trim whitespace
    
    if [ -z "$plugin" ]; then
        continue
    fi
    
    echo "Processing plugin: $plugin"
    
    # Check if we have a known repository for this plugin
    if [[ -v PLUGIN_REPOS["$plugin"] ]]; then
        install_plugin_from_github "$plugin" "${PLUGIN_REPOS[$plugin]}"
    else
        # Try to install from PyPI
        echo "Unknown plugin repository for $plugin, attempting PyPI installation"
        install_plugin_from_pypi "$plugin"
    fi
done

echo "Plugin installation completed"

# Create plugins configuration template
cat > /opt/netbox/netbox/plugins_config.py << 'EOF'
# Auto-generated plugin configuration
# This file is created during container build if plugins are enabled

PLUGINS = [
EOF

# Add each installed plugin to the configuration
for plugin in "${PLUGINS[@]}"; do
    plugin=$(echo "$plugin" | xargs)
    if [ -n "$plugin" ]; then
        echo "    '$plugin'," >> /opt/netbox/netbox/plugins_config.py
    fi
done

cat >> /opt/netbox/netbox/plugins_config.py << 'EOF'
]

# Plugin-specific settings can be configured here
PLUGINS_CONFIG = {
    # Example plugin configuration:
    # 'netbox_topology_views': {
    #     'ALLOW_CIRCUIT_TERMINATION': True,
    # },
}
EOF

echo "Plugin configuration file created at /opt/netbox/netbox/plugins_config.py"