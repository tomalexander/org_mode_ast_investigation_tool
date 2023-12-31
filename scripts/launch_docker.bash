#!/usr/bin/env bash
#
set -euo pipefail
IFS=$'\n\t'
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

function main {
    make --directory "$DIR/../docker"
    exec docker run --init --rm --read-only --mount type=tmpfs,destination=/tmp --publish 3000:3000/tcp org-investigation
}

main "${@}"
