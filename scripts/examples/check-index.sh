#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

IDX="$ROOT_DIR/examples/index.yaml"
[[ -f "$IDX" ]] || {
  echo "examples index missing: examples/index.yaml" >&2
  exit 1
}

if ! head -n 1 "$IDX" | grep -q '^# GENERATED FILE - DO NOT EDIT$'; then
  echo "examples/index.yaml must be generated-only with header" >&2
  exit 1
fi

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
expected="$(mktemp "$TMP_ROOT/examples-index.XXXXXX.yaml")"
trap 'rm -f "$expected"' EXIT INT TERM

"$ROOT_DIR/scripts/examples/generate-index.sh" "$expected" >/dev/null
if ! diff -u "$IDX" "$expected" >/dev/null; then
  echo "examples/index.yaml drift; regenerate with scripts/examples/generate-index.sh" >&2
  exit 1
fi

echo "examples index: OK"
