#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
ISO_TMP="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/tmp-scripts-smoke"
ensure_artifacts_dir "$ISO_TMP"
mkdir -p "$ISO_TMP"

fails=()
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  abs="$ROOT_DIR/$rel"
  if ! "$abs" --help >/dev/null 2>&1; then
    fails+=("$rel --help failed")
    continue
  fi
  # dry-run probe: prefer explicit --dry-run, else --help probe is the dry-run.
  if "$abs" --help 2>&1 | rg -n -- '--dry-run' >/dev/null 2>&1; then
    if ! ISO_ROOT="$ISO_TMP" "$abs" --dry-run >/dev/null 2>&1; then
      fails+=("$rel --dry-run failed")
    fi
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#fails[@]} -gt 0 ]]; then
  echo "test-scripts-smoke: failures:" >&2
  printf '%s\n' "${fails[@]}" >&2
  exit 1
fi

echo "test-scripts-smoke: OK"
