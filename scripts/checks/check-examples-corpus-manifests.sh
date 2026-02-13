#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for corpus_dir in "$ROOT_DIR"/examples/data/*; do
  [[ -d "$corpus_dir" ]] || continue
  manifest="$corpus_dir/MANIFEST.toml"
  if [[ ! -f "$manifest" ]]; then
    echo "corpus manifest missing: ${manifest#"$ROOT_DIR/"}" >&2
    errors=1
    continue
  fi
  for key in license source checksum_policy normalization_steps; do
    if ! rg -q "^${key}\s*=" "$manifest"; then
      echo "${manifest#"$ROOT_DIR/"} missing key: $key" >&2
      errors=1
    fi
  done
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples corpus manifests: OK"
