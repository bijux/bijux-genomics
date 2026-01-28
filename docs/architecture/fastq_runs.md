# FASTQ Runs (v1)

This document freezes the FASTQ v1 execution model.

## Discovery

1. Scan the input directory for FASTQ files.
2. Group files into samples.
3. Detect SE vs PE, R1/R2 pairing, gzip/plain, and naming inconsistencies.
4. Write `input_assessment.json` (immutable).

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
  <run_id>/
    meta/
      input_assessment.json
      run_manifest.json
      run_metadata.json
      events.jsonl
    stages/
      trim/
        tool/
          outputs/
          metrics/
            execution_metrics.json
            domain_metrics.json
          tool_invocation.json
          logs/
      filter/
      ...
    metrics/
    logs/
```

Every FASTQ run uses this layout exactly.

## Environment Fingerprint

Each run records (`run_metadata.json`):

- `run_id`
- `started_at` / `finished_at`
- `hostname`
- `os` / `arch`
- `cpu_model`, `cores`, `ram_mb`
- `platform` (docker/apptainer/local)
- `platform_version`
- `bijux_version`
- `git_commit`

## Analysis Separation

Running a pipeline never ranks or compares.

Benchmarking reads only:

- run directories
- `execution_metrics.json`
- `domain_metrics.json`
- manifests and metadata

Raw FASTQs can be deleted after the run.

## Run Index

`bijux-runs/index.jsonl` records (append-only):

- run_id
- domain
- pipeline
- stages executed
- SE/PE layout
- tools used
- platform
- objective (if any)
- success/failure

## Benchmarking

`bijux fastq benchmark` is a read-only operation. It must not execute tools.

## v1 Freeze

The run layout, discovery behavior, and analysis separation described here are frozen for FASTQ v1.
