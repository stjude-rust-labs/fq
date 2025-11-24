# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.91.1
ARG DEBIAN_CODENAME=bookworm

FROM rust:${RUST_VERSION}-${DEBIAN_CODENAME} AS builder

COPY .git /app/.git
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/

RUN cargo build --release --manifest-path /app/Cargo.toml

FROM debian:${DEBIAN_CODENAME}

COPY --from=builder /app/target/release/fq /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/fq"]
