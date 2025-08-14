---
name: Performance issue
about: Report performance problems or suggest optimizations
title: '[PERF] '
labels: 'performance, optimization'
assignees: ''

---

**Performance Issue Type**
- [ ] Slow execution time
- [ ] High memory usage
- [ ] High CPU usage
- [ ] Network-related delays
- [ ] Large output size
- [ ] Inefficient algorithms
- [ ] Other: [specify]

**Environment Details**
- **System**: [Hardware specs, VM, container]
- **OS**: [Operating system and version]
- **Architecture**: [CPU architecture]
- **Scale**: [Number of interfaces, routes, packages, etc.]

**Performance Metrics**
Please provide measurements if available:

**Execution Time**:
```bash
time ./collect_info.sh
# Output timing results here
```

**Memory Usage**:
```bash
/usr/bin/time -v ./collect_info.sh
# Memory usage stats here
```

**Output Size**:
```bash
./collect_info.sh | wc -c
# Size in bytes
```

**Specific Plugin Performance**:
If the issue is with a specific plugin:
```bash
time ./plugins/PLUGIN_NAME.sh x86_64
# Plugin-specific timing
```

**Problem Description**
Describe the performance issue:
- **Expected**: What performance did you expect?
- **Actual**: What performance are you seeing?
- **Impact**: How does this affect your use case?

**System Scale**
Please provide relevant scale information:
- **Network Interfaces**: [Number of interfaces on the system]
- **Routing Table**: [Number of routes: `ip route | wc -l`]
- **Installed Packages**: [Number of packages: `dpkg -l | wc -l` or equivalent]
- **Running Services**: [Number of listening ports: `ss -tuln | wc -l`]

**Environment Variables**
Current limits (if any):
```bash
echo "MAX_INTERFACES=${MAX_INTERFACES:-20}"
echo "MAX_ROUTES=${MAX_ROUTES:-50}"
echo "MAX_PACKAGES=${MAX_PACKAGES:-30}"
# Add other relevant limits
```

**Proposed Solution**
If you have ideas for optimization:
- **Approach**: [Algorithm improvement, caching, limiting, etc.]
- **Implementation**: [Specific code changes or approaches]
- **Trade-offs**: [What might be sacrificed for performance]

**Profiling Data**
If you've done any profiling:
```bash
# Commands used for profiling
strace -c ./collect_info.sh 2>&1 | head -20

# Or other profiling tools
```

**Use Case**
- **Frequency**: [How often do you run the collector?]
- **Automation**: [Is this part of automated monitoring?]
- **Real-time Requirements**: [Time constraints for collection]
- **Resource Constraints**: [Limited CPU, memory, or other resources]

**Workarounds**
Any current workarounds you're using:
- Environment variable limits
- Plugin disabling
- Custom modifications

**Additional Context**
Any other relevant performance information or requirements.