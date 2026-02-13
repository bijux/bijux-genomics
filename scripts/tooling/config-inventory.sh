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
OUT_FILE="$ROOT_DIR/artifacts/configs_inventory.txt"
mkdir -p "$(dirname "$OUT_FILE")"

{
  echo "# schema_version = 1"
  echo "# owner = bijux-dna-infra"
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort | while read -r rel; do
    printf '%s\n' "$rel"
  done
} > "$OUT_FILE"

echo "wrote $OUT_FILE"
