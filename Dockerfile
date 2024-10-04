# Rename containers to docker compatible naming
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl AS rust-musl-cross-amd64
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl AS rust-musl-cross-arm64

# Build program for target arch
ARG $TARGETARCH
FROM rust-musl-cross-$TARGETARCH AS builder
WORKDIR /usr/src/garage-door-rust/
COPY . .
RUN cargo build --release

# Copy into minimal image
FROM scratch
COPY --from=builder /usr/src/garage-door-rust/target/*-unknown-linux-musl/release/garage-door-rust /garage-door-rust
ENTRYPOINT ["/garage-door-rust"]
