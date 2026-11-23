# ==============================================================================
# Infocar Monolith — Multi-Stage Rust Dockerfile
# ==============================================================================

# Stage 1: Build environment
FROM rust:1.88-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies if needed
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies layer
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && \
    echo 'fn main() { println!("dependency cache"); }' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code and templates
COPY src ./src
COPY templates ./templates

# Force rebuild of application crate
RUN touch src/main.rs && cargo build --release

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /var/www/html
RUN mkdir -p /var/www/html/storage/logs

COPY --from=builder /app/target/release/infocar-monolith /usr/local/bin/infocar-monolith

EXPOSE 8000

CMD ["infocar-monolith"]
