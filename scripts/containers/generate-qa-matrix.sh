#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/30-operations/APPTAINER_QA_MATRIX.md}"
"$ROOT_DIR/scripts/tooling/generate-apptainer-qa-matrix.sh" "$OUT"
