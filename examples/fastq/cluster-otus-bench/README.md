# fastq_cluster_otus_bench

## Purpose
Run a deterministic OTU-clustering benchmark flow for amplicon-oriented FASTQ analysis.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_cluster_otus_bench`
Direct benchmark command: `cargo run -q -p bijux-dna bench fastq cluster-otus --sample-id SAMPLE --r1 reads.fastq.gz --out artifacts --tools auto`

## Stage
- Stage ID: `fastq.cluster_otus`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna-dev -- examples run run fastq_cluster_otus_bench`
3. Or submit the direct stage benchmark command above with your scheduler wrapper.
4. Collect outputs under artifacts/examples/fastq_cluster_otus_bench/
