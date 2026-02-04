# Multi-stage Dockerfile for Kowalski RLM
# Build stage: Compile Rust code
FROM rust:1.70 AS builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY kowalski-core kowalski-core/
COPY kowalski-code-agent kowalski-code-agent/
COPY kowalski-federation kowalski-federation/
COPY kowalski-agent-template kowalski-agent-template/
COPY kowalski-memory kowalski-memory/
COPY kowalski-tools kowalski-tools/
COPY kowalski-rlm kowalski-rlm/
COPY kowalski-cli kowalski-cli/

# Build the CLI binary with optimizations
RUN cargo build --release --package kowalski-cli

# Runtime stage: Minimal image with just the binary and runtime dependencies
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/kowalski-cli /usr/local/bin/kowalski

# Create non-root user for security
RUN useradd -m -u 1000 rlm && \
    mkdir -p /app /data && \
    chown -R rlm:rlm /app /data

USER rlm
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default configuration
ENV RUST_LOG=info
ENV KOWALSKI_HOST=0.0.0.0
ENV KOWALSKI_PORT=8080

# Expose API port
EXPOSE 8080

# Default command
CMD ["kowalski", "serve"]
