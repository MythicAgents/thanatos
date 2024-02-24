#!/bin/bash

REPO_BASE=""
MYTHIC_CODE="Payload_Type/thanatos/mythic"
AGENT_CODE="Payload_Type/thanatos/agent"

# Populates the 'REPO_BASE' to the base of the repo
repo_base() {
    # Get the path to the directory containing this script
    local _script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

    # Traverse up to the base of the git repository
    local _repo_base_dir=${_script_dir}/../..

    # Ensure that the repo base contains the '.git' directory
    if [ ! -d "${_repo_base_dir}/.git" ]; then
        echo "Could not find git repository base"
        exit 1
    fi

    # Set the REPO_BASE variable to the base of the repo
    REPO_BASE="$(realpath ${_repo_base_dir})"
}

# Run code format checking
checkformat() {
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

set -e
repo_base
pushd $REPO_BASE &> /dev/null
checkformat
popd &> /dev/null