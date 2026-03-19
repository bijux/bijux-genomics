# fastq_detect_adapters_bench

## Purpose
Run a deterministic FASTQ adapter-detection benchmark flow and validate expected contracts.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_detect_adapters_bench`
Direct benchmark command: `cargo run -q -p bijux-dna -- bench fastq detect-adapters --sample-id detect-adapters-hpc --r1 <reads_R1.fastq.gz> --r2 <reads_R2.fastq.gz> --out <bench-dir> --tools auto`

## Step 1 Containers
- Ensure benchmark images are resolved before execution.

## Step 2 Build/Verify
- Validate the example contract (`example.toml`) and corpus selection before execution.

## Step 3 Bench
- Execute the adapter-detection benchmark flow using the example-pinned suite.
- Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so adapter prevalence is benchmarked across the full fragment set.

## Step 4 Collect/Report
- Collect outputs in `artifacts/examples/fastq_detect_adapters_bench/` and produce `bundle.tar.gz`.
