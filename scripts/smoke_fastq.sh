#!/usr/bin/env bash
set -euo pipefail

bijux fastq preprocess \
  --r1 assets/golden/fastq/se/reads.fastq \
  --out artifacts/smoke_fastq \
  --sample-id smoke_fastq \
  --dry-run
