FROM rust:1.64-slim

RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown

WORKDIR /build
RUN cargo init --lib --name envoy-wasm-filter
COPY Cargo.toml /build/

RUN cargo build --release --target wasm32-unknown-unknown
RUN rm src/*.rs && rm target/wasm32-unknown-unknown/release/envoy_wasm_filter* 

COPY src /build/src
COPY proto /build/proto
COPY build.rs /build/

RUN ls -al src

RUN cargo build --release --target wasm32-unknown-unknown
