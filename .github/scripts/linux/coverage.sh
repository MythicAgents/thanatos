#!/bin/bash

REPO_BASE=""
MYTHIC_CODE="Payload_Type/thanatos"
AGENT_CODE="Payload_Type/thanatos/agent"

# Populates the 'REPO_BASE' to the base of the repo
repo_base() {
    # Get the path to the directory containing this script
    local _script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

    # Traverse up to the base of the git repository
    local _repo_base_dir=${_script_dir}/../../..

    # Ensure that the repo base contains the '.git' directory
    if [ ! -d "${_repo_base_dir}/.git" ]; then
        echo "Could not find git repository base"
        exit 1
    fi

    # Set the REPO_BASE variable to the base of the repo
    REPO_BASE="$(realpath ${_repo_base_dir})"
}

coverage_requirements() {
    cargo -V &> /dev/null
    go version &> /dev/null
    grcov -V &> /dev/null
    genhtml --version &> /dev/null
}

# Generate coverage
coverage() {
    find $AGENT_CODE -name "default*.profraw" -exec rm {} \;
    rm -f coverage/agent.lcov

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
    export RUSTFLAGS="-Ccodegen-units=1 -Copt-level=0 -Cinstrument-coverage"
    cargo test --workspace \
        --all-features \
        --exclude config \
        --exclude genconfig \
        --exclude thanatos_cdylib \
        --exclude thanatos_binary \
        --target x86_64-unknown-linux-gnu

    popd &> /dev/null

    grcov $AGENT_CODE \
        -s . \
        --binary-path ${AGENT_CODE}/target/x86_64-unknown-linux-gnu/debug/ \
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
        --dark-mode \
        --show-details \
        --show-navigation \
        --highlight \
        --ignore-errors source \
        --legend \
        coverage/agent.lcov

    find $AGENT_CODE -name "default*.profraw" -exec rm {} \;
}

set -e
repo_base
coverage_requirements
pushd $REPO_BASE &> /dev/null
coverage
popd &> /dev/null
