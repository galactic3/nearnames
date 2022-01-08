FROM rust:1.57.0

LABEL description="Container for builds"

RUN apt-get update && apt-get install -y \
    binaryen wabt \
    && rm -rf /var/lib/apt/lists/*

ARG USER_ID
ARG GROUP_ID

RUN addgroup --gid $GROUP_ID user
RUN adduser --disabled-password --gecos '' --uid $USER_ID --gid $GROUP_ID user
USER user

RUN rustup default 1.57.0
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-snip wasm-gc
