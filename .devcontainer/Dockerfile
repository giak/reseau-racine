FROM mcr.microsoft.com/devcontainers/rust:1-bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libsodium-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
