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

listed=$(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC" | sort -u)
all_scripts=$(find "$ROOT_DIR/scripts" -type f -name '*.sh' | sed "s#^$ROOT_DIR/##" | sort -u)

orphans=()
while IFS= read -r s; do
  [[ -n "$s" ]] || continue
  if [[ "$s" == scripts/experimental/* ]]; then
    continue
  fi
  if ! grep -qx "$s" <<< "$listed"; then
    orphans+=("$s")
  fi
done <<< "$all_scripts"

if [[ ${#orphans[@]} -gt 0 ]]; then
  echo "no-orphan-scripts: found scripts not in scripts/SUPPORTED.toml and not under scripts/experimental/:" >&2
  printf '%s\n' "${orphans[@]}" >&2
  exit 1
fi

echo "no-orphan-scripts: OK"
