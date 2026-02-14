#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/<group>/make.sh <command> [args...]
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

cmd="$1"
shift

target="${SCRIPT_DIR}/${cmd}.sh"
if [[ ! -f "$target" ]]; then
  # allow nested commands for hpc (e.g. lunarc/push)
  target="${SCRIPT_DIR}/${cmd}.sh"
fi

if [[ ! -f "$target" ]]; then
  echo "unsupported command for $(basename "$SCRIPT_DIR"): $cmd" >&2
  exit 2
fi

if [[ ! -x "$target" ]]; then
  chmod +x "$target"
fi

exec "$target" "$@"
