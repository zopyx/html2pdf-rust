# Dockerfile for html2pdf-rs
# Supports multiple build stages for different use cases

# ============================================================================
# Build Stage
# ============================================================================
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/html2pdf

# Copy Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir -p src && \
    echo 'fn main() { println!("placeholder") }' > src/main.rs && \
    echo 'pub fn placeholder() {}' > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Touch files to force rebuild
RUN touch src/main.rs src/lib.rs

# Build the actual application
RUN cargo build --release && \
    strip target/release/html2pdf

# ============================================================================
# Runtime Stage - Minimal
# ============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fonts-liberation \
    fonts-dejavu \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r html2pdf && useradd -r -g html2pdf -s /bin/false html2pdf

# Copy binary from builder
COPY --from=builder /usr/src/html2pdf/target/release/html2pdf /usr/local/bin/html2pdf

# Set ownership
RUN chown html2pdf:html2pdf /usr/local/bin/html2pdf

# Switch to non-root user
USER html2pdf

# Set working directory
WORKDIR /workspace

# Entry point
ENTRYPOINT ["html2pdf"]
CMD ["--help"]

# ============================================================================
# Alpine Runtime Stage (smaller image)
# ============================================================================
FROM alpine:3.19 AS alpine-runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    font-liberation \
    font-dejavu

# Create non-root user
RUN addgroup -S html2pdf && adduser -S html2pdf -G html2pdf

# Copy binary from builder
COPY --from=builder /usr/src/html2pdf/target/release/html2pdf /usr/local/bin/html2pdf

# Switch to non-root user
USER html2pdf

WORKDIR /workspace

ENTRYPOINT ["html2pdf"]
CMD ["--help"]

# ============================================================================
# Development Stage (includes build tools)
# ============================================================================
FROM rust:1.75-bookworm AS dev

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    gdb \
    valgrind \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Install cargo tools
RUN cargo install cargo-watch cargo-audit cargo-outdated

# Keep container running for development
CMD ["sleep", "infinity"]

# ============================================================================
# Default target
# ============================================================================
FROM runtime
