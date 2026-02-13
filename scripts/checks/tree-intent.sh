#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

failed=0

for dir in scripts/*; do
  [[ -d "$dir" ]] || continue
  readme="${dir}/README.md"
  if [[ ! -f "$readme" ]]; then
    echo "tree-intent: missing $readme" >&2
    failed=1
    continue
  fi
  if ! rg -q '^Purpose:' "$readme"; then
    echo "tree-intent: missing 'Purpose:' line in $readme" >&2
    failed=1
  fi
done

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "tree-intent: OK"
