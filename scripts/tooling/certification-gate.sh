#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

"$ROOT_DIR/scripts/run.sh" tooling certify-all
