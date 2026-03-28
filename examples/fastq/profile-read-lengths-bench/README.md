# fastq_profile_read_lengths_bench

## Purpose
Run a deterministic FASTQ read-length profiling benchmark and preserve histogram evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_profile_read_lengths_bench`

## Stage
- Stage ID: `fastq.profile_read_lengths`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `profile_read_lengths_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq profile-read-lengths --sample-id profile-read-lengths-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/profile_read_lengths/profile-read-lengths-hpc/`
