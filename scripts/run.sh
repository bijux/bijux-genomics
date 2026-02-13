#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <group> <command> [args...]" >&2
  exit 2
fi

group="$1"
command="$2"
shift 2

case "$group" in
  checks|tooling|containers|assets|docs|domain|smoke|test|lab)
    target="${SCRIPT_DIR}/${group}/${command}.sh"
    ;;
  hpc)
    # support hpc/lunarc style by allowing command to contain subpath
    target="${SCRIPT_DIR}/hpc/${command}.sh"
    ;;
  *)
    echo "unsupported group: $group" >&2
    exit 2
    ;;
esac

if [[ ! -x "$target" ]]; then
  if [[ -f "$target" ]]; then
    chmod +x "$target"
  else
    echo "script not found: ${target#"$ROOT_DIR/"}" >&2
    exit 1
  fi
fi

exec "$target" "$@"
