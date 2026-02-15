#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

doc="${ROOT_DIR}/docs/50-reference/MANIFEST_MIGRATION.md"
[[ -f "$doc" ]] || { echo "check-certification-schema-docs: missing $doc" >&2; exit 1; }

required=(
  "bijux.certification_bundle.v2"
  "bijux.certification_run_stamp.v1"
  "bijux.frontend.mini_domain_validation.v1"
)

missing=()
for v in "${required[@]}"; do
  if ! rg -n --fixed-strings "$v" "$doc" >/dev/null 2>&1; then
    missing+=("$v")
  fi
done

if ((${#missing[@]} > 0)); then
  echo "check-certification-schema-docs: FAILED" >&2
  for m in "${missing[@]}"; do
    echo " - missing schema version in migration doc: $m" >&2
  done
  exit 1
fi

echo "check-certification-schema-docs: OK"
