#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f assets/golden/bam/sample.bam ]]; then
  echo "Missing assets/golden/bam/sample.bam. Generate it with samtools (see assets/golden/README.md)." >&2
  exit 1
fi

bijux bam stage \
  --stage validate \
  --bam assets/golden/bam/sample.bam \
  --out artifacts/smoke_bam \
  --sample-id smoke_bam \
  --dry-run
