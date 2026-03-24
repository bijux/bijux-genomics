# fastq_profile_overrepresented_sequences_bench

## Purpose
Run a deterministic FASTQ overrepresented-sequence profiling benchmark and preserve sequence-profile evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_profile_overrepresented_sequences_bench`

## Stage
- Stage ID: `fastq.profile_overrepresented_sequences`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `overrepresented_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq profile-overrepresented-sequences --sample-id overrepresented-hpc --r1 <reads_r1.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/profile_overrepresented_sequences/overrepresented-hpc/`
