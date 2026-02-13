#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
bijux fastq preprocess \
  --r1 assets/golden/smoke-inputs-v1/fastq/se/reads.fastq \
  --out artifacts/smoke_fastq \
  --sample-id smoke_fastq \
  --dry-run
