#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
failed=0

while IFS= read -r file; do
  [[ -n "$file" ]] || continue
  base="$(basename "$file")"
  if [[ ! "$base" =~ ^[a-z0-9_]+\.(toml|ya?ml|md|snapshot|sha256|txt)$ ]]; then
    echo "config-filenames: non-snake_case name: ${file#$ROOT_DIR/}" >&2
    failed=1
  fi
done < <(find "$ROOT_DIR/configs" -type f | sort)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "config-filenames: OK"
