#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: scripts/containers/make.sh <subcommand> [args...]" >&2
  exit 0
fi

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <subcommand>" >&2
  exit 2
fi

cmd="$1"
shift
if [[ $# -eq 0 ]]; then
  exec cargo run -p bijux-dev-dna -- containers run "$cmd"
fi

exec cargo run -p bijux-dev-dna -- containers run "$cmd" -- "$@"
