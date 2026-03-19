# fastq_detect_adapters_bench

## Purpose
Run a deterministic FASTQ adapter-detection benchmark flow and validate expected contracts.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_detect_adapters_bench`

## Step 1 Containers
- Ensure benchmark images are resolved before execution.

## Step 2 Build/Verify
- Validate the example contract (`example.toml`) and corpus selection before execution.

## Step 3 Bench
- Execute the adapter-detection benchmark flow using the example-pinned suite.

## Step 4 Collect/Report
- Collect outputs in `artifacts/examples/fastq_detect_adapters_bench/` and produce `bundle.tar.gz`.
