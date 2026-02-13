#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT_DIR/configs/schema/config_tree.snapshot"
TMP="${TEST_TMP_DIR:-$ROOT_DIR/artifacts/tmp}/config_tree.snapshot.tmp"
mkdir -p "$(dirname "$OUT")" "$(dirname "$TMP")"

{
  echo '# schema_version = 1'
  echo '# owner = bijux-dna-infra'
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort
} > "$TMP"

mv "$TMP" "$OUT"
echo "generated $OUT"
