# example-103

`example-103` is FASTQ stage-series example 03 (`fastq-bench-stage-03`) and is fully self-contained.

## Purpose
Validate reproducible filter-stage benchmarking with deterministic example packaging and pinned probes.

## Stage
- Stage ID: `fastq.filter`
- Series mapping: `example-103` => FASTQ stage-03

## Inputs
- ENA project: `PRJEB44430`
- Selection policy: exactly `10 SE` + `10 PE`
- Species: `human` (canonical: `homo_sapiens`)
- Corpus ID: `example-103`

## Outputs
- ENA snapshot with selection reasons
- Normalized corpus + checksums
- Stage-03 benchmark manifests and telemetry
- Golden deterministic plan/explain artifacts

## Acceptance Criteria
- `bijux dna example validate example-103` passes
- `golden/plan.json` is deterministic for `bijux dna example plan example-103`
- `golden/explain.json` exists and matches stage/suite semantics
- `bench-suite.toml` pins exactly stage-03 tooling probes

## How To Run On HPC
```bash
bijux dna example run example-103 --hpc
```
