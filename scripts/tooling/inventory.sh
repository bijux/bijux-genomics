#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/artifacts/scripts"
OUT_FILE="$OUT_DIR/inventory.md"

mkdir -p "$OUT_DIR"

{
  echo "# Script Inventory"
  echo
  echo "Generated from Makefile references and scripts tree."
  echo
  echo "## Make-Referenced (Supported)"
  rg -No "scripts/[A-Za-z0-9_./-]+\\.(sh|py)" "$ROOT_DIR/Makefile" "$ROOT_DIR/makefiles" \
    | sort -u \
    | sed 's#^#- `#; s#$#`#'
  echo
  echo "## All Scripts"
  find "$ROOT_DIR/scripts" -type f \( -name "*.sh" -o -name "*.py" \) \
    | sed "s#^$ROOT_DIR/##" \
    | sort \
    | while IFS= read -r rel; do
        purpose=$(sed -n '2,8p' "$ROOT_DIR/$rel" | rg -m1 '^[[:space:]]*#' | sed 's/^[[:space:]]*#\s*//' || true)
        [ -n "$purpose" ] || purpose="(no purpose comment)"
        printf -- "- `%s`: %s\n" "$rel" "$purpose"
      done
} > "$OUT_FILE"

echo "wrote $OUT_FILE"
