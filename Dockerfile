# Dockerfile for Automation Nation Web Application
# Updated Rust toolchain to 1.89 to support Cargo.lock v4
FROM rust:1.89-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

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
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    sqlite3 \
    bash \
    # Container runtime tools (for runtime detection) \
    podman \
    # System tools needed by collect_info.sh \
    procps \
    net-tools \
    iproute2 \
    lshw \
    pciutils \
    usbutils \
    lldpd \
    && rm -rf /var/lib/apt/lists/*

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

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command
CMD ["/app/bin/web_server", "serve", "--host", "0.0.0.0", "--port", "3000"]