#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool=""
to_status=""
stage=""
reason=""
removal_after=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool) tool="${2:-}"; shift ;;
    --to) to_status="${2:-}"; shift ;;
    --stage) stage="${2:-}"; shift ;;
    --reason) reason="${2:-}"; shift ;;
    --removal-after) removal_after="${2:-}"; shift ;;
    --help|-h)
      cat <<'EOF'
Usage:
  scripts/containers/tool-lifecycle.sh --tool <id> --to experimental
  scripts/containers/tool-lifecycle.sh --tool <id> --to stable
  scripts/containers/tool-lifecycle.sh --tool <id> --to experimental --stage <domain.stage> --reason <text> --removal-after <YYYY-MM-DD>

Notes:
- `stable` is the lifecycle alias for production container status.
- Status changes must be done through this script (no manual edits).
EOF
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

[[ -n "$tool" ]] || { echo "--tool required" >&2; exit 2; }
[[ -n "$to_status" ]] || { echo "--to required" >&2; exit 2; }

case "$to_status" in
  experimental)
    "$SCRIPT_DIR/promote.sh" --tool "$tool" --to experimental
    ;;
  stable)
    "$SCRIPT_DIR/promote.sh" --tool "$tool" --to production
    ;;
  *)
    echo "--to must be experimental|stable" >&2
    exit 2
    ;;
esac

