#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

viol=0

if rg -n "RUSTFLAGS\s*=\s*\"" "$ROOT_DIR/crates" --glob '**/.cargo/config.toml' >/dev/null 2>&1; then
  echo "check-rustflags-consistency: crate-local .cargo/config.toml may not set RUSTFLAGS" >&2
  rg -n "RUSTFLAGS\s*=\s*\"" "$ROOT_DIR/crates" --glob '**/.cargo/config.toml' >&2 || true
  viol=1
fi

matches="$(rg -n "RUSTFLAGS=" "$ROOT_DIR/scripts" "$ROOT_DIR/makefiles" "$ROOT_DIR/Makefile" -S || true)"
if [[ -n "$matches" ]]; then
  filtered="$(printf '%s\n' "$matches" | rg -v "scripts/tooling/ci-coverage.sh|check-rustflags-consistency.sh" || true)"
  if [[ -n "$filtered" ]]; then
    echo "check-rustflags-consistency: undocumented RUSTFLAGS env usage detected" >&2
    printf '%s\n' "$filtered" >&2
    viol=1
  fi
fi

if [[ "$viol" -ne 0 ]]; then
  echo "check-rustflags-consistency: FAILED" >&2
  exit 1
fi

echo "check-rustflags-consistency: OK"
