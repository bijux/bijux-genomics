# fastq_normalize_primers_bench

## Purpose
Run a deterministic FASTQ primer-normalization benchmark and preserve primer-trim evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_normalize_primers_bench`

## Stage
- Stage ID: `fastq.normalize_primers`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `normalize_primers_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq normalize-primers --sample-id normalize-primers-hpc --r1 <reads_r1.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/normalize_primers/normalize-primers-hpc/`
