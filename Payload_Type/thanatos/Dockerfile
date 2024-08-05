# Build the Mythic server side code
FROM docker.io/library/golang:1.22-alpine as mythic-builder

RUN apk update \
    && apk add binutils \
    && apk cache clean

RUN mkdir -p /usr/local/share/app
WORKDIR /usr/local/share/app

COPY mythic .

RUN go mod download
RUN go mod tidy
RUN go build -ldflags='-extldflags=-static' -o mythic-server .
RUN strip -s mythic-server

# Pull in the base image for the payload builder
FROM docker.io/library/debian:bookworm-slim

# Install required system packages
RUN dpkg --add-architecture i386 && \
    apt-get update -y && apt-get install -y \
    make \
    curl \
    gcc \
    libc6:i386 \
    libc6-dev:i386 \
    lib32gcc-12-dev \
    mingw-w64 \
    && apt-get clean

WORKDIR /tmp

# Install sccache
ENV SCCACHE_VERSION v0.7.4
RUN curl -L \
    "https://github.com/mozilla/sccache/releases/download/${SCCACHE_VERSION}/sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    -o sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN tar xf sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
RUN mv sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/bin/sccache
RUN chmod +x /usr/bin/sccache
RUN rm -rf sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl
RUN rm -f sccache-${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz
ENV SCCACHE_DIR /Mythic/.cache/sccache

# Install Rust with the required toolchains
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
    --profile minimal \
    --default-toolchain stable \
    -t x86_64-unknown-linux-gnu \
    -t x86_64-pc-windows-gnu \
    -t i686-unknown-linux-gnu \
    -t i686-pc-windows-gnu

RUN rm -f rustup.sh

# Copy the cargo config.toml
COPY .docker/config.toml /root/.cargo/config.toml
ENV PATH=$PATH:/root/.cargo/bin

RUN mkdir /thanatos
WORKDIR /thanatos

COPY agent agent

RUN mkdir mythic
WORKDIR /thanatos/mythic
COPY mythic/assets assets
COPY mythic/browserscript browserscript

COPY --from=mythic-builder /opt/mythic/mythic-server .
RUN chmod +x ./mythic-server
CMD ["./mythic-server"]
