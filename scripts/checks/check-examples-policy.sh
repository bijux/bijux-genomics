#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

policy="$ROOT_DIR/examples/POLICY.md"
allowlist="$ROOT_DIR/examples/notebooks_allowlist.txt"

[[ -f "$policy" ]] || { echo "examples-policy: missing examples/POLICY.md" >&2; exit 1; }
[[ -f "$allowlist" ]] || { echo "examples-policy: missing examples/notebooks_allowlist.txt" >&2; exit 1; }

failed=0
for section in "Purpose:" "Scope:" "Contracts:" "Notebook Optional Path Rule"; do
  if ! rg -q "$section" "$policy"; then
    echo "examples-policy: examples/POLICY.md missing section '$section'" >&2
    failed=1
  fi
done

if ! rg -q "no notebooks unless allowlisted" "$policy"; then
  echo "examples-policy: policy must explicitly state notebook allowlist rule" >&2
  failed=1
fi

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "examples-policy: OK"
