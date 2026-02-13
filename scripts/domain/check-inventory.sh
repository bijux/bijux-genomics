#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_JSON="$ROOT_DIR/artifacts/domain/inventory.json"
OUT_MD="$ROOT_DIR/artifacts/domain/inventory.md"
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp1="$(mktemp "$TMP_ROOT/domain-inv-1.XXXXXX.json")"
tmp2="$(mktemp "$TMP_ROOT/domain-inv-2.XXXXXX.json")"
tmpm1="$(mktemp "$TMP_ROOT/domain-inv-1.XXXXXX.md")"
tmpm2="$(mktemp "$TMP_ROOT/domain-inv-2.XXXXXX.md")"
trap 'rm -f "$tmp1" "$tmp2" "$tmpm1" "$tmpm2"' EXIT INT TERM

"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$tmp1" "$tmpm1" >/dev/null
"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$tmp2" "$tmpm2" >/dev/null

if ! diff -u "$tmp1" "$tmp2" >/dev/null; then
  echo "domain inventory is non-deterministic across consecutive generations" >&2
  exit 1
fi
if ! diff -u "$tmpm1" "$tmpm2" >/dev/null; then
  echo "domain inventory markdown is non-deterministic across consecutive generations" >&2
  exit 1
fi

"$ROOT_DIR/scripts/domain/generate-inventory.sh" "$OUT_JSON" "$OUT_MD" >/dev/null
echo "domain inventory: OK ($OUT_JSON, $OUT_MD)"
