# fastq_deplete_rrna_bench

## Purpose
Run a deterministic rRNA-depletion benchmark flow for pre-HPC screening.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_deplete_rrna_bench`
Direct benchmark command: `cargo run -q -p bijux-dna bench fastq deplete-rrna --sample-id SAMPLE --r1 reads_R1.fastq.gz --r2 reads_R2.fastq.gz --out artifacts --tools auto`

## Stage
- Stage ID: `fastq.deplete_rrna`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dna-dev -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna-dev -- examples run run fastq_deplete_rrna_bench`
3. Or submit the direct stage benchmark command above with your scheduler wrapper.
4. Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so depletion and retention fractions stay fragment-aware.
5. Collect outputs under artifacts/examples/fastq_deplete_rrna_bench/
