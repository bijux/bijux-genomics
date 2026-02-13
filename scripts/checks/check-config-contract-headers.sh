#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
failed=0

while IFS= read -r file; do
  [[ -n "$file" ]] || continue
  rel="${file#$ROOT_DIR/}"
  head6=$(sed -n '1,6p' "$file")
  if ! printf '%s\n' "$head6" | grep -Eq '^# schema_version = 1$'; then
    echo "config-contract-headers: missing '# schema_version = 1' in $rel" >&2
    failed=1
  fi
  if ! printf '%s\n' "$head6" | grep -Eq '^# owner = [A-Za-z0-9._/-]+$'; then
    echo "config-contract-headers: missing '# owner = <...>' in $rel" >&2
    failed=1
  fi
done < <(find "$ROOT_DIR/configs/ci" "$ROOT_DIR/configs/runtime" "$ROOT_DIR/configs/bench" -type f \( -name '*.toml' -o -name '*.yaml' -o -name '*.yml' \) 2>/dev/null | sort)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "config-contract-headers: OK"
