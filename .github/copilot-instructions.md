# GitHub Copilot Instructions for Automation Nation

## Project Overview

Automation Nation is a comprehensive automation platform for container deployment, system profiling, and infrastructure management. The project combines:

- **Rust Applications**: High-performance web server (Axum), CI test suite, system profilers, and container managers
- **Bash Scripts**: System information collection plugins and orchestration scripts
- **Container Orchestration**: Multi-runtime support (Docker, Podman, LXC)
- **Monitoring Stack**: Integration with ELK, Prometheus, and Grafana
- **Infrastructure Management**: NetBox integration for network infrastructure tracking

### Technology Stack
- **Primary Languages**: Rust (2021 edition), Bash/Shell scripting
- **Web Framework**: Axum 0.7 with Tower middleware
- **Database**: SQLx with PostgreSQL and SQLite support
- **Container Runtimes**: Docker, Podman, LXC
- **Build System**: Cargo with custom release profiles

## Architecture

### Core Components

1. **Web Server** (`src/bin/web_server.rs`): RESTful API for container deployment and management
2. **System Profiler** (`src/system_profiler.rs`): Multi-architecture system information collection
3. **Container Managers**: Docker, Podman, and LXC orchestration modules
4. **Plugin System** (`plugins/*.sh`): Numbered bash scripts for system information gathering
5. **CI Test Suite** (`src/bin/ci_runner.rs`): Comprehensive testing framework

### Multi-Architecture Support

The platform supports 10 CPU architectures:
- x86_64 (Intel/AMD 64-bit)
- arm64/aarch64 (ARM 64-bit, Apple Silicon, AWS Graviton)
- aarch32 (ARM 32-bit)
- i386 (x86 32-bit legacy)
- ppc64le (IBM POWER)
- s390x (IBM Z mainframes)
- riscv64 (RISC-V 64-bit)
- mips64 (MIPS 64-bit)
- sparc64 (Oracle SPARC)
- loongarch64 (Chinese LoongArch)

## Coding Standards

### Rust Code

1. **Style**:
   - Follow standard Rust formatting (`cargo fmt`)
   - Use `clippy` for linting (`cargo clippy`)
   - Edition: 2021
   - Prefer explicit error handling with `anyhow` or `Result` types

2. **Dependencies**:
   - Web: `axum`, `tower`, `tower-http`
   - Async: `tokio` with full features
   - Serialization: `serde`, `serde_json`
   - Database: `sqlx` with runtime-tokio-rustls
   - HTTP Client: `reqwest` for GitHub API integration

3. **Error Handling**:
   - Use `anyhow::Result` for application-level errors
   - Implement proper error context with `.context()`
   - Log errors appropriately with `log` crate

4. **Async/Await**:
   - All I/O operations should be async
   - Use Tokio runtime consistently
   - Prefer structured concurrency patterns

### Bash/Shell Scripts

1. **Plugin System**:
   - Plugins are numbered: `NN_name.sh` (e.g., `10_os_info.sh`, `20_hardware_info.sh`)
   - Lower numbers execute first (10, 20, 30...)
   - Each plugin must output valid JSON
   - Use consistent error handling and privilege checking

2. **Standards**:
   - POSIX-compliant where possible
   - Use `#!/usr/bin/env bash` shebang
   - Include error handling: `set -e` for critical scripts
   - Quote variables to prevent word splitting
   - Use `$(command)` instead of backticks

3. **Output Format**:
   - JSON output for structured data
   - Use `jq` for JSON manipulation when available
   - Include error messages in JSON format with proper structure

## Testing Guidelines

### Rust Tests

1. **Unit Tests**:
   ```bash
   cargo test --lib
   ```

2. **Integration Tests**:
   ```bash
   cargo test --bins
   ```

3. **All Tests**:
   ```bash
   cargo test
   ```

4. **Test Organization**:
   - Unit tests in same file as code with `#[cfg(test)]` module
   - Integration tests use `dev-dependencies`: `tokio-test`, `pretty_assertions`, `axum-test`
   - Use `tempfile` for file system tests

### Bash Tests

1. **CI Test Suite**:
   ```bash
   ./comprehensive_test_suite.sh
   ```

2. **Script Validation**:
   - Use `shellcheck` for static analysis
   - Test on multiple distributions (Ubuntu, Alpine, Debian)
   - Verify privilege escalation handling

3. **Plugin Testing**:
   - Each plugin should be testable independently
   - Validate JSON output format
   - Test across different architectures when possible

### Build Commands

1. **Development Build**:
   ```bash
   cargo build
   ```

2. **Release Build**:
   ```bash
   cargo build --release
   ```

3. **Optimized Builds** (for production):
   ```bash
   cargo build --profile release-lto  # Link-Time Optimization
   cargo build --profile release-size # Size-optimized
   ```

## Security Guidelines

1. **Privilege Escalation**:
   - Scripts must check for root/sudo requirements
   - Use `check_root_or_sudo()` pattern in bash scripts
   - Never assume elevated privileges

