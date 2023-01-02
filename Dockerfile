FROM rust:1.64-slim as builder

RUN rustup target add wasm32-unknown-unknown

WORKDIR /build
RUN cargo init --lib --name envoy-wasm-filter
COPY Cargo.toml /build/

RUN cargo build --release --target wasm32-unknown-unknown
RUN rm src/*.rs && rm target/wasm32-unknown-unknown/release/envoy_wasm_filter* 

COPY src /build/src
COPY proto /build/proto
COPY build.rs /build/

RUN cargo build --release --target wasm32-unknown-unknown

# Manually build OCI Image.
# https://github.com/solo-io/wasm/blob/master/spec/spec-compat.md
FROM scratch

LABEL org.opencontainers.image.title envoy-wasm-filter
COPY --from=builder /build/target/wasm32-unknown-unknown/release/envoy_wasm_filter.wasm ./plugin.wasm
