# fastq_filter_reads_bench

## Purpose
Run a deterministic FASTQ filtering benchmark flow and validate expected contracts.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_filter_reads_bench`

## Step 1 Containers
- Ensure benchmark images are resolved before execution.

## Step 2 Build/Verify
- Validate the example contract (`example.toml`) and corpus selection before execution.

## Step 3 Bench
- Execute the filtering benchmark flow using the example-pinned suite.

## Step 4 Collect/Report
- Collect outputs in artifacts/examples/fastq_filter_reads_bench/ and produce `bundle.tar.gz`.
