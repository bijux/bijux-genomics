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
  readme="$ROOT_DIR/$(dirname "$rel")/README.md"
  if [[ ! -f "$readme" ]]; then
    viol+=("$rel -> missing README.md in $(dirname "$rel")")
    continue
  fi
  if ! rg -n '^Requires:' "$readme" >/dev/null 2>&1; then
    viol+=("$rel -> $(dirname "$rel")/README.md missing 'Requires:' list")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-script-deps: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-script-deps: OK"
