# Stage 1: Build
FROM rust:1.95-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies — copy manifests first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release && rm -rf src target/release/deps/typeset*

# Build the actual binary
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

# Stage 2: Fetch Typst CLI
FROM debian:bookworm-slim AS typst-fetch
ARG TYPST_VERSION=0.13.0
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates xz-utils \
    && curl -fsSL "https://github.com/typst/typst/releases/download/v${TYPST_VERSION}/typst-x86_64-unknown-linux-musl.tar.xz" \
       -o /tmp/typst.tar.xz \
    && tar -xJf /tmp/typst.tar.xz -C /tmp \
    && mv /tmp/typst-x86_64-unknown-linux-musl/typst /usr/local/bin/typst \
    && chmod +x /usr/local/bin/typst \
    && rm -rf /tmp/typst*

# Stage 3: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl3 \
      fonts-dejavu \
      fonts-inter \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/typeset /usr/local/bin/typeset
COPY --from=typst-fetch /usr/local/bin/typst /usr/local/bin/typst
COPY migrations/ /app/migrations/

# Pre-cache the fletcher package so first render doesn't hit the network
ENV TYPST_PACKAGE_CACHE_PATH=/app/.typst-cache
RUN mkdir -p /app/.typst-cache && \
    printf '#import "@preview/fletcher:0.5.7"\n' > /tmp/cache.typ && \
    (cd /tmp && typst compile cache.typ /dev/null 2>/dev/null || true) && \
    rm -f /tmp/cache.typ

WORKDIR /app
ENV RUST_LOG=info

EXPOSE 3200

CMD ["typeset"]
