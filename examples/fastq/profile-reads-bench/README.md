# fastq_profile_reads_bench

## Purpose
Run a deterministic FASTQ read-statistics benchmark and preserve composition evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_profile_reads_bench`

## Stage
- Stage ID: `fastq.profile_reads`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `stats_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq profile-reads --sample-id profile-reads-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/profile_reads/profile-reads-hpc/`
