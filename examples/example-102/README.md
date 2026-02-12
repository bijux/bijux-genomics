# example-102

`example-102` is FASTQ stage-series example 02 (`fastq-bench-stage-02`) and is fully self-contained.

## Purpose
Validate reproducible trim-stage benchmarking with deterministic example packaging and pinned probes.

## Stage
- Stage ID: `fastq.trim`
- Series mapping: `example-102` => FASTQ stage-02

## Inputs
- ENA project: `PRJEB44430`
- Selection policy: exactly `10 SE` + `10 PE`
- Species: `human` (canonical: `homo_sapiens`)
- Corpus ID: `example-102`

## Outputs
- ENA snapshot with selection reasons
- Normalized corpus + checksums
- Stage-02 benchmark manifests and telemetry
- Golden deterministic plan/explain artifacts

## Acceptance Criteria
- `bijux dna example validate example-102` passes
- `golden/plan.json` is deterministic for `bijux dna example plan example-102`
- `golden/explain.json` exists and matches stage/suite semantics
- `bench-suite.toml` pins exactly stage-02 tooling probes

## How To Run On HPC
```bash
bijux dna example run example-102 --hpc
```
