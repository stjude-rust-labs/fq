FROM rust:1.40.0-buster AS builder

RUN apt-get update \
      && apt-get --yes install --no-install-recommends \
        musl-tools \
      && rm -r /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY .git /app/.git
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/

RUN cargo build \
      --release \
      --manifest-path /app/Cargo.toml \
      --target x86_64-unknown-linux-musl

FROM alpine:3.11

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/fq /usr/local/bin/
