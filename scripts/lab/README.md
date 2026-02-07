# Lab / HPC Harness

This directory contains **manual** workflows for running real pipelines and benchmarks on scripts/lab/HPC
hardware. Nothing here is executed by CI.

## Requirements
- Docker or Apptainer installed (per `runner_kind`).
- Tool images built/pulled ahead of time.
- Real FASTQ/BAM corpora available on disk.

## Quick Start
1. Copy `scripts/lab/config.example.toml` to `scripts/lab/config.toml`.
2. Set `CORPUS_ROOT` (required) and optionally `RUNNER_KIND`, `PIPELINE_IDS`, `OUTPUT_DIR`.
3. Run:
```sh
make lab-fastq CORPUS_ROOT=/path/to/corpus
make lab-bam CORPUS_ROOT=/path/to/corpus
```

## Notes
- These scripts are intentionally simple and explicit.
- For production HPC workflows, wire these into your scheduler (Slurm, PBS, etc.).
