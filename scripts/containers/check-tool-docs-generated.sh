#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
tmp_dir="$(mktemp -d "$tmp_root/tool-docs.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

"$SCRIPT_DIR/generate-tool-docs.sh" "$tmp_dir" >/dev/null

if ! diff -ru "$ROOT_DIR/containers/docs/tools" "$tmp_dir" >/dev/null; then
  echo "tool docs drift: regenerate with scripts/containers/generate-tool-docs.sh" >&2
  exit 1
fi
echo "tool docs generated: OK"

