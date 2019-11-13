FROM rust:1.39.0 AS builder

RUN apt-get update \
    && apt-get --yes install --no-install-recommends \
      musl-tools \
    && rm -r /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.lock Cargo.toml /tmp/fqlib/
COPY src/ /tmp/fqlib/src/

RUN cargo build \
      --release \
      --manifest-path /tmp/fqlib/Cargo.toml \
      --target x86_64-unknown-linux-musl

FROM alpine:3.10

COPY --from=builder /tmp/fqlib/target/x86_64-unknown-linux-musl/release/fq /usr/local/bin/
