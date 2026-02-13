# fastq_qc_pre_bench

## Purpose
Run a deterministic FASTQ pre-QC benchmark flow and validate expected contracts.

Canonical invocation: `./scripts/examples/run.sh fastq_qc_pre_bench`

## Step 1 Containers
- Ensure image plan is resolved by the runner.

## Step 2 Build/Verify
- Validate the example contract (`example.toml`) and corpus selection before execution.

## Step 3 Bench
- Execute the benchmark flow using the example-pinned suite.

## Step 4 Collect/Report
- Collect outputs in `artifacts/examples/fastq_qc_pre_bench/` and produce `bundle.tar.gz`.
