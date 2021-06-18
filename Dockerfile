# syntax=docker/dockerfile:1

FROM rust:1.53.0-buster AS builder

COPY .git /app/.git
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/

RUN cargo build --release --manifest-path /app/Cargo.toml

FROM debian:buster

COPY --from=builder /app/target/release/fq /usr/local/bin/
