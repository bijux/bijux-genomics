# fastq_correct_errors_bench

## Purpose
Run a deterministic FASTQ error-correction benchmark and preserve corrected-read evidence for backend and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_correct_errors_bench`

## Stage
- Stage ID: `fastq.correct_errors`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `correct_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq correct --sample-id correct-errors-hpc --r1 <reads_r1.fastq.gz> --r2 <reads_r2.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/correct_errors/correct-errors-hpc/`
