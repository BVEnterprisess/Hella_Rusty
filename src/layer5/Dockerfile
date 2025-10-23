# Multi-stage build for Layer5 optimization engine

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

# Copy layer5 source
COPY src/layer5 ./src/layer5

# Build the application
RUN cargo build --release --package chimera-layer5

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false layer5

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chimera-layer5 /usr/local/bin/layer5

# Copy configuration files
COPY configs/ /app/configs/

# Set ownership
RUN chown -R layer5:layer5 /app

# Switch to non-root user
USER layer5

# Expose port for metrics
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

# Run the application
CMD ["layer5", "--config", "/app/configs/layer5.toml"]