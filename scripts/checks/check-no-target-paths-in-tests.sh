#!/usr/bin/env bash
set -euo pipefail
LC_ALL=C
export LC_ALL

if ! command -v rg >/dev/null 2>&1; then
  echo "no-target-paths-in-tests: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

offenders="$(
  {
    rg -n "target/" crates \
      --glob "**/tests/**/*.rs" \
      --glob "!crates/bijux-dna-policies/tests/**" \
      || true
    rg -n "target/" scripts \
      --glob "**/*.sh" \
      --glob "!scripts/checks/check-no-target-paths-in-tests.sh" \
      || true
    rg -n "target/" makefiles \
      --glob "**/*.mk" \
      || true
  } | sed '/^$/d'
)"
if [[ -n "${offenders}" ]]; then
  echo "hardcoded target/ paths in tests/scripts/makefiles are forbidden; use env vars/current_exe:" >&2
  echo "${offenders}" >&2
  exit 1
fi

echo "no-target-paths-in-tests: OK"
