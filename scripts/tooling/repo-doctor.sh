#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

./scripts/run.sh checks check-root-layout
./scripts/run.sh checks check-config-layout
./scripts/run.sh docs check-docs-graph
./scripts/run.sh checks check-supported-scripts
./scripts/run.sh checks check-no-orphan-scripts

echo "repo-doctor: OK"
