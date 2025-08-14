---
name: Bug report
about: Report a bug in the system information collector
title: '[BUG] '
labels: 'bug'
assignees: ''

---

**Describe the bug**
A clear and concise description of what the bug is.

**Environment**
- **Operating System**: [e.g., Ubuntu 24.04, CentOS 7, macOS 14]
- **Architecture**: [e.g., x86_64, arm64, detected architecture if known]
- **Shell**: [e.g., bash 5.1, zsh]
- **System Type**: [e.g., bare metal, VM, container, WSL]

**Command and Output**
Please provide the exact command you ran and the output:

```bash
# Command that failed
./collect_info.sh

# Or with output file
./collect_info.sh -o output.json
```

**Error Output**
Please include any error messages from stderr:
```
[Paste error messages here]
```

**Expected vs Actual Behavior**
- **Expected**: What you expected to happen
- **Actual**: What actually happened

**Plugin-Specific Issues**
If the issue is with a specific plugin, please indicate:
- **Plugin**: [e.g., 10_os_info.sh, 31_network_stats.sh]
- **Manual Test**: Result of running the plugin directly:
  ```bash
  ./plugins/PLUGIN_NAME.sh x86_64
  ```

**JSON Output**
If applicable, please provide the JSON output (sanitized of sensitive information):
```json
{
  "sample": "output showing the issue"
}
```

**Additional Context**
- Missing system tools or commands
- Custom system configuration
- Network environment specifics
- Any other relevant details
