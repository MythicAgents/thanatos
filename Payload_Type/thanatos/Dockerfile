### Image for building the server binary
FROM docker.io/library/golang:1.22-alpine as builder

RUN apk update \
    && apk add \
        binutils \
        make \
        protoc \
        git \
    && apk cache clean

RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest

RUN mkdir -p /usr/local/share/app
WORKDIR /usr/local/share/app

COPY go.mod go.mod
COPY go.sum go.sum

COPY Makefile Makefile
COPY protobuf protobuf

COPY cmd cmd
COPY pkg pkg

RUN go mod download
RUN make protobuf
RUN go mod tidy
RUN make GOFLAGS=-ldflags=-extldflags=-static server
RUN strip -s server


### Image for running the server
FROM docker.io/library/debian:bookworm-slim

COPY --from=builder /usr/local/share/app/server /usr/local/bin/thanatos-server
RUN chmod +x /usr/local/bin/thanatos-server

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

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup.sh \
    && sh rustup.sh -y \
    --profile minimal \
    --default-toolchain stable \
    -t x86_64-unknown-linux-gnu \
    -t x86_64-pc-windows-gnu \
    -t i686-unknown-linux-gnu \
    -t i686-pc-windows-gnu

RUN rm -f rustup.sh

ENV PATH=$PATH:/root/.cargo/bin

RUN mkdir /thanatos
WORKDIR /thanatos

COPY agent agent
COPY assets assets
COPY browserscripts browserscripts

CMD ["thanatos-server"]
