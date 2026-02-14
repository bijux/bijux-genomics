#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

expected="$ROOT_DIR/containers/versions/index.sha256"
tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
tmp="$(mktemp "$tmp_root/versions-index.XXXXXX.sha256")"
trap 'rm -f "$tmp"' EXIT

"$SCRIPT_DIR/generate-versions-index-sha.sh" "$tmp" >/dev/null
if ! diff -u "$expected" "$tmp" >/dev/null; then
  echo "versions index sha drift: regenerate with scripts/containers/generate-versions-index-sha.sh" >&2
  exit 1
fi

echo "versions index sha: OK"
