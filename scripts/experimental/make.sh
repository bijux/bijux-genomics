#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

cmd="${1:-}"
shift || true
case "$cmd" in
  flake-hunt) exec "$SCRIPT_DIR/flake-hunt.sh" "$@" ;;
  *) echo "unknown experimental command: $cmd" >&2; exit 2 ;;
esac
