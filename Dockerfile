FROM rust:1.66.0-slim-bullseye as builder
WORKDIR /root

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    cmake \
    curl \
    libssl-dev \
    pkg-config \
    python3 \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target install wasm32-wasi

COPY Cargo.* .
COPY Makefile Makefile
COPY ./crates ./crates
RUN make cli

FROM debian:bullseye-slim
COPY --from=builder /root/target/release/javy /usr/local/bin
