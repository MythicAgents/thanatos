### Real Dockerfile for the thanatos payload
FROM docker.io/library/debian:bookworm-slim

# Version of sccache to install
ARG SCCACHE_VERSION=0.7.7

# Install packages
RUN apt-get update -y && apt-get install -y \
    curl \
    python3 \
    python3-pip \
    gcc \
    gcc-multilib \
    mingw-w64-x86-64-dev \
    gcc-mingw-w64-x86-64 \
    mingw-w64-i686-dev \
    && apt-get clean

WORKDIR /tmp

# Download static linux-x86_64 libssl
RUN curl -L \
    'https://github.com/MEhrn00/static-containers/releases/download/libssl-3.2.1/libssl-linux-x86_64.tar.gz' \
    -o libssl-linux-x86_64.tar.gz
RUN tar xf libssl-linux-x86_64.tar.gz
RUN cp -v libssl-linux-x86_64/lib64/libcrypto.a /usr/lib/x86_64-linux-gnu/
RUN cp -v libssl-linux-x86_64/lib64/libssl.a /usr/lib/x86_64-linux-gnu/
RUN rm -rf libssl-linux-x86_64 libssl-linux-x86_64.tar.gz

# Download static linux-x86 libssl
RUN curl -L \
    'https://github.com/MEhrn00/static-containers/releases/download/libssl-3.2.1/libssl-linux-x86.tar.gz' \
    -o libssl-linux-x86.tar.gz
RUN tar xf libssl-linux-x86.tar.gz
RUN cp -v libssl-linux-x86/lib/libcrypto.a /usr/lib32/
RUN cp -v libssl-linux-x86/lib/libssl.a /usr/lib32/
RUN rm -rf libssl-linux-x86 libssl-linux-x86.tar.gz

# Download static mingw64 libssl
RUN curl -L \
    'https://github.com/MEhrn00/static-containers/releases/download/libssl-3.2.1/libssl-mingw64.tar.gz' \
    -o libssl-mingw64.tar.gz
RUN tar xf libssl-mingw64.tar.gz
RUN cp -v libssl-mingw64/lib64/libcrypto.a /usr/x86_64-w64-mingw32/lib/
RUN cp -v libssl-mingw64/lib64/libssl.a /usr/x86_64-w64-mingw32/lib/
RUN rm -rf libssl-mingw64 libssl-mingw64.tar.gz

# Download static mingw libssl
RUN curl -L \
    'https://github.com/MEhrn00/static-containers/releases/download/libssl-3.2.1/libssl-mingw.tar.gz' \
    -o libssl-mingw.tar.gz
RUN tar xf libssl-mingw.tar.gz
RUN cp -v libssl-mingw/lib/libcrypto.a /usr/i686-w64-mingw32/lib/
RUN cp -v libssl-mingw/lib/libssl.a /usr/i686-w64-mingw32/lib/
RUN rm -rf libssl-mingw libssl-mingw.tar.gz

# Install the python dependencies
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
RUN rm requirements.txt

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

ENV PATH=$PATH:/root/.cargo/bin

# Install sccache
RUN curl -L \
    "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    -o sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN tar xf sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN mv sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/bin/sccache
RUN chmod +x /usr/bin/sccache
RUN rm -rf sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl
RUN rm -vf sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz

WORKDIR /Mythic
COPY main.py main.py
COPY thanatos thanatos

# Fetch Rust dependencies
WORKDIR /Mythic/thanatos/agent_code
RUN cargo fetch

WORKDIR /Mythic

COPY .docker/config.toml /root/.cargo/config.toml
ENV SCCACHE_DIR /Mythic/.cache/sccache
CMD ["python3", "main.py"]
