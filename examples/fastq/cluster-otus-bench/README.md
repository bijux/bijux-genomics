# fastq_cluster_otus_bench

## Purpose
Run a deterministic OTU-clustering benchmark flow for amplicon-oriented FASTQ analysis.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_cluster_otus_bench`

## Stage
- Stage ID: `fastq.cluster_otus`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_cluster_otus_bench`
3. Collect outputs under `artifacts/examples/fastq_cluster_otus_bench/`
