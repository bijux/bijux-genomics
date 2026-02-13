#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/amd64}" \
DOCKER_ARCH="${DOCKER_ARCH:-amd64}" \
RUNTIME_NAME="${RUNTIME_NAME:-docker-amd64}" \
sh "$SCRIPT_DIR/smoke-docker-arm64.sh" "$@"
