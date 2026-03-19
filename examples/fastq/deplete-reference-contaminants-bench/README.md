# fastq_deplete_reference_contaminants_bench

## Purpose
Run a deterministic reference-contaminant depletion benchmark with explicit decoy-bank expectations.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_reference_contaminants_bench`

## Stage
- Stage ID: `fastq.deplete_reference_contaminants`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_reference_contaminants_bench`
3. Collect outputs under `artifacts/examples/fastq_deplete_reference_contaminants_bench/`
