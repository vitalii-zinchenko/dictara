# CI image for Linux Rust builds (clippy/tests) of the Tauri backend.
#
# Rust itself is deliberately NOT baked in here — it's installed per-job via
# dtolnay/rust-toolchain, so toolchain/version changes in the workflow don't
# require rebuilding this image. Only the system/apt dependency list below
# should trigger a rebuild.
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    wget \
    file \
    git \
    build-essential \
    pkg-config \
    clang \
    libclang-dev \
    cmake \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libasound2-dev \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*
