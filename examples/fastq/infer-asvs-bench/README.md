# fastq_infer_asvs_bench

## Purpose
Run a deterministic FASTQ ASV-inference benchmark and preserve feature-table evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_infer_asvs_bench`

## Stage
- Stage ID: `fastq.infer_asvs`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `infer_asvs_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna -- bench fastq infer-asvs --sample-id infer-asvs-hpc --r1 <reads_r1.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/infer_asvs/infer-asvs-hpc/`
