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


set -e
populate_thanatos_path
pushd $REPO_PATH &> /dev/null
./.github/scripts/checkformat.sh
echo ""

./.github/scripts/lint.sh
echo ""

./.github/scripts/test.sh
echo ""

./.github/scripts/sanitizers.sh
echo ""

popd &> /dev/null
