# Multi-stage build for Layer7 Evolution System

# Build stage
FROM rust:1.75-slim as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy layer7 source
COPY src/layer7 ./src/layer7

# Build the application
RUN cargo build --release --package chimera-layer7

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false layer7

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chimera-layer7 /usr/local/bin/layer7

# Copy configuration files
COPY configs/ /app/configs/

# Copy monitoring configurations
COPY src/layer7/monitoring-config.yaml /app/configs/
COPY src/layer7/alerting-rules.yaml /app/configs/

# Set ownership
RUN chown -R layer7:layer7 /app

# Switch to non-root user
USER layer7

# Expose ports
EXPOSE 8080  # Health/API
EXPOSE 9090  # Metrics

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["layer7", "--config", "/app/configs/layer7.toml"]