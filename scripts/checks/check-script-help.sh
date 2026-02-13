#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

viol=()
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  abs="$ROOT_DIR/$rel"
  if [[ ! -x "$abs" ]]; then
    viol+=("$rel -> not executable")
    continue
  fi
  if ! out="$("$abs" --help 2>&1)"; then
    viol+=("$rel -> --help failed")
    continue
  fi
  if ! grep -Eq '^Usage:' <<<"$out"; then
    viol+=("$rel -> --help must print Usage:")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-script-help: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-script-help: OK"
