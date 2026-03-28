# fastq_remove_chimeras_bench

## Purpose
Run a deterministic FASTQ chimera-removal benchmark and preserve non-chimera evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_remove_chimeras_bench`

## Stage
- Stage ID: `fastq.remove_chimeras`
- Domain family: `fastq`

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `remove_chimeras_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq remove-chimeras --sample-id remove-chimeras-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/remove_chimeras/remove-chimeras-hpc/`
