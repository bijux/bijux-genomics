#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-cargo-config-policy.sh
USAGE
  exit 0
fi

cfg="$ROOT_DIR/.cargo/config.toml"
[[ -f "$cfg" ]] || { echo "cargo-config-policy: missing .cargo/config.toml" >&2; exit 1; }

viol=()
if rg -q '^\[target\.' "$cfg"; then
  viol+=(".cargo/config.toml must not contain [target.*] blocks")
fi
if rg -q '^jobs\s*=' "$cfg"; then
  viol+=(".cargo/config.toml must not set machine-specific build.jobs")
fi
if rg -q 'rustflags\s*=' "$cfg"; then
  viol+=(".cargo/config.toml must not set rustflags; use configs/runtime/cargo_build.toml")
fi
if ! rg -q '^\[alias\]' "$cfg"; then
  viol+=(".cargo/config.toml must keep alias definitions")
fi

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "cargo-config-policy: FAILED" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "cargo-config-policy: OK"
