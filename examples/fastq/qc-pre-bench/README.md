# fastq_qc_pre_bench

## Purpose
Run a deterministic FASTQ pre-QC benchmark flow and validate expected contracts.

## Steps
1. Ensure images: `./scripts/run.sh containers ensure-images --plan`
2. Run bench: `./scripts/run.sh examples run fastq_qc_pre_bench`
3. Collect artifacts: `artifacts/examples/fastq_qc_pre_bench/`
4. Generate report: `artifacts/examples/fastq_qc_pre_bench/report.json`

## Runner
Use `./scripts/run.sh examples run fastq_qc_pre_bench`.
