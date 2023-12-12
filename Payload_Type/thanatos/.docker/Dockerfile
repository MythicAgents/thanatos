# Base container image for the thanatos payload.
# Note: Invoke this Dockerfile from the `Payload_Type/thanatos` directory using the command
#   `docker build -t ghcr.io/mythicagents/thanatos:{TAG} -f .docker/Dockerfile .`

# Build the Mythic code
FROM docker.io/library/golang:alpine as mythic-builder

RUN apk update \
    && apk add binutils \
    && apk cache clean

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

COPY mythic mythic
WORKDIR /usr/src/app/mythic

RUN go mod download
RUN go mod tidy
RUN go build -ldflags='-extldflags=-static' -o mythic-server .
RUN strip -s mythic-server

# Image containing the dependencies for building the agent
FROM docker.io/library/debian:bookworm-slim

# Install required system packages
RUN apt-get update -y && apt-get install -y \
        curl \
        && apt-get clean

# Install sccache
WORKDIR /tmp
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

# Install the 64 bit Rust toolchains (32 bit get installed during the build if needed)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
        --profile minimal \
        --default-toolchain stable \
        -t x86_64-unknown-linux-gnu \
        -t x86_64-pc-windows-gnu

RUN rm -f rustup.sh

# Copy the cargo config.toml
COPY .docker/config.toml /root/.cargo/config.toml

ENV PATH=$PATH:/root/.cargo/bin

WORKDIR /Mythic

COPY --from=mythic-builder /usr/src/app/mythic/mythic-server /mythic-server
RUN chmod +x /mythic-server
CMD ["/mythic-server"]
