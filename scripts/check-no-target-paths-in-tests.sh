#!/usr/bin/env bash
set -euo pipefail

if ! command -v rg >/dev/null 2>&1; then
  echo "no-target-paths-in-tests: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

offenders="$(rg -n "target/" crates --glob "**/tests/**/*.rs" || true)"
if [[ -n "${offenders}" ]]; then
  echo "hardcoded target/ paths in tests are forbidden; use env vars/current_exe:" >&2
  echo "${offenders}" >&2
  exit 1
fi

echo "no-target-paths-in-tests: OK"
