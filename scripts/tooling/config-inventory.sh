#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/artifacts/configs"
OUT_FILE="$OUT_DIR/inventory.md"
mkdir -p "$OUT_DIR"

{
  echo "# Config Inventory"
  echo
  echo "Generated from repository config tree."
  echo
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort | while read -r rel; do
    dir="$(dirname "$rel")"
    base="$(basename "$rel")"
    purpose=""
    case "$dir" in
      configs/ci) purpose="CI/SSOT contract input" ;;
      configs/coverage) purpose="Coverage gate input" ;;
      configs/nextest) purpose="Nextest runner config" ;;
      configs/runtime) purpose="Runtime default input" ;;
      configs/bench) purpose="Benchmark suite input" ;;
      configs/logging) purpose="Logging config/policy docs" ;;
      configs/schema) purpose="Schema config/policy docs" ;;
      configs) purpose="Top-level index" ;;
      *) purpose="Misc config" ;;
    esac
    printf -- '- `%s` — %s\n' "$rel" "$purpose"
  done
} > "$OUT_FILE"

echo "wrote $OUT_FILE"
