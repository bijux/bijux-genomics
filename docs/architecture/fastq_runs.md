# FASTQ Runs (v1)

This document freezes the FASTQ v1 execution model.

## Discovery

1. Scan the input directory for FASTQ files.
2. Group files into samples.
3. Detect SE vs PE, R1/R2 pairing, gzip/plain, and naming inconsistencies.
4. Write `input_assessment.json`.

Pipelines must not guess layout; they must use the assessment output.

## Sample Identity

Each sample is identified by:

- `sample_name`
- `layout` (SE or PE)
- `r1_path`
- `r2_path` (optional)

All downstream names derive from this identity.

## Canonical Run Layout

```
runs/
  run_<timestamp>_<uuid>/
    input_assessment.json
    run_manifest.json
    environment.json
    stages/
      trim/
        tool/
          outputs/
          metrics.json
          logs/
      filter/
      ...
    summary/
      metrics_aggregate.json
```

Every FASTQ run uses this layout exactly.

## Environment Fingerprint

Each run records:

- `run_id`
- `timestamp`
- `hostname`
- OS / arch
- runner (docker/apptainer/native)
- tool images + digests

## Analysis Separation

Running a pipeline never ranks or compares.

Benchmarking reads only:

- run directories
- `metrics.json`
- manifests

Raw FASTQs can be deleted after the run.

## Run Index

`runs/index.json` records:

- run_id
- pipeline
- stages executed
- SE/PE layout
- tools used
- objective (if any)

## Benchmarking

`bijux fastq benchmark` is a read-only operation. It must not execute tools.

## v1 Freeze

The run layout, discovery behavior, and analysis separation described here are frozen for FASTQ v1.
