#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

bad_files=$(find "$ROOT_DIR/containers" -type f \( -name '*.digest' -o -name '*digests*.json' -o -name '*.sha256' \) \
  ! -path "$ROOT_DIR/containers/versions/*" || true)
if [[ -n "${bad_files:-}" ]]; then
  echo "digest output policy failed: generated digest artifacts must not live under containers/ tree" >&2
  printf '%s\n' "$bad_files" >&2
  exit 1
fi

echo "digest output policy: OK"
