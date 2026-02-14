#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

API_DOC="$ROOT_DIR/scripts/_lib/API.md"
COMMON="$ROOT_DIR/scripts/_lib/common.sh"
RUNTIME="$ROOT_DIR/scripts/_lib/runtime.sh"
SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

exported=$(rg -o '`[a-zA-Z_][a-zA-Z0-9_]*' "$API_DOC" | tr -d '`' | sort -u)
common_funcs=$(sed -n -E 's/^([a-zA-Z_][a-zA-Z0-9_]*)\(\)\s*\{.*/\1/p' "$COMMON" | sort -u)
runtime_funcs=$(sed -n -E 's/^([a-zA-Z_][a-zA-Z0-9_]*)\(\)\s*\{.*/\1/p' "$RUNTIME" | sort -u)
all_funcs=$(printf '%s\n%s\n' "$common_funcs" "$runtime_funcs" | sed '/^$/d' | sort -u)

viol=()
while IFS= read -r fn; do
  [[ -n "$fn" ]] || continue
  if ! grep -qx "$fn" <<< "$exported" && [[ "$fn" != _internal_* ]]; then
    viol+=("common.sh function not declared in API or private-prefixed: $fn")
  fi
done <<< "$all_funcs"

while IFS= read -r p; do
  [[ -n "$p" ]] || continue
  [[ "$p" == scripts/_lib/* ]] && continue
  f="$ROOT_DIR/$p"
  while IFS= read -r fn; do
    [[ -n "$fn" ]] || continue
    [[ "$fn" == require_stable_env ]] && continue
    if rg -n "^\s*${fn}\(\)\s*\{" "$f" >/dev/null 2>&1; then
      viol+=("$p duplicates _lib function $fn")
    fi
  done <<< "$all_funcs"
  if ! rg -n 'source "\$\{ROOT_DIR\}/scripts/_lib/common\.sh"' "$f" >/dev/null 2>&1; then
    viol+=("$p must source scripts/_lib/common.sh")
  fi
  if ! rg -n '\brequire_stable_env\b' "$f" >/dev/null 2>&1; then
    viol+=("$p must call require_stable_env")
  fi
  if ! rg -n 'SCRIPT_DIR=\$\(cd "\$\(dirname "\$\{BASH_SOURCE\[0\]\}"\)" && pwd\)' "$f" >/dev/null 2>&1; then
    viol+=("$p must define SCRIPT_DIR using BASH_SOURCE path resolution")
  fi
  if ! rg -n 'ROOT_DIR=\$\(cd "\$\{SCRIPT_DIR\}/\.\./' "$f" >/dev/null 2>&1 && ! rg -n 'ROOT_DIR=\$\(cd "\$\{SCRIPT_DIR\}/\.\." && pwd\)' "$f" >/dev/null 2>&1; then
    viol+=("$p must define ROOT_DIR from SCRIPT_DIR (no CWD assumptions)")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "lib-api: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "lib-api: OK"
