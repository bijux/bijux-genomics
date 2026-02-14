#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/30-operations/APPTAINER_QA_MATRIX.md}"
case "$OUT" in
  -*)
    echo "refusing unsafe output path (starts with '-'): $OUT" >&2
    exit 2
    ;;
esac
"$ROOT_DIR/scripts/tooling/generate-apptainer-qa-matrix.sh" "$OUT"
