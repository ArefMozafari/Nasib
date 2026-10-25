# The Linux build and test environment. Ubuntu 22.04 is the oldest target, so binaries built here run on
# every newer target distro (glibc 2.35).
FROM ubuntu:22.04

ARG RUST_VERSION=1.99.1
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl pkg-config \
        libgtk-4-dev libdbus-1-dev \
        xvfb xauth dbus-x11 rpm \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION}
