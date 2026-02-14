#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

violations=$(rg -n --iglob '*.md' -e '\bTODO\b|\bTBD\b|\bWIP\b|\bplaceholder\b' "$ROOT_DIR/docs" \
  | rg -v '^.*/docs/overrides/' || true)

if [[ -n "$violations" ]]; then
  echo "docs-placeholder-policy: forbidden placeholder language found outside docs/overrides/" >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

echo "docs-placeholder-policy: OK"
