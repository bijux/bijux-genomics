#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
if [[ ! -f assets/golden/smoke-inputs-v1/bam/sample.bam ]]; then
  echo "Missing assets/golden/smoke-inputs-v1/bam/sample.bam. Generate it with samtools (see assets/golden/README.md)." >&2
  exit 1
fi

bijux bam stage \
  --stage validate \
  --bam assets/golden/smoke-inputs-v1/bam/sample.bam \
  --out artifacts/smoke_bam \
  --sample-id smoke_bam \
  --dry-run
