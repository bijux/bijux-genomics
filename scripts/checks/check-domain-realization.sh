#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tmp_dir="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/check-domain-realization"
ensure_artifacts_dir "$tmp_dir"
hits_file="$tmp_dir/hits.txt"
: > "$hits_file"

# Domain realization policy:
# Stage planners must not pass through pre-baked command templates.
rg -n "tool\.command\.template\.clone\(\)" \
  "$ROOT_DIR/crates/bijux-dna-planner-bam/src" \
  "$ROOT_DIR/crates/bijux-dna-planner-vcf/src" \
  "$ROOT_DIR/crates/bijux-dna-planner-fastq/src" \
  >"$hits_file" || true

if [[ -s "$hits_file" ]]; then
  echo "domain-realization: FAIL"
  echo "Found non-realized command passthrough(s):"
  cat "$hits_file"
  exit 1
fi

echo "domain-realization: OK"
