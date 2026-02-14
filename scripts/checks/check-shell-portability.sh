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
while IFS= read -r p; do
  [[ -n "$p" ]] || continue
  f="$ROOT_DIR/$p"
  first=$(head -n1 "$f" || true)
  [[ "$first" == '#!/bin/sh' ]] && viol+=("$p: uses #!/bin/sh")
  if rg -n '\bsed\s+-i(\s|$)' "$f" >/dev/null 2>&1 && ! rg -n 'compat_sed_inplace' "$f" >/dev/null 2>&1; then
    viol+=("$p: uses sed -i without compatibility wrapper")
  fi
  if rg -n 'readlink -f' "$f" >/dev/null 2>&1 && ! rg -n 'compat_readlink_f' "$f" >/dev/null 2>&1; then
    viol+=("$p: uses readlink -f without compatibility wrapper")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$ROOT_DIR/scripts/SUPPORTED.toml")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "shell-portability: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "shell-portability: OK"
