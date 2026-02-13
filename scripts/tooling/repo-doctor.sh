#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/tooling/repo-doctor.sh --fast|--full
EOF
}

mode="${1:---fast}"
case "$mode" in
  --help|-h)
    usage
    exit 0
    ;;
  --fast)
    ./scripts/run.sh checks check-root-layout
    ./scripts/run.sh docs check-docs-graph
    ./scripts/run.sh checks check-supported-scripts
    ./scripts/run.sh checks check-no-orphan-scripts
    ;;
  --full)
    ./scripts/run.sh checks check-root-layout
    ./scripts/run.sh checks check-config-layout
    ./scripts/run.sh docs check-docs-graph
    ./scripts/run.sh checks check-supported-scripts
    ./scripts/run.sh checks check-no-orphan-scripts
    ./scripts/run.sh tooling generate-configs
    ./scripts/run.sh tooling check-config-snapshot
    ./scripts/run.sh domain check-inventory
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

echo "repo-doctor: OK ($mode)"
