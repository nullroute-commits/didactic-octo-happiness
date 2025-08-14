---
name: Plugin development
about: Report issues or ask questions about plugin development
title: '[PLUGIN] '
labels: 'plugin, help wanted'
assignees: ''

---

**Plugin Information**
- **Plugin Name**: [e.g., 60_custom_info.sh]
- **Purpose**: [Brief description of what the plugin collects]
- **Status**: [Planning/In Development/Testing/Complete]

**Development Question/Issue**
A clear description of what you need help with or what issue you're encountering.

**Plugin Code**
If you have plugin code to share or review:
```bash
#!/bin/bash
# Your plugin code here
```

**Architecture Support**
Which architectures does your plugin support or need to support?
- [ ] x86_64
- [ ] arm64
- [ ] i386
- [ ] ppc64le
- [ ] s390x
- [ ] riscv64
- [ ] mips64
- [ ] aarch32
- [ ] sparc64
- [ ] loongarch64

**Expected JSON Output**
What should your plugin output look like?
```json
{
  "your_field": "example_value",
  "another_field": "example"
}
```

**Testing Results**
If you've tested your plugin:
- **Test Command**: `./plugins/your_plugin.sh x86_64`
- **Test Results**: [Working/Error/Partial]
- **Error Messages**: [If any]

**Dependencies**
What system tools or files does your plugin require?
- Commands: [e.g., lscpu, dmidecode]
- Files: [e.g., /proc/version, /sys/class/net]
- Packages: [e.g., util-linux, pciutils]

**Questions**
- [ ] How do I handle missing dependencies?
- [ ] How do I test across different architectures?
- [ ] How do I handle errors gracefully?
- [ ] How do I format the JSON output properly?
- [ ] Other: [Please specify]

**Additional Context**
Any other information about your plugin development needs.