# fastq_deplete_reference_contaminants_bench

## Purpose
Run a deterministic reference-contaminant depletion benchmark with explicit decoy-bank expectations.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_reference_contaminants_bench`
Direct benchmark command: `cargo run -q -p bijux-dna bench fastq deplete-reference-contaminants --sample-id SAMPLE --r1 reads_R1.fastq.gz --r2 reads_R2.fastq.gz --out artifacts --tools auto`

## Stage
- Stage ID: `fastq.deplete_reference_contaminants`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_reference_contaminants_bench`
3. Or submit the direct stage benchmark command above with your scheduler wrapper.
4. Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so depletion metrics are computed across the full fragment set.
5. Collect outputs under artifacts/examples/fastq_deplete_reference_contaminants_bench/
