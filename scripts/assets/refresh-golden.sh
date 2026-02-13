#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT}/artifacts/tmp/golden-refresh"
TARGET_DIR="${ROOT}/assets/golden/toy-runs-v1"

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

"${ROOT}/scripts/test/toy_runs.sh" refresh --accept --profile all --out "${OUT_DIR}"

rm -rf "${TARGET_DIR}"
mkdir -p "$(dirname "${TARGET_DIR}")"
cp -R "${OUT_DIR}" "${TARGET_DIR}"
echo "golden refresh: wrote ${TARGET_DIR}"
