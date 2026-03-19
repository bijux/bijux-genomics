# fastq_deplete_rrna_bench

## Purpose
Run a deterministic rRNA-depletion benchmark flow for pre-HPC screening.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_rrna_bench`

## Stage
- Stage ID: `fastq.deplete_rrna`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_rrna_bench`
3. Collect outputs under `artifacts/examples/fastq_deplete_rrna_bench/`
