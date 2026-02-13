#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

failed=0
while IFS= read -r file; do
  [[ -n "$file" ]] || continue
  rel="${file#$ROOT_DIR/}"
  head8=$(sed -n '1,8p' "$file")
  if ! printf '%s\n' "$head8" | rg -q '^# schema_version = [0-9]+$'; then
    echo "config-headers: missing '# schema_version = N' in $rel" >&2
    failed=1
  fi
  if ! printf '%s\n' "$head8" | rg -q '^# owner = [A-Za-z0-9._/-]+$'; then
    echo "config-headers: missing '# owner = <crate>' in $rel" >&2
    failed=1
  fi
done < <(find "$ROOT_DIR/configs" -type f \( -name '*.toml' -o -name '*.yaml' -o -name '*.yml' \) | sort)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "config-headers: OK"
