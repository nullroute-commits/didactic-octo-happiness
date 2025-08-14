# Rust CI Test Suite for collect_info.sh

This Rust-based CI test suite provides comprehensive testing infrastructure for the `collect_info.sh` script, supporting both privileged and non-privileged execution across multiple Unix operating systems and architectures.

## Features

- **Memory-Safe Rust Implementation**: Built with latest stable Rust for maximum safety and performance
- **Privilege Escalation Testing**: Tests both normal and sudo-escalated execution modes
- **Cross-Platform Support**: Tests across top 3 Unix OS for FY 2025 Q1 (Ubuntu, Alpine, Rocky)
- **Architecture Testing**: Supports all 10 architectures from the main script
- **Comprehensive Reporting**: Generates detailed JSON, CSV, and Markdown reports
- **Regression Detection**: Compares outputs across different test runs
- **Performance Analysis**: Measures execution time and privilege impact
- **CI/CD Integration**: GitHub Actions workflow for automated testing

## Requirements

- Rust 1.88.0+ (latest stable)
- Bash shell
- sudo access (optional, for privilege testing)
- Docker (optional, for cross-OS testing)

## Quick Start

### Build the CI Suite

```bash
cargo build --release
```

### Run Basic Tests

```bash
# Show system information and capabilities
cargo run --bin ci_runner -- info

# Validate script output format
cargo run --bin ci_runner -- validate

# Test privilege escalation
cargo run --bin ci_runner -- privilege --verbose
```

### Run Full Test Suite

```bash
# Development profile (current OS/arch only)
cargo run --bin ci_runner -- run --profile dev --verbose

# CI profile (comprehensive testing)
cargo run --bin ci_runner -- run --profile ci --verbose

# Custom configuration
cargo run --bin ci_runner -- run \
  --architectures x86_64,arm64 \
  --operating-systems ubuntu,alpine \
  --parallel \
  --verbose
```

## Test Profiles

### Development Profile (`dev`)
- Tests only current architecture (x86_64)
- Tests only current OS (Ubuntu)
- Shorter timeouts
- Good for local development

### CI Profile (`ci`)
- Tests multiple architectures
- Tests top 3 Unix OS
- Longer timeouts
- Non-parallel execution for stability
- Suitable for CI environments

### Default Profile (`default`)
- Balanced configuration
- Can be customized via environment variables

## Architecture Support

The CI suite tests all architectures supported by `collect_info.sh`:

- x86_64 (AMD64)
- arm64 (aarch64)
- i386 (i686)
- ppc64le (PowerPC 64-bit LE)
- s390x (IBM Z/Architecture)
- riscv64 (RISC-V 64-bit)
- mips64 (MIPS 64-bit)
- aarch32 (ARM 32-bit)
- sparc64 (SPARC 64-bit)
- loongarch64 (LoongArch 64-bit)

## Operating System Support

Tests the top 3 Unix operating systems for fiscal year 2025 Q1:

1. **Ubuntu** - Most popular for containers/cloud
2. **Alpine** - Popular for containers due to small size
3. **Rocky** - Enterprise standard (successor to CentOS)

Additional supported OS:
- CentOS
- Debian

## Test Types

### Functional Testing
- Script execution validation
- JSON output format verification
- Plugin discovery and execution
- Error handling

### Integration Testing
- End-to-end workflow testing
- Cross-component interaction
- Configuration validation

### Regression Testing
- Output comparison between runs
- Data consistency verification
- Performance regression detection

### Privilege Testing
- Normal vs escalated execution comparison
- Performance impact analysis
- Data enhancement detection

## Configuration

### Environment Variables

```bash
# Script configuration
CI_SCRIPT_PATH="./collect_info.sh"
CI_TIMEOUT_SECONDS=300
CI_OUTPUT_DIR="./test_results"

# Execution configuration
CI_PARALLEL=false
CI_MAX_RETRIES=3

# Feature toggles
CI_ENABLE_REGRESSION=true
CI_ENABLE_PRIVILEGE_COMPARISON=true
```

