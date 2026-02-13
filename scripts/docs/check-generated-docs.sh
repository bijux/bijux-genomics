#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

check_header() {
  local file="$1"
  local head
  head=$(sed -n '1,6p' "$file")
  printf '%s\n' "$head" | grep -q '^<!-- GENERATED FILE - DO NOT EDIT -->$' || {
    echo "generated-docs: missing generated header in ${file#$ROOT/}" >&2
    return 1
  }
}

check_header "$ROOT/docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"
check_header "$ROOT/docs/20-science/TOOL_INDEX.md"

echo "generated docs headers: OK"
