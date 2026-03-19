# fastq_deplete_host_bench

## Purpose
Run a deterministic host-depletion benchmark flow with a pinned host-reference contract.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_host_bench`
Direct benchmark command: `cargo run -q -p bijux-dna -- bench fastq deplete-host --sample-id SAMPLE --r1 reads_R1.fastq.gz --r2 reads_R2.fastq.gz --out artifacts --tools auto`

## Stage
- Stage ID: `fastq.deplete_host`
- Domain family: `fastq`

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_deplete_host_bench`
3. Or submit the direct stage benchmark command above with your scheduler wrapper.
4. Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so the benchmark records `pairs_in` and `pairs_out` correctly.
5. Collect outputs under `artifacts/examples/fastq_deplete_host_bench/`
