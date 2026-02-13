#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for ex in "$ROOT_DIR"/examples/* "$ROOT_DIR"/examples/*/*; do
  [[ -d "$ex" ]] || continue
  [[ -f "$ex/example.toml" ]] || continue
  rel="${ex#"$ROOT_DIR/"}"
  if [[ ! -f "$ex/golden/plan.json" ]]; then
    echo "examples golden: $rel missing golden/plan.json" >&2
    errors=1
  fi
  if [[ ! -f "$ex/golden/explain.json" ]]; then
    echo "examples golden: $rel missing golden/explain.json" >&2
    errors=1
  fi
  if [[ ! -f "$ex/golden/report.html" && ! -f "$ex/golden/report.json" ]]; then
    echo "examples golden: $rel requires golden/report.html or golden/report.json" >&2
    errors=1
  fi
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples golden: OK"
