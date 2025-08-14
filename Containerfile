# Containerfile for Automation Nation Web Application (Podman-optimized)
FROM registry.access.redhat.com/ubi9/ubi:latest as builder

# Install build dependencies
RUN dnf update -y && dnf install -y \
    curl \
    gcc \
    openssl-devel \
    sqlite-devel \
    pkg-config \
    && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create app user
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release --bin web_server --bin ci_runner

# Runtime stage
FROM registry.access.redhat.com/ubi9/ubi-minimal:latest

# Install runtime dependencies
RUN microdnf update -y && microdnf install -y \
    ca-certificates \
    curl \
    sqlite \
    bash \
    procps-ng \
    net-tools \
    iproute \
    pciutils \
    usbutils \
    && microdnf clean all

# Create app user
RUN useradd -m -u 1000 appuser

# Create directories
RUN mkdir -p /app/plugins /app/templates /app/data && \
    chown -R appuser:appuser /app

# Copy binaries from builder
COPY --from=builder /app/target/release/web_server /app/target/release/ci_runner /app/bin/

# Copy application files
COPY collect_info.sh /app/
COPY plugins/ /app/plugins/
COPY templates/ /app/templates/

# Make scripts executable
RUN chmod +x /app/collect_info.sh /app/plugins/*.sh

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV ENABLE_HASHING=1
ENV ENABLE_SUDO_SUPPORT=0

# Expose port
EXPOSE 3000

# Labels for better metadata
LABEL name="automation-nation" \
      version="1.0" \
      description="Automation Nation web application with system profiling" \
      maintainer="Automation Nation Team"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command
CMD ["/app/bin/web_server", "serve", "--host", "0.0.0.0", "--port", "3000"]