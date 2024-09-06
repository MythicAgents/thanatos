#!/bin/bash


: "${GOVERSION:=1.22.0}"
: "${PROTOCVERSION:=28.0}"
: "${LCOVVERSION:=2.1}"
: "${GRCOVVERSION:=0.8.19}"
: "${GOLANGCILINTVERSION:=1.60.3}"

: "${PREFIX:=/usr/local}"

BINPATH="${PREFIX}/bin"

function install_packages() {
    if [[ "$(id -u )" == 0 ]]; then
        local SUDO=''
    else
        local SUDO="sudo"
    fi

    ${SUDO} apt-get install -y "$@"
}

function install_cmds() {
    for cmd in "$@"
    do
        if ! command -v "$cmd" &> /dev/null; then
            install_packages $cmd
        fi
    done
}

function install_go() {
    if [[ ! -d "${PREFIX}/go" ]]; then
        install_cmds curl tar gzip

        local url="https://go.dev/dl/go${GOVERSION}.linux-amd64.tar.gz"
        local dl="/tmp/${url##*/}"

        curl -L "$url" -o "$dl"

        rm -rf "${PREFIX}/go"
        tar -C "$PREFIX" -xf "$dl"
        rm "$dl"
    fi
}

function install_rust() {
    if [[ ! -f "${HOME}/.cargo/bin/cargo" ]]; then
        install_cmds curl
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - \
            -y --profile minimal

        #-c clippy llvm-tools-preview
    fi
}

function install_protoc() {
    if [[ ! -f "${PREFIX}/bin/protoc" ]]; then
        install_cmds curl unzip

        local url="https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOCVERSION}/protoc-${PROTOCVERSION}-linux-x86_64.zip"
        local dl="/tmp/${url##*/}"

        curl -L "$url" -o "$dl"
        unzip "$dl" -d "$PREFIX" -x readme.txt
        rm "$dl"
    fi
}

function install_grcov() {
    if [[ ! -f "${PREFIX}/bin/grcov" ]]; then
        install_cmds curl tar bzip2

        local url="https://github.com/mozilla/grcov/releases/download/v${GRCOVVERSION}/grcov-x86_64-unknown-linux-gnu.tar.bz2"
        local dl="/tmp/${url##*/}"

        curl -L "$url" -o "$dl"
        tar -C "$BINPATH" -xf "$dl"
        rm "$dl"
    fi
}

function install_golangci-lint() {
    if [[ ! -f "${PREFIX}/bin/golangci-lint" ]]; then
        install_cmds curl tar gzip

        local url="https://github.com/golangci/golangci-lint/releases/download/v${GOLANGCILINTVERSION}/golangci-lint-${GOLANGCILINTVERSION}-linux-amd64.tar.gz"
        local dl="/tmp/${url##*/}"

        curl -L "$url" -o "$dl"
        tar -C "$BINPATH" -xf "$dl" --strip-components=1 --exclude=LICENSE --exclude=README.md
        rm "$dl"
    fi
}

function agent-dev() {
    mythic-dev
    install_rust
    install_cmds gcc pkg-config
    install_packages libssl-dev
}

function agent-test() {
    agent-dev
    $HOME/.cargo/bin/rustup toolchain install nightly
    $HOME/.cargo/bin/rustup component add rust-src --toolchain nightly
}

function agent-lint() {
    agent-dev
    $HOME/.cargo/bin/rustup component add clippy rustfmt
}

function mythic-dev() {
    install_protoc
    install_go
    ${PREFIX}/go/bin/go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
}

function mythic-test() {
    mythic-dev
}

function mythic-lint() {
    mythic-dev
    install_golangci-lint
}

function mythic-all() {
    mythic-dev
    mythic-test
    mythic-lint
}

function agent-all() {
    agent-dev
    agent-test
    agent-lint
}

if [ "$1" == "env" ]; then
    : "${GOPATH:=$HOME/go}"
    echo "export PATH=$(echo $PATH | tr ':' '\n' | grep -v $PREFIX/go/bin | grep -v $HOME/.cargo/bin | grep -v $GOPATH/bin | grep . | tr '\n' ':')$PREFIX/go/bin:$GOPATH/bin:$HOME/.cargo/bin"
    exit 0
fi

install_cmds make

set -e
for arg in $@
do
    if [ "$arg" = "mythic-dev" ]; then mythic-dev; fi
    if [ "$arg" = "mythic-test" ]; then mythic-test; fi
    if [ "$arg" = "mythic-lint" ]; then mythic-lint; fi
    if [ "$arg" = "mythic-all" ]; then mythic-all; fi

    if [ "$arg" = "agent-dev" ]; then agent-dev; fi
    if [ "$arg" = "agent-test" ]; then agent-test; fi
    if [ "$arg" = "agent-lint" ]; then agent-lint; fi
    if [ "$arg" = "agent-all" ]; then agent-all; fi

    if [ "$arg" = "all" ]; then mythic-all; agent-all; fi
done
