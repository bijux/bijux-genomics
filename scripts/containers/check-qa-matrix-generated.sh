#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

target="$ROOT_DIR/docs/30-operations/APPTAINER_QA_MATRIX.md"
tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
tmp_file="$(mktemp "$tmp_root/apptainer-qa.XXXXXX.md")"
trap 'rm -f "$tmp_file"' EXIT

"$SCRIPT_DIR/generate-qa-matrix.sh" "$tmp_file" >/dev/null
if ! diff -u "$target" "$tmp_file" >/dev/null; then
  echo "qa matrix drift: regenerate with scripts/containers/generate-qa-matrix.sh" >&2
  exit 1
fi
echo "qa matrix generated: OK"

