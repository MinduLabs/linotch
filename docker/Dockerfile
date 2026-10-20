# Build environment for Linotch. Ubuntu 22.04 on purpose: Tauri's docs say to build on the oldest
# distro that still ships WebKitGTK 4.1, otherwise the AppImage/deb demand a newer glibc.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl wget xz-utils file rsync git build-essential pkg-config \
        libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

ARG NODE_VERSION=22.22.3
RUN curl -fsSL https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz | tar -xJ -C /usr/local --strip-components=1 \
    && corepack enable

ARG PNPM_VERSION=10.34.7
RUN corepack prepare pnpm@${PNPM_VERSION} --activate

# The crate uses edition 2024, so it needs a current stable Rust.
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal

# mksquashfs, to repack the AppImage with stronger compression (see build.sh). Own layer so the ones above stay cached.
RUN apt-get update && apt-get install -y --no-install-recommends squashfs-tools && rm -rf /var/lib/apt/lists/*

# linuxdeploy (used for the AppImage) is itself an AppImage; containers have no FUSE.
ENV APPIMAGE_EXTRACT_AND_RUN=1 CI=true
WORKDIR /work
