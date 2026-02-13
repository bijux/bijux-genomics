#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="$ROOT_DIR/configs/schema/config-tree.snapshot"
ACTUAL="${TEST_TMP_DIR:-$ROOT_DIR/artifacts/tmp}/config-tree.snapshot.actual"
mkdir -p "$(dirname "$ACTUAL")"

find "$ROOT_DIR/configs" -type f \
  | sed "s#^$ROOT_DIR/##" \
  | sort > "$ACTUAL"

if [ ! -f "$BASELINE" ]; then
  cp "$ACTUAL" "$BASELINE"
  echo "created baseline $BASELINE"
  exit 0
fi

if ! diff -u "$BASELINE" "$ACTUAL"; then
  echo "config snapshot drift detected; update $BASELINE intentionally" >&2
  exit 1
fi

echo "config snapshot: OK"
