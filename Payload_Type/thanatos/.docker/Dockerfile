### Real Dockerfile for the thanatos payload

# Pull Alpine for musl libssl files
FROM docker.io/library/alpine:3.19 as musl-ssl
RUN apk update && apk add --no-cache \
    openssl-dev \
    openssl-libs-static

# Pull in Fedora for mingw libssl files
FROM docker.io/library/fedora:39 as mingw-ssl
RUN dnf install -y \
    mingw64-openssl-static \
    mingw32-openssl-static

# Thanatos builder image
FROM docker.io/library/debian:bookworm-slim

# Copy over openssl files
# x86_64-linux-musl
RUN mkdir -p /usr/lib/x86_64-linux-musl
COPY --from=musl-ssl /usr/lib/libcrypto.a /usr/lib/x86_64-linux-musl/libcrypto.a
COPY --from=musl-ssl /usr/lib/libssl.a /usr/lib/x86_64-linux-musl/libssl.a

RUN mkdir -p /usr/include/x86_64-linux-musl
COPY --from=musl-ssl /usr/include/openssl /usr/include/x86_64-linux-musl/openssl

# x86_64-w64-mingw32
RUN mkdir -p /usr/x86_64-w64-mingw32/lib
COPY --from=mingw-ssl \
    /usr/x86_64-w64-mingw32/sys-root/mingw/lib/libcrypto.a \
    /usr/x86_64-w64-mingw32/lib/libcrypto.a
COPY --from=mingw-ssl \
    /usr/x86_64-w64-mingw32/sys-root/mingw/lib/libssl.a \
    /usr/x86_64-w64-mingw32/lib/libssl.a

RUN mkdir -p /usr/x86_64-w64-mingw32/include
COPY --from=mingw-ssl \
    /usr/x86_64-w64-mingw32/sys-root/mingw/include/openssl \
    /usr/x86_64-w64-mingw32/include

# i686-w64-mingw32
RUN mkdir -p /usr/i686-w64-mingw32/lib
COPY --from=mingw-ssl \
    /usr/i686-w64-mingw32/sys-root/mingw/lib/libcrypto.a \
    /usr/i686-w64-mingw32/lib/libcrypto.a
COPY --from=mingw-ssl \
    /usr/i686-w64-mingw32/sys-root/mingw/lib/libssl.a \
    /usr/i686-w64-mingw32/lib/libssl.a

RUN mkdir -p /usr/i686-w64-mingw32/include
COPY --from=mingw-ssl \
    /usr/i686-w64-mingw32/sys-root/mingw/include/openssl \
    /usr/i686-w64-mingw32/include

# Install packages
RUN dpkg --add-architecture i386
RUN apt-get update -y && apt-get install -y \
    curl \
    python3 \
    python3-venv \
    gcc \
    gcc-multilib \
    mingw-w64-x86-64-dev \
    gcc-mingw-w64-x86-64 \
    mingw-w64-i686-dev \
    gcc-mingw-w64-i686 \
    libssl-dev \
    libssl-dev:i386 \
    musl \
    musl-dev \
    && apt-get clean

# Create a thanatos service user for running the Mythic payload service
RUN useradd \
    -r \
    -c "Thanatos service account" \
    -m \
    -d /thanatos \
    -s /usr/sbin/nologin \
    thanatos

COPY mythic /thanatos/mythic
COPY agent /thanatos/agent
RUN chown -R thanatos:thanatos /thanatos

USER thanatos
WORKDIR /thanatos

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
    --profile minimal \
    --default-toolchain stable \
    -t x86_64-unknown-linux-gnu \
    -t i686-unknown-linux-gnu \
    -t x86_64-unknown-linux-musl \
    -t x86_64-pc-windows-gnu \
    -t i686-pc-windows-gnu

RUN rm -vf rustup.sh
ENV PATH=$PATH:/thanatos/.cargo/bin

# Fetch Rust dependencies
WORKDIR /thanatos/agent
RUN cargo fetch

# Copy Rust Cargo config.toml
COPY .docker/config.toml /thanatos/.cargo/config.toml

WORKDIR /thanatos

# Setup poetry
RUN python3 -m venv .poetry-venv
RUN .poetry-venv/bin/pip install -U pip setuptools
RUN .poetry-venv/bin/pip install poetry
ENV PATH=$PATH:/thanatos/.poetry-venv/bin

# Install dependencies
WORKDIR /thanatos/mythic
RUN poetry install --only main

CMD ["poetry", "run", "thanatos"]
