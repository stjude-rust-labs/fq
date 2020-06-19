FROM rust:1.44.1-buster AS builder

COPY .git /app/.git
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/

RUN cargo build --release --manifest-path /app/Cargo.toml

FROM debian:buster-slim

COPY --from=builder /app/target/release/fq /usr/local/bin/
