#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

lock="$ROOT_DIR/containers/versions/lock.json"
tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
tmp="$(mktemp "$tmp_root/version-lock.XXXXXX.json")"
trap 'rm -f "$tmp"' EXIT

"$SCRIPT_DIR/generate-version-lock.sh" "$tmp" >/dev/null
if ! diff -u "$lock" "$tmp" >/dev/null; then
  echo "version lock drift: regenerate with scripts/containers/generate-version-lock.sh" >&2
  exit 1
fi
echo "version lock: OK"
