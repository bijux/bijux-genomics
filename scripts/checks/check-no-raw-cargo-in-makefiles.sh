#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
if ! command -v rg >/dev/null 2>&1; then
  echo "raw-cargo-policy: ripgrep (rg) is required" >&2
  exit 127
fi

matches=$(rg -n "cargo (fmt|clippy|test|run|deny|nextest|llvm-cov|insta|build|check|doc|install)\\b" Makefile makefiles || true)
if [ -z "$matches" ]; then
  echo "raw-cargo-policy(makefiles): OK"
  exit 0
fi

violations=$(printf '%s\n' "$matches" | rg -v "(\\./bin/isolate cargo |RUN_IN_ISOLATE)" || true)
if [ -n "$violations" ]; then
  echo "raw-cargo-policy(makefiles): direct cargo invocation found; use ./bin/isolate cargo ..." >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

echo "raw-cargo-policy(makefiles): OK"
