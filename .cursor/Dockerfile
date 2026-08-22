FROM nvidia/cuda:12.6.3-devel-ubuntu24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        curl \
        git \
        libclang-dev \
        libibverbs-dev \
        libnl-3-dev \
        libnl-route-3-dev \
        libnuma-dev \
        librdmacm-dev \
        ninja-build \
        rdma-core \
        pkg-config \
        python3 \
        python3-venv \
        unzip \
    && rm -rf /var/lib/apt/lists/*

# protoc >= 3.15 required for proto3 optional fields
RUN curl -fsSL -o /tmp/protoc.zip \
        https://github.com/protocolbuffers/protobuf/releases/download/v28.3/protoc-28.3-linux-x86_64.zip \
    && unzip -o /tmp/protoc.zip -d /usr/local 'bin/*' 'include/*' \
    && rm /tmp/protoc.zip

# uv package manager
RUN curl -LsSf https://astral.sh/uv/install.sh | sh \
    && mv /root/.local/bin/uv /usr/local/bin/uv \
    && mv /root/.local/bin/uvx /usr/local/bin/uvx

# Rust toolchain for Phoenix serving engine (edition 2024 requires Rust 1.85+)
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && . /root/.cargo/env \
    && rustup component add rustfmt clippy \
    && rustc --version

ENV PATH="/root/.cargo/bin:${PATH}"
ENV RUSTUP_HOME=/root/.rustup
ENV CARGO_HOME=/root/.cargo
ENV CC=gcc
ENV CXX=g++
ENV CARGO_TARGET_DIR=/tmp/phoenix-cargo
