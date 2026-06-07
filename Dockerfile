# Builder stage
FROM rust:1.88.0-slim AS builder

WORKDIR /app

# Install build dependencies (openssl-sys vendor need perl + gcc)
RUN apt-get update && apt-get install -y --no-install-recommends \
    perl gcc make pkg-config libc6-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy dependency files first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Create dummy source and build dependencies (cache efficient)
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/eel_bot.rs && \
    cargo build --release --bin eel_bot && \
    rm -rf src

# Copy source code and build the actual binary
COPY src ./src
COPY tests ./tests
COPY scripts ./scripts

RUN cargo build --release --bin eel_bot

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/eel_bot /usr/local/bin/

# Set entrypoint
entrypoint: ["sh", "-c"]
command: ["eel_bot --name eel-bot 2>&1 | tee -a /app/logs/bot.log"]
