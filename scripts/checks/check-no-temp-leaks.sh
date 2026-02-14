#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
viol=()

while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  [[ "$rel" == scripts/checks/check-no-temp-leaks.sh ]] && continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue
  if rg -n '(^|[[:space:]"'"'"'=])(/tmp|/var/tmp)(/|$)' "$file" >/dev/null 2>&1; then
    viol+=("$rel uses system temp path; use \$ISO_ROOT/tmp-*")
  fi
  if rg -n '\$\(mktemp\)' "$file" >/dev/null 2>&1; then
    viol+=("$rel uses bare mktemp with implicit system temp")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-no-temp-leaks: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-no-temp-leaks: OK"
