#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
refs_file="$(mktemp "$tmp_root/tmp-config-paths.XXXXXX")"
trap 'rm -f "$refs_file"' EXIT

rg -No --no-filename 'configs/[A-Za-z0-9_./-]+\.(toml|md|sha256)' Makefile makefiles crates scripts docs .github \
  | perl -pe 's/[`"'"'"',;:)]*$//' \
  | sort -u > "$refs_file"

missing=0
while IFS= read -r rel; do
  [ -z "$rel" ] && continue
  case "$rel" in
    configs/runtime/profiles/hpc.toml|configs/tools.toml|configs/lab/config.toml)
      continue
      ;;
  esac
  if [ ! -e "$rel" ]; then
    echo "missing config reference: $rel"
    missing=1
  fi
done < "$refs_file"

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "config path references: OK"
