#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

viol=()
while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  if rg -n '(^|[[:space:]"'"'"'=])(/tmp|/var/tmp)(/|$)' "$file" >/dev/null 2>&1; then
    viol+=("$rel uses hidden system tmp path; use runtime tmp root contracts")
  fi
done < <(find \
  "$ROOT_DIR/crates/bijux-dna-api/src/runtime" \
  "$ROOT_DIR/crates/bijux-dna-api/src/internal/handlers/cross" \
  -type f -name '*.rs' -print)

if ((${#viol[@]} > 0)); then
  printf '%s\n' "check-hidden-tmp-usage: FAILED"
  printf ' - %s\n' "${viol[@]}"
  exit 1
fi

echo "check-hidden-tmp-usage: OK"
