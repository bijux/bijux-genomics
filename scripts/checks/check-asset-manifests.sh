#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

status=0
while IFS= read -r pub_dir; do
  [[ -d "$pub_dir" ]] || continue
  manifest="$pub_dir/MANIFEST.toml"
  rel="${manifest#"$ROOT_DIR/"}"
  if [[ ! -f "$manifest" ]]; then
    echo "asset-manifests: missing $rel" >&2
    status=1
    continue
  fi
  for key in license provenance citation; do
    if ! rg -q "^[[:space:]]*${key}[[:space:]]*=" "$manifest"; then
      echo "asset-manifests: $rel missing key '${key}'" >&2
      status=1
    fi
  done
done < <(find "$ROOT_DIR/assets/publications" -mindepth 1 -maxdepth 1 -type d | sort)

while IFS= read -r gen; do
  [[ -f "$gen" ]] || continue
  rel="${gen#"$ROOT_DIR/"}"
  for heading in "## Command(s)" "## Tool versions"; do
    if ! rg -F -q "${heading}" "$gen"; then
      echo "asset-manifests: $rel missing heading '${heading}'" >&2
      status=1
    fi
  done
done < <(find "$ROOT_DIR/assets/golden" -type f -name GENERATE.md | sort)

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

echo "asset-manifests: OK"
