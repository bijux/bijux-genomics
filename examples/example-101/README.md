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
- `bijux dna example validate example-101` passes
- `golden/plan.json` is deterministic for `bijux dna example plan example-101`
- `golden/explain.json` exists and matches stage/suite semantics
- `bench-suite.toml` pins exactly stage-01 tooling probes

## How To Run On HPC
```bash
bijux dna example run example-101 --hpc
```
