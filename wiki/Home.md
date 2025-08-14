# Automation Nation Wiki

Welcome to the comprehensive documentation for the Automation Nation platform - a sophisticated automation platform for container deployment, system profiling, and infrastructure management.

## 🏠 Quick Navigation

### Getting Started
- **[Installation Guide](Installation-Guide)** - Complete setup instructions for all deployment methods
- **[Quick Start](Quick-Start)** - Get up and running in under 5 minutes
- **[Architecture Overview](Architecture-Overview)** - Understanding the platform design
- **[Configuration](Configuration)** - Environment variables and settings

### User Guides
- **[Web Interface Guide](Web-Interface-Guide)** - Using the web dashboard and API
- **[Container Management](Container-Management)** - Deploy and manage containers
- **[System Profiling](System-Profiling)** - Collect and analyze system information
- **[Monitoring Setup](Monitoring-Setup)** - Configure observability stack

### Development
- **[Developer Guide](Developer-Guide)** - Building and contributing to the project
- **[API Reference](API-Reference)** - Complete API documentation
- **[Plugin Development](Plugin-Development)** - Creating custom system info plugins
- **[Testing Guide](Testing-Guide)** - Running and writing tests

### Operations
- **[Deployment Guide](Deployment-Guide)** - Production deployment strategies
- **[Security Guide](Security-Guide)** - Security considerations and best practices
- **[Troubleshooting](Troubleshooting)** - Common issues and solutions
- **[Performance Tuning](Performance-Tuning)** - Optimization and scaling

### Reference
- **[CLI Reference](CLI-Reference)** - Command-line interface documentation
- **[Environment Variables](Environment-Variables)** - Complete configuration reference
- **[Plugin Reference](Plugin-Reference)** - Built-in plugin documentation
- **[Glossary](Glossary)** - Terms and definitions

## 🚀 What is Automation Nation?

Automation Nation is a comprehensive automation platform that combines:

- **🔍 System Profiling**: Multi-architecture system information collection
- **🐳 Container Orchestration**: Docker, Podman, and LXC management
- **🌐 Web Interface**: RESTful API and web-based management
- **📊 Monitoring Stack**: Complete observability with ELK + Prometheus/Grafana
- **🔧 Infrastructure Management**: NetBox integration for network documentation
- **🚀 CI/CD Integration**: Automated testing and release management

## 🏗️ Platform Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Automation Nation Platform                   │
├─────────────────────────────────────────────────────────────────┤
│  Web Interface (Rust + Axum)                                   │
│  ├── REST API for container deployment                         │
│  ├── GitHub API integration                                    │
│  ├── System profiling endpoints                                │
│  └── Container runtime management                              │
├─────────────────────────────────────────────────────────────────┤
│  Container Orchestration Layer                                 │
│  ├── Docker Manager                                            │
│  ├── Podman Manager                                            │
│  ├── LXC Manager                                               │
│  └── Runtime Detection & Optimization                          │
├─────────────────────────────────────────────────────────────────┤
│  System Information Collection (collect_info.sh)              │
│  ├── Plugin-based architecture                                 │
│  ├── Multi-architecture support (10 CPU architectures)        │
│  ├── JSON output with integrity verification                   │
│  └── Cross-platform Unix compatibility                         │
├─────────────────────────────────────────────────────────────────┤
│  Monitoring & Observability Stack                              │
│  ├── ELK Stack (Elasticsearch, Logstash, Kibana)             │
│  ├── Prometheus + Grafana                                      │
│  ├── Application metrics and logging                           │
│  └── Container and infrastructure monitoring                   │
├─────────────────────────────────────────────────────────────────┤
│  Infrastructure Management                                      │
│  ├── NetBox (Network infrastructure database)                  │
│  ├── PostgreSQL (Application database)                         │
│  ├── Redis (Caching and session storage)                       │
│  └── Automated backup and maintenance                          │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Use Cases

### Infrastructure Automation
- Automated container deployment based on repository analysis
- System profiling for capacity planning and optimization
- Network discovery and documentation via NetBox integration
- Multi-runtime container orchestration (Docker/Podman/LXC)

### DevOps Integration
- CI/CD pipeline integration with automated testing
- GitHub repository analysis and deployment suggestions
- Monitoring and alerting for deployed applications
- Release management with automated binary builds

### System Administration
- Cross-platform system information collection (10 architectures)
- Network topology discovery and LLDP neighbor detection
- Performance monitoring and resource utilization tracking
- Security analysis and compliance reporting

## 📚 Documentation Structure

This wiki is organized into logical sections:

### **Getting Started** - New user onboarding
Everything you need to get the platform running in your environment.

### **User Guides** - Feature documentation
Detailed guides for using each major platform feature.

### **Development** - Contributing and extending
Information for developers working on or extending the platform.

### **Operations** - Production deployment
Guides for running the platform in production environments.

### **Reference** - Complete documentation
Comprehensive reference materials for all platform components.

## 🤝 Contributing

We welcome contributions! See the [Developer Guide](Developer-Guide) for:
- Development environment setup
- Coding standards and practices
- Pull request process
- Testing requirements

## 📞 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/nullroute-commits/Automation_nation/issues)
- **Discussions**: [Community discussions and Q&A](https://github.com/nullroute-commits/Automation_nation/discussions)
- **Documentation**: Start with the [Troubleshooting](Troubleshooting) guide

## 📄 License

This project is licensed under the MIT License - see the main repository for details.

---

**Last Updated**: January 2025  
**Platform Version**: Latest  
**Documentation Version**: 1.0