# Configuration Guide

This guide covers configuration options and tuning parameters for the Automation_nation system information collector.

## Environment Variables

### Global Configuration

No global configuration variables are currently supported. All configuration is done at the plugin level through environment variables.

### Plugin-Specific Configuration

#### Network Interface Plugin (30_ip_info.sh)

Controls network interface discovery and address collection:

```bash
# Maximum number of network interfaces to process (default: 20)
export MAX_INTERFACES=20

# Maximum IPv4/IPv6 addresses per interface (default: 10)
export MAX_ADDRESSES_PER_INTERFACE=10
```

**Use Cases:**
- **High-density servers**: Increase `MAX_INTERFACES` for systems with many virtual interfaces
- **Container hosts**: Increase limits for Docker/Kubernetes environments with extensive networking
- **Performance tuning**: Decrease limits on resource-constrained systems

#### Network Statistics Plugin (31_network_stats.sh)

Controls network statistics and routing table collection:

```bash
# Maximum number of interfaces for statistics (default: 20)
export MAX_INTERFACES=20

# Maximum number of routing table entries for IPv4/IPv6 (default: 50)
export MAX_ROUTES=50

# Maximum multicast group entries (default: 30)
export MAX_MCAST_GROUPS=30

# Maximum listening ports to report (default: 50)
export MAX_LISTENING_PORTS=50
```

**Use Cases:**
- **Network appliances**: Increase `MAX_ROUTES` for routers with extensive routing tables
- **Monitoring systems**: Increase `MAX_LISTENING_PORTS` for systems running many services
- **Embedded systems**: Decrease all limits for resource conservation

#### LLDP/ARP Discovery Plugin (32_lldp_neighbors.sh)

Controls network neighbor discovery and bridge detection:

```bash
# Maximum LLDP/CDP neighbors to discover (default: 20)
export MAX_NEIGHBORS=20

# Maximum ARP table entries (default: 50)
export MAX_ARP_ENTRIES=50

# Maximum bridge configurations (default: 20)
export MAX_BRIDGES=20

# Maximum network namespaces (default: 20)
export MAX_NETNS=20

# Maximum Docker bridge networks (default: 10)
export MAX_DOCKER_NETWORKS=10
```

**Use Cases:**
- **Container platforms**: Increase `MAX_NETNS` and `MAX_DOCKER_NETWORKS` for Kubernetes/Docker environments
- **Network discovery**: Increase `MAX_NEIGHBORS` and `MAX_ARP_ENTRIES` for comprehensive network mapping
- **Security monitoring**: Higher limits help detect all network connections and neighbors

## Configuration Examples

### Performance-Optimized (Resource-Constrained Systems)

```bash
#!/bin/bash
# Minimal resource usage configuration
export MAX_INTERFACES=5
export MAX_ROUTES=10
export MAX_MCAST_GROUPS=5
export MAX_LISTENING_PORTS=10
export MAX_NEIGHBORS=5
export MAX_ARP_ENTRIES=10
export MAX_BRIDGES=5
export MAX_NETNS=5
export MAX_DOCKER_NETWORKS=3

./collect_info.sh -o minimal-info.json
```

### Comprehensive Discovery (High-End Systems)

```bash
#!/bin/bash
# Maximum detail configuration
export MAX_INTERFACES=100
export MAX_ADDRESSES_PER_INTERFACE=20
export MAX_ROUTES=200
export MAX_MCAST_GROUPS=100
export MAX_LISTENING_PORTS=200
export MAX_NEIGHBORS=50
export MAX_ARP_ENTRIES=500
export MAX_BRIDGES=50
export MAX_NETNS=100
export MAX_DOCKER_NETWORKS=50

./collect_info.sh -o comprehensive-info.json
```

### Container Platform Focus

```bash
#!/bin/bash
# Optimized for Kubernetes/Docker environments
export MAX_INTERFACES=50
export MAX_ROUTES=100
export MAX_BRIDGES=30
export MAX_NETNS=100
export MAX_DOCKER_NETWORKS=30
export MAX_ARP_ENTRIES=200

./collect_info.sh -o container-platform-info.json
```

### Network Appliance/Router

```bash
#!/bin/bash
# Optimized for network infrastructure devices
export MAX_INTERFACES=30
export MAX_ROUTES=500
export MAX_NEIGHBORS=100
export MAX_ARP_ENTRIES=1000
export MAX_LISTENING_PORTS=100

./collect_info.sh -o network-device-info.json
```

## Integration with Configuration Management

### Ansible

```yaml
- name: Collect system information with custom limits
  command: ./collect_info.sh -o {{ ansible_hostname }}-info.json
  environment:
    MAX_INTERFACES: "{{ max_interfaces | default(20) }}"
    MAX_ROUTES: "{{ max_routes | default(50) }}"
  args:
    chdir: /opt/automation_nation
```

### Docker

```dockerfile
FROM alpine:latest
RUN apk add --no-cache bash
COPY . /app
WORKDIR /app

# Set default configuration for container environment
ENV MAX_DOCKER_NETWORKS=20
ENV MAX_NETNS=50
ENV MAX_BRIDGES=30

CMD ["./collect_info.sh"]
```

### systemd Service

```ini
[Unit]
Description=System Information Collector
After=network.target

[Service]
Type=oneshot
ExecStart=/opt/automation_nation/collect_info.sh -o /var/log/system-info.json
Environment=MAX_INTERFACES=30
Environment=MAX_ROUTES=100
User=sysinfo
Group=sysinfo

[Install]
WantedBy=multi-user.target
```

## Performance Considerations

### Memory Usage

Each plugin's memory usage scales with the configured limits:

- **Network interfaces**: ~200 bytes per interface
- **Routes**: ~150 bytes per route entry
- **ARP entries**: ~100 bytes per entry
- **Listening ports**: ~80 bytes per port

### Execution Time

Plugin execution time is generally linear with configured limits:

- **Network discovery**: ~10ms per interface
- **Route parsing**: ~1ms per route
- **ARP table parsing**: ~0.5ms per entry

### Recommendations

1. **Start with defaults**: The default limits work well for most systems
2. **Monitor performance**: Use `time ./collect_info.sh` to measure execution time
3. **Adjust incrementally**: Increase limits gradually based on actual needs
4. **Consider system type**: Match configuration to system role (container host, network appliance, etc.)

## Troubleshooting

### Common Issues

1. **"No output from plugin"**: Check if limits are set too low
2. **Slow execution**: Reduce limits for better performance  
3. **Missing data**: Increase relevant limits or check for missing dependencies

### Debug Mode

Set plugin-specific debug mode:

```bash
DEBUG=1 ./collect_info.sh
```

This enables verbose logging from plugins that support debug mode.

### Validation

Test configuration before deployment:

```bash
# Dry run with verbose output
./collect_info.sh | python3 -m json.tool | wc -l
```

This shows the total number of lines in the JSON output, which correlates with data volume.