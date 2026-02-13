#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

violations=$(find "$ROOT_DIR/configs" -maxdepth 1 -type f \
  | sed "s#^$ROOT_DIR/##" \
  | grep -v '^configs/index.md$' || true)

if [[ -n "$violations" ]]; then
  echo "config-layout: files are forbidden directly under configs/ (except configs/index.md):" >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

ci_entries=$(find "$ROOT_DIR/configs/ci" -mindepth 1 -maxdepth 1 \( -type f -o -type l \) \
  | sed "s#^$ROOT_DIR/##" \
  | grep -v '^configs/ci/index.md$' || true)
if [[ -n "$ci_entries" ]]; then
  echo "config-layout: configs/ci must contain only index.md + subdirs:" >&2
  printf '%s\n' "$ci_entries" >&2
  exit 1
fi

for required in configs/ci/registry configs/ci/stages configs/ci/tools configs/ci/params; do
  [[ -d "$ROOT_DIR/$required" ]] || { echo "config-layout: missing $required" >&2; exit 1; }
done

echo "config-layout: OK"
