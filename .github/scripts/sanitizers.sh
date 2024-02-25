#!/bin/bash

REPO_BASE=""
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

sanitizers_requirements() {
    cargo +nightly -V &> /dev/null
}

sanitizers() {
    pushd $AGENT_CODE &> /dev/null
    local _cmd="RUSTFLAGS='-Zsanitizer=address' cargo +nightly test -Zbuild-std --color always -p ffiwrappers --all-features --target x86_64-unknown-linux-gnu"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd

    local _cmd="RUSTFLAGS='-Zsanitizer=memory' cargo +nightly test -Zbuild-std --color always -p ffiwrappers --all-features --target x86_64-unknown-linux-gnu"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd

    local _cmd="RUSTFLAGS='-Zsanitizer=leak' cargo +nightly test -Zbuild-std --color always -p ffiwrappers --all-features --target x86_64-unknown-linux-gnu"
    echo "current directory: $PWD"
    echo "command: $_cmd"
    eval $_cmd
    popd &> /dev/null
}

set -e
repo_base
sanitizers_requirements
pushd $REPO_BASE &> /dev/null
sanitizers
popd &> /dev/null
