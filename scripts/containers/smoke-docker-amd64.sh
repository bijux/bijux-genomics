#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/amd64}" \
DOCKER_ARCH="${DOCKER_ARCH:-amd64}" \
RUNTIME_NAME="${RUNTIME_NAME:-docker-amd64}" \
sh "$SCRIPT_DIR/smoke-docker-arm64.sh" "$@"
