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
  cat <<'USAGE'
Usage: scripts/test/require-isolate-smoke.sh
USAGE
  exit 0
fi

if env -u ISO_TAG -u ISO_RUN_ID -u ISO_ROOT -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP \
  "$ROOT_DIR/bin/require-isolate" >/dev/null 2>&1; then
  echo "require-isolate-smoke: expected non-isolated invocation to fail" >&2
  exit 1
fi

echo "require-isolate-smoke: OK"
