# Quickstart

## What
A minimal local dry‑run to verify planning and artifacts without executing tools.

## Why
Dry‑run provides explainability and stable graphs without requiring tool binaries.

## Non-goals
- Full execution with real inputs.

## Contracts
- Dry‑run emits `graph.json` and `run_manifest.json`.

## Examples
```
bijux fastq preprocess --dry-run --r1 reads.fastq --out out --sample-id sample
```

## Failure modes
- Missing required inputs (e.g., `--r1`) causes validation error.
- Invalid pipeline ID fails planning.
