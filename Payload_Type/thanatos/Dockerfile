FROM docker.io/library/debian:stable-slim

# Install packages
RUN dpkg --add-architecture i386
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
        curl \
        python3 \
        python3-poetry \
        gcc \
        gcc-multilib \
        mingw-w64-x86-64-dev \
        mingw-w64-i686-dev \
        gcc-mingw-w64-x86-64 \
        gcc-mingw-w64-i686 \
        libssl-dev \
        libssl-dev:i386 \
        musl-dev \
        && apt-get clean

# Create service user for running the payload service
RUN useradd \
    -c "Thanatos Service User" \
    -m \
    -d /thanatos \
    -r \
    -s /usr/sbin/nologin \
    thanatos

WORKDIR /thanatos

# Copy files
COPY agent/Cargo.toml agent/Cargo.toml
COPY agent/src agent/src
COPY mythic mythic
RUN chown -R thanatos:thanatos /thanatos

USER thanatos

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y \
        --profile minimal \
        --default-toolchain stable \
        -t x86_64-unknown-linux-gnu \
        -t i686-unknown-linux-gnu \
        -t x86_64-unknown-linux-musl \
        -t x86_64-pc-windows-gnu \
        -t i686-pc-windows-gnu \
        && rm -f /tmp/rustup.sh

ENV PATH=$PATH:/thanatos/.cargo/bin

# Fetch Rust dependencies
WORKDIR /thanatos/agent
RUN cargo fetch

# Fetch Python dependencies
WORKDIR /thanatos/mythic
RUN poetry install --only main

CMD ["poetry", "run", "thanatos"]
