#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/artifacts/policies"
OUT_FILE="$OUT_DIR/index.md"

mkdir -p "$OUT_DIR"

{
  echo "# Policy Test Index"
  echo
  echo "Generated from crates/bijux-dna-policies/tests."
  echo
  find "$ROOT_DIR/crates/bijux-dna-policies/tests" -type f -name '*.rs' | sort | while IFS= read -r f; do
    rel=$(echo "$f" | sed "s#^$ROOT_DIR/##")
    echo "## $rel"
    rg -n "^fn policy__" "$f" | sed 's/^/- /'
    echo
  done
} > "$OUT_FILE"

echo "wrote $OUT_FILE"
