#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/artifacts/containers/docker-arm64}"

echo "docker-build-all: artifact_dir=$ARTIFACT_DIR"
SMOKE_LEVEL=contract \
SAVE_TAR=0 \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$SCRIPT_DIR/smoke-docker-arm64.sh"
cargo run -q -p bijux-dev-dna -- containers run summary -- --json "$ROOT_DIR/artifacts/containers/summary.json" >/dev/null
cargo run -q -p bijux-dev-dna -- containers run generate-version-lock -- "$ROOT_DIR/containers/versions/lock.json" >/dev/null
"$SCRIPT_DIR/check-lock-matches-built-output.sh"
echo "docker-build-all: OK"
