#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="$ROOT_DIR/artifacts/domain/inventory.json"
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp1="$(mktemp "$TMP_ROOT/domain-inv-1.XXXXXX.json")"
tmp2="$(mktemp "$TMP_ROOT/domain-inv-2.XXXXXX.json")"
trap 'rm -f "$tmp1" "$tmp2"' EXIT INT TERM

"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$tmp1" >/dev/null
"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$tmp2" >/dev/null

if ! diff -u "$tmp1" "$tmp2" >/dev/null; then
  echo "domain inventory is non-deterministic across consecutive generations" >&2
  exit 1
fi

"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$OUT" >/dev/null
echo "domain inventory: OK ($OUT)"
