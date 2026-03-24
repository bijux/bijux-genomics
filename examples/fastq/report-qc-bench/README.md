# fastq_report_qc_bench

## Purpose
Run a deterministic FASTQ QC reporting benchmark and preserve stage evidence for backend and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_report_qc_bench`

## Stage
- Stage ID: `fastq.report_qc`
- Domain family: `fastq`

## Inputs
- Mini corpus FASTQ from `corpus-01-mini`
- QC reporting backends selected through the benchmark suite for identical input hashes
- A report-only contract: this stage evaluates evidence and report generation, not read mutation

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `qc_report.json`
- `metrics.json`
- `bench.jsonl`
- `bench.sqlite`

## Acceptance Criteria
- Every benchmark record uses the same input hash and report-only stage contract
- The benchmark preserves MultiQC/FastQC evidence paths when present
- The top-level report contains persisted benchmark records and failure classification

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dna bench fastq report-qc --sample-id report-qc-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --explain`
3. Collect outputs under `<bench-dir>/report_qc/report-qc-hpc/`
