# fastq_validate_reads_bench

## Purpose
Run a deterministic single-stage FASTQ validation benchmark and preserve the resulting evidence for backend comparison.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_validate_reads_bench`

## Stage
- Stage ID: `fastq.validate_reads`
- Domain family: `fastq`

## Inputs
- Mini corpus FASTQ from `corpus-01-mini`
- Validator backends selected through the benchmark suite for identical input hashes
- Strict-mode validation policy held constant across candidate tools

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- validation stage metrics and reports under `artifacts/examples/fastq_validate_reads_bench/`

## Acceptance Criteria
- Every benchmark record uses the same input hash and strict-mode policy
- Validation remains report-only; no downstream mutation stage is included
- Report output includes `validation_report` evidence and benchmark metrics for each backend

## HPC Run
1. `cargo run -q -p bijux-dev-dna -- hpc run validate-frontend-constraints --confirm`
2. `cargo run -q -p bijux-dev-dna -- examples run run fastq_validate_reads_bench`
3. Collect outputs under `artifacts/examples/fastq_validate_reads_bench/`

Preferred direct benchmark command:
`cargo run -q -p bijux-dna -- bench fastq validate-reads --sample-id validate-reads-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --strict --replicates 3 --jobs 8`
