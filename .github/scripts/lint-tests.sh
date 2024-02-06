#!/bin/bash

THANATOS_PATH=""
MYTHIC_CODE="thanatos/mythic/agent_functions"
AGENT_CODE="thanatos/agent_code"

CARGO_VARS=$(cat <<EOF
RUSTFLAGS='--cfg http'
UUID=''
AESPSK=''
callback_host=''
callback_interval=''
callback_jitter=''
callback_port=''
connection_retries=''
encrypted_exchange_check=''
get_uri=''
headers=''
post_uri=''
working_hours=''
EOF
)

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
    # Ensure python3 exists
    python3 --version &> /dev/null

    # Ensure python3-pylint exists
    python3 -m pylint --version &> /dev/null

    # Ensure python3-black exists
    python3 -m black --version &> /dev/null

    # Ensure cargo exists
    cargo -V &> /dev/null

    # Ensure cargo fmt exists
    cargo fmt --version &> /dev/null

    # Ensure cargo clippy exists
    cargo fmt --version &> /dev/null
}

# Run syntax checking
syntax_check() {
    echo "[*] Running syntax checks"

    local _cmd="python3 -m pylint --rcfile pylintrc -f colorized --errors-only main.py ${MYTHIC_CODE}/*.py"
    echo "[*] current directory: $PWD"
    echo "[*] command: $_cmd"
    eval $_cmd

    pushd $AGENT_CODE &> /dev/null
    local _cmd="env ${CARGO_VARS} cargo check --color always --all-targets --all-features"
    echo "[*] current directory: $PWD"
    echo "[*] command: $(echo $_cmd | tr '\n' ' ')"
    eval $_cmd
    popd &> /dev/null
}

# Run code format checking
format_check() {
    echo "[*] Running code format checks"

    local _cmd="python3 -m black --color --diff --check main.py ${MYTHIC_CODE}/*.py"
    echo "[*] current directory: $PWD"
    echo "[*] command: $_cmd"
    eval $_cmd

    pushd $AGENT_CODE &> /dev/null
    local _cmd="env ${CARGO_VARS} cargo fmt -- --color always --check"
    echo "[*] current directory: $PWD"
    echo "[*] command: $(echo $_cmd | tr '\n' ' ')"
    eval $_cmd
    popd &> /dev/null
}

# Run lint checks
lint_check() {
    echo "[*] Running lint checks"

    local _cmd="python3 -m pylint --rcfile pylintrc -f colorized main.py ${MYTHIC_CODE}/*.py"
    echo "[*] current directory: $PWD"
    echo "[*] command: $_cmd"
    eval $_cmd

    pushd $AGENT_CODE &> /dev/null

    local _cmd="env ${CARGO_VARS} cargo clippy --color always --all-targets --all-features -- -D warnings"
    echo "[*] current directory: $PWD"
    echo "[*] command: $(echo $_cmd | tr '\n' ' ')"
    eval $_cmd
    popd &> /dev/null
}

set -e

populate_thanatos_path
check_requirements

pushd $THANATOS_PATH &> /dev/null
syntax_check
echo ""

format_check
echo ""

lint_check
echo ""

popd &> /dev/null
