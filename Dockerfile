# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

WORKDIR /build

# Install OpenSSL dev headers (needed by some crates)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies first
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core/engine/Cargo.toml              core/engine/Cargo.toml
COPY core/plugin-manager/Cargo.toml      core/plugin-manager/Cargo.toml
COPY core/configuration-manager/Cargo.toml core/configuration-manager/Cargo.toml
COPY core/permission-manager/Cargo.toml  core/permission-manager/Cargo.toml
COPY core/api-gateway/Cargo.toml         core/api-gateway/Cargo.toml
COPY server/Cargo.toml                   server/Cargo.toml

# Create stub lib.rs / main.rs so cargo can build the dependency layer
RUN mkdir -p core/engine/src && echo "pub fn _stub(){}" > core/engine/src/lib.rs \
 && mkdir -p core/plugin-manager/src && echo "pub fn _stub(){}" > core/plugin-manager/src/lib.rs \
 && mkdir -p core/configuration-manager/src && echo "pub fn _stub(){}" > core/configuration-manager/src/lib.rs \
 && mkdir -p core/permission-manager/src && echo "pub fn _stub(){}" > core/permission-manager/src/lib.rs \
 && mkdir -p core/api-gateway/src && echo "pub fn _stub(){}" > core/api-gateway/src/lib.rs \
 && mkdir -p server/src && echo "fn main(){}" > server/src/main.rs

RUN cargo build --release --package aio-server 2>/dev/null; exit 0

# Now copy real source
COPY core/ core/
COPY server/ server/

# Bust the cache for real crates
RUN touch core/engine/src/lib.rs \
         core/plugin-manager/src/lib.rs \
         core/configuration-manager/src/lib.rs \
         core/permission-manager/src/lib.rs \
         core/api-gateway/src/lib.rs \
         server/src/main.rs

RUN cargo build --release --package aio-server

# ── Stage 2: Runtime image ────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/aio-server /app/aio-server

RUN chmod +x /app/aio-server

# Railway injects $PORT automatically; we default to 3000
ENV PORT=3000
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=5s --retries=3 \
  CMD curl -f http://localhost:${PORT}/health || exit 1

ENTRYPOINT ["/app/aio-server"]