### Command Line Options

```bash
# Test specific components
--architectures x86_64,arm64,i386
--operating-systems ubuntu,alpine,rocky

# Execution options
--parallel              # Enable parallel execution
--skip-privilege       # Skip privilege escalation tests
--verbose              # Enable verbose logging

# Output options
--output ./results     # Specify output directory
```

## Report Structure

The CI suite generates comprehensive reports:

### JSON Report (`test_report.json`)
- Complete test results with metadata
- Detailed comparison results
- Performance metrics
- Machine-readable format

### Markdown Summary (`test_summary.md`)
- Human-readable summary
- Test statistics
- Recommendations
- System information

### CSV Results (`test_results.csv`)
- Tabular test data
- Easy import into analysis tools
- OS/Architecture/Privilege breakdown

## GitHub Actions Integration

The included workflow (`.github/workflows/ci_test_suite.yml`) provides:

- **Automated Testing**: Runs on push, PR, and schedule
- **Matrix Testing**: Tests multiple profiles and configurations
- **Artifact Collection**: Saves detailed reports
- **PR Summaries**: Automatic test result summaries
- **Manual Triggers**: Workflow dispatch with options

### Workflow Jobs

1. **Build**: Rust compilation, formatting, linting
2. **Validation**: Script output format validation
3. **Privilege Testing**: Escalated vs normal execution
4. **Full Suite**: Complete test matrix
5. **Architecture Tests**: Per-architecture testing
6. **OS Tests**: Per-operating-system testing
7. **Reporting**: Comprehensive result aggregation

## Memory Safety

This implementation emphasizes memory safety through:

- **Rust Language**: Memory-safe by design
- **No Unsafe Code**: Zero `unsafe` blocks
- **Safe Concurrency**: Tokio async runtime
- **Input Validation**: Comprehensive parameter validation
- **Error Handling**: Explicit error propagation with `Result<T>`

## Performance Characteristics

- **Minimal Overhead**: Efficient script execution and monitoring
- **Concurrent Testing**: Optional parallel test execution
- **Streaming Logs**: Real-time test progress
- **Resource Management**: Automatic cleanup and timeout handling

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Specific test
cargo test test_privilege_analysis
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for unused dependencies
cargo machete
```

### Contributing

1. Follow Rust coding standards
2. Add tests for new functionality
3. Update documentation
4. Ensure CI passes

## Troubleshooting

### Common Issues

**Build Failures**
```bash
# Update Rust toolchain
rustup update stable

# Clear cache
cargo clean
```

**Permission Errors**
```bash
# Make scripts executable
chmod +x collect_info.sh plugins/*.sh

# Check sudo access
sudo -v
```

**Test Failures**
```bash
# Run with verbose logging
cargo run --bin ci_runner -- validate --verbose

# Check script manually
./collect_info.sh
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin ci_runner -- run --verbose

# Enable backtrace on panic
RUST_BACKTRACE=1 cargo run --bin ci_runner -- run
```

## Security Considerations

- **Privilege Isolation**: Tests run with minimal required privileges
- **Input Sanitization**: All user inputs are validated
- **Safe Execution**: No shell injection vulnerabilities
- **Audit Trail**: Comprehensive logging of all operations

## License

This CI test suite is provided under the same license as the main project.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CI Runner     │────│  Script         │────│  Output         │
│   (cli_runner)  │    │  Executor       │    │  Validator      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐             │
         │              │  Privilege      │             │
         └──────────────│  Manager        │─────────────┘
                        └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐             │
         │              │  OS Support     │             │
         └──────────────│  Manager        │─────────────┘
                        └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐             │
         │              │  Test           │             │
         └──────────────│  Reporter       │─────────────┘
                        └─────────────────┘
```

The architecture emphasizes:
- **Modularity**: Clear separation of concerns
- **Testability**: Each component is independently testable
- **Extensibility**: Easy to add new test types or OS support
- **Maintainability**: Clean interfaces and documentation