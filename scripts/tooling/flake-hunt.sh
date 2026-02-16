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
  cat <<'EOF'
Usage: scripts/tooling/flake-hunt.sh --expr '<nextest-filter>' [--runs N]
EOF
}

expr=""
runs="20"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --expr)
      expr="${2:-}"
      shift 2
      ;;
    --runs)
      runs="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$expr" ]]; then
  echo "--expr is required" >&2
  usage >&2
  exit 2
fi

exec "${ROOT_DIR}/scripts/experimental/flake-hunt.sh" "$expr" "$runs"
