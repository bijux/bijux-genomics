#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

index="$ROOT_DIR/containers/docs/index.md"
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp="$(mktemp "$TMP_ROOT/containers-index.XXXXXX.md")"
trap 'rm -f "$tmp"' EXIT

"$SCRIPT_DIR/generate-index.sh" "$tmp" >/dev/null
if ! diff -u "$index" "$tmp" >/dev/null; then
  echo "containers/docs/index.md drift; regenerate with scripts/containers/generate-index.sh" >&2
  exit 1
fi
echo "containers index: OK"
