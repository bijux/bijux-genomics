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
Usage: scripts/<group>/make.sh [--dry-run|--confirm] <command> [args...]
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

dry_run=1
confirm=0

while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --dry-run)
      dry_run=1
      confirm=0
      shift
      ;;
    --confirm)
      dry_run=0
      confirm=1
      shift
      ;;
    --)
      shift
      break
      ;;
    *)
      break
      ;;
  esac
done

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

if [[ "$confirm" -eq 1 ]]; then
  exec "$target" --confirm "$@"
fi

if [[ "$dry_run" -eq 1 ]]; then
  echo "[dry-run] $cmd (pass --confirm to execute)"
  exec "$target" --dry-run "$@"
fi

exec "$target" "$@"
