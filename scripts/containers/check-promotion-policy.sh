#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

policy="$ROOT_DIR/containers/docs/PROMOTION_POLICY.md"
if [[ ! -f "$policy" ]]; then
  echo "missing containers/docs/PROMOTION_POLICY.md" >&2
  exit 1
fi

required=(
  "License clarity"
  "Provenance"
  "Reproducibility"
  "Smoke quality"
  "scripts/containers/tool-lifecycle.sh"
  "scripts/containers/demote.sh"
)
for marker in "${required[@]}"; do
  if ! grep -Fq "$marker" "$policy"; then
    echo "promotion policy missing marker: $marker" >&2
    exit 1
  fi
done

echo "promotion policy: OK"
