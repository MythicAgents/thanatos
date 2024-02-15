#!/bin/bash

THANATOS_PATH=""
MYTHIC_CODE="mythic"
AGENT_CODE="agent"

# Populates the 'THANATOS_PATH' variable with the path to the thanatos payload base directory
populate_thanatos_path() {
    # Get the path to the directory containing this script
    local _script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

    # Traverse up to the base of the git repository
    local _repo_base=${_script_dir}/../..

    # Ensure that the repo base contains the '.git' directory
    if [ ! -d "${_repo_base}/.git" ]; then
        echo "Could not find git repository base"
        exit 1
    fi

    # Set the THANATOS_PATH variable to the base of the payload
    THANATOS_PATH="$(realpath ${_repo_base}/Payload_Type/thanatos)"
}

# Check that python3, python3-pylint, python3-black, cargo, cargo-fmt, and cargo-clippy exist
check_requirements() {
    # Ensure go exists
    go version &> /dev/null

    # Ensure gofmt exists
    gofmt --help &> /dev/null

    # Ensure golangci-lint exists
    golangci-lint --version &> /dev/null

    # Ensure cargo exists
    cargo -V &> /dev/null

    # Ensure cargo fmt exists
    cargo fmt --version &> /dev/null

    # Ensure cargo clippy exists
    cargo clippy --version &> /dev/null
}

# Run code format checking
format_check() {
    echo "[*] Running code format checks"

    echo "[*] Mythic code"
    pushd $MYTHIC_CODE &> /dev/null
    local _cmd="gofmt -l -d . | diff -u /dev/null -"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null

    echo "[*] Agent code"
    pushd $AGENT_CODE &> /dev/null
    local _cmd="cargo build -p genconfig && cargo fmt --all -- --color always --check"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null
}

# Run lint checks
lint_check() {
    echo "[*] Running lint checks"

    echo "[*] Mythic code"
    pushd $MYTHIC_CODE &> /dev/null
    local _cmd="golangci-lint run"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null

    echo "[*] Agent code"
    pushd $AGENT_CODE &> /dev/null
    local _cmd="cargo build -p genconfig && cargo clippy --color always --all-features --all-targets -- -D warnings"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null
}

# Run tests
tests() {
    echo "[*] Running tests"

    echo "[*] Mythic code"
    pushd $MYTHIC_CODE &> /dev/null
    local _cmd="go test ./commands/..."
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    local _cmd="go test -run \"^TestPayloadMockBuild/\" ./builder"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null

    echo "[*] Agent code"
    pushd $AGENT_CODE &> /dev/null
    local _cmd="cargo build -p genconfig && cargo test --color always --workspace --exclude genconfig --all-features"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null
}

set -e

populate_thanatos_path
check_requirements

pushd $THANATOS_PATH &> /dev/null

format_check
echo ""

lint_check
echo ""

tests

popd &> /dev/null
