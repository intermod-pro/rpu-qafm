FROM rust:1.81-slim

RUN \
  rustup target add armv7r-none-eabihf && \
  rustup component add clippy

RUN \
  mkdir -p /root/workspace
WORKDIR /root/workspace

CMD bash
