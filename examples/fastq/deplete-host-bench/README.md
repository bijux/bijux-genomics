# fastq_deplete_host_bench

## Purpose
Run a deterministic host-depletion benchmark flow with a pinned host-reference contract.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_host_bench`

## Stage
- Stage ID: `fastq.deplete_host`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_host_bench`
3. Collect outputs under `artifacts/examples/fastq_deplete_host_bench/`
