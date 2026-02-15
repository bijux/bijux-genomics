#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

files=$(rg --files \
  crates/bijux-dna-api/src \
  crates/bijux-dna-engine/src \
  crates/bijux-dna-runner/src \
  crates/bijux-dna-stages-bam/src \
  crates/bijux-dna-stages-fastq/src \
  crates/bijux-dna-stages-vcf/src)
files=$(printf '%s\n' "$files" | rg -v '(_contracts\.rs|/tests?/)' || true)

if [[ -z "$files" ]]; then
  echo "check-executor-no-unwrap: no source files found" >&2
  exit 1
fi

violations=$(printf '%s\n' "$files" | xargs rg -n "\\.(unwrap|expect)\\(" -S || true)
if [[ -n "$violations" ]]; then
  echo "ERROR: unwrap/expect are banned in executor crate source (tests may use them)." >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

echo "check-executor-no-unwrap: OK"
