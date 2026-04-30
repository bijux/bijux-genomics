# fastq_essential_qc

## Purpose
Run the essential FASTQ QC path with deterministic governed outputs.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_essential_qc`

## Step 1 Containers
- Ensure image planning is resolved for the pinned QC tools.

## Step 2 Build/Verify
- Validate the mini corpus contract before execution.
- Validate that the governed QC manifest and aggregation report paths remain deterministic.

## Step 3 Bench
- Execute the essential QC stages:
  - `fastq.validate_reads`
  - `fastq.detect_adapters`
  - `fastq.trim_reads`
  - `fastq.profile_reads`
  - `fastq.report_qc`

## Step 4 Collect/Report
- Collect plan, explain, and governed report outputs under `artifacts/examples/fastq_essential_qc/`.
- Preserve the governed QC manifest for downstream inspection and evidence bundling.
