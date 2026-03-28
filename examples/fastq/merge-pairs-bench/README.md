# fastq_merge_pairs_bench

## Purpose
Run a deterministic paired-end FASTQ merge benchmark and preserve merged-read evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_merge_pairs_bench`

## Stage
- Stage ID: `fastq.merge_pairs`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `merge_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq merge --sample-id merge-pairs-hpc --r1 <reads_r1.fastq.gz> --r2 <reads_r2.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/merge_pairs/merge-pairs-hpc/`
