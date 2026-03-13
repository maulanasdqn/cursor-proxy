FROM rust:1.92-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY cursor-proxy-errors/Cargo.toml cursor-proxy-errors/Cargo.toml
COPY cursor-proxy-types/Cargo.toml cursor-proxy-types/Cargo.toml
COPY cursor-proxy-agent/Cargo.toml cursor-proxy-agent/Cargo.toml
COPY cursor-proxy-models/Cargo.toml cursor-proxy-models/Cargo.toml
COPY cursor-proxy-auth/Cargo.toml cursor-proxy-auth/Cargo.toml
COPY cursor-proxy-openai/Cargo.toml cursor-proxy-openai/Cargo.toml
COPY cursor-proxy-anthropic/Cargo.toml cursor-proxy-anthropic/Cargo.toml
COPY cursor-proxy-server/Cargo.toml cursor-proxy-server/Cargo.toml
COPY cursor-proxy-test/Cargo.toml cursor-proxy-test/Cargo.toml

RUN mkdir -p cursor-proxy-errors/src cursor-proxy-types/src cursor-proxy-agent/src \
    cursor-proxy-models/src cursor-proxy-auth/src cursor-proxy-openai/src \
    cursor-proxy-anthropic/src cursor-proxy-server/src cursor-proxy-test/src && \
    echo "pub fn stub() {}" > cursor-proxy-errors/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-types/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-agent/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-models/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-auth/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-openai/src/lib.rs && \
    echo "pub fn stub() {}" > cursor-proxy-anthropic/src/lib.rs && \
    echo "fn main() {}" > cursor-proxy-server/src/main.rs && \
    echo "pub fn stub() {}" > cursor-proxy-test/src/lib.rs

RUN cargo build --release -p cursor-proxy-server 2>/dev/null || true

COPY . .

RUN touch cursor-proxy-errors/src/lib.rs cursor-proxy-types/src/lib.rs \
    cursor-proxy-agent/src/lib.rs cursor-proxy-models/src/lib.rs \
    cursor-proxy-auth/src/lib.rs cursor-proxy-openai/src/lib.rs \
    cursor-proxy-anthropic/src/lib.rs cursor-proxy-server/src/main.rs

RUN cargo build --release -p cursor-proxy-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cursor-proxy-server /usr/local/bin/cursor-proxy-server

ENV CURSOR_BRIDGE_HOST=0.0.0.0
ENV CURSOR_BRIDGE_PORT=8765

EXPOSE 8765

ENTRYPOINT ["cursor-proxy-server"]
