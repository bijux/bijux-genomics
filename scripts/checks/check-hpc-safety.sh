#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

viol=()
while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  if ! rg -n -- '--dry-run' "$file" >/dev/null 2>&1; then
    viol+=("$rel missing --dry-run flag handling")
  fi
  if ! rg -n -- '--confirm' "$file" >/dev/null 2>&1; then
    viol+=("$rel missing --confirm flag handling")
  fi
  if ! rg -n 'dry_run=1' "$file" >/dev/null 2>&1; then
    viol+=("$rel must default to dry_run=1")
  fi
  if ! rg -n 'confirm=0' "$file" >/dev/null 2>&1; then
    viol+=("$rel must default to confirm=0")
  fi
  if ! rg -n 'pass --confirm to execute' "$file" >/dev/null 2>&1; then
    viol+=("$rel must document confirm requirement in dry-run output")
  fi
done < <(find "$ROOT_DIR/scripts/hpc" -type f -name '*.sh' | sort)

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-hpc-safety: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-hpc-safety: OK"
