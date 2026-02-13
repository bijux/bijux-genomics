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

violations=$(printf '%s\n' "$matches" | rg -v "(\\./bin/isolate cargo |RUN_IN_ISOLATE|install once: cargo install|policy-no-raw-cargo)" || true)
if [ -n "$violations" ]; then
  echo "raw-cargo-policy(makefiles): direct cargo invocation found; use ./bin/isolate cargo ..." >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

echo "raw-cargo-policy(makefiles): OK"

tool_matches=$(rg -n "(^|[^[:alnum:]_])(rustup|pip)([[:space:]]|$)|python[0-9.]*[[:space:]]+-m[[:space:]]+venv\\b" Makefile makefiles || true)
if [ -z "$tool_matches" ]; then
  echo "raw-tooling-policy(makefiles): OK"
  exit 0
fi

tool_violations=$(printf '%s\n' "$tool_matches" | rg -v "scripts/tooling/" || true)
if [ -n "$tool_violations" ]; then
  echo "raw-tooling-policy(makefiles): direct rustup/pip/python -m venv found; route via scripts/tooling/*.sh" >&2
  printf '%s\n' "$tool_violations" >&2
  exit 1
fi

echo "raw-tooling-policy(makefiles): OK"
