#!/usr/bin/env sh
set -eu
LC_ALL=C
export LC_ALL

./bin/require-isolate >/dev/null

if ! command -v rg >/dev/null 2>&1; then
  echo "isolation-contract: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

if rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >/dev/null 2>&1; then
  echo "absolute host paths leaked into snapshots" >&2
  rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >&2 || true
  exit 1
fi

echo "isolation-contract: OK"
