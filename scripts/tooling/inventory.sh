#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_DIR="$ROOT_DIR/artifacts/inventory"
ensure_artifacts_dir "$OUT_DIR"
mkdir -p "$OUT_DIR"

SCRIPTS_OUT="$OUT_DIR/scripts_inventory.txt"
CONFIGS_OUT="$OUT_DIR/configs_inventory.txt"
DOCS_OUT="$OUT_DIR/docs_index_coverage.txt"
ASSETS_OUT="$OUT_DIR/assets_inventory.txt"

find "$ROOT_DIR/scripts" -type f -name '*.sh' | sed "s#^$ROOT_DIR/##" | sort > "$SCRIPTS_OUT"
find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort > "$CONFIGS_OUT"
find "$ROOT_DIR/assets" -type f | sed "s#^$ROOT_DIR/##" | sort > "$ASSETS_OUT"

{
  echo "docs_index_coverage"
  while IFS= read -r dir; do
    rel="${dir#"$ROOT_DIR/"}"
    if [[ -f "$dir/index.md" ]]; then
      echo "$rel/index.md:present"
    else
      echo "$rel/index.md:missing"
    fi
  done < <(find "$ROOT_DIR/docs" -type d | sort)
} > "$DOCS_OUT"

echo "wrote $SCRIPTS_OUT"
echo "wrote $CONFIGS_OUT"
echo "wrote $DOCS_OUT"
echo "wrote $ASSETS_OUT"
