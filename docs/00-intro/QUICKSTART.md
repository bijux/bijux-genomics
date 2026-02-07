# QUICKSTART

## What
A copy/paste walkthrough that produces a first successful run and artifacts.

## Why
Provides a minimal end-to-end proof that Bijux is installed and functioning.

## Non-goals
- Benchmark-quality runs.
- Performance tuning.

## Contracts
All outputs are contract artifacts: manifest, report, and step records.

## Examples
```bash
# Plan + execute a minimal FASTQ pipeline
bijux plan --pipeline fastq.default.v1 > graph.json
bijux execute --pipeline fastq.default.v1 --out runs/demo
```

Artifacts created:
- `runs/demo/run_manifest.json`
- `runs/demo/report.json`
- `runs/demo/report.html`
- `runs/demo/summary.tsv`
- `runs/demo/stage_0/*`

See `../30-operations/RUN_ARTIFACTS.md` for artifact meanings.

## Failure modes
- Missing tools => `ToolError` in execution.
- Missing artifacts => `ContractError` after step execution.
