---
name: Architecture support
about: Request support for a new CPU architecture
title: '[ARCH] Add support for '
labels: 'architecture, enhancement'
assignees: ''

---

**Architecture Details**
- **Architecture Name**: [e.g., riscv32, m68k, alpha]
- **Common Names/Aliases**: [Alternative names for this architecture]
- **Bit Width**: [32-bit/64-bit/Other]
- **Endianness**: [Little/Big/Bi-endian]

**Detection Information**
- **uname -m output**: [What does `uname -m` return on this architecture?]
- **Alternative detection**: [Any other ways to detect this architecture]
- **Sample system**: [Where can this architecture be found/tested?]

**Market Relevance**
- **Use Cases**: [Where is this architecture commonly used?]
- **Market Adoption**: [Current and projected usage]
- **Priority Level**: [High/Medium/Low and why]

**System Information Sources**
How should system information be collected on this architecture?

**CPU Information**:
- **Source**: [/proc/cpuinfo fields, specific commands]
- **Special Handling**: [Any architecture-specific parsing needed]

**Memory Information**:
- **Source**: [Standard /proc/meminfo or different approach]
- **Considerations**: [Memory layout differences, special cases]

**Hardware Information**:
- **Disk**: [Standard df output or different tools needed]
- **Network**: [Standard tools available or architecture-specific]

**Architecture-Specific Considerations**
- **Special Requirements**: [Any unique aspects of this architecture]
- **Tool Availability**: [Which standard tools are/aren't available]
- **Performance Considerations**: [Any performance-specific needs]
- **Compatibility Issues**: [Known issues with standard approaches]

**Testing Environment**
- **Availability**: [Do you have access to test systems?]
- **Emulation**: [Can this be tested via emulation?]
- **Sample Output**: [Can you provide sample system information?]

**Implementation Priority**
Why should this architecture be prioritized?
- [ ] High market adoption
- [ ] Growing use in specific domains
- [ ] Critical business need
- [ ] Educational/research importance
- [ ] Personal/hobbyist interest

**Sample Data**
If you have access to a system with this architecture, please provide:
```bash
# uname -a output
[paste here]

# uname -m output
[paste here]

# Sample /proc/cpuinfo (first few lines)
[paste here]

# Available system tools
which ip ifconfig ss netstat lscpu dmidecode
```

**Additional Context**
Any other relevant information about this architecture or its requirements.