#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

viol=()
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    path="$line"
    case "$path" in
      artifacts/*/*) ;;
      artifacts/isolates|artifacts/tmp|artifacts/docs|artifacts/container|artifacts/containers|artifacts/coverage|artifacts/inventory|artifacts/test-logs|artifacts/assets-refresh|artifacts/policies) ;;
      *)
        viol+=("$rel -> non-standard artifact path literal: $path")
        ;;
    esac
  done < <(rg -n -o 'artifacts/[A-Za-z0-9._/-]+' "$file" | cut -d: -f3)
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-artifacts-layout: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-artifacts-layout: OK"
