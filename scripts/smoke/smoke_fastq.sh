#!/usr/bin/env bash
set -euo pipefail
LC_ALL=C
export LC_ALL

bijux fastq preprocess \
  --r1 assets/golden/fastq/se/reads.fastq \
  --out artifacts/smoke_fastq \
  --sample-id smoke_fastq \
  --dry-run
