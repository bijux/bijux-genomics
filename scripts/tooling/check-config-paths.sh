#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

refs_file="$(mktemp)"
trap 'rm -f "$refs_file"' EXIT

rg -No --no-filename 'configs/[A-Za-z0-9_./-]+\.(toml|md|sha256)' Makefile makefiles crates scripts docs .github \
  | perl -pe 's/[`"'"'"',;:)]*$//' \
  | sort -u > "$refs_file"

missing=0
while IFS= read -r rel; do
  [ -z "$rel" ] && continue
  case "$rel" in
    configs/profile.hpc.toml|configs/tools.toml)
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
