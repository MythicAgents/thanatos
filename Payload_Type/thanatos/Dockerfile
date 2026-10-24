# Base image for all other multi-stage images
FROM docker.io/library/debian:13 AS base

# Base image with updated apt cache
FROM base AS base-apt
RUN apt-get update -y

# Rustup image
FROM base-apt AS rustup
RUN apt-get install -y curl
ADD https://sh.rustup.rs /tmp/rustup.sh
RUN <<EOF
sh /tmp/rustup.sh \
    -y \
    --no-modify-path \
    --no-update-default-toolchain \
    --profile minimal \
    --default-toolchain stable
EOF
ENV PATH="$PATH:/root/.cargo/bin"

# Rust toolchains
FROM rustup AS thanatos-rust-toolchains
COPY --from=rustup /root/.cargo /root/.cargo
COPY --from=rustup /root/.rustup /root/.rustup
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-unknown-linux-musl \
    x86_64-pc-windows-gnu \
    i686-unknown-linux-gnu \
    i686-pc-windows-gnu

# Vendored Rust dependencies
FROM thanatos-rust-toolchains AS thanatos-rust-vendored
RUN mkdir -p /Mythic
WORKDIR /Mythic
RUN --mount=type=bind,source=agent,target=agent \
    cd agent && cargo vendor /vendor

# Sccache image
FROM base AS sccache
ARG SCCACHE_VERSION=v0.19.0
ADD https://github.com/mozilla/sccache/releases/download/${SCCACHE_VERSION}/sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz /tmp/sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN tar xf /tmp/sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN mv sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/bin/sccache

# Thanatos server builder
FROM base-apt AS thanatos-server-builder
RUN apt-get install -y python3 python3-venv python3-dev gcc g++
RUN mkdir -p /Mythic
WORKDIR /Mythic
RUN --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    --mount=type=bind,source=server,target=server <<EOF
python3 -m venv /thanatos-venv
/thanatos-venv/bin/pip install .
EOF

# Runtime dependencies
FROM base-apt AS thanatos-runtime-deps
RUN apt-get install -y \
    python3 \
    make \
    gcc \
    gcc-i686-linux-gnu \
    musl-tools \
    libc6-dev \
    gcc-mingw-w64-x86-64-win32 \
    mingw-w64-x86-64-dev \
    gcc-mingw-w64-i686-win32 \
    mingw-w64-i686-dev \
    perl

# Main server image
FROM thanatos-runtime-deps AS runner
COPY --link --from=thanatos-server-builder /thanatos-venv /thanatos-venv
COPY --link --from=thanatos-rust-toolchains /root/.cargo /root/.cargo
COPY --link --from=thanatos-rust-toolchains /root/.rustup /root/.rustup
COPY --link --from=thanatos-rust-vendored /vendor /vendor
COPY --from=sccache /usr/bin/sccache /usr/bin/sccache
RUN mkdir -p /Mythic
WORKDIR /Mythic
ENV PATH="/thanatos-venv/bin:/root/.cargo/bin:$PATH"
COPY docker/config.toml /root/.cargo/config.toml
CMD ["thanatos-server"]
