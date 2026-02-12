# example-101

`example-101` is FASTQ stage-series example 01 (`fastq-bench-stage-01`) and is fully self-contained.

## Purpose
Validate end-to-end reproducibility for FASTQ stage-01 benchmarking on HPC using ENA-backed corpus selection.

## Stage
- Stage ID: `fastq.validate_pre`
- Series mapping: `example-101` => FASTQ stage-01

## Inputs
- ENA project: `PRJEB44430`
- Selection policy: exactly `10 SE` + `10 PE`
- Species: `human` (canonical: `homo_sapiens`)
- Corpus ID: `example-101`

## Outputs
- ENA snapshot with selection reasons
- Normalized corpus + checksums
- Stage-01 benchmark manifests and telemetry
- Golden deterministic plan/explain artifacts

## Acceptance Criteria
- Example validates against schema (`bijux example validate example-101`)
- Plan output matches `golden/plan.json` deterministically
- Explain output matches `golden/explain.json` deterministically
- Stage-01 suite only includes `fastq.validate_pre`

## How To Run On HPC
```bash
bijux example run example-101 --hpc
```