2. **Input Validation**:
   - Sanitize all user inputs
   - Validate JSON schemas
   - Use parameterized queries for database operations

3. **Secrets Management**:
   - Never commit secrets or tokens
   - Use environment variables or `.env` files (see `.env.template`)
   - GitHub tokens should be passed via environment variables

4. **Container Security**:
   - Run containers with least privilege
   - Validate container images before deployment
   - Use security contexts appropriately

## Best Practices

### Plugin Development

1. **Naming Convention**: `[NN]_[descriptive_name].sh`
   - 10-19: OS and distribution info
   - 20-29: Hardware and virtualization info
   - 30-39: Network information
   - 40-49: Packages and executables
   - 50-59: System state and uptime

2. **Structure**:
   ```bash
   #!/usr/bin/env bash
   # Description of what the plugin does
   
   # Check privileges if needed
   # Detect architecture/OS
   # Collect information
   # Output JSON to stdout
   ```

3. **JSON Output Format**:
   - Use consistent key naming (snake_case)
   - Include metadata: timestamp, architecture, hostname
   - Validate JSON with `jq` before output

### Rust Module Organization

1. **Binary Crates** (`src/bin/`):
   - `ci_runner.rs`: CI/CD test execution
   - `web_server.rs`: HTTP API server
   - `comprehensive_test_runner.rs`: Full test suite
   - `precompiled_builder.rs`: Release artifact builder

2. **Library Modules**:
   - Keep modules focused on single responsibility
   - Use public API (`pub`) judiciously
   - Document public interfaces with doc comments

3. **Dependencies**:
   - Add only necessary dependencies
   - Keep versions up-to-date but stable
   - Prefer well-maintained crates

## Database Schema

- Uses SQLx with PostgreSQL for primary data
- SQLite for development/testing
- Migrations in `migrations/` directory
- Run migrations: `sqlx migrate run`

## CI/CD Integration

### GitHub Actions Workflows

1. **CI Test Suite** (`.github/workflows/ci_test_suite.yml`):
   - Rust unit tests
   - Script validation with shellcheck
   - Privilege escalation tests
   - Full integration suite

2. **Release Workflow** (`.github/workflows/release.yml`):
   - Automated versioning
   - Multi-architecture builds
   - Docker image publishing
   - Release artifact generation

### Running CI Locally

```bash
# Run Rust tests
cargo test

# Run bash script validation
shellcheck *.sh plugins/*.sh

# Run full test suite
./comprehensive_test_suite.sh --profile dev
```

## Development Workflow

1. **Setup**:
   ```bash
   # Install Rust
   # Download the installer script and inspect it before running for security.
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init.sh
   less rustup-init.sh   # <-- Review the script before executing!
   sh rustup-init.sh
   
   # Install dependencies
   cargo build
   
   # Copy environment template
   cp .env.template .env
   ```

2. **Running Locally**:
   ```bash
   # Web server
   cargo run --bin web_server
   
   # System profiler
   ./collect_info.sh
   
   # CI test suite
   cargo run --bin ci_runner
   ```

3. **Testing Changes**:
   - Write tests for new functionality
   - Run `cargo test` before committing
   - Use `cargo clippy` to catch common issues
   - Run `cargo fmt` to format code

## Container Deployment

### Supported Runtimes

1. **Docker**: Primary container runtime
2. **Podman**: Daemonless alternative
3. **LXC**: System containers

### Docker Compose

- `docker-compose.yml`: Standard deployment
- `docker-compose.swarm.yml`: Docker Swarm configuration

## Monitoring and Observability

- ELK Stack for log aggregation
- Prometheus for metrics
- Grafana for visualization
- Application metrics exported via web server

## Documentation

- **README.md**: User-facing documentation
- **TECHNICAL.md**: Implementation details
- **COMPREHENSIVE_ARCHITECTURE_DOCUMENTATION.md**: Full architecture guide
- **CONFIGURATION.md**: Configuration options
- **DEVELOPMENT_AND_DEPLOYMENT_PROCESSES.md**: Deployment procedures

## Common Tasks

### Adding a New Plugin

1. Create `plugins/[NN]_[name].sh` with appropriate number
2. Follow JSON output format
3. Test independently: `bash plugins/[NN]_[name].sh | jq`
4. Test with main script: `./collect_info.sh`

### Adding a New API Endpoint

1. Add route in `src/bin/web_server.rs`
2. Implement handler function
3. Add tests in `src/web_test_suite.rs`
4. Update API documentation

### Adding a Database Migration

1. Create migration: `sqlx migrate add [name]`
2. Write up/down SQL in `migrations/`
3. Test migration: `sqlx migrate run`
4. Update schema documentation

## Performance Considerations

- Use async I/O for all network and file operations
- Implement caching where appropriate
- Consider memory usage for large data structures
- Profile code with `cargo bench` or `perf` when optimizing
- Use optimized release profiles for production

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Documentation](https://docs.rs/axum/)
- [Bash Reference Manual](https://www.gnu.org/software/bash/manual/)
- [Advanced Bash-Scripting Guide](https://tldp.org/LDP/abs/html/)
