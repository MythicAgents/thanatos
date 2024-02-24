#!/bin/bash

REPO_PATH=""
MYTHIC_CODE="Payload_Type/thanatos/mythic"
AGENT_CODE="Payload_Type/thanatos/agent"

# Populates the 'REPO_PATH' to the base of the repo
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

    # Set the REPO_PATH variable to the base of the payload
    REPO_PATH="$(realpath ${_repo_base})"
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
    local _cmd="cargo build -p genconfig && cargo clippy --workspace --color always --all-features --all-targets -- -D warnings"
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

# Generate coverage
coverage() {
    rm -rf coverage/html
    mkdir -p coverage/html/{agent,mythic}

    echo "[*] Generating Mythic code coverage"

    pushd $MYTHIC_CODE &> /dev/null
    go test -coverprofile ../../../coverage/mythic.builder.gocov -run "Mock" ./builder/...
    sed -i '/^thanatos\/builder\/testing.*\.go:.*$/d' ../../../coverage/mythic.builder.gocov
    sed -i '/^thanatos\/builder\/handlers\.go:.*$/d' ../../../coverage/mythic.builder.gocov

    go test -coverprofile ../../../coverage/mythic.commands.gocov ./commands/...
    sed -i '/^thanatos\/commands\/testing\/.*$/d' ../../../coverage/mythic.commands.gocov
    sed -i '/^thanatos\/commands\/commands\.go:.*$/d' ../../../coverage/mythic.commands.gocov
    sed -i '/^thanatos\/commands\/utils\/mythicrpc\.go:.*$/d' ../../../coverage/mythic.commands.gocov

    cat ../../../coverage/mythic.builder.gocov > ../../../coverage/mythic.gocov
    grep "^thanatos" ../../../coverage/mythic.commands.gocov >> ../../../coverage/mythic.gocov

    rm ../../../coverage/mythic.commands.gocov
    rm ../../../coverage/mythic.builder.gocov

    go tool cover -html ../../../coverage/mythic.gocov -o ../../../coverage/html/mythic/index.html
    popd &> /dev/null

    echo "[*] Generating Agent code coverage"
    pushd $AGENT_CODE &> /dev/null
    export RUSTFLAGS="-Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Cinstrument-coverage"
    cargo test --workspace --all-features --exclude config --exclude genconfig

    popd &> /dev/null

    grcov Payload_Type/thanatos/agent/ \
        -s . \
        --binary-path Payload_Type/thanatos/agent/target/debug/ \
        -t lcov \
        --branch \
        --ignore-not-existing \
        --ignore "$HOME/.cargo/registry/*" \
        --ignore "*/target/*" \
        --ignore "**/build.rs" \
        --ignore "*/config/*" \
        -o coverage/agent.lcov

    genhtml -o coverage/html/agent \
        -f \
        --show-details \
        --show-navigation \
        --highlight \
        --ignore-errors source \
        --legend \
        coverage/agent.lcov

    find Payload_Type/thanatos/agent -name "default*.profraw" -exec rm {} \;
}

set -e

populate_thanatos_path

pushd $REPO_PATH &> /dev/null

check_requirements

format_check
echo ""

lint_check
echo ""

tests
echo ""

coverage
echo ""

popd &> /dev/null
