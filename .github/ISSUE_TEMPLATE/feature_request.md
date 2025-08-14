---
name: Feature request
about: Suggest an idea for this project
title: '[FEATURE] '
labels: 'enhancement'
assignees: ''

---

**Feature Category**
Please check the type of feature you're requesting:
- [ ] New plugin for additional system information
- [ ] Enhancement to existing plugin
- [ ] New architecture support
- [ ] Output format improvement
- [ ] Performance optimization
- [ ] Integration with external tools
- [ ] Documentation improvement
- [ ] Other (please specify)

**Is your feature request related to a problem?**
A clear and concise description of what the problem is. Ex. "I need to collect [specific system information] but it's not currently supported..."

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**For New Plugin Requests**
If requesting a new plugin, please provide:
- **Plugin Purpose**: What system information should it collect?
- **Data Sources**: Where would this information come from? (e.g., /proc files, commands)
- **Architecture Support**: Which architectures need this information?
- **Sample Output**: What should the JSON output look like?
  ```json
  {
    "example_field": "example_value"
  }
  ```

**For Architecture Support**
If requesting support for a new architecture:
- **Architecture Name**: [e.g., riscv32, m68k]
- **Detection Method**: How is this architecture identified? (`uname -m` output)
- **Market Relevance**: Why is this architecture important?
- **Specific Considerations**: Any architecture-specific requirements?

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Implementation Considerations**
- **Complexity**: Simple/Medium/Complex
- **Dependencies**: Would this require new system tools or packages?
- **Backward Compatibility**: Any concerns about existing functionality?
- **Security Impact**: Any security considerations?

**Additional context**
Add any other context, links to documentation, or examples about the feature request here.
